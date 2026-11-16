//! IPv6-only independent/both-end changes with an actually IPv6-reachable Relay.
use super::both::{changed, present, read, rules, sample, snapshot};
use super::mobile::diagnostics;
use super::mobility::{addresses, command, entry};
use super::*;
use std::io::{Read, Write};
use std::net::{Ipv6Addr, SocketAddr};

#[derive(Clone, Copy, Debug)]
enum Movement {
    OneIid,
    OnePrefix,
    BothIid,
    BothPrefix,
    BothPrefixRelay,
}

impl Movement {
    fn iid(self) -> bool {
        matches!(self, Self::OneIid | Self::BothIid)
    }
    fn both(self) -> bool {
        !matches!(self, Self::OneIid | Self::OnePrefix)
    }
    fn relay_only(self) -> bool {
        matches!(self, Self::BothPrefixRelay)
    }
    fn name(self) -> &'static str {
        match self {
            Self::OneIid => "v6-only-one-iid",
            Self::OnePrefix => "v6-only-one-prefix",
            Self::BothIid => "v6-only-both-iid",
            Self::BothPrefix => "v6-only-both-prefix",
            Self::BothPrefixRelay => "v6-only-both-prefix-relay",
        }
    }
}

fn only_v6(n: &LabNode) -> bool {
    n.dev.ip().is_none()
        && addresses(&n.dev, "-4")
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["ifname"] != "lo")
            .all(|i| i["addr_info"].as_array().unwrap().is_empty())
}

