# Air access and transparent navigation follow-up

Base: main `df47c140`. Implementation branch: `fix/air-session-glass-20261116`.
The preceding rc.17 release remains complete; this follow-up has not shipped.

## Restored access

The root holder's signed roster revoked the current Air identity. It was not a
pending approval, which explains why the holder had no pending notification.
The old refusal combined both situations into one misleading message.

After the user's explicit approval, normal one-time pairing through the root
holder restored the same identity. No identity reset, root recovery, App
restart, session restart or preference changes were needed. Air-to-MacLab
directory access and the existing terminal's connection entry succeeded.
The user then confirmed the original session opens normally on Air.
The signed roster does not establish who originally requested revocation.

`Node::admit` now distinguishes waiting approval from revoked access. Admission
rules and protocol structures are unchanged. Run `43897246750` proves the old
message fails the new assertion, then passes all 20 trust and 7 catalog tests.

## Transparent macOS navigation

Air reports macOS 15.2; only navigation and tab capsules were opaque while the
rest of the native window was transparent. Native WK diagnostics using rc.17's
exact CSS ran on macOS 15.8.2 and 26.7.2 (`43897651143`). They measured different
compositing under an additional CSS backdrop, but did not reproduce Air's
15.2 opacity. The measurements are not Air acceptance.

The compatibility candidate lets transparent macOS 15 navigation and outer tab
capsules use the native window material without another CSS backdrop filter.
Shared tint, selected ink, rim, opacity and geometry stay unchanged. Ordinary
glass controls, sticky list titles, solid windows, other operating systems,
macOS 26, Web and Mobile retain the existing path. The native OS version emits
the capability; no browser User-Agent detection or user setting is added.

The full appearance gate additionally checks 18 platform/capability/material
combinations. Run `43900245994` proves the old CSS/renderer fails the new native
chrome assertion, then passes the complete appearance gate; the actual native
App compiles on macOS 15 and 26. These checks use product source `443f652b`.
Official GUI/settings validation `43898898350` passed the frontend build,
full GUI gates, App Lab cross-screen checks and source hot-update checks.
The final follow-up commit changes only documentation and formatting of the
new trust test; the tested product UI and native sources remain `443f652b`.

## Remaining actual-device acceptance

A newly built App must be checked on the reported Air 15.2: dark and light,
clear and frost at approximately 20%, navigation and session tabs over a
recognizable desktop background; switch back to solid and confirm normal
rendering. Existing terminal content, size and scroll position must remain.
Web Lab and newer macOS CI cannot close this item.
