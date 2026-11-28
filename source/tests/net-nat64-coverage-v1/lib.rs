//! Actual IPv6-only NAT64/DNS bootstrap, independent of the product build.
#![cfg(target_os = "linux")]
#[path = "@ROOT@/crates/cli/tests/netlab/lab.rs"]
mod lab;
#[path = "@ROOT@/crates/cli/tests/scratch/mod.rs"]
mod scratch;
use patchbay::{Lab, RouterPreset};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

#[ctor::ctor(unsafe)]
fn userns() {
    patchbay::init_userns().expect("isolated user/network namespaces");
}
fn translated(v4: Ipv4Addr) -> Ipv6Addr {
    let [a, b, c, d] = v4.octets();
    Ipv6Addr::new(
        0x64,
        0xff9b,
        0,
        0,
        0,
        0,
        u16::from_be_bytes([a, b]),
        u16::from_be_bytes([c, d]),
    )
}
struct Owned<T>(tokio::task::JoinHandle<T>);
impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ipv6_only_nat64_cold_product_pair_and_directory() {
    let lab = Lab::new().await.unwrap();
    let relay = lab::relay(&lab, "nat64-cold", "relay", None).await;
    let relay_v4 = relay.dev.ip().expect("IPv4 relay");
    // lab::relay binds its actual listener to relay_v4; the synthesized AAAA never names its native IPv6 interface.
    let nat64_relay = translated(relay_v4);
    let dns = lab.dns_server().unwrap();
    dns.set_host("relay.nat64.test.", IpAddr::V4(relay_v4))
        .unwrap();
    dns.set_host("relay.nat64.test.", IpAddr::V6(nat64_relay))
        .unwrap();
    let carrier = lab
        .add_router("carrier")
        .preset(RouterPreset::IspV6)
        .build()
        .await
        .unwrap();
    let phone_dev = lab
        .add_device("phone")
        .uplink(carrier.id())
        .build()
        .await
        .unwrap();
    assert!(phone_dev.ip().is_none(), "phone has no IPv4 address");
    assert!(phone_dev.ip6().is_some());
    let dc = lab
        .add_router("peer-dc")
        .preset(RouterPreset::PublicV4)
        .build()
        .await
        .unwrap();
    let peer_dev = lab
        .add_device("peer")
        .uplink(dc.id())
        .build()
        .await
        .unwrap();
    let raw_state = phone_dev
        .spawn(|_| async {
            let output = tokio::process::Command::new("ip")
                .args(["-j", "addr", "show"])
                .output()
                .await
                .unwrap();
            assert!(output.status.success());
            serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap()
        })
        .unwrap()
        .await
        .unwrap();
    // A real cross-family UDP oracle before any product assertion.
    let (ready, receive) = tokio::sync::oneshot::channel();
    let mut server = Owned(
        relay
            .dev
            .spawn(move |_| async move {
                let socket = tokio::net::UdpSocket::bind((relay_v4, 0)).await.unwrap();
                ready.send(socket.local_addr().unwrap().port()).unwrap();
                let mut buffer = [0u8; 64];
                let (n, source) =
                    tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut buffer))
                        .await
                        .unwrap()
                        .unwrap();
                assert!(source.is_ipv4());
                assert_eq!(&buffer[..n], b"nat64-independent-positive-v1");
                socket.send_to(&buffer[..n], source).await.unwrap();
                source
            })
            .unwrap(),
    );
    let port = receive.await.unwrap();
    let reflected = phone_dev
        .spawn(move |_| async move {
            let socket = tokio::net::UdpSocket::bind("[::]:0").await.unwrap();
            socket
                .send_to(
                    b"nat64-independent-positive-v1",
                    SocketAddr::new(nat64_relay.into(), port),
                )
                .await
                .unwrap();
            let mut bytes = [0u8; 64];
            let (n, source) =
                tokio::time::timeout(Duration::from_secs(3), socket.recv_from(&mut bytes))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(&bytes[..n], b"nat64-independent-positive-v1");
            assert!(source.is_ipv6());
            source
        })
        .unwrap()
        .await
        .unwrap();
    let observed = (&mut server.0).await.unwrap();
    let system_dns = phone_dev
        .spawn(|_| async {
            let resolver = iroh::dns::DnsResolver::new();
            let addresses = resolver
                .lookup_ipv6("relay.nat64.test.", Duration::from_secs(2))
                .await
                .unwrap()
                .collect::<Vec<_>>();
            assert!(!addresses.is_empty());
            addresses
        })
        .unwrap()
        .await
        .unwrap();
    assert!(system_dns.contains(&IpAddr::V6(nat64_relay)));
    let list = vec![json!({"url":"http://relay.nat64.test:3340"})];
    let phone = lab::node_on(
        "nat64-cold",
        phone_dev,
        "phone",
        list.clone(),
        &[("KHOR_RELAY_ONLY", "1")],
    )
    .await;
    let peer = lab::node_on(
        "nat64-cold",
        peer_dev,
        "peer",
        list,
        &[("KHOR_RELAY_ONLY", "1")],
    )
    .await;
    let began = Instant::now();
    let paired = tokio::time::timeout(Duration::from_secs(10), lab::pair(&peer, &phone)).await;
    let pairing_ok = paired.is_ok();
    let mut reads = Vec::new();
    if pairing_ok {
        for (a, b) in [(&phone, &peer), (&peer, &phone)] {
            let target = b.name.clone();
            let started = Instant::now();
            let answer = tokio::time::timeout(
                Duration::from_secs(2),
                a.with(move |n| async move { n.ls_of(&target, "/tmp").await }),
            )
            .await;
            reads.push(json!({"from":a.name,"to":b.name,"ok":matches!(answer,Ok(Ok(_))),"elapsed_us":started.elapsed().as_micros()}));
        }
    }
    eprintln!(
        "[netlab-nat64] {}",
        json!({"schema":1,"revision":"nat64_cold_product_v1","phone_kernel":raw_state,
        "relay_v4":relay_v4,"synthetic_aaaa":nat64_relay,"udp_received_by_v4_server_from":observed,
        "udp_received_by_v6_client_from":reflected,"system_dns":system_dns,"pairing_ok":pairing_ok,
        "pairing_elapsed_us":began.elapsed().as_micros(),"pairing_bound_seconds":10,"reads":reads,
        "model":"IspV6 SIIT+IPv4 masquerade; fixed synthetic DNS64 AAAA and real A; not dynamic DNS64 algorithm, CLAT or mobile device"})
    );
    assert!(
        pairing_ok,
        "cold product bootstrap honors working network DNS/NAT64"
    );
    assert_eq!(reads.len(), 2);
    assert!(reads.iter().all(|r| r["ok"] == true));
}
