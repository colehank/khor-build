//! Real ordinary requests on cached wires, with a scratch peer at loopback.

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

/// Both directions traverse these sockets; closing the gate drops real UDP
/// packets without closing either QUIC endpoint or changing host firewall rules.
struct Gate {
    address: std::net::SocketAddr,
    open: Arc<AtomicBool>,
    dropped: Arc<AtomicUsize>,
    pump: Aborts<()>,
}

impl Gate {
    async fn new(server: std::net::SocketAddr) -> Self {
        let front = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let back = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = front.local_addr().unwrap();
        let open = Arc::new(AtomicBool::new(true));
        let dropped = Arc::new(AtomicUsize::new(0));
        let pump = Aborts(Some(tokio::spawn({
            let (open, dropped) = (open.clone(), dropped.clone());
            async move {
                let (mut from_client, mut from_server) = ([0u8; 65536], [0u8; 65536]);
                let mut client = None;
                loop {
                    tokio::select! {
                        packet = front.recv_from(&mut from_client) => {
                            let (n, source) = packet.unwrap();
                            client = Some(source);
                            if open.load(Ordering::SeqCst) {
                                back.send_to(&from_client[..n], server).await.unwrap();
                            } else {
                                dropped.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                        packet = back.recv_from(&mut from_server) => {
                            let (n, source) = packet.unwrap();
                            assert_eq!(source, server);
                            if open.load(Ordering::SeqCst) {
                                if let Some(client) = client {
                                    front.send_to(&from_server[..n], client).await.unwrap();
                                }
                            } else {
                                dropped.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                }
            }
        })));
        Self {
            address,
            open,
            dropped,
            pump,
        }
    }

    async fn close(&mut self) {
        self.pump.0.as_ref().unwrap().abort();
        let _ = self.pump.0.as_mut().unwrap().await;
    }
}

struct Fixture {
    _root: tempfile::TempDir,
    node: Arc<Node>,
    peer: DeviceInfo,
    ep: iroh::Endpoint,
    server: iroh::Endpoint,
    local: iroh::endpoint::Connection,
    remote: iroh::endpoint::Connection,
}

impl Fixture {
    async fn new() -> Self {
        Self::new_via(false).await.0
    }

    async fn new_via(gated: bool) -> (Self, Option<Gate>) {
        let server = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
            .relay_mode(iroh::RelayMode::Disabled)
            .alpns(vec![ALPN.to_vec()])
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .bind()
            .await
            .unwrap();
        let (root, node, mut peer, ep) = fixture_for(server.id().to_string()).await;
        peer.addrs = server
            .addr()
            .addrs
            .iter()
            .filter_map(|addr| match addr {
                iroh::TransportAddr::Ip(addr) => Some(addr.to_string()),
                _ => None,
            })
            .collect();
        assert!(!peer.addrs.is_empty());
        let gate = if gated {
            Some(Gate::new(peer.addrs[0].parse().unwrap()).await)
        } else {
            None
        };
        let mut address = server.addr();
        if let Some(gate) = &gate {
            address.addrs = std::iter::once(iroh::TransportAddr::Ip(gate.address)).collect();
        }
        // Finish the handshake and adopt the wire before starting any deadline.
        let (local, remote) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(ep.connect(address, ALPN), async {
                server.accept().await.unwrap().accept().unwrap().await
            })
        })
        .await
        .unwrap();
        let (local, remote) = (local.unwrap(), remote.unwrap());
        node.wires
            .adopt(&peer.id, wires::Proto::Main, local.clone());
        (
            Self {
                _root: root,
                node: Arc::new(node),
                peer,
                ep,
                server,
                local,
                remote,
            },
            gate,
        )
    }

    fn line(&self) -> Line {
        Line {
            id: self.peer.id.clone(),
            name: self.peer.name.clone(),
            proto: wires::Proto::Main,
            conn: self.local.clone(),
        }
    }

    async fn close(&self) {
        self.ep.close().await;
        self.server.close().await;
    }
}

pub(super) async fn observed_by<T>(task: &mut Aborts<T>, by: tokio::time::Instant) -> Option<T> {
    match tokio::time::timeout_at(by, task.0.as_mut().unwrap()).await {
        Ok(Ok(answer)) => Some(answer),
        Ok(Err(_)) => None,
        Err(_) => {
            task.0.as_ref().unwrap().abort();
            let _ = task.0.as_mut().unwrap().await;
            None
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_expiry_stops_its_stream_and_preserves_a_concurrent_request() {
    observe_ordinary_expiry(Duration::from_secs(1)).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_long_wait_on_an_acknowledging_wire_keeps_its_concurrent_request() {
    observe_ordinary_expiry(Duration::from_millis(4100)).await;
}

async fn observe_ordinary_expiry(within: Duration) {
    let f = Fixture::new().await;
    let heard_before = wires::heard_count(&f.local);
    let patience = wires::Patience::of(&f.local).ack;
    assert_eq!(patience, Duration::from_secs(4));
    let (entered, ready) = tokio::sync::oneshot::channel();
    let (control_entered, control_ready) = tokio::sync::oneshot::channel();
    let (stopped, stop_seen) = tokio::sync::oneshot::channel();
    let requests = Arc::new(AtomicUsize::new(0));
    let mut serving = Aborts(Some(tokio::spawn({
        let conn = f.remote.clone();
        let requests = requests.clone();
        async move {
            let (delayed, mut recv) = conn.accept_bi().await.unwrap();
            let bytes = recv.read_to_end(MAX_FRAME).await.unwrap();
            assert!(matches!(
                proto::question(&bytes).unwrap(),
                Request::Sessions
            ));
            requests.fetch_add(1, Ordering::SeqCst);
            entered.send(()).unwrap();
            let release = tokio::sync::Notify::new();
            tokio::join!(
                async {
                    let _ = stopped.send(matches!(delayed.stopped().await, Ok(Some(_))));
                    release.notify_one();
                },
                async {
                    let (mut send, mut recv) = conn.accept_bi().await.unwrap();
                    let bytes = recv.read_to_end(MAX_FRAME).await.unwrap();
                    assert!(matches!(
                        proto::question(&bytes).unwrap(),
                        Request::Sessions
                    ));
                    requests.fetch_add(1, Ordering::SeqCst);
                    control_entered.send(()).unwrap();
                    // This second request is in flight at expiry, and answers
                    // only after the first stream has been stopped.
                    release.notified().await;
                    let answer = proto::encode(&Response::SessionRows { rows: vec![] }).unwrap();
                    send.write_all(&answer).await.unwrap();
                    send.finish().unwrap();
                }
            );
            conn.closed().await;
        }
    })));
    let by = tokio::time::Instant::now() + within;
    let mut asking = Aborts(Some(tokio::spawn({
        let node = f.node.clone();
        let line = f.line();
        crate::deadline::scope(by, async move {
            node.exchange_on(&line, &Request::Sessions).await
        })
    })));
    tokio::time::timeout_at(by, ready).await.unwrap().unwrap();
    let mut control = Aborts(Some(tokio::spawn({
        let node = f.node.clone();
        let ep = f.ep.clone();
        let peer = f.peer.clone();
        async move { node.ask_on(&ep, &peer, &Request::Sessions).await }
    })));
    tokio::time::timeout_at(by, control_ready)
        .await
        .unwrap()
        .unwrap();
    assert!(tokio::time::Instant::now() < by);
    let answer = observed_by(&mut asking, by + Duration::from_millis(500)).await;
    let stream_stopped = matches!(
        tokio::time::timeout(Duration::from_millis(500), stop_seen).await,
        Ok(Ok(true))
    );
    let control = observed_by(
        &mut control,
        tokio::time::Instant::now() + Duration::from_secs(3),
    )
    .await;
    let live = f.node.wires.live(&f.peer.id, wires::Proto::Main);
    let counts = f.node.wires.counts.snapshot();
    let heard_after = wires::heard_count(&f.local);
    let down = f.node.wires.link(&f.peer.id).down;
    let backoff = f.node.wires.backing_off(&f.peer.id);
    f.close().await;
    serving.0.as_ref().unwrap().abort();
    let _ = serving.0.as_mut().unwrap().await;

    assert!(matches!(answer, Some(Err(Miss::CallerExpired))));
    assert!(
        stream_stopped,
        "expiry must stop only the abandoned response stream"
    );
    assert!(matches!(control, Some(Ok(Response::SessionRows { .. }))));
    assert_eq!(live.map(|conn| conn.stable_id()), Some(f.local.stable_id()));
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(counts.reads_retried, 0);
    assert_eq!(counts.dial_failed, 0);
    assert_eq!(counts.answers_late, 0);
    assert_eq!(counts.retired_unanswered, 0);
    assert!(
        heard_after > heard_before,
        "the peer's transport actually acknowledged"
    );
    if within > patience {
        assert!(
            tokio::time::Instant::now() >= by,
            "this control crosses the real ACK floor"
        );
    }
    assert_eq!(down, None);
    assert_eq!(backoff, None);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn abandoning_a_long_silent_exchange_still_retires_the_dead_wire() {
    use std::future::Future;
    let (f, gate) = Fixture::new_via(true).await;
    let mut gate = gate.unwrap();
    assert!(f.local.close_reason().is_none() && f.remote.close_reason().is_none());
    gate.open.store(false, Ordering::SeqCst);
    // Drain packets already in flight before the guard takes its own baseline.
    tokio::time::sleep(Duration::from_millis(200)).await;
    let heard_before = wires::heard_count(&f.local);
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        wires::heard_count(&f.local),
        heard_before,
        "gate fixture must first be quiet"
    );
    let patience = wires::Patience::of(&f.local).ack;
    assert_eq!(patience, Duration::from_secs(4));
    let grace = wires::ask_every_path(&f.local);
    let line = f.line();
    let by = tokio::time::Instant::now() + patience + grace + Duration::from_millis(100);
    let mut asking = Box::pin(crate::deadline::scope(
        by,
        f.node.exchange_on(&line, &Request::Sessions),
    ));
    let pending = std::future::poll_fn(|cx| {
        std::task::Poll::Ready(matches!(asking.as_mut().poll(cx), std::task::Poll::Pending))
    })
    .await;
    assert!(pending, "the real exchange must enter its response wait");
    // Keep the application future unpolled while transport I/O continues.
    // This models an operation abandoned after a scheduler delay and makes
    // Drop, rather than the inner liveness beat, own the retirement. Wait out
    // both ACK patience and all-path grace; do not pin an early-retirement policy.
    tokio::time::sleep_until(by + Duration::from_millis(100)).await;
    assert_eq!(
        f.node.wires.live(&f.peer.id, wires::Proto::Main).map(|c| c.stable_id()),
        Some(f.local.stable_id())
    );
    drop(asking);
    let unheard = wires::heard_count(&f.local) == heard_before;
    let dropped = gate.dropped.load(Ordering::SeqCst);
    let remote_open = f.remote.close_reason().is_none();
    let live = f.node.wires.live(&f.peer.id, wires::Proto::Main);
    let counts = f.node.wires.counts.snapshot();
    f.close().await;
    gate.close().await;

    assert!(
        unheard,
        "fixture bypassed its UDP gate; this is not a blackhole"
    );
    assert!(
        dropped > 0,
        "the real request/probes must hit the closed gate"
    );
    assert!(
        remote_open,
        "fixture must not substitute a connection close for silence"
    );
    assert!(
        live.is_none(),
        "long real silence must not keep handing out a dead wire"
    );
    assert_eq!(counts.retired_unanswered, 1);
    assert_eq!(counts.retired_closed, 0);
    assert_eq!(counts.reads_retried, 0);
    assert_eq!(counts.dial_failed, 0);
    assert_eq!(counts.answers_late, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_read_retry_uses_the_original_operations_remaining_time() {
    let f = Fixture::new().await;
    let by = tokio::time::Instant::now() + Duration::from_secs(2);
    let (entered, ready) = tokio::sync::oneshot::channel();
    let (stopped, stop_seen) = tokio::sync::oneshot::channel();
    let requests = Arc::new(AtomicUsize::new(0));
    let mut serving = Aborts(Some(tokio::spawn({
        let first = f.remote.clone();
        let server = f.server.clone();
        let requests = requests.clone();
        async move {
            let (_send, mut recv) = first.accept_bi().await.unwrap();
            let bytes = recv.read_to_end(MAX_FRAME).await.unwrap();
            assert!(matches!(
                proto::question(&bytes).unwrap(),
                Request::Sessions
            ));
            requests.fetch_add(1, Ordering::SeqCst);
            // Spend most of the one operation budget on the first attempt.
            tokio::time::sleep_until(by - Duration::from_millis(700)).await;
            first.close(0u32.into(), b"lost first answer");
            let second = server
                .accept()
                .await
                .unwrap()
                .accept()
                .unwrap()
                .await
                .unwrap();
            let (delayed, mut recv) = second.accept_bi().await.unwrap();
            let bytes = recv.read_to_end(MAX_FRAME).await.unwrap();
            assert!(matches!(
                proto::question(&bytes).unwrap(),
                Request::Sessions
            ));
            requests.fetch_add(1, Ordering::SeqCst);
            entered.send(tokio::time::Instant::now()).unwrap();
            let _ = stopped.send(matches!(delayed.stopped().await, Ok(Some(_))));
            second.closed().await;
        }
    })));
    let mut asking = Aborts(Some(tokio::spawn({
        let node = f.node.clone();
        let ep = f.ep.clone();
        let peer = f.peer.clone();
        crate::deadline::scope(by, async move {
            node.ask_on(&ep, &peer, &Request::Sessions).await
        })
    })));
    let retry_at = tokio::time::timeout_at(by, ready).await.unwrap().unwrap();
    assert!(
        retry_at < by,
        "fixture must reach the second exchange before expiry"
    );
    let answer = observed_by(&mut asking, by + Duration::from_millis(500)).await;
    let ended_at = tokio::time::Instant::now();
    let stream_stopped = matches!(
        tokio::time::timeout(Duration::from_millis(500), stop_seen).await,
        Ok(Ok(true))
    );
    let live = f.node.wires.live(&f.peer.id, wires::Proto::Main);
    let counts = f.node.wires.counts.snapshot();
    f.close().await;
    serving.0.as_ref().unwrap().abort();
    let _ = serving.0.as_mut().unwrap().await;

    assert!(
        matches!(answer, Some(Err(_))),
        "retry ignored the operation deadline"
    );
    assert!(ended_at >= by && ended_at < by + Duration::from_millis(500));
    assert!(stream_stopped);
    assert!(live.is_some_and(|conn| conn.stable_id() != f.local.stable_id()));
    assert_eq!(requests.load(Ordering::SeqCst), 2);
    assert_eq!(counts.reads_retried, 1);
    assert_eq!(counts.dial_failed, 0);
    assert_eq!(counts.answers_late, 0);
    assert_eq!(counts.retired_closed, 1);
    assert_eq!(counts.retired_unanswered, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_unanswered_write_is_not_replayed() {
    lost_answers(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_unanswered_read_is_retried_at_most_once() {
    lost_answers(true).await;
}

async fn lost_answers(reading: bool) {
    let f = Fixture::new().await;
    let path = f._root.path().join("written-once");
    let req = if reading {
        Request::Sessions
    } else {
        Request::Mkdir {
            path: path.to_string_lossy().into_owned(),
        }
    };
    assert_eq!(req.is_reading(), reading);
    let requests = Arc::new(AtomicUsize::new(0));
    let mut serving = Aborts(Some(tokio::spawn({
        let mut next = Some(f.remote.clone());
        let server = f.server.clone();
        let requests = requests.clone();
        let path = path.clone();
        async move {
            // Continue serving possible replays after dropping every answer;
            // an unserved second connection would hide a repeated write.
            for _ in 0..4 {
                let conn = match next.take() {
                    Some(conn) => conn,
                    None => match tokio::time::timeout(Duration::from_millis(800), server.accept())
                        .await
                    {
                        Ok(Some(incoming)) => incoming.accept().unwrap().await.unwrap(),
                        _ => break,
                    },
                };
                let (_send, mut recv) = conn.accept_bi().await.unwrap();
                let bytes = recv.read_to_end(MAX_FRAME).await.unwrap();
                match proto::question(&bytes).unwrap() {
                    Request::Sessions if reading => {}
                    Request::Mkdir { path: requested } if !reading => {
                        assert_eq!(std::path::Path::new(&requested), path);
                        std::fs::create_dir_all(&path).unwrap();
                    }
                    _ => panic!("unexpected request in replay fixture"),
                }
                requests.fetch_add(1, Ordering::SeqCst);
                conn.close(0u32.into(), b"answer deliberately lost after execution");
            }
        }
    })));
    let mut asking = Aborts(Some(tokio::spawn({
        let node = f.node.clone();
        let ep = f.ep.clone();
        let peer = f.peer.clone();
        async move { node.ask_on(&ep, &peer, &req).await }
    })));
    let answer = observed_by(
        &mut asking,
        tokio::time::Instant::now() + Duration::from_secs(5),
    )
    .await;
    let server_done = observed_by(
        &mut serving,
        tokio::time::Instant::now() + Duration::from_secs(3),
    )
    .await;
    let counts = f.node.wires.counts.snapshot();
    f.close().await;

    assert!(matches!(answer, Some(Err(_))));
    assert!(server_done.is_some());
    assert_eq!(requests.load(Ordering::SeqCst), if reading { 2 } else { 1 });
    assert_eq!(counts.reads_retried, u64::from(reading));
    assert_eq!(counts.dial_failed, 0);
    assert_eq!(path.is_dir(), !reading);
}
