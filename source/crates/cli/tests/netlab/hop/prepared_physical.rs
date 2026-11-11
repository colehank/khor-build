//! Four actual residents: a frozen first middle must not delay the second.
//! This observes actual attempts and carried business, not a transport adapter.
use super::*;
use crate::lab::LabNode;
use khor_core::RoadKind;
use std::collections::BTreeSet;

fn peer_hop(nodes: &[&LabNode], on: bool) {
    for node in nodes {
        let value = if on { "on" } else { "off" };
        khor_node::config::set(&node.home.0, "peer-hop", value).unwrap();
        assert_eq!(khor_node::config::on(&node.home.0, "peer-hop"), on);
    }
}

async fn warm_tunnel(from: &LabNode, to: &LabNode) {
    let name = to.name.clone();
    // The IPC pipe door uses the resident's existing endpoint. Port 9 is
    // unused in these fresh namespaces; its refusal leaves the tunnel up.
    let _ = tokio::time::timeout(
        Duration::from_secs(10),
        from.with(move |node| async move {
            node.pipe_via(&name, "127.0.0.1", 9, tokio::io::empty(), tokio::io::sink())
                .await
        }),
    )
    .await
    .expect("the resident completes its bounded tunnel probe");
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if from
                .journal(Some(to))
                .iter()
                .any(|row| row["e"] == "dial" && row["tunnel"] == true && !row["won"].is_null())
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("an actual successful resident-owned Tunnel dial is recorded");
}

fn carried_wires(node: &LabNode, peer: &LabNode, since: u64) -> BTreeSet<u64> {
    node.journal(Some(peer))
        .into_iter()
        .filter(|row| {
            row["e"] == "path"
                && row["change"] == "started"
                && row["road"] == "carried"
                && row["at"].as_u64().is_some_and(|at| at >= since)
        })
        .filter_map(|row| row["wire"].as_u64())
        .collect()
}

fn retired(node: &LabNode, peer: &LabNode, wires: &BTreeSet<u64>, since: u64) -> bool {
    node.journal(Some(peer)).iter().any(|row| {
        row["e"] == "retired"
            && row["at"].as_u64().is_some_and(|at| at >= since)
            && row["wire"]
                .as_u64()
                .is_some_and(|wire| wires.contains(&wire))
    })
}

async fn measured_candidates(
    x: &LabNode,
    y: &LabNode,
    m: &LabNode,
    n: &LabNode,
) -> Option<(u32, u32)> {
    let (target, first, second) = (y.id.clone(), m.id.clone(), n.id.clone());
    x.with(move |node| async move {
        let devices = node.devices().ok()?;
        if !devices
            .iter()
            .any(|device| device.id == target && device.hop_staged)
        {
            return None;
        }
        let links = node.links().await.ok()?;
        let cost = |id: &str| {
            let device = devices.iter().find(|device| device.id == id)?;
            let first = links.iter().find(|(peer, _)| peer == id)?;
            if first.1.hop != Hop::Direct {
                return None;
            }
            let second = node.roads_of(device).into_iter().find(|road| {
                road.to == target && road.road == RoadKind::Direct && road.via.is_none()
            })?;
            Some(first.1.rtt_ms? + u32::from(second.ms?))
        };
        Some((cost(&first)?, cost(&second)?))
    })
    .await
}

