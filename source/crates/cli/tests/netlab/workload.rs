//! Actual resident directory reads, a remote PTY, and a file pull together.
//! This records a baseline, not a latency SLO or a queue-policy comparison.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use khor_core::Hop;
use khor_node::{Node, host};
use patchbay::{Lab, LinkCondition, LinkDirection};
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::lab::{mesh, node, relay, result};
use crate::nat::{Net, router};

const REVISION: &str = "resident_mixed_workload_v1";
const SAMPLES: usize = 120;
const PERIOD: Duration = Duration::from_millis(100);
// Measurement bounds, not user-approved UX targets. Failed/blocked samples
// remain in the population; a small successful subset cannot pass this test.
const CALLER: Duration = Duration::from_secs(2);
const BYTES: usize = 16 * 1024 * 1024;

fn emit(value: Value) {
    eprintln!("[netlab-workload] {value}");
}

fn at(epoch: Instant) -> u64 {
    epoch.elapsed().as_micros().try_into().unwrap()
}

fn framed(op: &host::ClientOp) -> Vec<u8> {
    let mut bytes = Vec::new();
    host::write_frame(&mut bytes, op).unwrap();
    bytes
}

/// The expected prefix is absent from the input. Even if terminal echo is
/// accidentally on, an echoed input cannot impersonate the shell's response.
async fn echo(stream: &mut TcpStream, token: &str) -> Result<(), String> {
    let frame = framed(&host::ClientOp::Input(format!("{token}\n").into_bytes()));
    stream.write_all(&frame).await.map_err(|e| e.to_string())?;
    let expected = format!("NETLAB-REPLY-{token}");
    let mut received = Vec::new();
    loop {
        let mut bytes = [0; 4096];
        let n = stream.read(&mut bytes).await.map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("terminal stream ended before its response".into());
        }
        received.extend_from_slice(&bytes[..n]);
        if received.windows(expected.len()).any(|b| b == expected.as_bytes()) {
            return Ok(());
        }
        if received.len() > 1024 * 1024 {
            return Err("terminal response exceeded the replay bound".into());
        }
    }
}

fn sample(
    epoch: Instant,
    phase: &str,
    operation: &str,
    index: usize,
    scheduled_us: u64,
    started_us: u64,
    outcome: &str,
    error: Option<String>,
) -> Value {
    let ended_us = at(epoch);
    let row = json!({
        "schema": 1, "revision": REVISION, "event": "sample",
        "phase": phase, "operation": operation, "index": index,
        "scheduled_us": scheduled_us, "started_us": started_us,
        "ended_us": ended_us, "elapsed_us": ended_us - started_us,
        "schedule_lag_us": started_us.saturating_sub(scheduled_us),
        "caller_us": CALLER.as_micros(), "outcome": outcome, "error": error,
    });
    emit(row.clone());
    row
}

async fn directory(
    node: &Node,
    path: &str,
    epoch: Instant,
    phase_start: Instant,
    phase: &str,
) -> Vec<Value> {
    let mut rows = Vec::with_capacity(SAMPLES);
    for index in 0..SAMPLES {
        let scheduled = phase_start + PERIOD * index as u32;
        tokio::time::sleep_until(scheduled.into()).await;
        let started_us = at(epoch);
        let (outcome, error) = match tokio::time::timeout(CALLER, node.ls_of("y", path)).await {
            Ok(Ok((_, entries, false))) if entries.iter().any(|e| e.name == "payload.bin" && e.size == BYTES as u64) => ("ok", None),
            Ok(Ok(_)) => ("error", Some("directory omitted the known payload".into())),
            Ok(Err(e)) => ("error", Some(e)),
            Err(_) => ("timeout", Some("directory caller deadline".into())),
        };
        rows.push(sample(epoch, phase, "directory", index,
            scheduled.duration_since(epoch).as_micros().try_into().unwrap(),
            started_us, outcome, error));
    }
    rows
}

