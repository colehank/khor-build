#!/bin/bash
# Trusted control driver: portable network checks on Linux/Mac; netlab requires Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|net-node|net-vendor|net-wan-build|net-wan-lifetime|net-wan-peer|net-congestion|netlab-measure-check|netlab-cpu-check|netlab-mobility-calibrate|netlab-mobility-node|netlab-mobility-roam|netlab-peer-mobility|netlab-peer-mobility-samples|netlab-nat-rebind|netlab-nat-rebind-lifecycle|netlab-hop-reference|netlab-hop-reference-lifecycle|netlab-hop-ownership|netlab-hop-lifecycle|netlab-smoke|netlab-dynamic|netlab-compare|netlab-hop|netlab-recovery|netlab-stability|netlab-stalls|netlab-diagnostic|netlab-hop-diagnostic|netlab) ;; *) exit 2 ;; esac
mkdir -p "$REPORT_DIR"
cp "$SOURCE_PACKAGE/source.json" "$REPORT_DIR/source.json"
source_dir="${KHOR_NETWORK_SOURCE_DIR:-$RUNNER_TEMP/network-source}"
if [[ -z "${KHOR_NETWORK_SOURCE_DIR:-}" ]]; then
  git clone --no-hardlinks "$SOURCE_PACKAGE/source.git" "$source_dir"
fi
cd "$source_dir"
python3 - <<'PY'
import json, os, platform, subprocess
from pathlib import Path
report = Path(os.environ['REPORT_DIR'])
source = json.loads((report / 'source.json').read_text())
if os.environ.get('KHOR_NETWORK_SOURCE_DIR'):
    cache = json.loads((report / 'source-cache.json').read_text())
    assert cache['path'] == os.getcwd() == os.environ['KHOR_NETWORK_SOURCE_DIR']
    assert cache['tree'] == source['tree'] and cache['tracked_clean'] is True
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == source['build_commit']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], text=True).strip() == source['tree']
(report / 'environment.json').write_text(json.dumps(dict(task=os.environ['DEV_TASK'], system=platform.platform(),
    rust=subprocess.check_output(['rustc', '--version'], text=True).strip(), run_id=os.environ['GITHUB_RUN_ID'],
    control_commit=os.environ['GITHUB_SHA'], jobs=os.environ['CARGO_BUILD_JOBS']), indent=2) + '\n')
PY

python3 - <<'NATIVE_WEBKIT_DIAG'
"""Fixed real preaccept-abort diagnosis, not product acceptance."""
from pathlib import Path
import base64
import hashlib
import json
import os
import subprocess
import tomllib

S = Path.cwd()
R = Path(os.environ["REPORT_DIR"])
H = R / "rust-loopback-harness"
H.mkdir()
(H / "src").mkdir()
s = (S / "apps/gui/core/src/web.rs").read_text()
production = s[s.index("#[derive(Clone, Debug)]\npub struct LoopbackLease"):s.index("\n#[cfg(test)]\nmod loopback_tests")]
test = s[s.index('    #[cfg(target_os = "macos")]\n    #[tokio::test]'):s.index("\n}\n\n#[cfg(test)]\nmod navigation_contract_tests")]
old = "                        let Ok((sock, _)) = accepted else { break };"
assert production.count(old) == 1
baseline = production.replace(old, r"""
                        let (sock, _) = match accepted {
                            Ok(peer) => peer,
                            Err(error) => {
                                eprintln!("ACCEPT_ERROR target={target} kind={:?} raw={:?}: {error}", error.kind(), error.raw_os_error());
                                break;
                            }
                        };""")
candidate = baseline.replace("                                break;", r"""
                                if matches!(error.kind(), std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::Interrupted) {
                                    continue;
                                }
                                break;""", 1)
assert candidate != baseline
needle = r'''            let lease = hold_loopback(host, port, &format!("127.0.0.1:{proxy_port}"))
                .await
                .unwrap();'''
assert test.count(needle) == 1
test = test.replace(needle, needle + r'''
            if host == "[::1]" {
                let address = std::net::SocketAddr::from((std::net::Ipv6Addr::LOCALHOST, port));
                // No await: queue real abortive preconnections before accept polls.
                for _ in 0..16 {
                    let stream = std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_secs(2)).unwrap();
                    socket2::SockRef::from(&stream).set_linger(Some(std::time::Duration::ZERO)).unwrap();
                    drop(stream);
                }
            }
''')
needle = "            let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();"
assert test.count(needle) == 1
test = test.replace(needle, needle + r'''
            if result["body"] != "BORROWED_EXIT" {
                eprintln!("FAILURE_LIVENESS target={} proxy={} forward_finished={:?} proxy_finished={} result={result}", lease.0.target, lease.0.proxy, lease.0.tasks.iter().map(|task| task.is_finished()).collect::<Vec<_>>(), answering.is_finished());
            }
''')
stub = 'mod khor_catalog { pub mod msg { pub fn page_port_taken(port:u16)->String { format!("occupied port {port}") } } }\n'

