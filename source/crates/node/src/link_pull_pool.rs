//! Active-file leases, scoped to the resident's actual Endpoint instance.
//! Map locks are released before closing connections; no pool operation takes
//! the cached-slot, served-connection or Node::ears locks.
use super::{DIAL_MOST, PreparationFailure, TransferLine};
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, Weak},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, watch};
use tokio::time::{Duration, Instant};

const EXTRA: usize = 4;
const BULK: usize = 32;
const PEER_BULK: usize = 8;
const FIRST: usize = 8;

pub(crate) struct Pools {
    scope: Mutex<Option<Arc<Scope>>>,
    extra: Arc<Semaphore>,
    bulk: Arc<Semaphore>,
    first: Arc<Semaphore>,
}

impl Default for Pools {
    fn default() -> Self {
        Self {
            scope: Mutex::new(None),
            extra: Arc::new(Semaphore::new(EXTRA)),
            bulk: Arc::new(Semaphore::new(BULK)),
            first: Arc::new(Semaphore::new(FIRST)),
        }
    }
}

struct Scope {
    owner: Weak<iroh::Endpoint>,
    peers: Mutex<HashMap<String, Weak<Peer>>>,
    dead: std::sync::atomic::AtomicBool,
    extra: Arc<Semaphore>,
    bulk: Arc<Semaphore>,
    first: Arc<Semaphore>,
    #[cfg(test)]
    ready: Arc<std::sync::atomic::AtomicUsize>,
}

impl Pools {
    #[cfg(test)]
    pub(super) fn available_extra(&self) -> usize {
        self.extra.available_permits()
    }

    /// Called only while the Node holds its canonical ears guard.
    pub(super) fn peer(&self, owner: &Arc<iroh::Endpoint>, id: &str) -> Arc<Peer> {
        let (scope, old) = {
            let mut slot = self.scope.lock().unwrap_or_else(|p| p.into_inner());
            let same = slot.as_ref().is_some_and(|s| {
                s.owner.upgrade().is_some_and(|ep| Arc::ptr_eq(&ep, owner))
                    && !s.dead.load(std::sync::atomic::Ordering::Acquire)
            });
            let old = if same { None } else { slot.take() };
            let scope = slot
                .get_or_insert_with(|| {
                    Arc::new(Scope {
                        owner: Arc::downgrade(owner),
                        peers: Mutex::new(HashMap::new()),
                        dead: false.into(),
                        extra: self.extra.clone(),
                        bulk: self.bulk.clone(),
                        first: self.first.clone(),
                        #[cfg(test)]
                        ready: Default::default(),
                    })
                })
                .clone();
            (scope, old)
        };
        if let Some(old) = old {
            old.retire(b"endpoint replaced");
        }
        let mut peers = scope.peers.lock().unwrap_or_else(|p| p.into_inner());
        peers.retain(|_, p| p.strong_count() > 0);
        if let Some(peer) = peers.get(id).and_then(Weak::upgrade) {
            return peer;
        }
        let peer = Arc::new(Peer {
            scope: scope.clone(),
            opening: tokio::sync::Mutex::new(()),
            live: Mutex::new(Weak::new()),
            cancelled: watch::channel(false).0,
            bulk: Arc::new(Semaphore::new(PEER_BULK)),
            #[cfg(test)]
            ready: Default::default(),
        });
        peers.insert(id.to_owned(), Arc::downgrade(&peer));
        peer
    }

    pub(crate) fn retire_peer(&self, id: &str, reason: &[u8]) {
        let scope = self.scope.lock().unwrap_or_else(|p| p.into_inner()).clone();
        let peer = scope.and_then(|s| {
            let peer = s.peers.lock().unwrap_or_else(|p| p.into_inner()).remove(id);
            peer.and_then(|p| p.upgrade())
        });
        if let Some(peer) = peer {
            peer.retire(reason);
        }
    }

    pub(crate) fn clear(&self, reason: &[u8]) {
        let scope = self.scope.lock().unwrap_or_else(|p| p.into_inner()).take();
        if let Some(scope) = scope {
            scope.retire(reason);
        }
    }
}

impl Scope {
    fn extra_permit(&self) -> Result<OwnedSemaphorePermit, tokio::sync::TryAcquireError> {
        if let Ok(permit) = self.extra.clone().try_acquire_owned() {
            return Ok(permit);
        }
        // A closed connection may still be owned by a file. Reclaim only its
        // logical slot, on demand; no polling task or idle timer is needed.
        let peers: Vec<_> = self
            .peers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .values()
            .filter_map(Weak::upgrade)
            .collect();
        for peer in peers {
            let lease = peer
                .live
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .upgrade();
            if let Some(lease) = lease
                && lease.line.0.conn.close_reason().is_some()
            {
                lease.close(b"file connection ended");
            }
        }
        self.extra.clone().try_acquire_owned()
    }

