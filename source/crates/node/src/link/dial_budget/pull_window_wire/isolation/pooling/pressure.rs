//! One actual Node and five authenticated loopback peers, not a WAN benchmark.
use super::*;
use std::collections::BTreeMap;

const SLOW: &str = "/pressure-block.bin";

#[derive(Default)]
struct Traffic {
    issued: BTreeMap<(String, u64), usize>,
    ended: BTreeMap<String, &'static str>,
    holds: BTreeMap<String, Arc<Notify>>,
    extras: Vec<iroh::endpoint::Connection>,
    aborted_handshakes: usize,
}

async fn serve(
    conn: iroh::endpoint::Connection,
    traffic: Arc<Mutex<Traffic>>,
    bytes: Arc<Vec<u8>>,
) {
    let mut streams = tokio::task::JoinSet::new();
    loop {
        tokio::select! {
            stream = conn.accept_bi() => {
                let Ok((mut send, mut recv)) = stream else { break };
                let (traffic, bytes) = (traffic.clone(), bytes.clone());
                streams.spawn(async move {
                    let Ok(raw) = recv.read_to_end(MAX_FRAME).await else { return };
                    match proto::question(&raw).unwrap() {
                        Request::Sessions => {
                            send.write_all(&proto::encode(&Response::SessionRows { rows: vec![] }).unwrap()).await.unwrap();
                            send.finish().unwrap();
                        }
                        Request::FetchPath { path, offset } => {
                            assert_eq!(offset % proto::SLICE, 0);
                            let index = offset / proto::SLICE;
                            assert!(index <= 12);
                            let hold = {
                                let mut seen = traffic.lock().unwrap();
                                *seen.issued.entry((path.clone(), index)).or_default() += 1;
                                seen.holds.get(&path).cloned()
                            };
                            if index == 1 && let Some(hold) = hold {
                                tokio::select! {
                                    _ = hold.notified() => {}
                                    ended = send.stopped() => {
                                        let kind = match ended {
                                            Ok(Some(_)) => "stream_stop",
                                            Err(_) => "connection_closed",
                                            Ok(None) => "unexpected_finish",
                                        };
                                        traffic.lock().unwrap().ended.insert(path, kind);
                                        return;
                                    }
                                }
                            }
                            let start = offset as usize;
                            let end = (start + proto::SLICE as usize).min(bytes.len());
                            let reply = Response::PathSlice { total:bytes.len() as u64, at_ms:123, bytes:bytes[start..end].to_vec().into() };
                            if send.write_all(&proto::encode(&reply).unwrap()).await.is_ok() {
                                let _ = send.finish();
                                let _ = send.stopped().await;
                            }
                        }
                        other => panic!("unexpected pressure request: {other:?}"),
                    }
                });
            }
            done = streams.join_next(), if !streams.is_empty() => { done.unwrap().unwrap(); }
        }
    }
    streams.shutdown().await;
}

async fn server(
    ep: Arc<iroh::Endpoint>,
    cached: iroh::endpoint::Connection,
    traffic: Arc<Mutex<Traffic>>,
    bytes: Arc<Vec<u8>>,
) {
    let mut connections = tokio::task::JoinSet::new();
    connections.spawn(serve(cached, traffic.clone(), bytes.clone()));
    loop {
        tokio::select! {
            incoming = ep.accept() => {
                let Some(incoming) = incoming else { break };
                let connected = match incoming.accept() {
                    Ok(connecting) => connecting.await.ok(),
                    Err(_) => None,
                };
                if let Some(conn) = connected {
                    traffic.lock().unwrap().extras.push(conn.clone());
                    connections.spawn(serve(conn, traffic.clone(), bytes.clone()));
                } else { traffic.lock().unwrap().aborted_handshakes += 1; }
            }
            done = connections.join_next(), if !connections.is_empty() => { done.unwrap().unwrap(); }
        }
    }
    connections.shutdown().await;
}

type ResultFile = Result<(u64, std::path::PathBuf), String>;
type Done = (usize, String, ResultFile);

