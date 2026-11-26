# Next round: session priority independent of grouping

Status: implementation authorized by the user on 2026-11-18 and now active.
Implementation baseline: main329de18a67905cf843c849b9754fe91f67235c07.
Latest instruction: implement this round; include cross-device GUI drafts and globally synchronized unread/read progress.

## Confirmed behavior

- One priority order for every session list and every grouping, including inside the pinned section: Blocked > Errored > Done > Busy > Idle > Failed. Unknown/stale state must not be promoted as a live actionable state without evidence.
- Within equal priority, use most recent meaningful activity, then stable identity for deterministic ties.
- Pinned section remains first. Within it use exactly the same priority comparator.
- Ordinary groups compare their highest visible session priority, then most recent meaningful activity, then stable group identity. One actionable session suffices; do not weight by group size.
- Derive group ordering after filtering; rows already placed in pinned section do not also affect their former ordinary group.
- Meaningful activity: user submission, task/turn start, real status transition (approval/error/completion). Stream chunks, terminal redraw, heartbeat, polls and duplicate state delivery must not bump ordering.
- Recompute on priority decrease as well as increase; recently resumed Busy must not outrank another Blocked.
- Defer visible reordering during scroll, held pointer/touch, open row menus or keyboard row interaction; preserve scroll anchor, selected identity and collapsed-group state.
- Grouping chooses membership only: none / machine / category / directory / state. Priority comparator is shared backend policy, not copied per view.
- Parent/child rows remain a tree. Surface a child requiring attention through a derived parent priority/reason without falsifying the parent's own execution state.

## Validation when resumed

Backend: all grouping modes, pinned/nonpinned, tie determinism, priority up/down, filter effects, unknown/stale state and meaningful-event timestamps.
GUI/Lab: root, workspace and machine session hosts; group promotion/demotion; pinned internal order; children; menu/touch/keyboard/scroll stability. Verify actual source artifact remotely; local source-only restriction remains.
Implementation and remote verification are authorized. Main integration and release need this round's later instruction.

## Draft attention addition (2026-11-18)

User requested: a session with partly edited, unsent content should gain list
priority. Show `[草稿]` in red followed by normally colored draft text. Desired
coverage includes GUI conversations, shell terminals and agent TUIs. Implementation is active in the isolated session-attention worktree; validation is pending.

Confirmed by the user on 2026-11-18: Blocked > Errored > Draft > Done >
Busy > Idle > Failed. Draft is an attention overlay, not a seventh execution
state: a busy session remains busy, and approval/error outranks its draft.
Apply the same effective priority within pinned rows, all groups and parent
attention summaries.

Confirmed presentation: retain the session title; replace the second-line preview
with a red `[草稿]` prefix and one-line draft excerpt in the ordinary preview
color, ellipsized as necessary. Sending or explicitly clearing the draft removes
the overlay and restores the normal preview/priority. Creating a draft promotes
the row; continuous typing updates the excerpt without per-keystroke reordering.
Pending attachments without text should also count; an attachment-only summary
is a proposal, not yet confirmed wording.

Current implementation evidence: ChatView and MachineChat use useSessionDraft
and a retained local SessionDraftStore. TerminalPane forwards onData immediately
to the PTY; its predictive-echo queue is transport speculation, not the remote
application's complete editable input. Shell prompt/command markers exist, but
a reliable live editing-buffer report is not established by those markers.

GUI can expose existing draft metadata. Shell requires authoritative editing
buffer integration. TUI needs provider/input integration and verification;
Enter may insert a newline, completion/history may change the buffer, and
programs have different submission rules. Do not infer a universal draft by
concatenating keystrokes or extracting arbitrary terminal output. Unsupported
TUI draft visibility remains an explicit capability limitation.

Confirmed: GUI drafts synchronize across devices in this round. Validate concurrent edits and delivery receipts. Session read progress must clear unread on every device and after offline rejoin, without acknowledging unseen later content. Open: exact shell/TUI integrations and capability coverage. Preserve
current input timing and native TUI editing behavior while adding detection.

### State-machine implementation proposal

Use explicit, orthogonal draft lifecycle transitions; keep the existing
execution state authoritative and independent. Do not enumerate every
execution-state/draft-state combination. Derive effective attention rank from
execution, verified draft presence and meaningful timestamps in one pure rule;
derive group priority from those same results.

Separate the current editable buffer (unknown/unsupported, empty, editing) from
immutable submitted snapshots and their delivery lifecycle (pending, confirmed,
failed, cancelled). Submission must not prevent a newer editable draft. A
receipt can retire only its matching token/revision; an old acknowledgement
must never erase newer input, and failure recovery must not overwrite it.
Use actual existing receipt semantics for each transport rather than inventing
confirmation from an Enter key or echoed output.

GUI, shell and TUI adapters report authoritative draft changes/submission
evidence into this shared model. Missing support or lost ownership is unknown,
not an empty draft. Draft appearance, clear and confirmed submission drive
attention changes; continuous text edits update the excerpt with stable list
interaction behavior. Final lifecycle details will be verified against the
existing composer/receipt implementation when work resumes.

## Implementation checkpoint 2026-11-18

First source checkpoint 69d82fd6e on feat/session-attention-20261118. Do not merge
or ship yet. DraftDoc uses observed-remove immutable editor revision identities,
retains concurrent alternatives and exact submission receipts, and shares the existing session-metadata (`titles`) CRDT pile and wire name. Old peers relay the additional maps without understanding them. GUI edits are durably queued locally;
legacy text migration is one-time and closing tabs no longer discards unsent text.
The backend assigns global attention ordinals; filtered lists group the surviving
ordinals, and interaction freezes positions while updating current row content.

Read intent retries now survive a frontend restart. Added actual two-node QUIC
coverage for draft alternatives and for old reads versus newly completed turns.
Shell draft evidence is currently implemented for zsh through ZLE pre-redraw and
finish hooks. Bash/arbitrary full-screen TUI editors remain unknown; do not infer
their contents from the predictor or terminal drawing. Khor-owned GUI text composers are covered. Attachment-only draft ranking remains an unimplemented proposal and is not part of the current claim.

Child status summaries are bounded background requests; their version-qualified
attention reports replicate without changing the parent's execution state. Active
waiting/working children may raise the parent; terminal children stop raising it.
All changes still require exact-source remote checks and Lab verification.

Compatibility follow-up: drafts and child reports share the existing session-metadata
pile/wire name (`titles`); no additional document name is sent to older peers.
The compatibility regression passes the new maps through a legacy TitleDoc and
renames concurrently, proving the opaque fields survive. Draft creation time is
stable across keystrokes and participates in attention recency without changing
the execution/read timestamp. Newer input and a concurrent send receipt are also
covered by a two-replica merge test.
