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

## Registered worker

- Runner: `khor-mac-mini-m4` (repository runner ID 2).
- Labels: `khor-mac-mini`, `khor-dev`; no default routing labels.
- Installer: official ARM64 runner 2.338.2, SHA256 `25a065af409ffa317ab4d5a3b614dd15fc9c40f2172bc1b35a1b1258e919a388`.
- Provisioning source: `infra/register_runner.py`, identical to the helper used for this registration.
- Live validation: [Actions smoke](https://github.com/colehank/khor-build/actions/runs/41707655972), successful on runner ID 2.

Run the smoke check from a trusted machine with repository access:

```sh
gh workflow run worker-smoke.yml --repo colehank/khor-build
```

Inspect the service with `sudo launchctl print system/com.khor.github-runner.macmini`.
After checking that no job is running, stop it with `sudo launchctl bootout system/com.khor.github-runner.macmini`.
Start it with `sudo launchctl bootstrap system /Library/LaunchDaemons/com.khor.github-runner.macmini.plist`.

The service runs as z13 and does not depend on an SSH login remaining connected.
It is configured to launch when the operating system starts; a full machine reboot has not been tested.
The existing public Khor quality and release workflows have not been migrated to this runner.
