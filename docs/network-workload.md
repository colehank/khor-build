# Resident workload iteration task

`netlab-workload` is an SSH-only Turing task. It keeps the existing UID, lock,
namespace, disk, toolchain, jobs and deadline guards. It does not change GitHub
dispatch or Mac admission. It runs the fixed ignored `workload::w1_` fixture
sequentially three times, retaining each failure and all raw scenario logs.

The current source protocol is v4. This lane still validates only the fixed
streaming functional baseline; the separate `netlab-workload-observer` lane
validates the guard, timer probes, actual profile and paired recording modes.
Historical v3 source must use its recorded control revision.

The frozen product protocol is `docs/net-optimization/WORKLOAD.md`. This is an
uncalibrated diagnostic baseline, not a performance SLO, capacity calibration,
policy comparison, p99 or phone-energy acceptance. Current production terminal
traffic uses Tunnel; directory and file requests use Main.

The inline evidence validator requires three logs, one manifest/bulk/completion
per log, one successful actual test summary, and exactly 120 unique raw samples
per operation in each of three phases. It recomputes actual bulk overlap and
outcome counts; a missing/duplicate sample, timeout, forged completion or empty
round cannot produce a green result. Successful-sample percentiles are labelled
with their denominator and kept beside all failure counts. No p99 is emitted.

`scripts/tests/test_network_workload.py` runs that validator with synthetic logs
only. It never invokes Cargo, namespaces or product code locally. Actual Linux
compilation and the entire resident workload still require a pinned remote run.
