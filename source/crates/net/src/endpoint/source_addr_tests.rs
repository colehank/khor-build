//! Verify the kernel-observed source, not merely successful sendmsg.
//! No packet leaves the host: UDP connect only asks for an assigned source;
//! all three actual transmits go to a loopback receiver.

#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
#[test]
fn wildcard_ipv4_honors_each_packet_source_without_rebinding() {
    use std::net::{IpAddr, Ipv4Addr, UdpSocket};
    use std::time::Duration;

    let route = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    route.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).unwrap();
    let source = route.local_addr().unwrap().ip();
    assert!(
        !source.is_loopback() && !source.is_unspecified(),
        "test host needs an assigned non-loopback IPv4 address"
    );
    let receiver = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    receiver
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let sender = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    let port = sender.local_addr().unwrap().port();
    let state = noq_udp::UdpSocketState::new((&sender).into()).unwrap();
    for (requested, expected) in [
        (Some(source), source),
        (None, IpAddr::V4(Ipv4Addr::LOCALHOST)),
        (Some(source), source),
    ] {
        state
            .try_send(
                (&sender).into(),
                &noq_udp::Transmit {
                    destination: receiver.local_addr().unwrap(),
                    ecn: None,
                    contents: b"source-control",
                    segment_size: None,
                    src_ip: requested,
                },
            )
            .unwrap();
        let mut buf = [0; 32];
        let (len, observed) = receiver.recv_from(&mut buf).unwrap();
        assert_eq!(&buf[..len], b"source-control");
        assert_eq!(observed.ip(), expected, "requested source {requested:?}");
        assert_eq!(
            observed.port(),
            port,
            "the pinned UDP port must stay intact"
        );
        assert!(sender.local_addr().unwrap().ip().is_unspecified());
    }
}

/// A loopback-bound peer dials a wildcard endpoint at its assigned physical
/// IPv4. Its handshake and replies must retain that destination as their source.
/// This is encrypted iroh traffic with mutual identity checks, all on-host.
#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
#[tokio::test]
async fn wildcard_ipv4_authenticates_and_echoes_from_the_address_dialed() {
    use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
    use std::time::Duration;

    use iroh::endpoint::presets;
    use iroh::{Endpoint, EndpointAddr, RelayMode};

    let route = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    route.connect((Ipv4Addr::new(192, 0, 2, 1), 9)).unwrap();
    let source = route.local_addr().unwrap().ip();
    assert!(!source.is_loopback() && !source.is_unspecified());
    const ALPN: &[u8] = b"khor/source-control/test";
    let server = Endpoint::builder(presets::Minimal)
        .ca_tls_config(super::ca_roots())
        .relay_mode(RelayMode::Disabled)
        .clear_ip_transports()
        .bind_addr((Ipv4Addr::UNSPECIFIED, 0))
        .unwrap()
        .alpns(vec![ALPN.to_vec()])
        .bind()
        .await
        .unwrap();
    let client = Endpoint::builder(presets::Minimal)
        .ca_tls_config(super::ca_roots())
        .relay_mode(RelayMode::Disabled)
        .clear_ip_transports()
        .bind_addr((Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .bind()
        .await
        .unwrap();
    let destination = SocketAddr::new(source, server.bound_sockets()[0].port());
    let serve = async {
        let conn = server.accept().await.unwrap().await.unwrap();
        assert_eq!(conn.remote_id(), client.id());
        let (mut send, mut recv) = conn.accept_bi().await.unwrap();
        let body = recv.read_to_end(64).await.unwrap();
        assert_eq!(body, b"authenticated-source-probe");
        send.write_all(&body).await.unwrap();
        send.finish().unwrap();
        let _ = conn.closed().await;
    };
    let dial = async {
        let conn = client
            .connect(
                EndpointAddr::new(server.id()).with_ip_addr(destination),
                ALPN,
            )
            .await
            .unwrap();
        assert_eq!(conn.remote_id(), server.id());
        let (mut send, mut recv) = conn.open_bi().await.unwrap();
        send.write_all(b"authenticated-source-probe").await.unwrap();
        send.finish().unwrap();
        assert_eq!(
            recv.read_to_end(64).await.unwrap(),
            b"authenticated-source-probe"
        );
        assert_eq!(crate::path::classify(&conn), crate::path::PathKind::Direct);
        conn.close(0u32.into(), b"done");
    };
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        tokio::join!(serve, dial);
    })
    .await;
    tokio::join!(client.close(), server.close());
    result.expect("authenticated direct echo must finish without a relay");
}
