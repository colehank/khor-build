//! Decode real replies while both file prefixes are held; credits stay owned.
use super::*;
use std::collections::BTreeMap;

const A: &str = "/pool-a.bin";
const B: &str = "/pool-b.bin";

#[derive(Default)]
struct Traffic {
    issued: BTreeMap<(String, u64), bool>,
    duplicates: usize,
    stopped: BTreeSet<String>,
    handshakes: usize,
    extra: Option<iroh::endpoint::Connection>,
}

async fn serve_files(
    conn: iroh::endpoint::Connection,
    extra: bool,
    traffic: Arc<Mutex<Traffic>>,
    bytes: Arc<Vec<u8>>,
    release_a: Arc<Notify>,
    release_b: Arc<Notify>,
) {
    let mut children = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            stream = conn.accept_bi() => {
                let Ok((mut send, mut recv)) = stream else { break };
                let (traffic, bytes, release_a, release_b) =
                    (traffic.clone(), bytes.clone(), release_a.clone(), release_b.clone());
                children.spawn(async move {
                    let Ok(raw) = recv.read_to_end(MAX_FRAME).await else { return };
                    match proto::question(&raw).unwrap() {
                        Request::Sessions => {
                            send.write_all(&proto::encode(&Response::SessionRows { rows: vec![] }).unwrap()).await.unwrap();
                            send.finish().unwrap();
                        }
                        Request::FetchPath { path, offset } => {
                            assert!(path == A || path == B);
                            assert_eq!(offset % proto::SLICE, 0);
                            let index = offset / proto::SLICE;
                            {
                                let mut seen = traffic.lock().unwrap();
                                if seen.issued.insert((path.clone(), index), extra).is_some() { seen.duplicates += 1; }
                            }
                            if index == 1 {
                                let release = if path == A { &release_a } else { &release_b };
                                tokio::select! {
                                    _ = release.notified() => {}
                                    stopped = send.stopped() => {
                                        if matches!(stopped, Ok(Some(_))) { traffic.lock().unwrap().stopped.insert(path); }
                                        return;
                                    }
                                }
                            }
                            let start = offset as usize;
                            let end = (start + proto::SLICE as usize).min(bytes.len());
                            let response = Response::PathSlice { total:bytes.len() as u64, at_ms:123, bytes:bytes[start..end].to_vec().into() };
                            if send.write_all(&proto::encode(&response).unwrap()).await.is_ok() {
                                let _ = send.finish();
                                let _ = send.stopped().await;
                            }
                        }
                        other => panic!("unexpected file fixture request: {other:?}"),
                    }
                });
            }
            child = children.join_next(), if !children.is_empty() => { child.unwrap().unwrap(); }
        }
    }
    children.shutdown().await;
}

async fn controlled() -> (Fixture, Arc<Mutex<Traffic>>, Arc<Notify>, Arc<Notify>) {
    let mut f = Fixture::with_identity(Held::All, true).await;
    let old = f.serving.0.take().unwrap();
    old.abort();
    let _ = old.await;
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&peer(&f).id, "peer", &peer(&f).addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    let traffic = Arc::new(Mutex::new(Traffic::default()));
    let release_a = Arc::new(Notify::new());
    let release_b = Arc::new(Notify::new());
    f.serving = Aborts(Some(tokio::spawn({
        let (server, cached, traffic, bytes, ra, rb) = (
            f.server.clone(),
            f.remote.clone(),
            traffic.clone(),
            f.bytes.clone(),
            release_a.clone(),
            release_b.clone(),
        );
        async move {
            let mut connections = tokio::task::JoinSet::new();
            connections.spawn(serve_files(
                cached,
                false,
                traffic.clone(),
                bytes.clone(),
                ra.clone(),
                rb.clone(),
            ));
            loop {
                tokio::select! {
                    incoming = server.accept() => {
                        let Some(incoming) = incoming else { break };
                        let conn = incoming.accept().unwrap().await.unwrap();
                        {
                            let mut seen = traffic.lock().unwrap();
                            seen.handshakes += 1;
                            seen.extra = Some(conn.clone());
                        }
                        connections.spawn(serve_files(conn, true, traffic.clone(), bytes.clone(), ra.clone(), rb.clone()));
                    }
                    child = connections.join_next(), if !connections.is_empty() => { child.unwrap().unwrap(); }
                }
            }
            connections.shutdown().await;
        }
    })));
    (f, traffic, release_a, release_b)
}

