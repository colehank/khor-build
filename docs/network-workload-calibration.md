# Kernel TCP workload calibration

`netlab-workload-calibrate` is a fixed SSH-only task, using the existing Turing
UID, namespace, lock, toolchain, disk, jobs and deadline guards. It runs only
the ignored `workload::calibration::w0_` test three times sequentially.

The product protocol is `docs/net-optimization/WORKLOAD.md`. The raw validator
requires exactly three logs, 4 warmup and 64 steady echo outcomes, two complete
8 MiB directions, four namespace-local qdisc/interface snapshots, one actual
successful test summary and independently recomputed rates/minimum RTT. Missing,
duplicate or forged records cannot pass. No record is converted into a p99 claim.

Even a green functional/bounds report remains `calibrated:false`: actual qdisc
rate/delay/queue units must be reviewed from the archived readback separately.
This is kernel TCP fixture calibration, not Node/Noq, policy or energy acceptance.
The synthetic log tests execute no local Cargo, namespaces or product binary.
