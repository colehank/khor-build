//! Passive desktop cost fixture. The installed product is never opened.
#![cfg(target_os = "linux")]
#[path = "@ROOT@/crates/cli/tests/netlab/lab.rs"]
mod lab;
#[path = "@ROOT@/crates/cli/tests/scratch/mod.rs"]
mod scratch;

use patchbay::{Lab, RouterPreset};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

#[ctor::ctor(unsafe)]
fn userns() {
    patchbay::init_userns().expect("private unprivileged namespaces");
}

fn proc_sample(pid: i32) -> Value {
    let root = std::path::PathBuf::from(format!("/proc/{pid}"));
    let raw = std::fs::read_to_string(root.join("stat")).unwrap();
    let fields: Vec<_> = raw
        .rsplit_once(") ")
        .unwrap()
        .1
        .split_whitespace()
        .collect();
    let number = |index: usize| fields[index].parse::<u64>().unwrap();
    let status = std::fs::read_to_string(root.join("status")).unwrap();
    let value = |key: &str| {
        status
            .lines()
            .find_map(|line| line.strip_prefix(key))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<u64>()
            .unwrap()
    };
    let mut threads = BTreeMap::new();
    for task in std::fs::read_dir(root.join("task")).unwrap().flatten() {
        let Ok(raw) = std::fs::read_to_string(task.path().join("status")) else {
            continue;
        };
        let n = |key: &str| {
            raw.lines()
                .find_map(|line| line.strip_prefix(key))
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u64>()
                .unwrap()
        };
        threads.insert(
            task.file_name().to_string_lossy().into_owned(),
            json!({
                "voluntary": n("voluntary_ctxt_switches:"),
                "involuntary": n("nonvoluntary_ctxt_switches:")
            }),
        );
    }
    // Linux stat: state is field 3; utime/stime/starttime are 14/15/22.
    json!({"pid":pid,"start_ticks":number(19),"utime_ticks":number(11),
        "stime_ticks":number(12),"rss_kib":value("VmRSS:"),
        "threads_count":value("Threads:"),"threads":threads,
        "executable":std::fs::read_link(root.join("exe")).unwrap()})
}

async fn interfaces(node: &lab::LabNode) -> Value {
    node.with(|_| async {
        let mut command = tokio::process::Command::new("ip");
        command
            .kill_on_drop(true)
            .args(["-s", "-j", "link", "show"]);
        let output = tokio::time::timeout(Duration::from_secs(2), command.output())
            .await
            .expect("bounded namespace snapshot")
            .unwrap();
        assert!(output.status.success());
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    })
    .await
}

async fn snapshots(nodes: &[lab::LabNode], epoch: Instant) -> Value {
    let mut result = Vec::new();
    for node in nodes {
        let started = epoch.elapsed().as_micros();
        let process = proc_sample(node.pid());
        let interfaces = interfaces(node).await;
        result.push(json!({"name":node.name,"device_id":node.id,
            "started_us":started,"ended_us":epoch.elapsed().as_micros(),
            "process":process,"interfaces":interfaces}));
    }
    json!(result)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn idle_personal_mesh_cost() {
    let count: usize = std::env::var("KHOR_COST_NODES").unwrap().parse().unwrap();
    assert!([1, 3, 5].contains(&count));
    let network = std::env::var("KHOR_COST_NETWORK").unwrap();
    assert!(matches!(network.as_str(), "direct" | "relay"));
    let lab = Lab::new().await.unwrap();
    let relay = lab::relay(&lab, "idle-cost", "relay", None).await;
    let mut nodes = Vec::new();
    for index in 0..count {
        let name = format!("node{index}");
        let router = lab
            .add_router(&format!("router{index}"))
            .preset(if network == "relay" {
                RouterPreset::IspCgnatSymmetric
            } else {
                RouterPreset::PublicV4
            })
            .build()
            .await
            .unwrap();
        let dev = lab
            .add_device(&name)
            .uplink(router.id())
            .build()
            .await
            .unwrap();
        let env = if network == "relay" {
            vec![("KHOR_RELAY_ONLY", "1")]
        } else {
            vec![]
        };
        nodes.push(lab::node("idle-cost", dev, &name, &[&relay], &env).await);
    }
    lab::mesh(&nodes.iter().collect::<Vec<_>>()).await;
    // Verify every directed application leg before the quiet interval.
    for from in &nodes {
        for to in &nodes {
            if from.id == to.id {
                continue;
            }
            let target = to.name.clone();
            tokio::time::timeout(
                Duration::from_secs(5),
                from.with(move |n| async move { n.ls_of(&target, "/tmp").await }),
            )
            .await
            .expect("preflight directory deadline")
            .expect("preflight directory");
        }
    }
    tokio::time::sleep(Duration::from_secs(30)).await;
    let epoch = Instant::now();
    let before = snapshots(&nodes, epoch).await;
    let quiet_started = epoch.elapsed().as_micros();
    tokio::time::sleep(Duration::from_secs(60)).await;
    let quiet_ended = epoch.elapsed().as_micros();
    let after = snapshots(&nodes, epoch).await;
    // First real operation after idle; never refresh a peer before timing it.
    let mut first_reads = Vec::new();
    for from in &nodes {
        let to = &nodes[(nodes.iter().position(|n| n.id == from.id).unwrap() + 1) % count];
        if from.id == to.id {
            continue;
        }
        let target = to.name.clone();
        let began = Instant::now();
        let answer = tokio::time::timeout(
            Duration::from_secs(2),
            from.with(move |n| async move { n.ls_of(&target, "/tmp").await }),
        )
        .await;
        let ok = matches!(answer, Ok(Ok(_)));
        first_reads.push(json!({"from":from.name,"to":to.name,
            "elapsed_us":began.elapsed().as_micros(),"ok":ok}));
    }
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    eprintln!(
        "[netlab-idle-cost] {}",
        json!({"schema":1,"revision":"idle_personal_mesh_cost_v1",
        "nodes":count,"network":network,"settle_seconds":30,"quiet_seconds":60,
        "quiet_started_us":quiet_started,"quiet_ended_us":quiet_ended,"clock_ticks_per_second":hz,
        "before":before,"after":after,"first_reads":first_reads,
        "diagnostics":false,"wakeups_available":false,
        "context_switches_are_wakeups":false,"phone_energy_accepted":false})
    );
    assert!(
        first_reads.iter().all(|r| r["ok"] == true),
        "every first read keeps its original two-second budget"
    );
}
