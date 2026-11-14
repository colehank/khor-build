//! Explicit separate-mode contracts. Existing mocks stay in shared mode.
use super::*;

mod pooling;
mod preparation;

fn peer(f: &Fixture) -> DeviceInfo {
    let mut peer = f
        .node
        .devices()
        .unwrap()
        .into_iter()
        .find(|p| p.id == f.server.id().to_string())
        .unwrap();
    peer.addrs = f
        .server
        .addr()
        .ip_addrs()
        .map(ToString::to_string)
        .collect();
    peer
}

async fn extra(f: &Fixture) -> (TransferLine, iroh::endpoint::Connection) {
    let peer = peer(f);
    let (line, remote) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(f.node.transfer_line_for(&f.ep, &peer), async {
            f.server.accept().await.unwrap().accept().unwrap().await
        })
    })
    .await
    .unwrap();
    let (line, remote) = (line.unwrap(), remote.unwrap());
    assert_ne!(line.0.conn.stable_id(), f.local.stable_id());
    assert_eq!(line.0.conn.alpn(), ALPN);
    (line, remote)
}

async fn cached_answer(f: &Fixture) {
    let peer = peer(f);
    let answer = tokio::time::timeout(
        Duration::from_secs(5),
        f.node.ask_on(&f.ep, &peer, &Request::Sessions),
    )
    .await
    .unwrap();
    assert!(matches!(answer, Ok(Response::SessionRows { .. })));
    assert_eq!(
        f.node
            .wires
            .live(&peer.id, wires::Proto::Main)
            .unwrap()
            .stable_id(),
        f.local.stable_id()
    );
    assert!(f.local.close_reason().is_none() && f.remote.close_reason().is_none());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dropping_transfer_closes_only_the_additional_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    drop(line);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    eprintln!("[pull-isolation] {{\"event\":\"drop_extra_keeps_cached\"}}");
    f.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn peer_pause_closes_cached_and_transfer_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    f.node
        .wires
        .retire_peer("unrelated-peer", wires::Retired::Paused);
    assert!(line.0.conn.close_reason().is_none() && remote.close_reason().is_none());
    cached_answer(&f).await;
    f.node
        .wires
        .retire_peer(&peer(&f).id, wires::Retired::Paused);
    tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(remote.closed(), f.remote.closed());
    })
    .await
    .unwrap();
    assert!(line.0.conn.close_reason().is_some() && f.local.close_reason().is_some());
    assert!(
        f.node
            .wires
            .live(&peer(&f).id, wires::Proto::Main)
            .is_none()
    );
    drop(line);
    eprintln!("[pull-isolation] {{\"event\":\"peer_pause_closes_both\"}}");
    f.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn side_cleanup_closes_transfer_without_retiring_cached_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    f.node.wires.let_go_served(wires::Retired::Suspended);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    assert!(line.0.conn.close_reason().is_some());
    cached_answer(&f).await;
    drop(line);
    eprintln!("[pull-isolation] {{\"event\":\"side_cleanup_keeps_cached\"}}");
    f.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failed_transfer_read_retries_once_on_the_original_cached_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    remote.close(0u32.into(), b"controlled transfer failure");
    tokio::time::timeout(Duration::from_secs(5), line.0.conn.closed())
        .await
        .unwrap();
    let before = f.node.wires.counts.snapshot().reads_retried;
    let peer = peer(&f);
    let answer = tokio::time::timeout(
        Duration::from_secs(5),
        f.node
            .ask_on_transfer(&f.ep, &peer, &Request::Sessions, &line.0),
    )
    .await
    .unwrap();
    assert!(matches!(answer, Ok(Response::SessionRows { .. })));
    assert_eq!(f.node.wires.counts.snapshot().reads_retried - before, 1);
    cached_answer(&f).await;
    drop(line);
    eprintln!("[pull-isolation] {{\"event\":\"one_read_retry_cached\"}}");
    f.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dropping_a_separate_pull_closes_its_real_held_rpcs_and_keeps_cached_main() {
    observe_separate_pull(true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn completing_a_separate_pull_closes_only_its_file_main() {
    observe_separate_pull(false).await;
}

async fn observe_separate_pull(cancel: bool) {
    let f = Fixture::with_identity(if cancel { Held::All } else { Held::Head }, true).await;
    // Supply the exact direct hint to the stored peer used by the actual pull.
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    let issued = Arc::new(Mutex::new(BTreeSet::new()));
    let changed = Arc::new(Notify::new());
    let (accepted_tx, accepted_rx) = tokio::sync::oneshot::channel();
    let mut serving = Aborts(Some(tokio::spawn({
        let (server, issued, changed, bytes) = (
            f.server.clone(),
            issued.clone(),
            changed.clone(),
            f.bytes.clone(),
        );
        async move {
            let remote = server
                .accept()
                .await
                .unwrap()
                .accept()
                .unwrap()
                .await
                .unwrap();
            accepted_tx.send(remote.clone()).unwrap();
            let mut children = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    stream = remote.accept_bi() => {
                        let Ok((mut send, mut recv)) = stream else { break };
                        let (issued, changed, bytes) = (issued.clone(), changed.clone(), bytes.clone());
                        children.spawn(async move {
                            let raw = recv.read_to_end(MAX_FRAME).await.unwrap();
                            let Request::FetchPath { path, offset } = proto::question(&raw).unwrap() else { panic!("file RPC required") };
                            assert_eq!(path, PATH);
                            assert!(issued.lock().unwrap().insert(offset));
                            changed.notify_one();
                            if offset == 0 || !cancel {
                                let start = offset as usize;
                                let end = (start + proto::SLICE as usize).min(bytes.len());
                                let reply = Response::PathSlice { total:bytes.len() as u64, at_ms:123, bytes:bytes[start..end].to_vec().into() };
                                send.write_all(&proto::encode(&reply).unwrap()).await.unwrap();
                                send.finish().unwrap();
                                let _ = send.stopped().await;
                            } else {
                                let _ = send.stopped().await;
                            }
                        });
                    }
                    child = children.join_next(), if !children.is_empty() => { child.unwrap().unwrap(); }
                }
            }
            children.shutdown().await;
        }
    })));
    let remote;
    {
        let landing = crate::files::landing(PATH, f.root.path()).unwrap();
        let pull = f
            .node
            .pull_into_with_layout(&f.ep, "peer", PATH, landing, None, None, false, true);
        tokio::pin!(pull);
        remote = tokio::select! {
            outcome = &mut pull => panic!("held pull returned: {outcome:?}"),
            remote = tokio::time::timeout(Duration::from_secs(5), accepted_rx) => remote.unwrap().unwrap(),
        };
        if !cancel {
            // The cached prefix stays responsive; release its optional pending
            // slice only after the actual extra connection has been accepted.
            f.release.notify_one();
        }
        if cancel {
            tokio::select! {
                outcome = &mut pull => panic!("held pull returned: {outcome:?}"),
                ready = tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        let notified = changed.notified();
                        if issued.lock().unwrap().len() + f.seen.lock().unwrap().issued.len() == PULL_AHEAD as usize + 1 { break; }
                        notified.await;
                    }
                }) => ready.unwrap(),
            }
            assert_eq!(
                issued.lock().unwrap().len() + f.seen.lock().unwrap().issued.len(),
                PULL_AHEAD as usize + 1
            );
            assert_eq!(std::fs::metadata(f.part()).unwrap().len(), proto::SLICE);
        } else {
            let (moved, dest) = tokio::time::timeout(Duration::from_secs(20), &mut pull)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(moved, f.bytes.len() as u64);
            assert_eq!(std::fs::read(dest).unwrap(), *f.bytes);
            assert!(!f.part().exists() && !PullNote::path_of(&f.part()).exists());
            let cached: BTreeSet<_> = f
                .seen
                .lock()
                .unwrap()
                .issued
                .iter()
                .map(|i| i * proto::SLICE)
                .collect();
            let additional = issued.lock().unwrap();
            assert!(
                cached.is_disjoint(&additional),
                "no slice is replayed at handoff"
            );
            assert_eq!(
                cached.union(&additional).copied().collect::<BTreeSet<_>>(),
                (0..13).map(|i| i * proto::SLICE).collect()
            );
        }
    }
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), serving.0.as_mut().unwrap())
        .await
        .unwrap()
        .unwrap();
    serving.0.take();
    cached_answer(&f).await;
    assert!(
        f.seen.lock().unwrap().issued.contains(&0),
        "first content uses cached Main"
    );
    assert!(
        !issued.lock().unwrap().contains(&0),
        "no duplicate first slice on the extra Main"
    );
    assert!(
        !issued.lock().unwrap().is_empty(),
        "remaining file uses the actual extra Main"
    );
    if cancel {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let notified = f.changed.notified();
                let stopped = {
                    let seen = f.seen.lock().unwrap();
                    seen.issued
                        .difference(&seen.delivered)
                        .all(|i| seen.stopped.contains(i))
                };
                if stopped {
                    break;
                }
                notified.await;
            }
        })
        .await
        .unwrap();
    }
    eprintln!(
        "[pull-isolation] {}",
        serde_json::json!({"event":if cancel {"cancel_real_pull_keeps_cached"} else {"complete_real_pull_keeps_cached"}})
    );
    f.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failed_transfer_write_is_not_replayed_on_cached_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let (line, remote) = extra(&f).await;
    remote.close(0u32.into(), b"controlled transfer failure");
    tokio::time::timeout(Duration::from_secs(5), line.0.conn.closed())
        .await
        .unwrap();
    let before = f.node.wires.counts.snapshot().reads_retried;
    let peer = peer(&f);
    let req = Request::Mkdir {
        path: "/must-not-replay".into(),
    };
    assert!(!req.is_reading());
    let answer = tokio::time::timeout(
        Duration::from_secs(5),
        f.node.ask_on_transfer(&f.ep, &peer, &req, &line.0),
    )
    .await
    .unwrap();
    assert!(answer.is_err());
    assert_eq!(f.node.wires.counts.snapshot().reads_retried, before);
    cached_answer(&f).await;
    drop(line);
    eprintln!("[pull-isolation] {{\"event\":\"write_not_replayed\"}}");
    f.close().await;
}
