//! A prepared handshake has its own initial route beside a live IP wire.
use super::*;
use khor_net::carry::{Carrier, Links, transport_addr};

async fn endpoint(key: iroh::SecretKey, links: &Links) -> iroh::Endpoint {
    iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .secret_key(key)
        .relay_mode(iroh::RelayMode::Disabled)
        .clear_ip_transports()
        .bind_addr("127.0.0.1:0")
        .unwrap()
        .add_custom_transport(links.transport())
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_prepared_handshake_uses_custom_path_zero_beside_a_live_ip_connection() {
    let (client_key, server_key) = (iroh::SecretKey::generate(), iroh::SecretKey::generate());
    let client_links = Links::new(client_key.public());
    let server_links = Links::new(server_key.public());
    let client = endpoint(client_key, &client_links).await;
    let server = endpoint(server_key, &server_links).await;
    let ip_only = iroh::EndpointAddr::from_parts(
        server.id(),
        server.addr().addrs.into_iter().filter(|addr| addr.is_ip()),
    );
    let (warm, warm_remote) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(client.connect(ip_only, ALPN), async {
            server.accept().await.unwrap().accept().unwrap().await
        })
    })
    .await
    .unwrap();
    let (warm, warm_remote) = (warm.unwrap(), warm_remote.unwrap());
    assert!(
        warm.paths()
            .iter()
            .any(|path| { path.id() == iroh::endpoint::PathId::ZERO && path.is_ip() })
    );
    let via = iroh::SecretKey::generate().public();
    let (a, b) = tokio::io::duplex(64 << 10);
    let (ar, aw) = tokio::io::split(a);
    let (br, bw) = tokio::io::split(b);
    let local = client_links.attach(server.id(), Carrier::Hop { via }, ar, aw);
    let remote = server_links.attach(client.id(), Carrier::Hop { via }, br, bw);
    let addr = iroh::EndpointAddr::from_parts(server.id(), [transport_addr(&server.id())]);
    let (carried, accepted) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(connect_prepared_carrier(&client, addr, ALPN), async {
            server.accept().await.unwrap().accept().unwrap().await
        })
    })
    .await
    .unwrap();
    let (carried, accepted) = (carried.unwrap(), accepted.unwrap());
    assert!(carried.paths().iter().any(|path| {
        path.id() == iroh::endpoint::PathId::ZERO
            && path.remote_addr() == &transport_addr(&server.id())
    }));
    assert_eq!(carried.remote_id(), server.id());
    tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(
            async {
                let (mut send, mut recv) = carried.open_bi().await.unwrap();
                send.write_all(b"initial-carrier").await.unwrap();
                send.finish().unwrap();
                assert_eq!(recv.read_to_end(64).await.unwrap(), b"initial-carrier");
            },
            async {
                let (mut send, mut recv) = accepted.accept_bi().await.unwrap();
                let bytes = recv.read_to_end(64).await.unwrap();
                send.write_all(&bytes).await.unwrap();
                send.finish().unwrap();
            }
        );
    })
    .await
    .unwrap();
    let wrong = iroh::SecretKey::generate().public();
    let addr = iroh::EndpointAddr::from_parts(wrong, [transport_addr(&server.id())]);
    let mut accepting = Aborts(Some(tokio::spawn({
        let server = server.clone();
        async move {
            if let Some(incoming) = server.accept().await
                && let Ok(connecting) = incoming.accept()
                && let Ok(conn) = connecting.await
            {
                conn.close(0u32.into(), b"identity control complete");
            }
        }
    })));
    let rejected = tokio::time::timeout(
        Duration::from_secs(5),
        connect_prepared_carrier(&client, addr, ALPN),
    )
    .await
    .unwrap()
    .expect_err("pinning a transport cannot bypass endpoint identity");
    if let Some(task) = accepting.0.take() {
        task.abort();
        let _ = task.await;
    }
    eprintln!("[initial-custom-identity-control] {rejected}");
    assert!(matches!(
        rejected,
        iroh::endpoint::ConnectError::Connecting { .. }
    ));
    assert!(warm.close_reason().is_none() && warm_remote.close_reason().is_none());
    carried.close(0u32.into(), b"initial route contract complete");
    warm.close(0u32.into(), b"warm route contract complete");
    client_links.detach_attached(&server.id(), Carrier::Hop { via }, local.serial());
    server_links.detach_attached(&client.id(), Carrier::Hop { via }, remote.serial());
    tokio::time::timeout(Duration::from_secs(2), async {
        tokio::join!(local.ended(), remote.ended());
    })
    .await
    .unwrap();
    tokio::join!(client.close(), server.close());
}

#[tokio::test]
async fn an_initial_custom_address_must_be_explicit_in_the_dial() {
    let client = iroh::Endpoint::builder(iroh::endpoint::presets::Minimal)
        .relay_mode(iroh::RelayMode::Disabled)
        .bind()
        .await
        .unwrap();
    let peer = iroh::SecretKey::generate().public();
    let iroh::TransportAddr::Custom(custom) = transport_addr(&peer) else {
        unreachable!()
    };
    let addr = iroh::EndpointAddr::from_parts(peer, std::iter::empty::<iroh::TransportAddr>());
    let result = client
        .connect_with_opts(
            addr,
            ALPN,
            iroh::endpoint::ConnectOptions::default().with_initial_custom_addr(custom),
        )
        .await;
    assert!(matches!(
        result,
        Err(iroh::endpoint::ConnectWithOptsError::InvalidInitialCustomAddr { .. })
    ));
    client.close().await;
}