pub(super) async fn run() {
    let s = "f4r";
    let lab = Lab::new().await.unwrap();
    let relay = relay(&lab, s, "relay", Some(Duration::from_millis(800))).await;
    let (rx, ry, public) = (
        router(&lab, "natx", Net::Strict).await,
        router(&lab, "naty", Net::Strict).await,
        router(&lab, "public", Net::Public).await,
    );
    let dx = lab.add_device("x").uplink(rx.id()).build().await.unwrap();
    let dy = lab.add_device("y").uplink(ry.id()).build().await.unwrap();
    let dm = lab
        .add_device("m")
        .uplink(public.id())
        .build()
        .await
        .unwrap();
    let dn = lab
        .add_device("n")
        .uplink(public.id())
        .build()
        .await
        .unwrap();
    for device in [&dx, &dy, &dm] {
        slow(device, LEG).await;
    }
    slow(&dn, Duration::from_millis(80)).await;
    let mut x = node(s, dx, "x", &[&relay], &[]).await;
    let mut y = node(s, dy, "y", &[&relay], &[]).await;
    let m = node(s, dm, "m", &[&relay], &[]).await;
    let n = node(s, dn, "n", &[&relay], &[]).await;
    // The lower identity owns an outgoing hop if both residents ask.
    if y.id < x.id {
        std::mem::swap(&mut x, &mut y);
    }
    peer_hop(&[&x, &y, &m, &n], false);
    tokio::time::sleep(Duration::from_secs(5)).await;
    mesh(&[&x, &y, &m, &n]).await;
    for end in [&x, &y] {
        for middle in [&m, &n] {
            assert!(
                end.until(middle, Duration::from_secs(90), |link| {
                    link.hop == Hop::Direct && link.rtt_ms.is_some()
                })
                .await
                .is_some(),
                "{s}: warm actual direct legs before the fault"
            );
        }
    }
    for middle in [&m, &n] {
        warm_tunnel(&x, middle).await;
        warm_tunnel(middle, &y).await;
    }
    let costs = tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            if let Some(costs) = measured_candidates(&x, &y, &m, &n).await {
                break costs;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .expect("prepared hint and both actual two-leg road facts arrive");
    assert!(
        costs.0 < costs.1,
        "{s}: frozen M is actually ranked ahead of N"
    );
    eprintln!(
        "[netlab-prepared] {s} x={} y={} m={} n={} costs={costs:?}",
        x.id, y.id, m.id, n.id
    );

    cut_as(&x.dev, relay.dev.ip().unwrap(), "prepared_no_relay");
    let name = y.name.clone();
    x.with(move |node| async move { node.let_go_of(&name).await })
        .await
        .unwrap();
    let name = x.name.clone();
    y.with(move |node| async move { node.let_go_of(&name).await })
        .await
        .unwrap();
    // No carried link has ever been allowed in this fresh fixture.
    assert!(
        x.link(&y)
            .await
            .is_none_or(|link| link.hop != Hop::Forwarded)
    );
    m.freeze();
    x.freeze();
    y.freeze();
    peer_hop(&[&x, &y, &m, &n], true);
    let began = now_ms();
    let activated = std::time::Instant::now();
    crate::attempts::phase(s, "prepared_race_started", began);
    y.thaw();
    x.thaw();
    let name = y.name.clone();
    let (first_time, first_error) = crate::attempts::measure(
        s,
        &x.name,
        &y.name,
        Duration::from_millis(1500),
        x.with(move |node| async move { node.ls_of(&name, "/tmp").await.map(|_| ()) }),
    )
    .await;
    let first_response = activated.elapsed();
    let stopped = std::fs::read_to_string(format!("/proc/{}/status", m.pid()))
        .unwrap()
        .lines()
        .any(|line| line.starts_with("State:") && line.contains("T (stopped)"));
    // Restore the resident even when the first business contract failed.
    m.thaw();
    let restored = now_ms();
    crate::attempts::phase(s, "first_middle_resumed", restored);
    let via_n = |link: &khor_core::Link| {
        link.hop == Hop::Forwarded && link.via.as_deref() == Some(n.id.as_str())
    };
    let carried = x.until(&y, Duration::from_secs(2), via_n).await;
    let x_wires = carried_wires(&x, &y, began);
    let y_wires = carried_wires(&y, &x, began);
    let two_asks = x.journal(Some(&y)).iter().any(|row| {
        row["e"] == "hop"
            && row["at"]
                .as_u64()
                .is_some_and(|at| at >= began && at <= restored)
            && row["via"].as_str() == Some(n.id.as_str())
            && row["tried"] == 2
    });
    let hold = std::time::Instant::now();
    let mut samples = 0;
    let mut held = true;
    while hold.elapsed() < Duration::from_secs(10) || samples < 30 {
        let name = y.name.clone();
        let (_, error) = crate::attempts::measure(
            s,
            &x.name,
            &y.name,
            Duration::from_secs(2),
            x.with(move |node| async move { node.ls_of(&name, "/tmp").await.map(|_| ()) }),
        )
        .await;
        samples += 1;
        held &= error.is_none()
            && x.link(&y).await.as_ref().is_some_and(via_n)
            && y.link(&x).await.as_ref().is_some_and(via_n);
        if !held {
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    let continuous = x_wires.len() == 1
        && y_wires.len() == 1
        && carried_wires(&x, &y, began) == x_wires
        && carried_wires(&y, &x, began) == y_wires
        && !retired(&x, &y, &x_wires, restored)
        && !retired(&y, &x, &y_wires, restored);
    let ok = first_error.is_none()
        && first_response < Duration::from_millis(1500)
        && stopped
        && two_asks
        && carried.is_some()
        && held
        && samples >= 30
        && hold.elapsed() >= Duration::from_secs(10)
        && continuous;
    for (owner, peer) in [(&x, &y), (&y, &x), (&x, &m), (&x, &n)] {
        path_diagnostics(owner, peer);
    }
    for owner in [&x, &y, &m, &n] {
        sample_diagnostics(s, owner);
    }
    eprintln!(
        "[netlab-prepared-limits] {s} late_M_OK=unobserved cancellation=unobserved cpu=unknown bytes=unknown; restoring M does not prove a late admission occurred"
    );
    result(
        s,
        ok,
        format!(
            "activation_to_content={first_response:?} request={first_time:?} error={first_error:?} M_stopped={stopped} two_asks={two_asks} via_N={} samples={samples} hold={:?} continuous={continuous} x_wires={x_wires:?} y_wires={y_wires:?}",
            carried.is_some(),
            hold.elapsed()
        ),
    );
    assert!(ok, "{s}: prepared race and actual carried continuity");
}
