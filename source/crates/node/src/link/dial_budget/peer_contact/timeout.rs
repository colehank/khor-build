//! An actual unanswered handshake finishes after newer authenticated contact.

use super::*;
use std::sync::atomic::AtomicBool;

async fn ask_after_timeout(
    node: &Node,
    ep: &iroh::Endpoint,
    peer: &DeviceInfo,
    early: bool,
) -> bool {
    let conn = if early {
        match node.ask_early(ep, peer, &Request::Sessions, None).await {
            Early::Failed(DialFailure::TimedOut) => return false,
            Early::NotTaken => node.wires.live(&peer.id, wires::Proto::Main).unwrap(),
            _ => panic!("the cold dial must time out or publish its actual connection"),
        }
    } else {
        match node
            .line_to_inner(ep, peer, wires::Proto::Main, None, false)
            .await
        {
            Err(DialFailure::TimedOut) => return false,
            Ok(line) => line.conn,
            _ => panic!("the ordinary dial must time out or establish its connection"),
        }
    };
    echo(&conn).await;
    true
}

async fn both_protocol_endpoint() -> iroh::Endpoint {
    iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .relay_mode(iroh::RelayMode::Disabled)
        .alpns(vec![
            ALPN.to_vec(),
            khor_net::endpoint::TUNNEL_ALPN.to_vec(),
        ])
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .bind()
        .await
        .unwrap()
}

async fn observe_timeout(early: bool, contact: Contact) {
    observe_timeout_on(early, contact, ALPN).await;
}

