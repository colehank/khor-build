# DeepSeek Harness integration

## Completed outcome

DeepSeek Harness is available directly to users and Khor agent through the
installed `dsh --profile acp` command. All required integration outcomes are
verified. Branch: `feat/deepseek-harness-20261117`.

- Worktree: `/Volumes/disk0/Khor/worktrees/current/khor-deepseek-harness-20261117`.
- Base: `81cc76c48b48b2900f37c70e0307a47df29dcaa1`.
- DSH implementation runtime source: `ab6b7e43468cf56f3a290ee6f95ad765e414b050`.
  Later feature-branch changes affect only tests, CI and this handoff.
- User approved main integration on 2026-11-17. The merge retains main's
  loopback and attachment updates through
  `2185f9c6a03ea6c09775d34fbbe2c092d284b223`, alongside the complete DSH branch
  through `aca94577ffb607957de84fecf66d2ee05b81d712`.
- Shared ChatView edits auto-merged; both the DSH and attachment patches are
  preserved exactly when projected from the other branch. The icon guide
  retains both additions. Other changed source blobs remain byte-identical
  to their owning branch. Combined Quality run `44012234369`, source
  `c97b7b3ddf22f2d428710a0ae9212bbae1c2c7e3`, passed frontend, GUI interaction,
  all 20 DSH complete-App scenarios and hot-update checks. This documentation
  follow-up does not change the verified application or CI sources.
- The existing `lab-preview/` and `output/` untracked directories are preserved.
  This integration does not release or replace the installed Khor app.

## Shipped behavior

- A shared `dsh` launch default is available without creating a synced agent
  registration. Explicit registrations override it; forgetting one restores
  the default. Missing optional DSH does not produce a doctor warning.
- GUI creation, shared session faces, list rows, tabs and mobile navigation
  use the official Harness whale. The asset geometry matches pinned official
  commit `39163dd30f703a79079e2d2446545cff3c453738`, with monochrome token color.
- Opaque model values remain unchanged on the wire while the GUI displays
  advertised model names. Setup explains execution-computer credentials and
  hides unsupported additional-directory input for official DSH.
- Discovery, launch, listing, explicit recovery and Khor agent delegation use
  the same default/registration precedence. Read-only recovery retains its
  pure snapshot lookup and now includes the shipped default.
- Existing ACP configuration, tools, permissions, cancellation, file delivery,
  canonical Khor history and recovery mechanisms provide the integration.

Usage and upstream limits: [DeepSeek Harness](../DEEPSEEK-HARNESS.md).

## Verification matrix

| Check | Source and evidence | Result |
| --- | --- | --- |
| Full App GUI | `9d086a5e`, Quality `43955048330` | 20 scenarios passed: Chromium/WebKit, four widths (1280/412/390/320), light/dark, and four setup flows; focused creation/configuration/brand/type/unit/mobile/hot-update gates passed |
| DSH defaults and recovery | `ab6b7e434`, workflow `43960937477` | Both controlled regression proofs failed before their fixes, then all three DSH tests passed; 11 sync-agent and six GUI-core agent tests passed |
| ACP lifecycle | Same source/run | 19 passed, two intentionally ignored; configuration, permissions, cancellation and recovery covered |
| Cross-machine user session | `525ffc8db`, workflow `43966829935` | Passed; remote creation, ownership, unknown-agent refusal and the GUI pipe verified |
| Khor runtime delegation | `dfd28a948`, workflow `43974921965` | Passed: actual embedded Khor runtime with deterministic local planner, permission answers, completion, cancellation and remote-task access across resident restart |
| Rust formatting | `36ea25be5`, workflow `43977866194` | Passed after applying the exact CI-generated formatter patch; the delegation source changed only layout and an optional trailing comma after its passing run |
| Installed DSH | Trusted `native-09` then `native-ab6` binaries; `runtime-acceptance-final.json` | Nine real runtime checks passed; final read-only support is `resume`, with zero new protocol prompts in the final recheck |

