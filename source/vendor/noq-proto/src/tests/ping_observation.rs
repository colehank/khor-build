//! Actual transmit/ACK/RTT evidence for path-specific recovery probes.

use std::time::Duration;

use testresult::TestResult;

use super::util::{ConnPair, ManyToManyRouting, subscribe};
use crate::{FourTuple, PathStatus, Side::*};

fn observe_backup_ping(available: bool) -> TestResult {
    let _guard = subscribe();
    let mut pair = ConnPair::builder().enable_multipath().connect();
    let routing = pair.routes.as_basic();
    let (mut ca, mut sa) = (routing.client_addr, routing.server_addr);
    let clients = vec![ca, {
        ca.set_port(ca.port() + 1);
        ca
    }];
    let servers = vec![sa, {
        sa.set_port(sa.port() + 1);
        sa
    }];
    pair.routes = ManyToManyRouting::simple_symmetric(clients.clone(), servers.clone()).into();
    let backup = pair.open_path(
        Client,
        FourTuple {
            local_ip: Some(clients[1].ip()),
            remote: servers[1],
        },
        PathStatus::Backup,
    )?;
    pair.drive();
    pair.set_path_status(Server, backup, PathStatus::Backup)?;
    pair.drive();
    let before = pair.path_stats(Client, backup).unwrap();
    let route = usize::from(available);
    pair.routes = ManyToManyRouting::simple_symmetric([clients[route]], [servers[route]]).into();
    let began = pair.time;
    let by = began + Duration::from_millis(200);
    pair.ping_path(Client, backup)?;
    for _ in 0..64 {
        if pair.time >= by
            || available && pair.path_stats(Client, backup).unwrap().rtt_samples > before.rtt_samples
        {
            break;
        }
        if !pair.step() {
            break;
        }
    }
    let after = pair.path_stats(Client, backup).unwrap();
    assert!(after.frame_tx.ping > before.frame_tx.ping);
    assert!(after.udp_tx.datagrams > before.udp_tx.datagrams);
    if available {
        assert!(pair.time < by, "backup answers while the data road is dead");
        assert!(after.frame_rx.path_acks > before.frame_rx.path_acks);
        assert!(after.rtt_samples > before.rtt_samples);
        assert!(after.latest_rtt_sent.is_some_and(|sent| sent >= began));
        assert!(after.latest_rtt_received.is_some_and(|at| at < by));
    } else {
        assert_eq!(after.rtt_samples, before.rtt_samples);
        assert_eq!(after.latest_rtt_sent, before.latest_rtt_sent);
        assert_eq!(after.latest_rtt_received, before.latest_rtt_received);
        assert_eq!(after.frame_rx.path_acks, before.frame_rx.path_acks);
    }
    Ok(())
}

#[test]
fn backup_ping_is_transmitted_acked_and_samples_a_new_packet() -> TestResult {
    observe_backup_ping(true)
}

#[test]
fn accepted_and_transmitted_ping_does_not_fake_an_rtt_on_a_blackhole() -> TestResult {
    observe_backup_ping(false)
}
