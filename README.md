# Khor build worker

Private control repository for the rented Mac mini development runner.
It contains no Khor source fork, signing credentials, model credentials, or cluster access.
Public Khor pull requests cannot submit jobs here. Workflows are manually dispatched by repository collaborators.

The runner executes as z13. Its host is not a release trust boundary.
Production releases and signing are rebuilt on GitHub-hosted runners without importing this worker’s compiled caches.

`worker-smoke.yml` verifies scheduling, the scoped toolchain, and the build admission guard.
It does not compile or publish the Khor product.

Mac service: `com.khor.github-runner.macmini`, backed by the official runner `runsvc.sh` entry point.
Build root: `/Users/z13/khor-builder`; run heavy commands via `bin/run-build`.
