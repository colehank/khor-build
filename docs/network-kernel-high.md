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

`netlab-workload-high-sockets` selects exactly three `hd0_` cases at the
unchanged high profile. Both endpoints observe only their actual transfer socket
with bounded `ss -tinm` calls. Node lifecycle diagnostics remain disabled.
Source-owned validation records observation integrity independently from the
physical bounds: a still-failing Rust calibration keeps its original nonzero
status even when all socket snapshots are verified. This is diagnostic evidence,
not a policy benchmark or acceptance of the high-bandwidth model.

`netlab-workload-high-long` runs one fixed `hl0_` case with 384MiB in each
direction at unchanged physical settings and deadlines. It retains later socket
observations to investigate flow-age effects. One longer diagnostic is not a
replacement for the original three 96MiB calibration cases; physical acceptance
remains false in the diagnostic validator. The original Rust failure status is
still preserved. Confirm at least 8GiB available memory before launch.
