# Focused route contracts on Turing

`net-node-routes` is an SSH-runner-only iteration task. It runs the six
`link::dial_budget::initial_custom::` tests and the two
`hop::tests_offer::published::incoming::` tests, with the existing source,
toolchain, format, cache, owner, resource and exclusive-run guards.
The filters and expected counts are fixed in the trusted control driver.

Both groups run even if the first fails. Each original log is preserved;
`route-contracts.json` binds its SHA-256 and actual passed/failed counts to
the source identity. Missing summaries, zero tests, ignored tests and a
changed test count fail the task even when Cargo exits successfully.

This task is neither the complete twelve-group `net-node` regression nor
a performance or physical-network acceptance run. Its report explicitly
sets `full_regression=false` and `slo=false`. A candidate still needs the
appropriate full regression, vendor, physical and device evidence.
No network timing, observation duration or production threshold is shortened.

The workflow-dispatch task list and Mac admission rules are unchanged.
SSH jobs must pin the control commit that introduced this task. Existing
jobs continue using their recorded control commit; their receipts and
results are not rewritten.

Control verification uses Python Cargo/rustc stubs, never a local compiler
or product binary. The driver tests exercise a successful run, failure of
either group, a zero-test success and a missing summary. The existing
network-driver and SSH-runner tests also pass.