Individual suites are attributed above. Earlier full native runs failed on
outdated integration fixtures; their passing replacements are explicit here.
The cross-machine fixture now uses the GUI connector and consumes the protocol
Capabilities frame. The delegation fixture starts its resident before work,
uses the normal `stop_serving` barrier and joins it before restart, then waits
for a newly published endpoint rather than accepting a stale file. Existing
hard-abort acceptance paths retain their task-field interface.

The last delegation job passed its functional test but failed formatting; CI's
exact source-only formatting patch was applied and independently checked by
the successful format-only run. Passed functional suites were not repeated.
Frontend reuse verified the artifact source SHA and refused reuse if any
non-fixture/frontend-input source changed.

## Real installed-DSH evidence

Tested CLI: `/opt/homebrew/bin/dsh`, version `0.2.1-rc.5`. Credentials remained
in DSH's existing store on the execution computer.

- Capability initialization and discovery used the actual installed ACP server.
- A file uploaded to the isolated execution workspace contained an unpredictable
  token. Actual DSH tool output and its reply reached canonical Khor history;
  reattaching retained the reply.
- A bounded second prompt was canceled and returned `Cancelled`.
- Native close and explicit recovery resumed the same provider identity and
  preserved Khor history. The final fixed binary projected support `resume`.
- The real embedded Khor agent called `agents`, `delegate` and `task_wait` twice.
  A real DSH child completed a separate token-read task. Its receipt bound to
  the parent run, and the parent received the actual token. One parent
  permission was approved. The parent planner used a synthetic localhost
  credential; the runtime, MCP transport and DSH child were actual.

Four bounded real DSH prompt rounds were used: initial raw read probe, GUI-path
attachment read, cancellation probe and parent delegate. A prompt can involve
multiple provider requests; no cost amount is inferred. Completed paid phases
were retained across acceptance-driver repairs. The final recheck added zero
DSH protocol-prompt records.

The official ACP profile advertises listing/resume/close and HTTP MCP, with
images false in this tested version. It emits committed messages rather than
raw token deltas and does not replay provider transcripts. Khor records its own
conversation history. The proof uses confirmed native close followed by
explicit recovery; unknown execution ownership is not force-restarted.

## Artifacts, review and cleanup

Evidence root: `/Volumes/disk0/Khor/evidence/deepseek-harness-20261117`.

- `validation-summary.json`: completed check/source mapping.
- `runtime-acceptance-final.json`: final nine-check result, source attribution,
  same-identity recovery and no-prompt recheck.
- `runtime-acceptance-baseline09.json`: original paid-call evidence.
- `real-probe.json`, `control-probe.json`, `icon-provenance.json`.
- `native-ab6.log`, `native-fixtures.log`, `native-delegation-rerun.log`,
  `format-final.log` and corresponding status files retain CI observations.
- `native-dfd-format/format.patch`: exact trusted formatter output. Binary
  artifacts have verified SHA-256 and source SHA/tree metadata.
- `lab-final`: successful complete App bundle and all 20 scenario results.
  The 320px dark, 1280px light and 390px setup screenshots were visually checked.

Isolated preview: <http://127.0.0.1:23971/app/?scene=dsh>. It uses fixture data;
real-model evidence is separate above. Existing user Lab 23961 was unchanged.

All test sessions and bridge processes were closed. Cleanup confirmed zero
owned live session metadata and zero remaining owned runtime processes. Local
source-only rules were respected: no local Node, Rust or Xcode build ran.
Twenty-three Python Lab lifecycle/configuration tests also passed locally.

Self-review used explore-codebase, khor-design, review-changes and debug-issue.
The worktree graph is partial/stale, so zero graph flows was not interpreted as
no impact. Direct call-site review and actual runtime acceptance identified the
doctor and read-only recovery omissions; both have behavioral regressions in
the permanent full-node `dsh_` test selection. No independent agent was
spawned, consistent with the serial-execution constraint.