    fn retire(&self, reason: &[u8]) {
        self.dead.store(true, std::sync::atomic::Ordering::Release);
        let peers = std::mem::take(&mut *self.peers.lock().unwrap_or_else(|p| p.into_inner()));
        for peer in peers.into_values().filter_map(|p| p.upgrade()) {
            peer.retire(reason);
        }
    }
}

pub(super) struct Peer {
    scope: Arc<Scope>,
    opening: tokio::sync::Mutex<()>,
    live: Mutex<Weak<Lease>>,
    cancelled: watch::Sender<bool>,
    bulk: Arc<Semaphore>,
    #[cfg(test)]
    ready: Arc<std::sync::atomic::AtomicUsize>,
}

pub(super) struct Lease {
    pub(super) line: TransferLine,
    permit: Mutex<Option<OwnedSemaphorePermit>>,
    _peer: Arc<Peer>,
}

impl Lease {
    fn close(&self, reason: &[u8]) {
        self.line.0.conn.close(0u32.into(), reason);
        self.permit.lock().unwrap_or_else(|p| p.into_inner()).take();
    }
}

/// Kept alongside a ready response until that response is committed/discarded.
pub(super) struct Credit {
    _peer: Option<OwnedSemaphorePermit>,
    _global: OwnedSemaphorePermit,
    #[cfg(test)]
    ready: Option<(
        Arc<std::sync::atomic::AtomicUsize>,
        Arc<std::sync::atomic::AtomicUsize>,
    )>,
    #[cfg(test)]
    decoded: bool,
}

#[cfg(test)]
impl Credit {
    pub(super) fn decoded(&mut self) {
        if let Some((peer, global)) = &self.ready {
            peer.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            global.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.decoded = true;
        }
    }
}

