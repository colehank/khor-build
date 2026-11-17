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
