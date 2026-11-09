//! Real ordinary requests on cached wires, with a scratch peer at loopback.

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

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
        // Finish the handshake and adopt the wire before starting any deadline.
        let (local, remote) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(ep.connect(server.addr(), ALPN), async {
                server.accept().await.unwrap().accept().unwrap().await
            })
        })
        .await
        .unwrap();
        let (local, remote) = (local.unwrap(), remote.unwrap());
        node.wires
            .adopt(&peer.id, wires::Proto::Main, local.clone());
        Self {
            _root: root,
            node: Arc::new(node),
            peer,
            ep,
            server,
            local,
            remote,
        }
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

async fn observed_by<T>(task: &mut Aborts<T>, by: tokio::time::Instant) -> Option<T> {
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
    let f = Fixture::new().await;
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
    let by = tokio::time::Instant::now() + Duration::from_secs(1);
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
    assert_eq!(down, None);
    assert_eq!(backoff, None);
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