def wrapper(name, production, test):
    return "mod " + name + " { use crate::khor_catalog;\n" + production + "\n#[cfg(test)] mod loopback_tests { use super::hold_loopback; use tokio::io::{AsyncReadExt,AsyncWriteExt};\n" + test + "\n}\n}\n"

(H / "src/lib.rs").write_text(stub + wrapper("baseline", baseline, test) + wrapper("candidate", candidate, test))
(H / "Cargo.toml").write_text('[package]\nname="khor-loopback-diagnostic"\nversion="0.0.0"\nedition="2024"\n[dependencies]\ntokio={version="=1.53.1",features=["full"]}\nserde_json={version="=1.0.151",features=["preserve_order"]}\nsocket2="=0.6.5"\n')
(H / "Cargo.lock").write_bytes((S / "Cargo.lock").read_bytes())
fixture = R / "webkit-loopback-source.swift"
data = base64.b64decode("aW1wb3J0IEFwcEtpdAppbXBvcnQgTmV0d29yawppbXBvcnQgV2ViS2l0CgovLyBBbiBpc29sYXRlZCBuYXRpdmUgbmV0d29ya2luZyBwcm9iZS4gTm8gYXBwL3VzZXIgd2Vic2l0ZSBkYXRhIGlzIG9wZW5lZC4KZmluYWwgY2xhc3MgUHJvYmU6IE5TT2JqZWN0LCBXS05hdmlnYXRpb25EZWxlZ2F0ZSB7CiAgICB2YXIgd2ViOiBXS1dlYlZpZXchCiAgICB2YXIgZmluaXNoZWQgPSBmYWxzZQogICAgbGV0IG1vZGUgPSBDb21tYW5kTGluZS5hcmd1bWVudHNbMV0KCiAgICBmdW5jIHN0YXJ0KCkgewogICAgICAgIGxldCBjb25maWcgPSBXS1dlYlZpZXdDb25maWd1cmF0aW9uKCkKICAgICAgICBsZXQgc3RvcmUgPSBXS1dlYnNpdGVEYXRhU3RvcmUubm9uUGVyc2lzdGVudCgpCiAgICAgICAgaWYgbW9kZSAhPSAiZGlyZWN0IiB7CiAgICAgICAgICAgIGxldCBlbmRwb2ludCA9IE5XRW5kcG9pbnQuaG9zdFBvcnQoaG9zdDogIjEyNy4wLjAuMSIsIHBvcnQ6IE5XRW5kcG9pbnQuUG9ydChDb21tYW5kTGluZS5hcmd1bWVudHNbM10pISkKICAgICAgICAgICAgdmFyIHByb3h5ID0gUHJveHlDb25maWd1cmF0aW9uKGh0dHBDT05ORUNUUHJveHk6IGVuZHBvaW50LCB0bHNPcHRpb25zOiBuaWwpCiAgICAgICAgICAgIHByb3h5LmFsbG93RmFpbG92ZXIgPSBmYWxzZQogICAgICAgICAgICBpZiBtb2RlID09ICJleHBsaWNpdCIgewogICAgICAgICAgICAgICAgcHJveHkubWF0Y2hEb21haW5zID0gWyIiLCAiMTI3LjAuMC4xIiwgImxvY2FsaG9zdCJdCiAgICAgICAgICAgICAgICBwcm94eS5leGNsdWRlZERvbWFpbnMgPSBbXQogICAgICAgICAgICB9CiAgICAgICAgICAgIHN0b3JlLnByb3h5Q29uZmlndXJhdGlvbnMgPSBbcHJveHldCiAgICAgICAgfQogICAgICAgIGNvbmZpZy53ZWJzaXRlRGF0YVN0b3JlID0gc3RvcmUKICAgICAgICB3ZWIgPSBXS1dlYlZpZXcoZnJhbWU6IENHUmVjdCh4OiAwLCB5OiAwLCB3aWR0aDogNDgwLCBoZWlnaHQ6IDMyMCksIGNvbmZpZ3VyYXRpb246IGNvbmZpZykKICAgICAgICB3ZWIubmF2aWdhdGlvbkRlbGVnYXRlID0gc2VsZgogICAgICAgIHdlYi5sb2FkKFVSTFJlcXVlc3QodXJsOiBVUkwoc3RyaW5nOiBDb21tYW5kTGluZS5hcmd1bWVudHNbMl0pISkpCiAgICAgICAgRGlzcGF0Y2hRdWV1ZS5tYWluLmFzeW5jQWZ0ZXIoZGVhZGxpbmU6IC5ub3coKSArIDIwKSB7CiAgICAgICAgICAgIHNlbGYuZmluaXNoKFsiZXJyb3IiOiAibmF2aWdhdGlvbiB0aW1lb3V0Il0pCiAgICAgICAgfQogICAgfQoKICAgIGZ1bmMgZGVzY3JpYmVGYWlsdXJlKF8gZXJyb3I6IEVycm9yKSAtPiBbU3RyaW5nOiBBbnldIHsKICAgICAgICBsZXQgZXJyb3IgPSBlcnJvciBhcyBOU0Vycm9yCiAgICAgICAgcmV0dXJuIFsKICAgICAgICAgICAgImVycm9yIjogZXJyb3IubG9jYWxpemVkRGVzY3JpcHRpb24sCiAgICAgICAgICAgICJkb21haW4iOiBlcnJvci5kb21haW4sCiAgICAgICAgICAgICJjb2RlIjogZXJyb3IuY29kZSwKICAgICAgICAgICAgInVuZGVybHlpbmciOiBTdHJpbmcoZGVzY3JpYmluZzogZXJyb3IudXNlckluZm9bTlNVbmRlcmx5aW5nRXJyb3JLZXldKSwKICAgICAgICBdCiAgICB9CgogICAgZnVuYyBmaW5pc2goXyByZXN1bHQ6IFtTdHJpbmc6IEFueV0pIHsKICAgICAgICBndWFyZCAhZmluaXNoZWQgZWxzZSB7IHJldHVybiB9CiAgICAgICAgZmluaXNoZWQgPSB0cnVlCiAgICAgICAgdmFyIHZhbHVlID0gcmVzdWx0CiAgICAgICAgdmFsdWVbIm1vZGUiXSA9IG1vZGUKICAgICAgICBsZXQgZGF0YSA9IHRyeSEgSlNPTlNlcmlhbGl6YXRpb24uZGF0YSh3aXRoSlNPTk9iamVjdDogdmFsdWUsIG9wdGlvbnM6IFsuc29ydGVkS2V5c10pCiAgICAgICAgcHJpbnQoU3RyaW5nKGRhdGE6IGRhdGEsIGVuY29kaW5nOiAudXRmOCkhKQogICAgICAgIGZmbHVzaChzdGRvdXQpCiAgICAgICAgTlNBcHBsaWNhdGlvbi5zaGFyZWQudGVybWluYXRlKG5pbCkKICAgIH0KCiAgICBmdW5jIHdlYlZpZXcoXyB3ZWJWaWV3OiBXS1dlYlZpZXcsIGRpZEZpbmlzaCBuYXZpZ2F0aW9uOiBXS05hdmlnYXRpb24hKSB7CiAgICAgICAgd2ViVmlldy5ldmFsdWF0ZUphdmFTY3JpcHQoImRvY3VtZW50LmJvZHkuaW5uZXJUZXh0IikgeyB2YWx1ZSwgZXJyb3IgaW4KICAgICAgICAgICAgaWYgbGV0IGVycm9yID0gZXJyb3IgeyBzZWxmLmZpbmlzaChzZWxmLmRlc2NyaWJlRmFpbHVyZShlcnJvcikpIH0KICAgICAgICAgICAgZWxzZSB7IHNlbGYuZmluaXNoKFsiYm9keSI6IHZhbHVlIGFzPyBTdHJpbmcgPz8gIiJdKSB9CiAgICAgICAgfQogICAgfQoKICAgIGZ1bmMgd2ViVmlldyhfIHdlYlZpZXc6IFdLV2ViVmlldywgZGlkRmFpbFByb3Zpc2lvbmFsTmF2aWdhdGlvbiBuYXZpZ2F0aW9uOiBXS05hdmlnYXRpb24hLCB3aXRoRXJyb3IgZXJyb3I6IEVycm9yKSB7CiAgICAgICAgZmluaXNoKGRlc2NyaWJlRmFpbHVyZShlcnJvcikpCiAgICB9CiAgICBmdW5jIHdlYlZpZXcoXyB3ZWJWaWV3OiBXS1dlYlZpZXcsIGRpZEZhaWwgbmF2aWdhdGlvbjogV0tOYXZpZ2F0aW9uISwgd2l0aEVycm9yIGVycm9yOiBFcnJvcikgewogICAgICAgIGZpbmlzaChkZXNjcmliZUZhaWx1cmUoZXJyb3IpKQogICAgfQp9CgpsZXQgYXBwID0gTlNBcHBsaWNhdGlvbi5zaGFyZWQKYXBwLnNldEFjdGl2YXRpb25Qb2xpY3koLnByb2hpYml0ZWQpCmxldCBwcm9iZSA9IFByb2JlKCkKcHJvYmUuc3RhcnQoKQphcHAucnVuKCkK")
assert hashlib.sha256(data).hexdigest() == "9902bc9974d57c64a5760e896280d7d843a19dc70d9d0a754162d0e10f3408f8"
fixture.write_bytes(data)
program = Path(os.environ["RUNNER_TEMP"]) / "webkit-loopback"
build = subprocess.run(["xcrun", "swiftc", "-swift-version", "5", str(fixture), "-o", str(program)], capture_output=True, text=True)
(R / "swift-build.log").write_text(build.stdout + build.stderr)
assert build.returncode == 0
order = ["baseline", "candidate", "candidate", "baseline", "baseline", "candidate"]
plan = dict(order=order, original_source_sha256=hashlib.sha256(s.encode()).hexdigest(),
            original_production_sha256=hashlib.sha256(production.encode()).hexdigest(),
            fixture_sha256="9902bc9974d57c64a5760e896280d7d843a19dc70d9d0a754162d0e10f3408f8", fixture_component_source="ed245f1ec28ab40363e78b91f69ba4488338e91e",
            fault="16 real IPv6 loopback TCP connections reset with zero linger before accept task polls; native request then unchanged",
            native_deadline_seconds=30, replacements=0, product_changes=0,
            limits=["Extracted Rust with only occupied-port text stub, not full product gate",
                    "All six retained; causal confirmation requires baseline failed native reads with observed aborted/reset accept, and candidate successes",
                    "Failure-only accept/liveness logs; no success probes or CFNetwork diagnostic env"])