async fn terminal(
    stream: &mut TcpStream,
    epoch: Instant,
    phase_start: Instant,
    phase: &str,
) -> Vec<Value> {
    let mut rows = Vec::with_capacity(SAMPLES);
    let mut broken = false;
    for index in 0..SAMPLES {
        let scheduled = phase_start + PERIOD * index as u32;
        tokio::time::sleep_until(scheduled.into()).await;
        let started_us = at(epoch);
        let (outcome, error) = if broken {
            ("blocked", Some("terminal failed earlier; no implicit reattach".into()))
        } else {
            match tokio::time::timeout(CALLER, echo(stream, &format!("{phase}-{index:04}"))).await {
                Ok(Ok(())) => ("ok", None),
                Ok(Err(e)) => { broken = true; ("error", Some(e)) },
                Err(_) => { broken = true; ("timeout", Some("terminal caller deadline".into())) },
            }
        };
        rows.push(sample(epoch, phase, "terminal", index,
            scheduled.duration_since(epoch).as_micros().try_into().unwrap(),
            started_us, outcome, error));
    }
    rows
}

async fn phase(
    node: &Arc<Node>,
    stream: &mut TcpStream,
    path: &str,
    landed: &std::path::Path,
    epoch: Instant,
    name: &str,
    bulk: bool,
) -> (Vec<Value>, Value, Option<PathBuf>) {
    // Snapshots are outside the sample window, on this resident's actual IPC.
    let before = node.wire_counts().await.map_err(|e| e.to_string());
    let phase_start = Instant::now();
    emit(json!({"schema":1,"revision":REVISION,"event":"phase_started","phase":name,"at_us":at(epoch),"wire_counts":before}));
    let pull = async {
        if !bulk {
            return (Value::Null, None);
        }
        let started_us = at(epoch);
        let file = format!("{path}/payload.bin");
        let reply = tokio::time::timeout(Duration::from_secs(120), node.pull_path("y", &file, landed)).await;
        let ended_us = at(epoch);
        let (outcome, error, moved, dest) = match reply {
            Ok(Ok((moved, dest))) => ("ok", None, Some(moved), Some(dest)),
            Ok(Err(e)) => ("error", Some(e), None, None),
            Err(_) => ("timeout", Some("bulk caller deadline".into()), None, None),
        };
        let row = json!({"schema":1,"revision":REVISION,"event":"bulk","phase":name,
            "started_us":started_us,"ended_us":ended_us,"moved":moved,"expected":BYTES,
            "outcome":outcome,"error":error,"caller_ms":120000});
        emit(row.clone());
        (row, dest)
    };
    // All three futures are borrowed by this parent: no detached measurement
    // tasks or extra endpoints sharing the resident's identity.
    let (mut directories, terminals, (bulk_row, dest)) = tokio::join!(
        directory(node, path, epoch, phase_start, name),
        terminal(stream, epoch, phase_start, name),
        pull,
    );
    directories.extend(terminals);
    emit(json!({"schema":1,"revision":REVISION,"event":"phase_finished","phase":name,
        "at_us":at(epoch),"wire_counts":node.wire_counts().await.map_err(|e|e.to_string())}));
    (directories, bulk_row, dest)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 6)]
