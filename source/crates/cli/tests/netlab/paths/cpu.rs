//! Admission and evidence for p1s's original one-CPU, two-contender load.

use std::collections::BTreeSet;
use std::fs;
use std::process::Command;
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::scratch::Reaped;

const REVISION: &str = "checked_affinity_v3";

fn allowed(text: &str) -> Result<Vec<usize>, String> {
    let list = text
        .lines()
        .find_map(|line| line.strip_prefix("Cpus_allowed_list:"))
        .ok_or("missing Cpus_allowed_list")?
        .trim();
    let mut cpus = BTreeSet::new();
    for item in list.split(',') {
        let (lo, hi) = item.split_once('-').unwrap_or((item, item));
        let lo: usize = lo.parse().map_err(|_| "invalid CPU ID")?;
        let hi: usize = hi.parse().map_err(|_| "invalid CPU range")?;
        if hi < lo || hi.saturating_sub(lo) > 65535 {
            return Err("invalid CPU range".to_owned());
        }
        cpus.extend(lo..=hi);
    }
    if cpus.is_empty() {
        return Err("empty CPU set".to_owned());
    }
    Ok(cpus.into_iter().collect())
}

fn current_allowed() -> Result<Vec<usize>, String> {
    allowed(&fs::read_to_string("/proc/thread-self/status").map_err(|e| e.to_string())?)
}

#[derive(Debug)]
struct Process {
    pid: u32,
    start: u64,
    ticks: u64,
    threads: Vec<(u32, Vec<usize>)>,
}

impl Process {
    fn read(pid: u32) -> Result<Self, String> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat")).map_err(|e| e.to_string())?;
        // The comm field may itself contain spaces and parentheses.
        let fields: Vec<_> = stat
            .rsplit_once(") ")
            .ok_or("missing stat comm terminator")?
            .1
            .split_whitespace()
            .collect();
        if matches!(fields.first().copied(), Some("Z" | "X")) {
            return Err(format!("process {pid} is no longer running"));
        }
        let field = |index: usize| -> Result<u64, String> {
            fields.get(index).ok_or("short proc stat")?.parse().map_err(|_| "invalid proc stat number".to_owned())
        };
        // After comm: state is field3/index0; utime14, stime15, starttime22.
        let ticks = field(11)?.checked_add(field(12)?).ok_or("CPU ticks overflow")?;
        let mut threads = Vec::new();
        for entry in fs::read_dir(format!("/proc/{pid}/task")).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let tid: u32 = entry.file_name().to_string_lossy().parse().map_err(|_| "invalid thread ID")?;
            let status = fs::read_to_string(entry.path().join("status")).map_err(|e| e.to_string())?;
            threads.push((tid, allowed(&status)?));
        }
        threads.sort_by_key(|(tid, _)| *tid);
        if threads.is_empty() {
            return Err("process has no observable thread".to_owned());
        }
        Ok(Self { pid, start: field(19)?, ticks, threads })
    }

    fn on(&self, cpu: usize) -> bool {
        self.threads.iter().all(|(_, cpus)| cpus.as_slice() == [cpu])
    }

    fn record(&self) -> Value {
        json!({"pid": self.pid, "start_ticks": self.start, "cpu_ticks": self.ticks, "threads": self.threads})
    }
}

fn pin(command: &str, residents: &[u32], cpu: usize) -> Result<(), String> {
    for pid in residents {
        if *pid == 0 || *pid == std::process::id() {
            return Err("only explicit child resident PIDs may be pinned".to_owned());
        }
        let output = Command::new(command)
            .args(["-a", "-p", "-c", &cpu.to_string(), &pid.to_string()])
            .output()
            .map_err(|e| format!("resident affinity command: {e}"))?;
        if !output.status.success() {
            return Err(format!("resident affinity command failed: {}: {}", output.status, String::from_utf8_lossy(&output.stderr)));
        }
        if !Process::read(*pid)?.on(cpu) {
            return Err(format!("resident {pid} did not bind all its threads to CPU {cpu}"));
        }
    }
    Ok(())
}

pub(super) struct BusyCpu {
    cpu: usize,
    allowed: Vec<usize>,
    residents: Vec<Process>,
    contenders: Vec<Reaped>,
    previous: Vec<Process>,
}

impl BusyCpu {
    pub(super) async fn start(residents: &[u32]) -> Result<Self, String> {
        let allowed = current_allowed()?;
        let cpu = allowed[std::process::id() as usize % allowed.len()];
        Self::start_on(residents, cpu).await
    }