async fn observe_timeout_on(early: bool, contact: Contact, contact_alpn: &[u8]) {
    let server = both_protocol_endpoint().await;
    let other = both_protocol_endpoint().await;
    let (_root, node, mut peer, initial_ep) = fixture_for(server.id().to_string()).await;
    initial_ep.close().await;
    let ep = both_protocol_endpoint().await;
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
    let node = Arc::new(node);
    let initial_contact_version = node.wires.contact_version(&peer.id);
    let ignore_handshakes = Arc::new(AtomicBool::new(true));
    let (first_ignored, observed_first) = tokio::sync::oneshot::channel();
    let (stop_server, mut stopped) = tokio::sync::oneshot::channel::<()>();
    let cancelled_handshakes = Arc::new(AtomicUsize::new(0));
    let closed_streams = Arc::new(AtomicUsize::new(0));
    let over_capacity = Arc::new(AtomicUsize::new(0));
    let mut serving = Aborts(Some(tokio::spawn({
        let server = server.clone();
        let ignore_handshakes = ignore_handshakes.clone();
        let cancelled_handshakes = cancelled_handshakes.clone();
        let closed_streams = closed_streams.clone();
        let over_capacity = over_capacity.clone();
        async move {
            let mut first_ignored = Some(first_ignored);
            let mut handlers = tokio::task::JoinSet::new();
            let mut panicked = false;
            loop {
                tokio::select! {
                    biased;
                    _ = &mut stopped => break,
                    result = handlers.join_next(), if !handlers.is_empty() => {
                        if result.is_some_and(|r| r.is_err()) {
                            panicked = true;
                            break;
                        }
                    }
                    incoming = server.accept() => {
                        let Some(incoming) = incoming else { break };
                        if ignore_handshakes.load(Ordering::SeqCst) {
                            incoming.ignore();
                            if let Some(signal) = first_ignored.take() {
                                signal.send(()).unwrap();
                            }
                            continue;
                        }
                        if handlers.len() >= 8 {
                            over_capacity.fetch_add(1, Ordering::SeqCst);
                            incoming.ignore();
                            continue;
                        }
                        let cancelled_handshakes = cancelled_handshakes.clone();
                        let closed_streams = closed_streams.clone();
                        handlers.spawn(async move {
                            let Ok(accepting) = incoming.accept() else {
                                cancelled_handshakes.fetch_add(1, Ordering::SeqCst);
                                return;
                            };
                            let Ok(conn) = accepting.await else {
                                cancelled_handshakes.fetch_add(1, Ordering::SeqCst);
                                return;
                            };
                            let Ok((mut send, mut recv)) = conn.accept_bi().await else {
                                closed_streams.fetch_add(1, Ordering::SeqCst);
                                return;
                            };
                            let Ok(bytes) = recv.read_to_end(64).await else {
                                closed_streams.fetch_add(1, Ordering::SeqCst);
                                return;
                            };
                            assert_eq!(bytes, b"peer-contact");
                            if send.write_all(&bytes).await.is_err() || send.finish().is_err() {
                                closed_streams.fetch_add(1, Ordering::SeqCst);
                                return;
                            }
                            conn.closed().await;
                        });
                    }
                }
            }
            handlers.abort_all();
            while let Some(result) = handlers.join_next().await {
                if result.is_err_and(|e| !e.is_cancelled()) {
                    panicked = true;
                }
            }
            assert!(!panicked, "the scratch connection handlers must not panic");
        }
    })));
    let mut owner = Aborts(Some(tokio::spawn({
        let (node, ep, peer) = (node.clone(), ep.clone(), peer.clone());
        async move { ask_after_timeout(&node, &ep, &peer, early).await }
    })));
    tokio::time::timeout(Duration::from_secs(5), observed_first)
        .await
        .unwrap()
        .unwrap();
    let mut queued = Box::pin(ask_after_timeout(&node, &ep, &peer, early));
    std::future::poll_fn(|cx| {
        assert!(queued.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    let contact_wires = match contact {
        Contact::None => None,
        Contact::SamePeer | Contact::OtherPeer => {
            let caller = if matches!(contact, Contact::SamePeer) {
                &server
            } else {
                &other
            };
            let (outbound, inbound) = tokio::time::timeout(Duration::from_secs(5), async {
                tokio::join!(caller.connect(ep.addr(), contact_alpn), async {
                    ep.accept().await.unwrap().accept().unwrap().await
                })
            })
            .await
            .unwrap();
            let (outbound, inbound) = (outbound.unwrap(), inbound.unwrap());
            assert_eq!(inbound.remote_id(), caller.id());
            assert_eq!(inbound.alpn(), contact_alpn);
            node.wires.serving(&inbound);
            tokio::time::timeout(Duration::from_secs(5), async {
                tokio::join!(echo(&outbound), answer_echo(&inbound));
            })
            .await
            .unwrap();
            Some((outbound, inbound))
        }
    };
    let version_after_contact = node.wires.contact_version(&peer.id);
    // The product's original10s dial limit runs to completion, without a
    // caller deadline or altered production timer fabricating the failure.
    let owner_answered = tokio::time::timeout(Duration::from_secs(15), owner.0.as_mut().unwrap())
        .await
        .unwrap()
        .unwrap();
    let down_after_old_timeout = node.wires.link(&peer.id).down;
    let backoff_after_old_timeout = node.wires.backing_off(&peer.id);
    let version_after_timeout = node.wires.contact_version(&peer.id);
    ignore_handshakes.store(false, Ordering::SeqCst);
    let queued_answered = tokio::time::timeout(Duration::from_secs(5), &mut queued).await;
    drop(queued);
    if let Some((outbound, inbound)) = contact_wires {
        outbound.close(0u32.into(), b"fixture finished");
        inbound.close(0u32.into(), b"fixture finished");
    }
    let _ = stop_server.send(());
    let server_stopped = serving.0.as_mut().unwrap().await;
    tokio::join!(ep.close(), server.close(), other.close());

    match server_stopped {
        Ok(()) => {}
        Err(error) => assert!(error.is_cancelled(), "the scratch server must not panic"),
    }
    eprintln!(
        "[dial-timeout-contact] {}",
        serde_json::json!({
            "schema": 1,
            "early": early,
            "contact_is_main": contact_alpn == ALPN,
            "contact": match contact {
                Contact::SamePeer => "same_peer",
                Contact::None => "none",
                Contact::OtherPeer => "other_peer",
            },
            "initial_contact_version": initial_contact_version,
            "version_after_contact": version_after_contact,
            "version_after_timeout": version_after_timeout,
            "owner_answered": owner_answered,
            "queued_answered": queued_answered.as_ref().ok().copied(),
            "down_after_old_timeout": down_after_old_timeout.is_some(),
            "backoff_after_old_timeout": backoff_after_old_timeout.is_some(),
            "endpoints_explicitly_closed": true,
            "fixture_revision": "cancellation_safe_concurrent_v2",
            "cancelled_handshakes": cancelled_handshakes.load(Ordering::SeqCst),
            "closed_streams": closed_streams.load(Ordering::SeqCst),
            "over_capacity": over_capacity.load(Ordering::SeqCst),
            "all_connection_handlers_joined": true,
        })
    );
    assert!(
        !owner_answered,
        "the old caller must keep its actual timeout"
    );
    let queued_answered = queued_answered.expect("the queued caller must finish within5s");
    if matches!(contact, Contact::SamePeer) && contact_alpn == ALPN {
        assert_eq!(
            down_after_old_timeout, None,
            "an old timeout cannot erase newer authenticated contact"
        );
        assert_eq!(backoff_after_old_timeout, None);
        assert!(
            queued_answered,
            "the waiter must not reuse the stale timeout"
        );
    } else {
        assert!(down_after_old_timeout.is_some());
        assert!(backoff_after_old_timeout.is_some());
        assert!(
            !queued_answered,
            "unproven peers still share the failed dial"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_timeout_cannot_overwrite_fresh_peer_contact() {
    observe_timeout(false, Contact::SamePeer).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_early_timeout_cannot_overwrite_fresh_peer_contact() {
    observe_timeout(true, Contact::SamePeer).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_timeout_without_fresh_contact_is_shared() {
    observe_timeout(false, Contact::None).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_early_timeout_without_fresh_contact_is_shared() {
    observe_timeout(true, Contact::None).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_timeout_is_not_invalidated_by_another_peer() {
    observe_timeout(false, Contact::OtherPeer).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_early_timeout_is_not_invalidated_by_another_peer() {
    observe_timeout(true, Contact::OtherPeer).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ordinary_main_timeout_is_not_invalidated_by_a_new_tunnel() {
    observe_timeout_on(false, Contact::SamePeer, khor_net::endpoint::TUNNEL_ALPN).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cold_early_main_timeout_is_not_invalidated_by_a_new_tunnel() {
    observe_timeout_on(true, Contact::SamePeer, khor_net::endpoint::TUNNEL_ALPN).await;
}
