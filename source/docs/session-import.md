# Importing agent sessions

The GUI and `khor import` use the same source-owned ACP catalog. Opening the
catalog reads bounded local snapshots on each execution machine. Explicit
refresh runs the registered ACP command on that machine and calls `initialize`
and paginated `session/list`; it never calls `session/new`, `load` or `resume`.
Claude, Codex and DeepSeek Harness have launch defaults. Other registrations
use exactly the same code and can override a default.

## Admission

| Source evidence | Action |
| --- | --- |
| Already listed under Khor, including a proven launch alias | Omitted |
| Archived | Omitted |
| Archive state unknown | Unavailable |
| Confirmed ended, unarchived, supports load or resume | Import original |
| Running or owner unknown, unarchived, advertises fork | Create independent branch |
| Owner unknown without fork | Unavailable |

Absence from a process scan is never proof that a session is unhosted. ACP v1
does not provide a universal host-occupancy query. Existing positive lifecycle
evidence and Journal reservations remain authoritative. Consequently, a
provider may list history yet offer no admissible import action. Optional
archive/fork support is not inferred from its brand or protocol suffix.

Identity comprises the source machine, registered launch and complete native
session ID. Titles are display text. Identical proven launch aliases deduplicate;
different native IDs with the same title remain separate. Provider archive
filtering and action eligibility are checked again at apply time.

Import publishes an observation row without running the provider. Its first
open rechecks the source and uses the existing epoch-fenced recovery path.
Older observations without the explicit adoption field remain read-only.
A branch calls the provider's `session/fork` and requires a different native
ID. No text-only clone or resume of the original substitutes for fork.
The fork intent is removed from saved startup context: subsequent opens resume
the new branch rather than forking its parent again.

## Retries and transport

The browser commits a bounded IndexedDB caller token before sending an action.
The execution machine holds a cross-process action lock and preserves the
exact request/result under `.khor/import-actions`. A lost result retries the
same token, including after reload. An uncertain started action is refused
instead of creating another branch. These receipts are not session authority;
Journal identity and recovery fences still decide execution ownership.

IPC and authenticated QUIC route to the specified execution home. Incoming
requests cannot forward themselves to another machine. Mutations never use
0-RTT. Catalog caches are local metadata, not synced provider histories.

CLI examples:

```sh
khor import --machine MacLab
khor import --machine MacLab --refresh dsh
khor import --machine MacLab --agent my-agent native-session-id
khor import --machine MacLab --agent my-agent --fork native-session-id --token unique-retry-token
```

Legacy native transcript readers remain for existing histories and terminal
observation. They do not supply the new import dialog's primary discovery path.
Provider credentials and native history files are not copied during discovery.

Verification is in `acp/tests/session_catalog.rs`, Node import policy and
source-identity tests, `node/tests/session_import_wire.rs`, existing recovery
fence regressions, and the real App Lab's `session-import` interaction checks.
The Lab includes unavailable DSH/unknown-owner examples rather than inventing
optional capabilities for those agents.

Permanent regression checks run in the main Quality workflow: `node-native`
checks ACP catalog/fork, source identity, lifecycle fences and resident IPC/QUIC;
`mac-updates` checks the actual macOS subprocess identity/cancellation boundary;
`developer-lab` exercises the real import UI in Chromium and WebKit; `gui`
retains shared dialog geometry, gestures, focus and observation regressions.
The feature-branch workflow was retired after these checks moved into Quality.
