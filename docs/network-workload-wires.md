# Workload stream diagnosis

`netlab-workload-wires` keeps the original SSH, UID, lock, namespace and resource
guards. It checks format, runs the actual observer ownership calibration once,
then runs three complete buffered workloads with lifecycle diagnostics enabled.
A guard failure skips those cases. A case failure retains the remaining cases;
the first failing status survives validation. No sampling diagnostics are enabled.

The exact source owns `scripts/netlab-workload-wires.py`, which checks the full
720-request workload/profile/probe contract and both retained resident logs.
It requires all 360 directory and 64 file-slice replies on matching real streams,
actual ALPN/connection identities and monotonic server stages, plus the one
separate terminal tunnel. Multiple Main connections are reported as observed;
the validator does not force the expected sharing hypothesis to be true.

This is an instrumented diagnostic cohort. Local write completion is not packet
delivery; clocks begin after accept_bi and do not measure the QUIC pre-accept
queue. It does not accept cancellation/host exit, CPU/energy, p99 or the full N3.