    async fn start_on(residents: &[u32], cpu: usize) -> Result<Self, String> {
        let allowed = current_allowed()?;
        if !allowed.contains(&cpu) {
            return Err("selected CPU is not allowed".to_owned());
        }
        if residents.is_empty() {
            return Err("CPU load needs its resident subjects".to_owned());
        }
        pin("taskset", residents, cpu)?;
        let residents: Vec<_> = residents.iter().map(|pid| Process::read(*pid)).collect::<Result<_, _>>()?;
        let mut owner = Self { cpu, allowed, residents, contenders: Vec::new(), previous: Vec::new() };
        for _ in 0..2 {
            let child = Command::new("taskset")
                .args(["-c", &cpu.to_string(), "sh", "-c", "while :; do :; done"])
                .spawn()
                .map_err(|e| format!("CPU contender spawn failed: {e}"))?;
            owner.contenders.push(Reaped(child));
        }
        let began = Instant::now();
        loop {
            let samples = owner.samples()?;
            let ready = samples.iter().all(|p| p.on(cpu)) && owner.exec_ready()?;
            if ready {
                if !owner.previous.is_empty()
                    && owner.previous.iter().zip(&samples).all(|(before, now)| {
                        now.pid == before.pid && now.start == before.start && now.ticks > before.ticks
                    })
                {
                    owner.previous = samples;
                    owner.log("admitted");
                    return Ok(owner);
                }
                if owner.previous.is_empty() {
                    owner.previous = samples;
                }
            }
            if began.elapsed() >= Duration::from_secs(2) {
                return Err("CPU contenders did not execute on the selected CPU".to_owned());
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    fn exec_ready(&self) -> Result<bool, String> {
        for contender in &self.contenders {
            let bytes = fs::read(format!("/proc/{}/cmdline", contender.0.id())).map_err(|e| e.to_string())?;
            let args: Vec<_> = bytes.split(|b| *b == 0).filter(|arg| !arg.is_empty()).collect();
            if args.as_slice() != [b"sh".as_slice(), b"-c".as_slice(), b"while :; do :; done".as_slice()] {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn samples(&mut self) -> Result<Vec<Process>, String> {
        for resident in &mut self.residents {
            let now = Process::read(resident.pid)?;
            if now.start != resident.start || !now.on(self.cpu) {
                return Err(format!("resident {} changed identity or CPU affinity", resident.pid));
            }
            *resident = now;
        }
        let mut samples = Vec::new();
        for contender in &mut self.contenders {
            if contender.0.try_wait().map_err(|e| e.to_string())?.is_some() {
                return Err("CPU contender exited before measurement ended".to_owned());
            }
            samples.push(Process::read(contender.0.id())?);
        }
        Ok(samples)
    }

    fn log(&self, phase: &str) {
        eprintln!("[netlab-cpu] {}", json!({
            "schema": 1, "measurement_revision": REVISION, "phase": phase,
            "cpu": self.cpu, "allowed_ids": self.allowed,
            "residents": self.residents.iter().map(Process::record).collect::<Vec<_>>(),
            "contenders": self.previous.iter().map(Process::record).collect::<Vec<_>>(),
        }));
    }

    pub(super) fn verify(&mut self, phase: &str) -> Result<(), String> {
        let samples = self.samples()?;
        if !self.exec_ready()? || samples.len() != 2 || !samples.iter().all(|p| p.on(self.cpu)) {
            return Err("CPU contender execution or affinity changed".to_owned());
        }
        for (before, now) in self.previous.iter().zip(&samples) {
            if now.pid != before.pid || now.start != before.start || now.ticks <= before.ticks {
                return Err("a CPU contender did not make real CPU progress".to_owned());
            }
        }
        self.previous = samples;
        self.log(phase);
        Ok(())
    }
}

#[test]
fn allowed_ids_are_explicit_and_invalid_lists_fail_closed() {
    assert_eq!(allowed("Cpus_allowed_list:\t3,7-8,12\n").unwrap(), [3, 7, 8, 12]);
    for bad in ["", "Cpus_allowed_list:", "Cpus_allowed_list: 5-3", "Cpus_allowed_list: x", "Cpus_allowed_list: 1,,2"] {
        assert!(allowed(bad).is_err(), "must reject {bad:?}");
    }
}

#[tokio::test]
async fn checked_load_proves_real_cpu_progress_and_reaps_its_contenders() {
    let first = Reaped(Command::new("sleep").arg("10").spawn().unwrap());
    let second = Reaped(Command::new("sleep").arg("10").spawn().unwrap());
    let mut load = BusyCpu::start(&[first.0.id(), second.0.id()]).await.unwrap();
    let pids: Vec<_> = load.contenders.iter().map(|p| p.0.id()).collect();
    tokio::time::sleep(Duration::from_millis(300)).await;
    load.verify("calibration").unwrap();
    drop(load);
    assert!(pids.iter().all(|pid| !std::path::Path::new(&format!("/proc/{pid}")).exists()));
    crate::lab::result("cpu-admit", true, "two checked residents and two real CPU contenders on one allowed CPU; contenders reaped");
}

#[tokio::test]
async fn failed_pinning_or_disallowed_cpu_cannot_admit_load() {
    let child = Reaped(Command::new("sleep").arg("10").spawn().unwrap());
    let before = Process::read(child.0.id()).unwrap();
    let cpu = current_allowed().unwrap()[0];
    let failed = pin("false", &[child.0.id()], cpu);
    assert!(failed.is_err());
    assert!(BusyCpu::start_on(&[child.0.id()], usize::MAX).await.is_err());
    let after = Process::read(child.0.id()).unwrap();
    assert_eq!(before.start, after.start);
    assert_eq!(before.threads, after.threads);
    crate::lab::result("cpu-reject", true, "nonzero pin command and disallowed CPU rejected; owned child affinity unchanged");
}