#[cfg(test)]
impl Drop for Credit {
    fn drop(&mut self) {
        if self.decoded
            && let Some((peer, global)) = &self.ready
        {
            peer.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            global.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
pub(super) struct Usage {
    pub(super) peer_held: usize,
    pub(super) global_held: usize,
    pub(super) peer_ready: usize,
    pub(super) global_ready: usize,
    pub(super) foreground_held: usize,
    pub(super) owners: usize,
}

impl Peer {
    #[cfg(test)]
    pub(super) fn usage(&self) -> Usage {
        Usage {
            peer_held: PEER_BULK - self.bulk.available_permits(),
            global_held: BULK - self.scope.bulk.available_permits(),
            peer_ready: self.ready.load(std::sync::atomic::Ordering::SeqCst),
            global_ready: self.scope.ready.load(std::sync::atomic::Ordering::SeqCst),
            foreground_held: FIRST - self.scope.first.available_permits(),
            owners: self
                .live
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .strong_count(),
        }
    }
    fn stopped(&self) -> bool {
        *self.cancelled.borrow() || self.scope.dead.load(std::sync::atomic::Ordering::Acquire)
    }

    fn retire(&self, reason: &[u8]) {
        self.cancelled.send_replace(true);
        let old =
            std::mem::take(&mut *self.live.lock().unwrap_or_else(|p| p.into_inner())).upgrade();
        if let Some(old) = old {
            old.close(reason);
        }
    }

    pub(super) async fn prepare<F, Fut>(
        self: &Arc<Self>,
        open: F,
    ) -> Result<Arc<Lease>, PreparationFailure>
    where
        F: FnOnce(Duration) -> Fut,
        Fut: Future<Output = Result<TransferLine, PreparationFailure>>,
    {
        let caller = crate::deadline::current();
        let started = Instant::now();
        let until = caller.map_or(started + DIAL_MOST, |t| t.min(started + DIAL_MOST));
        let exhausted = || {
            if crate::deadline::expired(caller) {
                PreparationFailure::Stop(khor_catalog::msg::dial_timed_out().to_owned())
            } else {
                PreparationFailure::RetryCached("file connection preparation budget ended".into())
            }
        };
        let stopped = || PreparationFailure::Stop("file connection owner retired".into());
        let endpoint = self.scope.owner.upgrade().ok_or_else(stopped)?;
        let mut cancelled = self.cancelled.subscribe();
        if self.stopped() || endpoint.is_closed() {
            return Err(stopped());
        }
        let _opening = tokio::select! {
            biased;
            _ = cancelled.changed() => return Err(stopped()),
            _ = endpoint.closed() => return Err(stopped()),
            lock = tokio::time::timeout_at(until, self.opening.lock()) => lock.map_err(|_| exhausted())?,
        };
        if crate::deadline::expired(caller) {
            return Err(exhausted());
        }
        if self.stopped() || endpoint.is_closed() {
            return Err(stopped());
        }
        let live = self
            .live
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .upgrade();
        if let Some(live) = live {
            if live.line.0.conn.close_reason().is_none() {
                return Ok(live);
            }
            live.close(b"file connection ended");
        }
        let permit = self.scope.extra_permit().map_err(|_| {
            PreparationFailure::RetryCached("file connection capacity in use".into())
        })?;
        let remaining = DIAL_MOST.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(exhausted());
        }
        let line = tokio::select! {
            biased;
            _ = cancelled.changed() => return Err(stopped()),
            _ = endpoint.closed() => return Err(stopped()),
            line = open(remaining) => line?,
        };
        if crate::deadline::expired(caller) {
            return Err(exhausted());
        }
        if self.stopped() || endpoint.is_closed() {
            return Err(stopped());
        }
        let lease = Arc::new(Lease {
            line,
            permit: Mutex::new(Some(permit)),
            _peer: self.clone(),
        });
        let mut live = self.live.lock().unwrap_or_else(|p| p.into_inner());
        // Serialize publication against retire, including the cancellation flag.
        if self.stopped() {
            return Err(stopped());
        }
        *live = Arc::downgrade(&lease);
        Ok(lease)
    }

    fn credit_of(
        &self,
        peer: Option<OwnedSemaphorePermit>,
        global: OwnedSemaphorePermit,
        bulk: bool,
    ) -> Credit {
        #[cfg(not(test))]
        let _ = bulk;
        Credit {
            _peer: peer,
            _global: global,
            #[cfg(test)]
            ready: bulk.then(|| (self.ready.clone(), self.scope.ready.clone())),
            #[cfg(test)]
            decoded: false,
        }
    }

    pub(super) async fn credit(
        &self,
        first: bool,
        at: u64,
        progress: &Progress,
    ) -> Result<Credit, String> {
        let by = crate::deadline::current();
        let mut cancelled = self.cancelled.subscribe();
        let endpoint = self.scope.owner.upgrade().ok_or("connection ended")?;
        let mut head = progress.head.subscribe();
        loop {
            if crate::deadline::expired(by) {
                return Err(khor_catalog::msg::dial_timed_out().to_owned());
            }
            if self.stopped() {
                return Err("file connection owner retired".into());
            }
            if endpoint.is_closed() {
                return Err("connection ended".into());
            }
            let current = *head.borrow() == at;
            // Prefer ordinary capacity for the cursor too. The reserved lane
            // is needed only when retained prefetch would block its progress.
            if current && !first {
                if let Ok(peer) = self.bulk.clone().try_acquire_owned() {
                    if let Ok(global) = self.scope.bulk.clone().try_acquire_owned() {
                        return Ok(self.credit_of(Some(peer), global, true));
                    }
                }
            }
            if first || current {
                return tokio::select! {
                    biased;
                    _ = cancelled.changed() => Err("file connection owner retired".into()),
                    _ = endpoint.closed() => Err("connection ended".into()),
                    result = crate::deadline::wait(by, self.scope.first.clone().acquire_owned()) => {
                        let global = result.map_err(|_| khor_catalog::msg::dial_timed_out().to_owned())?.map_err(|e| e.to_string())?;
                        Ok(self.credit_of(None, global, false))
                    }
                };
            }
            let acquire = async {
                let peer = self
                    .bulk
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|e| e.to_string())?;
                let global = self
                    .scope
                    .bulk
                    .clone()
                    .acquire_owned()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok::<Credit, String>(self.credit_of(Some(peer), global, true))
            };
            // A queued prefetch changes lane only when it becomes this file's
            // committed cursor. No RPC has started, so promotion replays none.
            let promoted = async {
                loop {
                    if *head.borrow_and_update() == at {
                        return Ok::<(), String>(());
                    }
                    head.changed().await.map_err(|_| "file ended".to_owned())?;
                }
            };
            tokio::select! {
                biased;
                _ = cancelled.changed() => return Err("file connection owner retired".into()),
                _ = endpoint.closed() => return Err("connection ended".into()),
                result = promoted => { result?; },
                result = crate::deadline::wait(by, acquire) => return result.map_err(|_| khor_catalog::msg::dial_timed_out().to_owned())?,
            }
        }
    }
}

pub(super) struct Progress {
    head: watch::Sender<u64>,
}

impl Progress {
    pub(super) fn new(at: u64) -> Self {
        Self {
            head: watch::channel(at).0,
        }
    }
    pub(super) fn advance(&self, at: u64) {
        self.head.send_replace(at);
    }
}
