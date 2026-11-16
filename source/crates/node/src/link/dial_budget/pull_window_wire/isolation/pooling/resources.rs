//! Actual preparation waiters and capacity retained by closed connections.
use super::*;

async fn owner_ends(expired: bool) {
    let f = Fixture::with_identity(Held::All, true).await;
    let target = peer(&f);
    let owner = f.node.pull_peer(&f.ep, &target).unwrap();
    let mut first = Some(Box::pin(crate::deadline::scope(
        tokio::time::Instant::now() + Duration::from_millis(500),
        f.node.pooled_transfer(&f.ep, &target, &owner),
    )));
    let incoming = tokio::select! {
        result = first.as_mut().unwrap().as_mut() => panic!("owner ended before controlled Incoming; success={}", result.is_ok()),
        incoming = tokio::time::timeout(Duration::from_secs(5), f.server.accept()) => incoming.unwrap().unwrap(),
    };
    let mut waiter = Box::pin(f.node.pooled_transfer(&f.ep, &target, &owner));
    // Poll the independent waiter while the first owner still holds the gate.
    tokio::select! {
        result = &mut waiter => panic!("waiter bypassed the preparation owner; success={}", result.is_ok()),
        _ = tokio::time::sleep(Duration::from_millis(30)) => {}
    }
    let owner_terminal = if expired {
        tokio::time::timeout(Duration::from_secs(2), async {
            tokio::select! {
                result = first.as_mut().unwrap().as_mut() => matches!(result, Err(PreparationFailure::Stop(_))),
                result = &mut waiter => panic!("waiter ended while owner was still held; success={}", result.is_ok()),
            }
        }).await.unwrap()
    } else {
        true
    };
    drop(first.take());
    drop(first);
    drop(incoming);
    let caller_deadline_leaked = crate::deadline::current().is_some();
    let mut accepted = None;
    let mut handshake_error = None;
    let observed = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                result = &mut waiter => break result,
                incoming = f.server.accept(), if accepted.is_none() => {
                    match incoming.unwrap().accept() {
                        Ok(connecting) => match connecting.await {
                            Ok(remote) => accepted = Some(remote),
                            Err(error) => handshake_error = Some(format!("{error:?}")),
                        },
                        Err(error) => handshake_error = Some(format!("{error:?}")),
                    }
                }
            }
        }
    })
    .await;
    let connected = matches!(&observed, Ok(Ok(_)));
    let failure = match &observed {
        Ok(Err(error)) => Some(format!("{error:?}")),
        Err(_) => Some("observer timeout".into()),
        _ => None,
    };
    let lease = observed.ok().and_then(Result::ok);
    let distinct = lease
        .as_ref()
        .is_some_and(|l| l.line.0.conn.stable_id() != f.local.stable_id());
    drop(waiter);
    drop(lease);
    let accepted_incoming = accepted.is_some();
    let extra_closed = if let Some(remote) = accepted {
        tokio::time::timeout(Duration::from_secs(5), remote.closed())
            .await
            .is_ok()
    } else {
        false
    };
    let capacity_returned = f.node.wires.pulls.available_extra() == 4;
    cached_answer(&f).await;
    f.close().await;
    eprintln!(
        "[pull-pool-resource] {}",
        serde_json::json!({
            "event":"preparation_waiter", "expired_owner":expired,
            "actual_unaccepted_incoming":true, "waiter_polled_pending":true,
            "owner_terminal_or_cancelled":owner_terminal, "waiter_connected":connected,
            "accepted_incoming":accepted_incoming, "failure":failure,
            "handshake_error":handshake_error, "caller_deadline_leaked":caller_deadline_leaked,
            "distinct_from_cached":distinct, "capacity_returned":capacity_returned,
            "extra_closed":extra_closed, "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(
        owner_terminal
            && connected
            && accepted_incoming
            && distinct
            && capacity_returned
            && extra_closed
            && !caller_deadline_leaked,
        "an ended preparation owner must not poison its independent waiter"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn expired_preparation_owner_does_not_poison_a_live_waiter() {
    owner_ends(true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cancelled_preparation_owner_does_not_poison_a_live_waiter() {
    owner_ends(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_by_value_endpoint_clone_finishes_on_cached_main_without_extra() {
    let f = Fixture::with_identity(Held::Head, true).await;
    let target = peer(&f);
    let copied = f.ep.as_ref().clone();
    assert_eq!(copied.id(), f.ep.id());
    assert!(f.node.pull_peer(&copied, &target).is_none());
    {
        let loaded = f.node.devices_loaded().unwrap();
        loaded
            .doc
            .upsert(&target.id, "peer", &target.addrs)
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
    }
    f.release.notify_one();
    let landing = crate::files::landing(PATH, f.root.path()).unwrap();
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        f.node
            .pull_into_with_layout(&copied, "peer", PATH, landing, None, None, false, true),
    )
    .await
    .unwrap();
    f.finish(result).await;
    let no_extra = tokio::time::timeout(Duration::from_millis(500), f.server.accept())
        .await
        .is_err();
    cached_answer(&f).await;
    let (calibration, remote) = pooled_extra(&f, &f.ep, &target, &f.server).await;
    drop(calibration);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    drop(copied);
    f.close().await;
    eprintln!(
        "[pull-pool-resource] {}",
        serde_json::json!({
            "event":"by_value_endpoint", "same_key":true, "cached_file_exact":true,
            "no_extra_handshake":no_extra, "positive_handshake_calibration":true,
            "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(no_extra);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_remotely_closed_extra_frees_capacity_while_old_owners_remain() {
    let f = Fixture::with_identity(Held::All, true).await;
    let mut servers = Vec::new();
    for _ in 0..4 {
        servers.push(endpoint(None).await);
    }
    let mut targets = vec![peer(&f)];
    for (index, server) in servers.iter().enumerate() {
        let loaded = f.node.devices_loaded().unwrap();
        let addrs = server
            .addr()
            .ip_addrs()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        loaded
            .doc
            .upsert(
                &server.id().to_string(),
                &format!("resource-{index}"),
                &addrs,
            )
            .unwrap();
        let mut store = loaded.store;
        store.flush(&loaded.doc).unwrap();
        targets.push(
            f.node
                .devices()
                .unwrap()
                .into_iter()
                .find(|p| p.id == server.id().to_string())
                .unwrap(),
        );
    }
    let mut leases = Vec::new();
    let mut remotes = Vec::new();
    for (index, target) in targets[..4].iter().enumerate() {
        let server = if index == 0 {
            &f.server
        } else {
            servers[index - 1].as_ref()
        };
        let (lease, remote) = pooled_extra(&f, &f.ep, target, server).await;
        leases.push(lease);
        remotes.push(remote);
    }
    assert_eq!(f.node.wires.pulls.available_extra(), 0);
    remotes[0].close(17u32.into(), b"controlled remote file connection close");
    tokio::time::timeout(Duration::from_secs(5), leases[0].line.0.conn.closed())
        .await
        .unwrap();
    let other_three_healthy = remotes[1..].iter().all(|c| c.close_reason().is_none());
    let owner = f.node.pull_peer(&f.ep, &targets[4]).unwrap();
    let mut attempt = Box::pin(f.node.pooled_transfer(&f.ep, &targets[4], &owner));
    let mut replacement_remote = None;
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                result = &mut attempt => break result,
                incoming = servers[3].accept(), if replacement_remote.is_none() => {
                    replacement_remote = Some(incoming.unwrap().accept().unwrap().await.unwrap());
                }
            }
        }
    })
    .await;
    let replacement_connected = matches!(&result, Ok(Ok(_)));
    let failure = match &result {
        Ok(Err(error)) => Some(format!("{error:?}")),
        Err(_) => Some("observer timeout".to_owned()),
        _ => None,
    };
    let incoming_observed = replacement_remote.is_some();
    let replacement = result.ok().and_then(Result::ok);
    let old_arc_retained_and_closed = leases[0].line.0.conn.close_reason().is_some();
    drop(attempt);
    // Only after observing the required replacement may any old owner go.
    drop(leases);
    drop(replacement);
    for remote in remotes {
        tokio::time::timeout(Duration::from_secs(5), remote.closed())
            .await
            .unwrap();
    }
    if let Some(remote) = replacement_remote {
        tokio::time::timeout(Duration::from_secs(5), remote.closed())
            .await
            .unwrap();
    }
    let capacity_returned = f.node.wires.pulls.available_extra() == 4;
    let (calibration, remote) = pooled_extra(&f, &f.ep, &targets[4], &servers[3]).await;
    drop(calibration);
    tokio::time::timeout(Duration::from_secs(5), remote.closed())
        .await
        .unwrap();
    cached_answer(&f).await;
    for server in servers {
        server.close().await;
    }
    f.close().await;
    eprintln!(
        "[pull-pool-resource] {}",
        serde_json::json!({
            "event":"remote_close_capacity", "initial_actual_extras":4,
            "old_arc_retained_and_closed":old_arc_retained_and_closed,
            "other_three_healthy":other_three_healthy,
            "replacement_connected_before_drop":replacement_connected,
            "replacement_incoming_observed":incoming_observed, "failure":failure,
            "capacity_returned":capacity_returned, "positive_peer5_calibration":true,
            "cached_main_control":true, "fixture_closed":true,
        })
    );
    assert!(
        replacement_connected
            && incoming_observed
            && old_arc_retained_and_closed
            && other_three_healthy
            && capacity_returned,
        "a remotely closed extra must free capacity before its old owner is dropped"
    );
}
