# Attachment picker — final Lab revision, 2026-11-17

Branch `fix/attachment-picker-regions-20261117`, base main `84e75c57`.

The panel and nested file/directory content overlays own an opaque
`canvas-solid` surface. Its body has three spaced, aligned sections:

1. 选择文件: local file/directory actions on the first row; other machines'
   avatars and names on the second. Only complete chips that fit remain inline;
   the rest are selectable in the More menu. The local machine is not duplicated.
2. 已选文件: hidden entirely when empty. Selected rows share AttachmentList and
   FileChoiceRow with the existing workspace geometry and size formatter.
3. 发送目录: follows selection directly, with the shared avatar/colon/path row.

Short keyboard panels scroll their body instead of squeezing controls. Long
selected lists retain independent scrolling. Missing native-path sizes are read
from the source through stat_text without staging, uploading, changing selection
or blocking send. Known sizes are reused; unknown directory totals stay unknown.
Desktop terminal's paperclip HeaderSeat is removed. Direct browser/native/internal
drops open the draft; Mobile retains its key row attachment entry.

Both final realApp runs43914447734/e12594ec and43915117547/c0264748 passed14
Chromium/WebKit attachment/GUI/Khor scenarios at1280/412/390/320, builds, unit,
token/mobile static and hot-update checks. The latter additionally forces80px
machine room, checks every overflow machine is reachable and restores inline
choices; it checks empty-section removal, source row separation, all three
section boundaries/order/alignment and nonoverlapping button content. Existing
file size/source, preview, cancel/send, scrolling,232px touch row and unchanged
PTY dimension checks remain. Desktop and narrow screenshots were inspected.

Live App: http://127.0.0.1:23961/app/?scene=terminal&machines=3
Pinned App: http://127.0.0.1:23961/snapshots/c026474864e655bd/app/?scene=terminal&machines=3
Desktop: drag a file into the terminal. Mobile: use the attachment key; scroll
panel content to reach the destination when the keyboard area is short.

Artifact14251060354 SHA2566933ef468c03d5c2c82907835448a1ad78a40738ef14f0e4cabde301106321ce
has verified ZIP CRC, exact sourcec0264748 and tree2beaf89d43335bb9d6f0471f836d8970a5763c4a,
14 error-free scenarios and passed hot-update checks. Only the verified owned
previous Lab server was replaced; historical snapshot URLs remain available.
The final documentation commit leaves tested App/native/crate sources unchanged.

This follow-up is not merged or shipped. Installed platform file pickers and
native drag feel remain separate actual-device checks.

## Eyeglasses preview follow-up

User chose eyeglasses for in-app previews. Product5ac1e0ce replaces the outward
arrow with shared IconPreview, using Khor line weight and the same box. Tooltip
and accessible name say预览. FileChoiceRow shares it across attachment drafts
and directory selection; preview handlers and ContentOverlay remain unchanged.
Run43921867791 passed all14 existing attachment flows plus build/shared gates
and hot-update checks. Artifact14254044348 digest
9bad38d9504c2513feef6923a89d127b41bc52a029ecf0e140d474d9a7becbde
verified against source5ac1e0ce/tree1577bc4c14a6a0ca6039c12eb85399c6c3889ee2.
Desktop1280 and WebKit320 screenshots inspected. Current Lab is now:
http://127.0.0.1:23961/snapshots/5ac1e0ce340e23a2/app/?scene=terminal&machines=3
All37 snapshot URLs are retained; four current entry assets byte-verified.
Only the owned prior Lab process was replaced. No main merge or ship.

## Latest accepted refinements and current Lab

Preview now uses size-6 eyeglasses to balance the size-5 selection disc.
Selected discs share settings' brand-strong and on-brand; unchecked stays outline.
Shared Button compactTouch keeps the mobile task primary face at ctl-lg32 and
its real target at least44. TaskOverlay and TermFiles use the same implementation;
wide desktop stays28. Neutral empty/busy behavior, focus, press and labels remain
owned by the real button. Destination heading and accessible name now say发送到.

Final source8882a960 (product code70ac67cb) passed run43941997857: all14 existing
Chromium/WebKit attachment/GUI/Khor cases, hot-update/build/shared gates, plus
shared dialog bounds/actions and compact face/touch checks. The mobile send
flow clicks the transparent strip above the green face and successfully sends.
Desktop1280, lightWebKit320 and darkChromium412 screenshots inspected.
Artifact14263193582 digest14b1bf78ba43bdfbc20ee39be64ce240b09557e0433c4764fb29cb4ef0c262cf
verified against exactsource/tree6aec2568634d6099cb6300635ef669738eb90cf8.

Current pinned App: http://127.0.0.1:23961/snapshots/8882a96033340cb5/app/?scene=terminal&machines=3
Four entry assets byte-verified and40 snapshots retained. Only owned Lab server
replaced. Temporary verifier b3c259ee must never merge. This UI follow-up remains
unmerged/unshipped; no user App or device changed.

## GUI plus and direct file drops — current Lab

The shared SessionAdd plus now opens AttachmentPicker directly in both ACP GUI
and Khor chat; the intermediate add menu and its find/export items are removed.
No search/export items are added to the model menu. Standard keyboard find and
read-only/raw-source utilities remain separate from attachment actions.
Readable conversation export no longer instantiates a GUI hook or notice.
ChatView browser File drops and native OS-path drops now open the same retained
picker after adding files. Khor already did so. Selection stays local until Send.
No per-agent picker was copied, and protocol file constraints remain intact.

Sourcef4aa291d (product187336e1) passed43946813584, all14 attachment/GUI/Khor
Chromium/WebKit flows, hot updates, shared dialogs and frontend gates. Desktop
browser drops in GUI/Khor, retained message text, no premature transfer, previews,
retry/send and direct plus across widths are covered. Native OS drop integration
is wired; actual installed-package drop behavior still needs device QA.
Artifact14266750160 digest18ccd1d92e907cdb7cdd27f7d79f3f92c8600d195787f123dc3bd052d679efe2
verified against exactsource/tree0dc9c7ad2cd5783c4b9a9e65d8f2038312a7d001.
Current GUI: http://127.0.0.1:23961/snapshots/f4aa291d46192582/app/?scene=gui&machines=3
Four current entry assets byte-verified and41 snapshots retained. Only the owned
previous static Lab process replaced. No main merge, release or user App restart.

## Main integration — 2026-11-17

User approved the Lab and explicitly authorized committing to main. Merge
7fef92d732a5123fc1587a1e73565ce44d3fef23 integrates approved branch99b998e8
onto main1b20eee9, preserving the newer network and loopback-listener fixes.
There were no overlapping changed files or conflicts. All20 approved files were
byte-identical to the feature branch at the merge; all other main files stayed
unchanged. This closeout only records that integration.

The GUI verification remains43946813584:14 scenarios, hot updates, shared dialogs
and frontend gates. Main Quality is triggered by the push to verify the combined
tree. Existing Lab sourcef4aa291d and historical snapshots stay available. Native
OS drop behavior still requires installed-package device QA. This authorization
covers main integration; no release tag or installation is part of it.
