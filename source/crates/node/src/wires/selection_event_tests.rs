//! Real transport selections reach the current cached wire without a polling delay.
use super::*;
use iroh::{Endpoint, RelayMode, endpoint::presets};

const ALPN: &[u8] = b"khor/selection-event-test";

async fn endpoints() -> (Endpoint, Endpoint) {
    async fn bind() -> Endpoint {
        Endpoint::builder(presets::Minimal)
            .relay_mode(RelayMode::Disabled)
            .clear_ip_transports()
            .bind_addr("127.0.0.1:0")
            .unwrap()
            .alpns(vec![ALPN.to_vec()])
            .bind()
            .await
            .unwrap()
    }
    (bind().await, bind().await)
}

async fn connect(a: &Endpoint, b: &Endpoint) -> (Connection, Connection) {
    tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(async { a.connect(b.addr(), ALPN).await.unwrap() }, async {
            b.accept().await.unwrap().await.unwrap()
        },)
    })
    .await
    .expect("two actual authenticated QUIC endpoints")
}

fn previous_word(wires: &Wires, id: &str, proto: Proto) {
    let mut slots = wires.slots.lock().unwrap();
    let slot = slots.get_mut(&(id.to_owned(), proto)).unwrap();
    slot.word = Some(PathKind::Relay);
    slot.last_word = Some(PathKind::Relay);
    slot.word_at = Some(Instant::now() - Duration::from_secs(1));
    slot.leaning = None;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_transport_snapshot_updates_the_live_word_without_ticks_or_journaling() {
    let (a, b) = endpoints().await;
    let (client, _server) = connect(&a, &b).await;
    let id = b.id().to_string();
    assert_eq!(road_now(&client), PathKind::Direct);
    let wires = Arc::new(Wires::default());
    wires.adopt(&id, Proto::Main, client.clone());
    previous_word(&wires, &id, Proto::Main);
    assert_eq!(wires.link(&id).road, Some(PathKind::Relay));

    let carry = khor_net::carry::Links::new(a.id());
    crate::roads::follow_inbound(&id, &client, None, &carry, &wires);
    tokio::time::timeout(Duration::from_secs(1), async {
        while wires.link(&id).road != Some(PathKind::Direct) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the existing event consumer publishes the actual selection without Wires::tick");

    let weak = Arc::downgrade(&wires);
    drop(wires);
    tokio::time::timeout(Duration::from_secs(1), async {
        while weak.upgrade().is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("the path observer must not keep the wire owner alive");
    a.close().await;
    b.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn only_the_live_connection_updates_its_protocol_and_keeps_the_same_word_age() {
    let (a, b) = endpoints().await;
    let (old, _old_server) = connect(&a, &b).await;
    let (current, _current_server) = connect(&a, &b).await;
    let (tunnel, _tunnel_server) = connect(&a, &b).await;
    assert_ne!(old.stable_id(), current.stable_id());
    assert_ne!(tunnel.stable_id(), current.stable_id());
    let id = b.id().to_string();
    let wires = Wires::default();
    wires.adopt(&id, Proto::Main, old.clone());
    wires.adopt(&id, Proto::Main, current.clone());
    wires.adopt(&id, Proto::Tunnel, tunnel.clone());
    previous_word(&wires, &id, Proto::Main);
    previous_word(&wires, &id, Proto::Tunnel);

    assert!(!wires.selected("another-peer", current.stable_id()));
    assert!(!wires.selected(&id, old.stable_id()));
    assert_eq!(wires.link(&id).road, Some(PathKind::Relay));
    assert!(wires.selected(&id, current.stable_id()));
    assert_eq!(wires.link(&id).road, Some(PathKind::Direct));
    let at = {
        let slots = wires.slots.lock().unwrap();
        assert_eq!(
            slots[&(id.clone(), Proto::Tunnel)].word,
            Some(PathKind::Relay)
        );
        slots[&(id.clone(), Proto::Main)].word_at
    };
    assert!(!wires.selected(&id, current.stable_id()));
    assert_eq!(
        wires.slots.lock().unwrap()[&(id.clone(), Proto::Main)].word_at,
        at
    );
    assert!(wires.selected(&id, tunnel.stable_id()));
    current.close(0u32.into(), b"test closed connection");
    previous_word(&wires, &id, Proto::Main);
    assert!(!wires.selected(&id, current.stable_id()));
    assert_eq!(
        wires.slots.lock().unwrap()[&(id, Proto::Main)].word,
        Some(PathKind::Relay)
    );
    a.close().await;
    b.close().await;
}
