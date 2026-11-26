use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_pending_write_recovers_when_the_live_middles_onward_route_returns() {
    let mut f = Publication::new().await;
    for task in [&mut f.middle_task, &mut f.echo_task] {
        let handle = task.0.take().unwrap();
        handle.abort();
        assert!(handle.await.unwrap_err().is_cancelled());
    }

    let asks = Arc::new(AtomicUsize::new(0));
    let (first_rejected, rejected) = tokio::sync::oneshot::channel();
    let (restore, restored) = tokio::sync::oneshot::channel();
    f.middle_task = Owned(Some(tokio::spawn({
        let conn = f.outer_remote.clone();
        let links = f.target_links.clone();
        let (me, via, to) = (f.ep.id(), f.middle_ep.id(), f.target_ep.id());
        let asks = asks.clone();
        async move {
            let expected = Ask::To(to).words();
            let (mut send, mut recv) = conn.accept_bi().await.unwrap();
            assert_eq!(
                crate::tunnel::read_target(&mut recv).await.unwrap(),
                expected
            );
            asks.fetch_add(1, Ordering::SeqCst);
            send.write_all(&[crate::tunnel::NO_ROUTE]).await.unwrap();
            send.finish().unwrap();
            first_rejected.send(()).unwrap();

            restored.await.unwrap();
            let (mut send, mut recv) = conn.accept_bi().await.unwrap();
            assert_eq!(
                crate::tunnel::read_target(&mut recv).await.unwrap(),
                expected
            );
            asks.fetch_add(1, Ordering::SeqCst);
            send.write_all(&[crate::tunnel::OK]).await.unwrap();
            links.attach(me, Carrier::Hop { via }, recv, send);
        }
    })));

    let writes = Arc::new(AtomicUsize::new(0));
    f.echo_task = Owned(Some(tokio::spawn({
        let ep = f.target_ep.clone();
        let writes = writes.clone();
        async move {
            let conn = ep.accept().await.unwrap().accept().unwrap().await.unwrap();
            while let Ok((mut send, mut recv)) = conn.accept_bi().await {
                let bytes = recv.read_to_end(crate::proto::MAX_FRAME).await.unwrap();
                assert!(matches!(
                    crate::proto::question(&bytes).unwrap(),
                    crate::proto::Request::Mkdir { path } if path == "/test-once"
                ));
                writes.fetch_add(1, Ordering::SeqCst);
                let answer = crate::proto::encode(&crate::proto::Response::Made).unwrap();
                send.write_all(&answer).await.unwrap();
                send.finish().unwrap();
            }
        }
    })));

    f.node.wires.network_changed();
    let started = tokio::time::Instant::now();
    let deadline = started + Duration::from_secs(5);
    let mut caller = Owned(Some(tokio::spawn({
        let (node, ep, target) = (f.node.clone(), f.ep.clone(), f.target.clone());
        async move {
            let write = crate::proto::Request::Mkdir {
                path: "/test-once".into(),
            };
            crate::journal::BY
                .scope(
                    crate::journal::By::Ask,
                    crate::deadline::scope(deadline, node.ask_on(&ep, &target, &write)),
                )
                .await
        }
    })));
    tokio::time::timeout_at(deadline, rejected)
        .await
        .expect("the first hop is actually tried")
        .unwrap();
    assert_eq!(writes.load(Ordering::SeqCst), 0);
    assert!(f.outer_local.close_reason().is_none());
    restore.send(()).unwrap();

    let result = tokio::time::timeout_at(
        deadline + Duration::from_millis(500),
        caller.0.as_mut().unwrap(),
    )
    .await;
    let response = match result {
        Ok(Ok(response)) => response,
        other => panic!("pending call did not finish: {other:?}"),
    };
    assert!(
        matches!(response, Ok(crate::proto::Response::Made)),
        "same pending call should recover through the already connected middle: {response:?}; hop asks={}",
        asks.load(Ordering::SeqCst)
    );
    assert_eq!(asks.load(Ordering::SeqCst), 2);
    assert_eq!(writes.load(Ordering::SeqCst), 1);
    assert!(f.outer_local.close_reason().is_none());
    assert!(started.elapsed() < Duration::from_secs(5));
    eprintln!("same pending write returned in {:?}", started.elapsed());
}