#[ignore = "real resident workload baseline; run alone through the guarded netlab task"]
async fn w1_real_terminal_and_directory_during_a_resident_file_pull() {
    let s = "w1";
    let lab = Lab::new().await.unwrap();
    let r = relay(&lab, s, "relay", None).await;
    let rx = router(&lab, "pubx", Net::Public).await;
    let ry = router(&lab, "puby", Net::Public).await;
    let dx = lab.add_device("x").uplink(rx.id()).build().await.unwrap();
    let dy = lab.add_device("y").uplink(ry.id()).build().await.unwrap();
    let x = node(s, dx, "x", &[&r], &[("SHELL", "/bin/sh")]).await;
    let y = node(s, dy, "y", &[&r], &[("SHELL", "/bin/sh")]).await;
    mesh(&[&x, &y]).await;
    assert!(x.until(&y, Duration::from_secs(60), |l| l.hop == Hop::Direct).await.is_some());
    let source = y.home.0.join("workload");
    let landed = x.home.0.join("landed");
    std::fs::create_dir_all(&source).unwrap();
    std::fs::create_dir_all(&landed).unwrap();
    let mut seed = 0x6d697865646c6f61u64;
    let payload: Vec<u8> = (0..BYTES).map(|_| {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
        (seed >> 32) as u8
    }).collect();
    std::fs::write(source.join("payload.bin"), &payload).unwrap();
    // Only x's interface is impaired, in both directions: nominal direct
    // RTT40ms, 8Mbit/s each direction, explicit40ms rate queue, zero random loss.
    x.dev.default_iface().unwrap().set_condition(
        LinkCondition::new().latency_ms(20).rate_mbit(8).buffer_ms(40),
        LinkDirection::Both,
    ).await.unwrap();
    let path = source.display().to_string();
    let cwd = path.clone();
    let (rows, bulk, destination, closed) = x.with(move |node| async move {
        let id = node.open_shell_on("y", khor_node::shell_start::ShellStart {
            token: "netlab-resident-mixed-workload-v1".into(), title: "netlab echo".into(),
            cwd, command: "stty -echo; while IFS= read -r line; do printf 'NETLAB-REPLY-%s\\n' \"$line\"; done".into(), cols: 100, rows: 30,
        }).await.expect("a scratch remote PTY");
        let (addr, cookie) = node.reach("y", &id).await.expect("the resident's terminal pipe");
        let socket = tokio::task::spawn_blocking(move || host::connect_terminal(&addr, &cookie, 100, 30))
            .await.unwrap().expect("current terminal protocol handshake");
        socket.set_nonblocking(true).unwrap();
        let mut stream = TcpStream::from_std(socket).unwrap();
        stream.write_all(&framed(&host::ClientOp::Ready)).await.unwrap();
        tokio::time::timeout(Duration::from_secs(10), echo(&mut stream, "warmup-unique"))
            .await.expect("bounded PTY startup").expect("a real shell response, not input echo");
        let epoch = Instant::now();
        emit(json!({"schema":1,"revision":REVISION,"event":"manifest","samples_per_operation_per_phase":SAMPLES,
            "phases":["idle_before","bulk","idle_after"],"operations":["directory","terminal"],
            "period_us":PERIOD.as_micros(),"caller_us":CALLER.as_micros(),"bulk_bytes":BYTES,
            "nominal_rtt_ms":40,"mbit_per_direction":8,"queue_ms":40,"random_loss_pct":0,
            "connection_layout":"directory_and_fetch_Main__terminal_Tunnel","slo":false,
            "calibrated":false,"seed":"6d697865646c6f61","diagnostic_recording":true}));
        let (mut rows, _, _) = phase(&node, &mut stream, &path, &landed, epoch, "idle_before", false).await;
        let (during, bulk, destination) = phase(&node, &mut stream, &path, &landed, epoch, "bulk", true).await;
        rows.extend(during);
        let (after, _, _) = phase(&node, &mut stream, &path, &landed, epoch, "idle_after", false).await;
        rows.extend(after);
        drop(stream);
        let closed = tokio::time::timeout(Duration::from_secs(10), node.close_on("y", &id)).await.is_ok_and(|r|r.is_ok());
        (rows, bulk, destination, closed)
    }).await;
    let exact_bytes = destination.as_ref().and_then(|p|std::fs::read(p).ok()).is_some_and(|bytes|bytes==payload);
    let overlap = |operation: &str| rows.iter().filter(|row| row["phase"]=="bulk" && row["operation"]==operation
        && row["started_us"].as_u64().unwrap()>=bulk["started_us"].as_u64().unwrap()
        && row["ended_us"].as_u64().unwrap()<=bulk["ended_us"].as_u64().unwrap()).count();
    let all_answers = rows.len()==SAMPLES*6 && rows.iter().all(|row|row["outcome"]=="ok");
    let direct_after = x.link(&y).await.is_some_and(|l|l.hop==Hop::Direct && l.down_ms.is_none());
    let ok = all_answers && exact_bytes && bulk["moved"]==BYTES as u64 && closed && direct_after
        && overlap("directory")>=30 && overlap("terminal")>=30;
    emit(json!({"schema":1,"revision":REVISION,"event":"complete","functional_ok":ok,
        "samples":rows.len(),"all_answers":all_answers,"exact_bytes":exact_bytes,"closed":closed,
        "direct_after":direct_after,"directory_overlap":overlap("directory"),"terminal_overlap":overlap("terminal"),
        "bulk":bulk,"calibrated":false,"slo":false,"p99_claimed":false,
        "x_journal":x.journal(Some(&y)),"y_journal":y.journal(Some(&x))}));
    result(s, ok, format!("real PTY+directory samples={} exact_bytes={exact_bytes} overlap={}/{} closed={closed}; uncalibrated diagnostic baseline, not a latency SLO",rows.len(),overlap("directory"),overlap("terminal")));
    assert!(ok, "w1: all logical outcomes, actual bulk overlap, intact payload and scratch PTY cleanup");
}