fn tcp_probe(dev: &Device, remote: SocketAddr) -> serde_json::Value {
    dev.run_sync(move || {
        let mut socket = std::net::TcpStream::connect_timeout(&remote, Duration::from_secs(2))
            .expect("literal IPv6 Relay TCP is reachable");
        socket
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        socket
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let local = socket.local_addr().unwrap();
        assert!(local.is_ipv6() && socket.peer_addr().unwrap() == remote);
        socket
            .write_all(b"GET / HTTP/1.1\r\nHost: netlab\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut bytes = [0u8; 256];
        let received = socket.read(&mut bytes).expect("IPv6 Relay HTTP response");
        assert!(bytes[..received].starts_with(b"HTTP/"));
        Ok(serde_json::json!({"local":local,"remote":remote,"response_bytes":received}))
    })
    .unwrap()
}

fn udp_rule(dev: &Device, reflector: SocketAddr) -> String {
    let script = format!(
        "table inet v6_only {{\n chain output {{\n type filter hook output priority 0; policy accept;\n ip6 daddr {} udp dport {} accept;\n meta l4proto udp counter drop;\n }}\n}}\n",
        reflector.ip(),
        reflector.port()
    );
    let mut cmd = Command::new("nft");
    cmd.args(["-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = dev.spawn_command_sync(cmd).unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(script.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "owned IPv6-only UDP isolation: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    rule_snapshot(dev)
}

fn rule_snapshot(dev: &Device) -> String {
    let output = command(dev, &["nft", "list", "table", "inet", "v6_only"]);
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

async fn move_endpoint(n: &LabNode, cell: &Router, movement: Movement) -> Ipv6Addr {
    let old = n.dev.ip6().expect("configured IPv6-only address");
    if !movement.iid() {
        n.dev
            .iface("eth0")
            .unwrap()
            .replug(cell.id())
            .await
            .unwrap();
        return n.dev.ip6().expect("new IPv6 prefix address");
    }
    let (interface, info) = entry(&addresses(&n.dev, "-6"), old).unwrap();
    assert_eq!(interface, "eth0");
    let prefix = info["prefixlen"].as_u64().unwrap();
    assert_eq!(prefix, 64, "routed original IPv6 prefix");
    let mut octets = old.octets();
    octets[15] ^= 64;
    let new = Ipv6Addr::from(octets);
    assert!(entry(&addresses(&n.dev, "-6"), new).is_none());
    let output = command(
        &n.dev,
        &[
            "ip",
            "-6",
            "address",
            "add",
            &format!("{new}/{prefix}"),
            "dev",
            &interface,
        ],
    );
    assert!(output.status.success(), "add owned IID");
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let (_, info) = entry(&addresses(&n.dev, "-6"), new).unwrap();
        assert_ne!(info["dadfailed"], serde_json::Value::Bool(true));
        if info["tentative"] != serde_json::Value::Bool(true) {
            break;
        }
        assert!(Instant::now() < until, "new IID completes DAD");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let output = command(
        &n.dev,
        &[
            "ip",
            "-6",
            "address",
            "del",
            &format!("{old}/{prefix}"),
            "dev",
            &interface,
        ],
    );
    assert!(output.status.success(), "remove only original owned IID");
    new
}

async fn session(
    a: &LabNode,
    b: &LabNode,
    cells: [&Router; 2],
    reflector: SocketAddr,
    blocked: SocketAddr,
    relay_addr: SocketAddr,
    movement: Movement,
) -> Result<(), String> {
    let s = movement.name();
    tokio::time::timeout(Duration::from_secs(120), async {
        mesh(&[a, b]).await;
        for (direction, from, to) in [("ab", a, b), ("ba", b, a)] {
            let (_, error) = read(
                from,
                to,
                "setup",
                &format!("{s}-setup-{direction}"),
                Duration::from_secs(30),
            )
            .await;
            if let Some(e) = error {
                return Err(e);
            }
        }
        let mut held = None;
        loop {
            let available = match (a.link(b).await, b.link(a).await) {
                (Some(a), Some(b)) => {
                    a.down_ms.is_none()
                        && b.down_ms.is_none()
                        && a.rtt_ms.is_some()
                        && b.rtt_ms.is_some()
                }
                _ => false,
            };
            if available {
                if held.get_or_insert_with(Instant::now).elapsed() >= Duration::from_secs(10) {
                    break;
                }
            } else {
                held = None;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        Ok::<_, String>(())
    })
    .await
    .map_err(|_| "IPv6-only preparation exceeded120s".to_owned())??;
    let before = [snapshot(a), snapshot(b)];
    assert!(only_v6(a) && only_v6(b));
    let old = [a.dev.ip6().unwrap(), b.dev.ip6().unwrap()];
    let before_mapping = [
        a.dev.probe_udp_mapping(reflector).unwrap(),
        b.dev.probe_udp_mapping(reflector).unwrap(),
    ];
    assert!(before_mapping.iter().zip(old).all(|(m, ip)| m.ip() == ip));
    let before_tcp = [tcp_probe(&a.dev, relay_addr), tcp_probe(&b.dev, relay_addr)];
    if movement.relay_only() {
        assert!(
            a.dev.probe_udp_mapping(blocked).is_err() && b.dev.probe_udp_mapping(blocked).is_err()
        );
    }
    let epoch = Instant::now();
    let ma = async {
        let start = epoch.elapsed();
        let ip = move_endpoint(a, cells[0], movement).await;
        (ip, start.as_micros(), epoch.elapsed().as_micros())
    };
    let mb = async {
        let start = epoch.elapsed();
        let ip = if movement.both() {
            move_endpoint(b, cells[1], movement).await
        } else {
            old[1]
        };
        (ip, start.as_micros(), epoch.elapsed().as_micros())
    };
    let (ma, mb) = tokio::join!(ma, mb);
    let new = [ma.0, mb.0];
    let pivot = Instant::now();
    // First work after both changes is actual bidirectional business.
    let mut samples = Vec::with_capacity(40);
    for index in 0..20 {
        tokio::time::sleep_until((pivot + Duration::from_millis(500) * index).into()).await;
        let (ab, ba) = tokio::join!(
            sample(a, b, "ab", s, index, pivot),
            sample(b, a, "ba", s, index, pivot)
        );
        samples.extend([ab, ba]);
    }
    let after = [snapshot(a), snapshot(b)];
    let after_mapping = [
        a.dev.probe_udp_mapping(reflector).unwrap(),
        b.dev.probe_udp_mapping(reflector).unwrap(),
    ];
    let after_tcp = [tcp_probe(&a.dev, relay_addr), tcp_probe(&b.dev, relay_addr)];
    let geometry = (0..2).all(|i| {
        let moved = i == 0 || movement.both();
        if !moved {
            return old[i] == new[i] && before[i] == after[i];
        }
        changed(
            &before[i],
            &after[i],
            "v6",
            Some(old[i].to_string()),
            Some(new[i].to_string()),
        ) && (old[i].octets()[..8] == new[i].octets()[..8]) == movement.iid()
            && !present(&after[i], "v6", &old[i].to_string())
    }) && only_v6(a)
        && only_v6(b)
        && after_mapping.iter().zip(new).all(|(m, ip)| m.ip() == ip);
    let first: Vec<_> = ["ab", "ba"]
        .into_iter()
        .map(|direction| {
            samples
                .iter()
                .find(|r| r["direction"] == direction && r["error"].is_null())
                .and_then(|r| r["ended_us"].as_u64())
        })
        .collect();
    let route_ok = if movement.relay_only() {
        a.dev.probe_udp_mapping(blocked).is_err()
            && b.dev.probe_udp_mapping(blocked).is_err()
            && a.link(b).await.is_some_and(|l| l.hop == Hop::Relay)
            && b.link(a).await.is_some_and(|l| l.hop == Hop::Relay)
    } else {
        true
    };
    let passed = geometry
        && route_ok
        && samples.len() == 40
        && samples.iter().all(|r| r["error"].is_null())
        && first.iter().all(|v| v.is_some_and(|us| us <= 2_000_000));
    let report = serde_json::json!({"schema":1,"revision":"ipv6_only_mobility_v1","case":s,"event":"complete",
        "before":before,"after":after,"old":old,"new":new,"before_mapping":before_mapping,"after_mapping":after_mapping,
        "before_tcp":before_tcp,"after_tcp":after_tcp,"movement_intervals_us":[[ma.1,ma.2],[mb.1,mb.2]],
        "pivot_us":pivot.duration_since(epoch).as_micros(),"caller_us":2_000_000,"period_us":500_000,
        "samples":samples,"first_reply_us":first,"geometry":geometry,"route_ok":route_ok,"passed":passed});
    eprintln!("[netlab-v6-only] {report}");
    if passed {
        Ok(())
    } else {
        Err(format!("IPv6-only geometry/business failed: {report}"))
    }
}

async fn run(movement: Movement) {
    let s = movement.name();
    let lab = Lab::new().await.unwrap();
    let relay = crate::lab::relay_v6(&lab, s, "v6-relay", Some(Duration::from_millis(100))).await;
    let relay_addr = SocketAddr::new(relay.dev.ip6().unwrap().into(), 3340);
    let mut routers = Vec::new();
    for name in [
        "v6-wifi-a",
        "v6-wifi-b",
        "v6-cell-a",
        "v6-cell-b",
        "v6-reflector",
    ] {
        routers.push(
            lab.add_router(name)
                .preset(RouterPreset::Public)
                .ip_support(IpSupport::V6Only)
                .build()
                .await
                .unwrap(),
        );
    }
    let rd = lab
        .add_device("v6-reflector")
        .uplink(routers[4].id())
        .build()
        .await
        .unwrap();
    let reflector = SocketAddr::new(rd.ip6().unwrap().into(), 39999);
    let blocked = SocketAddr::new(rd.ip6().unwrap().into(), 39998);
    let _reflector = rd.spawn_reflector(reflector).await.unwrap();
    let _blocked = rd.spawn_reflector(blocked).await.unwrap();
    let da = lab
        .add_device("v6-a")
        .uplink(routers[0].id())
        .build()
        .await
        .unwrap();
    let db = lab
        .add_device("v6-b")
        .uplink(routers[1].id())
        .build()
        .await
        .unwrap();
    for dev in [&da, &db] {
        assert!(dev.ip().is_none() && dev.probe_udp_mapping(reflector).unwrap().is_ipv6());
        assert!(dev.probe_udp_mapping(blocked).unwrap().is_ipv6());
        if movement.relay_only() {
            eprintln!("[netlab-v6-only-rule] {}", udp_rule(dev, reflector));
        }
    }
    let a = node(s, da, "v6-only-a", &[&relay], &[]).await;
    let b = node(s, db, "v6-only-b", &[&relay], &[]).await;
    for r in &routers {
        eprintln!(
            "[netlab-v6-only-router] {}",
            serde_json::json!({"phase":"before","rules":rules(r)})
        );
    }
    let checked = tokio::time::timeout(
        Duration::from_secs(240),
        session(
            &a,
            &b,
            [&routers[2], &routers[3]],
            reflector,
            blocked,
            relay_addr,
            movement,
        ),
    )
    .await;
    diagnostics(s, &a, &b);
    diagnostics(s, &b, &a);
    if movement.relay_only() {
        for n in [&a, &b] {
            eprintln!("[netlab-v6-only-rule] {}", rule_snapshot(&n.dev));
        }
    }
    for r in &routers {
        eprintln!(
            "[netlab-v6-only-router] {}",
            serde_json::json!({"phase":"after","rules":rules(r)})
        );
    }
    let ok = matches!(&checked, Ok(Ok(())));
    result(
        s,
        ok,
        format!("IPv6-only changes before real requests: {checked:?}"),
    );
    assert!(
        ok,
        "IPv6-only callers recover under original2second contracts"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "guarded IPv6-only independent IID change"]
async fn one_iid_changes_with_no_ipv4_rescue() {
    run(Movement::OneIid).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "guarded IPv6-only independent prefix change"]
async fn one_prefix_changes_with_no_ipv4_rescue() {
    run(Movement::OnePrefix).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "guarded IPv6-only both IID changes"]
async fn both_iids_change_before_bidirectional_business() {
    run(Movement::BothIid).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "guarded IPv6-only both prefix changes"]
async fn both_prefixes_change_before_bidirectional_business() {
    run(Movement::BothPrefix).await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "guarded IPv6-only both prefix changes with UDP isolated"]
async fn both_prefixes_recover_through_ipv6_relay() {
    run(Movement::BothPrefixRelay).await;
}
