//! Bounded fixture recording, cancellation population and runtime scheduling.
//! These resources belong to the test, never to a production resident.
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::{CALLER, REVISION, at, emit};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Streaming,
    Buffered,
}

impl Mode {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::Buffered => "buffered",
        }
    }
}

pub(super) struct Owned<T>(pub(super) tokio::task::JoinHandle<T>);

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

struct Store {
    epoch: Instant,
    phase: String,
    mode: Mode,
    rows: Vec<Value>,
    active: usize,
    overflow: bool,
    closed: bool,
}

impl Store {
    fn push(&mut self, mut row: Value) {
        if self.rows.len() == 512 {
            self.overflow = true;
            return;
        }
        row["mode"] = json!(self.mode.name());
        if self.mode == Mode::Streaming {
            emit(row.clone());
        }
        self.rows.push(row);
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        if self.closed {
            return;
        }
        // Completed/cancelled rows survive ordinary future cancellation or a
        // panic. An abrupt process kill still makes the missing summary fail.
        if self.mode == Mode::Buffered {
            for row in &self.rows {
                emit(row.clone());
            }
        }
        emit(
            json!({"schema":1,"revision":REVISION,"event":"logger_aborted",
            "phase":self.phase,"mode":self.mode.name(),"rows":self.rows.len(),
            "active":self.active,"overflow":self.overflow}),
        );
    }
}

#[derive(Clone)]
pub(super) struct Log(Arc<Mutex<Store>>);

impl Log {
    pub(super) fn new(epoch: Instant, phase: &str, mode: Mode) -> Self {
        Self(Arc::new(Mutex::new(Store {
            epoch,
            phase: phase.into(),
            mode,
            rows: Vec::with_capacity(256),
            active: 0,
            overflow: false,
            closed: false,
        })))
    }

    pub(super) fn start(&self, operation: &str, index: usize, scheduled: Instant) -> Attempt {
        let mut store = self.0.lock().unwrap();
        assert!(!store.closed);
        store.active += 1;
        Attempt {
            log: self.clone(),
            operation: operation.into(),
            index,
            scheduled_us: scheduled
                .duration_since(store.epoch)
                .as_micros()
                .try_into()
                .unwrap(),
            started_us: at(store.epoch),
            finished: false,
        }
    }

    pub(super) fn push(&self, row: Value) {
        self.0.lock().unwrap().push(row);
    }

    pub(super) fn finish(&self) -> (Vec<Value>, bool) {
        let mut store = self.0.lock().unwrap();
        assert_eq!(
            store.active, 0,
            "every measured future released its attempt"
        );
        assert!(!store.closed);
        if store.mode == Mode::Buffered {
            for row in &store.rows {
                emit(row.clone());
            }
        }
        store.closed = true;
        (store.rows.clone(), store.overflow)
    }
}

pub(super) struct Attempt {
    log: Log,
    operation: String,
    index: usize,
    scheduled_us: u64,
    started_us: u64,
    finished: bool,
}

impl Attempt {
    fn complete(&mut self, outcome: &str, error: Option<String>) -> Value {
        let mut store = self.log.0.lock().unwrap();
        let ended_us = at(store.epoch);
        let row = json!({"schema":1,"revision":REVISION,"event":"sample",
            "phase":store.phase,"operation":self.operation,"index":self.index,
            "scheduled_us":self.scheduled_us,"started_us":self.started_us,
            "ended_us":ended_us,"elapsed_us":ended_us-self.started_us,
            "schedule_lag_us":self.started_us.saturating_sub(self.scheduled_us),
            "caller_us":CALLER.as_micros(),"outcome":outcome,"error":error,
            "mode":store.mode.name()});
        store.active -= 1;
        self.finished = true;
        store.push(row.clone());
        row
    }

    pub(super) fn finish(mut self, outcome: &str, error: Option<String>) -> Value {
        self.complete(outcome, error)
    }
}

impl Drop for Attempt {
    fn drop(&mut self) {
        if !self.finished {
            let outcome = if std::thread::panicking() {
                "aborted"
            } else {
                "cancelled"
            };
            self.complete(outcome, Some("measurement future dropped".into()));
        }
    }
}

/// Same observer runtime as the RPCs, with no task and no printing per tick.
/// Skip is explicit: gaps in scheduled times remain in the raw timestamp pairs.
pub(super) async fn probe<T>(
    epoch: Instant,
    start: Instant,
    phase: &str,
    mode: Mode,
    measured: impl Future<Output = T>,
) -> (T, Value) {
    const PERIOD: Duration = Duration::from_millis(20);
    const CAP: usize = 20000;
    let mut ticks = Vec::with_capacity(1024);
    let mut interval = tokio::time::interval_at((start + PERIOD).into(), PERIOD);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    tokio::pin!(measured);
    let value = loop {
        tokio::select! {
            result=&mut measured => break result,
            scheduled=interval.tick(), if ticks.len()<CAP => {
                let planned=scheduled.into_std().duration_since(epoch).as_micros();
                ticks.push([u64::try_from(planned).unwrap(),at(epoch)]);
            }
        }
    };
    let row = json!({"schema":1,"revision":REVISION,"event":"probe","phase":phase,
        "mode":mode.name(),"period_us":PERIOD.as_micros(),"ticks_us":ticks,
        "started_us":start.duration_since(epoch).as_micros(),"ended_us":at(epoch),
        "capped":ticks.len()==CAP,"capacity":CAP,"missed_tick_behavior":"skip"});
    (value, row)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "fixture cancellation/ownership calibration, separate from network SLOs"]
async fn w3_cancelled_observer_futures_keep_outcomes_in_both_recording_modes() {
    for mode in [Mode::Streaming, Mode::Buffered] {
        let epoch = Instant::now();
        let log = Log::new(epoch, "guard_calibration", mode);
        let cancelled = log.clone();
        let (started, ready) = tokio::sync::oneshot::channel();
        let owner = Owned(tokio::spawn(async move {
            let _attempt = cancelled.start("cancelled", 0, epoch);
            started.send(()).unwrap();
            std::future::pending::<()>().await;
        }));
        tokio::time::timeout(Duration::from_secs(1), ready)
            .await
            .expect("the owned task starts within its fixture budget")
            .unwrap();
        drop(owner);
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let active = log.0.lock().unwrap().active;
                if active == 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("dropping the owner cancels its actual task");
        let finished = log.start("completed", 1, epoch).finish("ok", None);
        assert_eq!(finished["outcome"], "ok");
        let (rows, overflow) = log.finish();
        assert!(!overflow);
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.iter().filter(|r| r["outcome"] == "cancelled").count(),
            1
        );
        assert_eq!(rows.iter().filter(|r| r["outcome"] == "ok").count(), 1);
        assert!(
            rows.iter()
                .all(|r| r["ended_us"].as_u64().unwrap() >= r["started_us"].as_u64().unwrap())
        );
    }
    emit(
        json!({"schema":1,"revision":REVISION,"event":"observer_guard_calibration",
        "modes":2,"cancelled":2,"completed":2,"passed":true,"slo":false}),
    );
    crate::lab::result(
        "w3",
        true,
        "two actual owned task cancellations and two completions retained in both modes; not a product cancellation SLO",
    );
}
