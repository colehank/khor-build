//! Preparation is optional: a healthy cached Main can deliver first content.
use super::*;

mod resting;
mod transport;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cached_main_delivers_first_content_without_accepting_an_extra_handshake() {
    let f = Fixture::with_identity(Held::All, true).await;
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    // No acceptor is started for another connection. The already authenticated
    // cached connection remains served by the fixture, including its first slice.
    let first_content = {
        let landing = crate::files::landing(PATH, f.root.path()).unwrap();
        let pull = f
            .node
            .pull_into_with_layout(&f.ep, "peer", PATH, landing, None, None, false, true);
        tokio::pin!(pull);
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if let Ok(bytes) = std::fs::read(f.part())
                    && bytes.len() >= proto::SLICE as usize
                {
                    return bytes[..proto::SLICE as usize] == f.bytes[..proto::SLICE as usize];
                }
                tokio::select! {
                    outcome = &mut pull => panic!("held pull returned before first content: {outcome:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(5)) => {}
                }
            }
        })
        .await
        .unwrap_or(false)
        // The actual pull, its pending requests and optional dial are dropped
        // here, before checking the original connection or closing the fixture.
    };
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-preparation] {}",
        serde_json::json!({"first_content":first_content,"cached_main_control":true,"fixture_closed":true})
    );
    assert!(
        first_content,
        "healthy cached Main must deliver exact first content before an optional extra handshake"
    );
}

async fn complete_small_file_without_extra_handshake(size: usize) {
    let f = Fixture::with_payload(Held::All, false, true, size).await;
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    let landing = crate::files::landing(PATH, f.root.path()).unwrap();
    let (moved, destination) = tokio::time::timeout(
        Duration::from_secs(5),
        f.node
            .pull_into_with_layout(&f.ep, "peer", PATH, landing, None, None, false, true),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(moved, size as u64);
    assert_eq!(std::fs::read(destination).unwrap(), *f.bytes);
    assert!(!f.part().exists() && !PullNote::path_of(&f.part()).exists());
    assert_eq!(f.seen.lock().unwrap().issued, BTreeSet::from([0]));
    let unexpected = tokio::time::timeout(Duration::from_millis(500), f.server.accept()).await;
    assert!(
        unexpected.is_err(),
        "small file must not initiate an extra handshake"
    );
    cached_answer(&f).await;
    // The same observer must actually detect a new authenticated connection.
    let (line, remote) = extra(&f).await;
    drop(line);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-preparation-boundary] {}",
        serde_json::json!({"event":"small_file_no_extra","bytes":size,"exact_bytes":true,"extra_handshakes":0,"observer_positive_control":true,"cached_main_control":true,"fixture_closed":true})
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn empty_file_finishes_without_an_extra_handshake() {
    complete_small_file_without_extra_handshake(0).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn sub_slice_file_finishes_without_an_extra_handshake() {
    complete_small_file_without_extra_handshake(1234).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn exactly_one_slice_finishes_without_an_extra_handshake() {
    complete_small_file_without_extra_handshake(proto::SLICE as usize).await;
}

async fn preparation_is_prevented(kind: &str) {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    match kind {
        "local_pause" => crate::pause::set_on(f.node.root(), true).unwrap(),
        "peer_pause" => {
            crate::pause::set_peer(f.node.root(), &target.id, &target.name, true).unwrap()
        }
        "resting" => f
            .node
            .resting
            .store(true, std::sync::atomic::Ordering::Relaxed),
        "caller_expired" => {}
        _ => panic!("unknown preparation control"),
    }
    let attempt = async {
        tokio::time::timeout(
            Duration::from_secs(5),
            f.node.transfer_line_for(&f.ep, &target),
        )
        .await
    };
    let result = if kind == "caller_expired" {
        crate::deadline::scope(
            tokio::time::Instant::now() - Duration::from_millis(1),
            attempt,
        )
        .await
    } else {
        attempt.await
    };
    let prevented = matches!(&result, Ok(Err(PreparationFailure::Stop(_))));
    drop(result);
    match kind {
        "local_pause" => crate::pause::set_on(f.node.root(), false).unwrap(),
        "peer_pause" => {
            crate::pause::set_peer(f.node.root(), &target.id, &target.name, false).unwrap()
        }
        "resting" => f
            .node
            .resting
            .store(false, std::sync::atomic::Ordering::Relaxed),
        "caller_expired" => {}
        _ => unreachable!(),
    }
    let unexpected = tokio::time::timeout(Duration::from_millis(500), f.server.accept()).await;
    let no_handshake = unexpected.is_err();
    drop(unexpected);
    cached_answer(&f).await;
    let (line, remote) = extra(&f).await;
    drop(line);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-preparation-boundary] {}",
        serde_json::json!({"event":"preparation_prevented","kind":kind,"terminal_failure":prevented,"extra_handshakes":if no_handshake {0} else {1},"observer_positive_control":true,"cached_main_control":true,"fixture_closed":true})
    );
    assert!(
        prevented && no_handshake,
        "policy or caller expiry must not become optional fallback or a new handshake"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn local_pause_prevents_preparation_without_poisoning_cached_main() {
    preparation_is_prevented("local_pause").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn peer_pause_prevents_preparation_without_poisoning_cached_main() {
    preparation_is_prevented("peer_pause").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resting_prevents_preparation_without_poisoning_cached_main() {
    preparation_is_prevented("resting").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn expired_caller_prevents_preparation_without_poisoning_cached_main() {
    preparation_is_prevented("caller_expired").await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelling_an_unaccepted_preparation_preserves_the_cached_main() {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let mut attempt = Box::pin(f.node.transfer_line_for(&f.ep, &target));
    let incoming = tokio::select! {
        result = &mut attempt => panic!("preparation completed before the controlled handshake was accepted; success={}", result.is_ok()),
        incoming = tokio::time::timeout(Duration::from_secs(5), f.server.accept()) => incoming.unwrap().unwrap(),
    };
    drop(attempt);
    drop(incoming);
    cached_answer(&f).await;
    let (line, remote) = extra(&f).await;
    drop(line);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-preparation-boundary] {}",
        serde_json::json!({"event":"cancel_unaccepted_preparation","actual_incoming_observed":true,"cached_main_control":true,"later_extra_connection_control":true,"fixture_closed":true})
    );
}