async fn pull(
    node: Arc<Node>,
    ep: Arc<iroh::Endpoint>,
    name: String,
    path: String,
    landing: crate::files::Landing,
) -> ResultFile {
    node.pull_into_with_layout(&ep, &name, &path, landing, None, None, false, true)
        .await
}

fn launch(
    jobs: &mut tokio::task::JoinSet<Done>,
    f: &Fixture,
    target: &DeviceInfo,
    index: usize,
    path: &str,
    dir: &std::path::Path,
) -> std::path::PathBuf {
    let landing = crate::files::landing(path, dir).unwrap();
    let part = landing.part.clone();
    let (node, ep, name, path) = (
        f.node.clone(),
        f.ep.clone(),
        target.name.clone(),
        path.to_owned(),
    );
    jobs.spawn(async move {
        let out = pull(node, ep, name, path.clone(), landing).await;
        (index, path, out)
    });
    part
}

async fn until(mut ready: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !ready() {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    })
    .await
    .expect("actual pressure precondition did not arrive");
}

fn finished(result: ResultFile, part: &std::path::Path, bytes: &[u8]) -> bool {
    result.is_ok_and(|(moved, dest)| {
        moved == bytes.len() as u64
            && std::fs::read(dest).is_ok_and(|v| v == bytes)
            && !part.exists()
            && !PullNote::path_of(part).exists()
    })
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn five_peers_keep_progress_and_cancellation_when_bulk_and_foreground_are_full() {
    let mut f = Fixture::with_identity(Held::All, true).await;
    let old = f.serving.0.take().unwrap();
    old.abort();
    let _ = old.await;
    let mut servers = vec![Arc::new(f.server.clone())];
    let mut locals = vec![f.local.clone()];
    let mut remotes = vec![f.remote.clone()];
    for _ in 0..4 {
        let ep = endpoint(None).await;
        let (local, remote) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(f.ep.connect(ep.addr(), ALPN), async {
                ep.accept().await.unwrap().accept().unwrap().await
            })
        })
        .await
        .unwrap();
        let (local, remote) = (local.unwrap(), remote.unwrap());
        f.node
            .wires
            .adopt(&ep.id().to_string(), wires::Proto::Main, local.clone());
        servers.push(ep);
        locals.push(local);
        remotes.push(remote);
    }
    let mut targets = Vec::new();
    let mut dirs = Vec::new();
    for (i, ep) in servers.iter().enumerate() {
        let loaded = f.node.devices_loaded().unwrap();
        let addrs = ep
            .addr()
            .ip_addrs()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        loaded
            .doc
            .upsert(&ep.id().to_string(), &format!("pressure-{i}"), &addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
        targets.push(
            f.node
                .devices()
                .unwrap()
                .into_iter()
                .find(|p| p.id == ep.id().to_string())
                .unwrap(),
        );
        let dir = f.root.path().join(format!("peer-{i}"));
        std::fs::create_dir_all(&dir).unwrap();
        dirs.push(dir);
    }
    let owners = targets
        .iter()
        .map(|d| f.node.pull_peer(&f.ep, d).unwrap())
        .collect::<Vec<_>>();
    let traffic = (0..5)
        .map(|_| Arc::new(Mutex::new(Traffic::default())))
        .collect::<Vec<_>>();
    for state in &traffic[..4] {
        state
            .lock()
            .unwrap()
            .holds
            .insert(SLOW.into(), Arc::new(Notify::new()));
    }
    let busy = (0..8)
        .map(|i| format!("/pressure-busy-{i}.bin"))
        .collect::<Vec<_>>();
    for path in &busy {
        traffic[4]
            .lock()
            .unwrap()
            .holds
            .insert(path.clone(), Arc::new(Notify::new()));
    }
    let mut controllers = Vec::new();
    for i in 0..5 {
        let task = tokio::spawn(server(
            servers[i].clone(),
            remotes[i].clone(),
            traffic[i].clone(),
            f.bytes.clone(),
        ));
        if i == 0 {
            f.serving.0 = Some(task);
        } else {
            controllers.push(Aborts(Some(task)));
        }
    }
    let mut jobs = tokio::task::JoinSet::<Done>::new();
    let mut parts = Vec::new();
    for i in 0..4 {
        parts.push(launch(&mut jobs, &f, &targets[i], i, SLOW, &dirs[i]));
    }
    until(|| {
        owners[..4].iter().all(|p| {
            let u = p.usage();
            u.peer_held == 8 && u.peer_ready == 7 && u.global_held == 32 && u.global_ready == 28
        })
    })
    .await;
    assert_eq!(f.node.wires.pulls.available_extra(), 0);
    for i in 0..4 {
        assert_eq!(traffic[i].lock().unwrap().extras.len(), 1);
        assert_eq!(
            std::fs::read(&parts[i]).unwrap(),
            f.bytes[..proto::SLICE as usize]
        );
    }
    let mut busy_parts = Vec::new();
    for (i, path) in busy.iter().enumerate() {
        let part = launch(&mut jobs, &f, &targets[4], 4, path, &dirs[4]);
        until(|| {
            let u = owners[4].usage();
            u.global_held == 32
                && u.global_ready == 28
                && u.foreground_held == i + 1
                && std::fs::read(&part).is_ok_and(|v| v == f.bytes[..proto::SLICE as usize])
                && traffic[4]
                    .lock()
                    .unwrap()
                    .issued
                    .contains_key(&(path.clone(), 1))
        })
        .await;
        busy_parts.push(part);
    }
    assert!(jobs.try_join_next().is_none());
    assert_eq!(traffic[4].lock().unwrap().extras.len(), 0);
    let foreground_full = owners[4].usage().foreground_held == 8;
    let responsive = tokio::time::timeout(
        Duration::from_secs(5),
        f.node.ask_on(&f.ep, &targets[4], &Request::Sessions),
    )
    .await
    .unwrap();
    assert!(matches!(responsive, Ok(Response::SessionRows { .. })));

    let expired_path = "/pressure-expired.bin";
    let landing = crate::files::landing(expired_path, &dirs[4]).unwrap();
    let expired_part = landing.part.clone();
    let expired = crate::deadline::scope(
        tokio::time::Instant::now() + Duration::from_millis(100),
        pull(
            f.node.clone(),
            f.ep.clone(),
            targets[4].name.clone(),
            expired_path.into(),
            landing,
        ),
    );
    let expired = tokio::time::timeout(Duration::from_secs(2), expired)
        .await
        .unwrap();
    let expired_without_rpc = expired.is_err()
        && !traffic[4]
            .lock()
            .unwrap()
            .issued
            .keys()
            .any(|(p, _)| p == expired_path)
        && !expired_part.exists()
        && !PullNote::path_of(&expired_part).exists();
    assert!(expired_without_rpc);

    let cancelled_path = "/pressure-cancelled.bin";
    let landing = crate::files::landing(cancelled_path, &dirs[4]).unwrap();
    let mut cancelled = Box::pin(pull(
        f.node.clone(),
        f.ep.clone(),
        targets[4].name.clone(),
        cancelled_path.into(),
        landing,
    ));
    tokio::select! {
        result = &mut cancelled => panic!("queued caller escaped full foreground: {result:?}"),
        _ = tokio::time::sleep(Duration::from_millis(30)) => {}
    }
    drop(cancelled);
    let cancelled_without_rpc = !traffic[4]
        .lock()
        .unwrap()
        .issued
        .keys()
        .any(|(p, _)| p == cancelled_path);
    let u = owners[4].usage();
    assert!(
        cancelled_without_rpc
            && u.global_held == 32
            && u.global_ready == 28
            && u.foreground_held == 8
    );
    assert!(crate::deadline::current().is_none());

    traffic[4].lock().unwrap().holds[&busy[0]].notify_one();
    let (peer_index, path, result) = tokio::time::timeout(Duration::from_secs(5), jobs.join_next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!((peer_index, path.as_str()), (4, busy[0].as_str()));
    let released_file_complete = finished(result, &busy_parts[0], &f.bytes);
    assert!(released_file_complete);
    let u = owners[4].usage();
    assert_eq!(
        (u.global_held, u.global_ready, u.foreground_held),
        (32, 28, 7)
    );

    let fast_path = "/pressure-fast.bin";
    let landing = crate::files::landing(fast_path, &dirs[4]).unwrap();
    let fast_part = landing.part.clone();
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        pull(
            f.node.clone(),
            f.ep.clone(),
            targets[4].name.clone(),
            fast_path.into(),
            landing,
        ),
    )
    .await
    .unwrap();
    let later_file_complete = finished(result, &fast_part, &f.bytes);
    let u = owners[4].usage();
    assert!(
        later_file_complete
            && u.global_held == 32
            && u.global_ready == 28
            && u.foreground_held == 7
    );
    {
        let seen = traffic[4].lock().unwrap();
        for path in [&busy[0], &fast_path.to_owned()] {
            let offsets = seen
                .issued
                .keys()
                .filter(|(p, _)| p == path)
                .map(|(_, i)| *i)
                .collect::<BTreeSet<_>>();
            assert_eq!(offsets, (0..13).collect());
        }
    }
    jobs.abort_all();
    while let Some(done) = jobs.join_next().await {
        assert!(done.unwrap_err().is_cancelled());
    }
    let extras = traffic
        .iter()
        .flat_map(|t| t.lock().unwrap().extras.clone())
        .collect::<Vec<_>>();
    for conn in extras {
        tokio::time::timeout(Duration::from_secs(5), conn.closed())
            .await
            .unwrap();
    }
    until(|| {
        traffic
            .iter()
            .map(|t| t.lock().unwrap().ended.len())
            .sum::<usize>()
            == 11
    })
    .await;
    let all_credits_returned = owners.iter().all(|p| {
        let u = p.usage();
        u.peer_held == 0
            && u.global_held == 0
            && u.peer_ready == 0
            && u.global_ready == 0
            && u.foreground_held == 0
    });
    assert!(all_credits_returned && f.node.wires.pulls.available_extra() == 4);
    let mut stops = 0;
    let mut closed = 0;
    let mut extra_connections = 0;
    for state in &traffic {
        let seen = state.lock().unwrap();
        assert!(seen.issued.values().all(|n| *n == 1));
        for kind in seen.ended.values() {
            match *kind {
                "stream_stop" => stops += 1,
                "connection_closed" => closed += 1,
                _ => panic!("held response ended without cancellation"),
            }
        }
        extra_connections += seen.extras.len();
        assert!(seen.extras.iter().all(|c| c.close_reason().is_some()));
    }
    assert_eq!(extra_connections, 4);
    assert_eq!(traffic[4].lock().unwrap().extras.len(), 0);
    for i in 0..5 {
        let answer = tokio::time::timeout(
            Duration::from_secs(5),
            f.node.ask_on(&f.ep, &targets[i], &Request::Sessions),
        )
        .await
        .unwrap();
        assert!(matches!(answer, Ok(Response::SessionRows { .. })));
        assert_eq!(
            f.node
                .wires
                .live(&targets[i].id, wires::Proto::Main)
                .unwrap()
                .stable_id(),
            locals[i].stable_id()
        );
        assert!(locals[i].close_reason().is_none());
    }
    for ep in &servers {
        ep.close().await;
    }
    for mut task in controllers {
        task.0.take().unwrap().await.unwrap();
    }
    f.close().await;
    eprintln!(
        "[pull-pool-pressure] {}",
        serde_json::json!({
            "peers":5, "one_process_loopback":true, "bulk_held":32, "bulk_decoded_ready":28,
            "foreground_full":foreground_full, "foreground_held":8, "initial_extras":4,
            "peer5_extra_handshakes":0, "queued_expiry_no_rpc":expired_without_rpc,
            "queued_cancel_no_rpc":cancelled_without_rpc, "released_file_complete":released_file_complete,
            "later_file_complete":later_file_complete, "all_credits_returned":all_credits_returned,
            "all_extra_capacity_returned":true, "held_stream_stops":stops,
            "held_connection_closes":closed, "unique_rpc_offsets":true,
            "cached_main_controls":5, "foreground_full_sessions_control":true, "fixture_closed":true,
        })
    );
}