(R / "abort-design.json").write_text(json.dumps(plan, indent=2) + "\n")
with (R / "rust-build.log").open("w") as log:
    build = subprocess.run(["cargo", "test", "--manifest-path", str(H / "Cargo.toml"), "--no-run"], stdout=log, stderr=subprocess.STDOUT)
assert build.returncode == 0
locked = tomllib.loads((H / "Cargo.lock").read_text())
original = tomllib.loads((S / "Cargo.lock").read_text())
ids = {(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in original["package"]}
used = [(p["name"], p["version"], p.get("source"), p.get("checksum")) for p in locked["package"] if p.get("source")]
assert all(p in ids for p in used)
(R / "dependency-identities.json").write_text(json.dumps(used, indent=2) + "\n")
rows = []
for i, name in enumerate(order, 1):
    env = dict(os.environ, KHOR_WEBKIT_PROBE=str(program))
    env.pop("CFNETWORK_DIAGNOSTICS", None)
    log = R / f"abort-case-{i}-{name}.log"
    with log.open("w") as output:
        result = subprocess.run(["cargo", "test", "--locked", "--manifest-path", str(H / "Cargo.toml"),
                                 name + "::loopback_tests::native_webkit_reaches_the_borrowed_loopback_origin",
                                 "--", "--exact", "--ignored", "--nocapture", "--test-threads=1"],
                                env=env, stdout=output, stderr=subprocess.STDOUT, timeout=110)
    text = log.read_text()
    observed = "ACCEPT_ERROR" in text and any("kind=" + x in text for x in ["ConnectionAborted", "ConnectionReset", "Interrupted"])
    row = dict(index=i, variant=name, returncode=result.returncode, observed_recoverable_accept_error=observed,
               connection_refused="Connection refused" in text, native_success="test result: ok. 1 passed;" in text)
    rows.append(row)
    confirmed = (len(rows) == 6
                 and all(x["returncode"] != 0 and x["observed_recoverable_accept_error"] and x["connection_refused"] for x in rows if x["variant"] == "baseline")
                 and all(x["returncode"] == 0 for x in rows if x["variant"] == "candidate"))
    (R / "abort-results.json").write_text(json.dumps(dict(plan=plan, results=rows, causal_fault_contract_confirmed=confirmed), indent=2) + "\n")
    print(json.dumps(row), flush=True)
raise SystemExit(0 if confirmed else 1)

NATIVE_WEBKIT_DIAG
