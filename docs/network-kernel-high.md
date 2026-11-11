# High-BDP kernel calibration

`netlab-workload-high-calibrate` uses the unchanged SSH executor resource,
UID, lock, namespace and deadline guards. It checks source format, explicitly
disables diagnostics, and runs exactly three serial fresh instances of
`workload::calibration::h0_ --ignored` with one test thread. All later cases
remain after a failure, and source-owned validation cannot overwrite the first
failing status.

The exact source supplies `scripts/netlab-kernel-high.py`, which checks the
predeclared 300ms/50Mbit geometry, 96MiB payload in both directions, four
separately retained warmup echoes and all 64 steady echoes. The independent
TCP server belongs to the parent test; no resident or Noq is started. Actual
qdisc JSON/text/interface counters and model bounds are mandatory.

Passing this task only validates the kernel model. It does not establish Node
policy performance, high-profile mixed workload, fairness, p99, phone, energy or
full N3 acceptance. This fixed task must not overlap a timed Turing job.
