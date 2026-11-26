# Transparent macOS WebKit diagnostic

Run on a dedicated macOS test host with an active desktop. The source-only
coordinator must not compile or launch it locally.

```
python3 apps/gui/scripts/mac-glass-probe/run.py <verified-release-lab> <external-output-directory>
```

The release artifact must contain exactly one `assets/main-*.css` stylesheet.
Verify its manifest and hash before running. The temporary verifier workflow
pins rc.17's source, tree and exact stylesheet hash independently of this script.

The probe compiles a disposable Swift executable and opens two fixture windows.
It uses a transparent WKWebView, the production stylesheet, light/dark native
appearance, and the same optional WindowServer background blur as the App.
Magenta/cyan native backgrounds show whether each surface responds to what is
behind the window. The alternate fixture state disables CSS backdrop filters
for comparison; it is broader than the product's navigation/tab-only fallback.

`result.json` and the PNGs are diagnostic measurements, not a claim that an
installed Khor App passed. Screen capture requires permission on the test host.
No user store, credentials, preferences, session or terminal is opened. macOS
15.8 results cannot substitute for the reported macOS 15.2 device.