async fn observe_two_files(cancel_a: bool) {
    let (f, traffic, release_a, release_b) = controlled().await;
    let owner = f.node.pull_peer(&f.ep, &peer(&f)).unwrap();
    let la = crate::files::landing(A, f.root.path()).unwrap();
    let lb = crate::files::landing(B, f.root.path()).unwrap();
    let pa = la.part.clone();
    let pb = lb.part.clone();
    let mut a = Box::pin(
        f.node
            .pull_into_with_layout(&f.ep, "peer", A, la, None, None, false, true),
    );
    let mut b = Box::pin(
        f.node
            .pull_into_with_layout(&f.ep, "peer", B, lb, None, None, false, true),
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let usage = owner.usage();
            let both_prefixes = [&pa, &pb]
                .into_iter()
                .all(|p| std::fs::read(p).is_ok_and(|v| v == f.bytes[..proto::SLICE as usize]));
            let issued = {
                let seen = traffic.lock().unwrap();
                seen.issued.contains_key(&(A.to_owned(), 1))
                    && seen.issued.contains_key(&(B.to_owned(), 1))
                    && seen.issued.len() == 10 + usage.foreground_held
            };
            if both_prefixes
                && issued
                && usage.peer_held == 8
                && usage.global_held == 8
                && usage.foreground_held <= 2
                && usage.peer_ready == 6 + usage.foreground_held
                && usage.global_ready == usage.peer_ready
                && usage.owners >= 2
            {
                break;
            }
            tokio::select! {
                result = &mut a => panic!("fileA finished before its held head: {result:?}"),
                result = &mut b => panic!("fileB finished before its held head: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(2)) => {}
            }
        }
    })
    .await
    .expect("two real files never reached decoded-ready bounded state");
    let before = owner.usage();
    let issued_before = traffic.lock().unwrap().issued.len();
    {
        let seen = traffic.lock().unwrap();
        assert_eq!(seen.issued.len(), 10 + before.foreground_held);
        assert_eq!(seen.duplicates, 0);
        assert_eq!(seen.handshakes, 1);
    }
    let remote = traffic.lock().unwrap().extra.clone().unwrap();
    let complete = |result: Result<(u64, std::path::PathBuf), String>, part: &std::path::Path| {
        result.is_ok_and(|(moved, dest)| {
            moved == f.bytes.len() as u64
                && std::fs::read(dest).is_ok_and(|v| v == *f.bytes)
                && !part.exists()
                && !PullNote::path_of(part).exists()
        })
    };
    let completed_a;
    let completed_b;
    if cancel_a {
        drop(a);
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                let usage = owner.usage();
                let progress = {
                    let seen = traffic.lock().unwrap();
                    seen.stopped.contains(A)
                        && seen
                            .issued
                            .iter()
                            .any(|((path, index), extra)| path == B && *index > 1 && *extra)
                };
                if progress && usage.peer_held + usage.foreground_held == 8 && usage.peer_ready == 7
                {
                    break;
                }
                tokio::select! {
                    result = &mut b => panic!("fileB ended before release: {result:?}"),
                    _ = tokio::time::sleep(Duration::from_millis(2)) => {}
                }
            }
        })
        .await
        .expect("cancelled file retained credits or closed the shared extra");
        assert!(remote.close_reason().is_none());
        release_b.notify_one();
        completed_a = false;
        completed_b = complete(
            tokio::time::timeout(Duration::from_secs(20), &mut b)
                .await
                .unwrap(),
            &pb,
        );
    } else {
        release_a.notify_one();
        release_b.notify_one();
        let (ra, rb) = tokio::time::timeout(Duration::from_secs(20), async {
            tokio::join!(&mut a, &mut b)
        })
        .await
        .unwrap();
        completed_a = complete(ra, &pa);
        completed_b = complete(rb, &pb);
        drop(a);
    }
    drop(b);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    let usage = owner.usage();
    let credits_returned = usage.peer_held == 0
        && usage.global_held == 0
        && usage.peer_ready == 0
        && usage.global_ready == 0
        && usage.foreground_held == 0;
    assert_eq!(f.node.wires.pulls.available_extra(), 4);
    let unique_offsets = {
        let seen = traffic.lock().unwrap();
        assert_eq!(seen.handshakes, 1);
        seen.duplicates == 0
            && [A, B]
                .into_iter()
                .filter(|path| !cancel_a || *path == B)
                .all(|path| {
                    seen.issued
                        .keys()
                        .filter(|(p, _)| p == path)
                        .map(|(_, i)| *i)
                        .collect::<BTreeSet<_>>()
                        == (0..13).collect()
                })
    };
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-pool-files] {}",
        serde_json::json!({
            "cancel_a":cancel_a, "both_exact_first_slices":true, "handshakes":1,
            "held_bulk_before_release":before.peer_held, "decoded_ready_before_release":before.peer_ready,
            "foreground_held_before_release":before.foreground_held,
            "issued_before_release":issued_before, "cancelled_head_actual_stop":cancel_a,
            "surviving_file_retained_extra":cancel_a, "surviving_decoded_ready":if cancel_a {7} else {before.peer_ready},
            "completed_a":completed_a, "completed_b":completed_b, "unique_offsets":unique_offsets,
            "all_credits_returned":credits_returned, "last_owner_closed_extra":true,
            "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(completed_b && (cancel_a || completed_a) && unique_offsets && credits_returned);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn two_real_files_share_an_extra_and_both_finish() {
    observe_two_files(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelling_one_real_file_releases_ready_credits_and_preserves_the_other() {
    observe_two_files(true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_late_file_finishes_while_another_files_head_remains_held() {
    let (f, traffic, _release_a, release_b) = controlled().await;
    let owner = f.node.pull_peer(&f.ep, &peer(&f)).unwrap();
    let la = crate::files::landing(A, f.root.path()).unwrap();
    let lb = crate::files::landing(B, f.root.path()).unwrap();
    let pa = la.part.clone();
    let pb = lb.part.clone();
    let mut a = Box::pin(
        f.node
            .pull_into_with_layout(&f.ep, "peer", A, la, None, None, false, true),
    );
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let usage = owner.usage();
            if usage.peer_held == 8 && usage.peer_ready == 7 {
                break;
            }
            tokio::select! {
                result = &mut a => panic!("fileA ended before its held head: {result:?}"),
                _ = tokio::time::sleep(Duration::from_millis(2)) => {}
            }
        }
    })
    .await
    .expect("fileA never reached the controlled decoded-ready state");
    assert_eq!(std::fs::read(pa).unwrap(), f.bytes[..proto::SLICE as usize]);
    release_b.notify_one();
    let mut b = Box::pin(
        f.node
            .pull_into_with_layout(&f.ep, "peer", B, lb, None, None, false, true),
    );
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::select! {
            result = &mut b => result,
            result = &mut a => panic!("fileA head was never released: {result:?}"),
        }
    })
    .await;
    let premature_error = result.as_ref().ok().and_then(|v| v.as_ref().err()).cloned();
    let completed_b = match result {
        Ok(Ok((moved, dest))) => {
            moved == f.bytes.len() as u64
                && std::fs::read(dest).is_ok_and(|v| v == *f.bytes)
                && !pb.exists()
                && !PullNote::path_of(&pb).exists()
        }
        _ => false,
    };
    let first_content_b = completed_b
        || std::fs::read(&pb).is_ok_and(|v| v.starts_with(&f.bytes[..proto::SLICE as usize]));
    let issued_b = traffic
        .lock()
        .unwrap()
        .issued
        .keys()
        .filter(|(path, _)| path == B)
        .map(|(_, i)| *i)
        .collect::<Vec<_>>();
    drop(a);
    drop(b);
    tokio::time::timeout(Duration::from_secs(5), async {
        while !traffic.lock().unwrap().stopped.contains(A) {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .unwrap();
    let remote = traffic.lock().unwrap().extra.clone().unwrap();
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    let usage = owner.usage();
    let credits_returned = usage.peer_held == 0
        && usage.global_held == 0
        && usage.peer_ready == 0
        && usage.global_ready == 0;
    let capacity_returned = f.node.wires.pulls.available_extra() == 4;
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-pool-late-file] {}",
        serde_json::json!({
            "initial_held_bulk":8, "initial_decoded_ready":7, "fileA_head_never_released":true,
            "fileB_first_content":first_content_b, "fileB_completed":completed_b,
            "fileB_issued_offsets":issued_b, "premature_error":premature_error,
            "fileA_actual_stop":true, "all_credits_returned":credits_returned,
            "extra_capacity_returned":capacity_returned, "extra_closed":true,
            "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(
        completed_b && first_content_b && credits_returned && capacity_returned,
        "a held file must not block another complete file on the same peer"
    );
}
