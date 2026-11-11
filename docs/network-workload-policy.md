# Response priority comparison

`netlab-workload-policy` retains the original UID, shared lock, namespace, tool,
disk, jobs and deadline guards. It checks format, runs the real observer
ownership calibration, then runs three complete buffered v4 workloads serially.
Lifecycle and sampling diagnostics are disabled explicitly. A failed guard skips
the workloads; a failed workload retains later outcomes and the first failing
status is preserved through source-owned validation.

The exact source parser checks all 720 requests per case, actual traffic/profile,
the bounded timer/logger and intact payload. It reports the full phase population
and the population that actually started while the file transfer was active.
It reads only the declared zero/minus-one response policy and unchanged prefetch
eight from the compiled source. Actual priority getter results are established
by a separate lifecycle cohort, with its logging cost retained.

The frozen block order and source pairs live in the product's net optimization
documents. This task provides one block; it does not select a winner or establish
high-BDP, fairness, device-scale, CPU/energy, phone or full N3 acceptance.
