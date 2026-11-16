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
"""Remote-only small Rust diagnosis extracted from exact source; not product acceptance."""
from pathlib import Path
import hashlib,json,os,subprocess,tomllib
S=Path.cwd();R=Path(os.environ['REPORT_DIR']);H=R/'rust-loopback-harness';H.mkdir();(H/'src').mkdir()
s=(S/'apps/gui/core/src/web.rs').read_text()
production=s[s.index('#[derive(Clone, Debug)]\npub struct LoopbackLease'):s.index('\n#[cfg(test)]\nmod loopback_tests')]
test=s[s.index('    #[cfg(target_os = "macos")]\n    #[tokio::test]'):s.index('\n}\n\n#[cfg(test)]\nmod navigation_contract_tests')]
assert 'native_webkit_reaches_the_borrowed_loopback_origin' in test
observed_production=production.replace('let Ok((sock, _)) = accepted else { break };','let Ok((sock, peer)) = accepted else { eprintln!("OBS accept error {accepted:?}"); break }; eprintln!("OBS accepted peer={peer} local={:?}", sock.local_addr());').replace('let _ = through(sock, &target, &proxy).await;','let result = through(sock, &target, &proxy).await; eprintln!("OBS through {target} via {proxy}: {result:?}");')
assert observed_production!=production
observed_test=test.replace('let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();','eprintln!("OBS native stderr {}", String::from_utf8_lossy(&output.stderr));\n            let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();')
assert observed_test!=test
stub='mod khor_catalog { pub mod msg { pub fn page_port_taken(port:u16)->String { format!("occupied port {port}") } } }\n'
wrapper=lambda name,p,t: 'mod '+name+' { use crate::khor_catalog;\n'+p+'\n#[cfg(test)] mod loopback_tests { use super::hold_loopback; use tokio::io::{AsyncReadExt,AsyncWriteExt};\n'+t+'\n}\n}\n'
(H/'src/lib.rs').write_text(stub+wrapper('original',production,test)+wrapper('observed',observed_production,observed_test))
(H/'Cargo.toml').write_text('[package]\nname="khor-loopback-diagnostic"\nversion="0.0.0"\nedition="2024"\n[dependencies]\ntokio={version="=1.53.1",features=["full"]}\nserde_json={version="=1.0.151",features=["preserve_order"]}\n')
(H/'Cargo.lock').write_bytes((S/'Cargo.lock').read_bytes())
fixture=S/'apps/gui/core/tests/fixtures/webkit_loopback.swift'
programs={}
for name,body in [('original',fixture.read_text()),('observed',fixture.read_text().replace('"error": error.localizedDescription','"error": error.localizedDescription, "domain": (error as NSError).domain, "code": (error as NSError).code, "underlying": String(describing: (error as NSError).userInfo[NSUnderlyingErrorKey])'))]:
 f=R/('webkit-'+name+'.swift');f.write_text(body);out=R/('webkit-'+name)
 b=subprocess.run(['xcrun','swiftc','-swift-version','5',str(f),'-o',str(out)],capture_output=True,text=True);(R/('swift-'+name+'-build.log')).write_text(b.stdout+b.stderr);assert b.returncode==0
 programs[name]=str(out)
order=['original','observed','observed','original','original','observed']
plan=dict(order=order,source_sha256=hashlib.sha256(s.encode()).hexdigest(),production_sha256=hashlib.sha256(production.encode()).hexdigest(),test_sha256=hashlib.sha256(test.encode()).hexdigest(),original_fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(),product_changes=0,replacements=0,limits=['Minimal extracted Rust module diagnostic, not full original product gate','Only catalog occupied-port text is stubbed; all original transport/test bodies retained','Observer adds logs and NSError fields; no deadlines/retries/acceptance predicates changed'])
(R/'rust-loopback-design.json').write_text(json.dumps(plan,indent=2)+'\n')
with (R/'rust-build.log').open('w') as log:
 built=subprocess.run(['cargo','test','--manifest-path',str(H/'Cargo.toml'),'--no-run'],stdout=log,stderr=subprocess.STDOUT)
assert built.returncode==0
locked=tomllib.loads((H/'Cargo.lock').read_text());original=tomllib.loads((S/'Cargo.lock').read_text());original_ids={(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in original['package']}
used=[(p['name'],p['version'],p.get('source'),p.get('checksum')) for p in locked['package'] if p.get('source')];assert all(p in original_ids for p in used),[p for p in used if p not in original_ids]
(R/'rust-dependency-identities.json').write_text(json.dumps(used,indent=2)+'\n')
rows=[]
for i,name in enumerate(order,1):
 env=dict(os.environ,KHOR_WEBKIT_PROBE=programs[name],CFNETWORK_DIAGNOSTICS='3')
 with (R/f'rust-case-{i}-{name}.log').open('w') as log:
  result=subprocess.run(['cargo','test','--locked','--manifest-path',str(H/'Cargo.toml'),name+'::loopback_tests::native_webkit_reaches_the_borrowed_loopback_origin','--','--exact','--ignored','--nocapture','--test-threads=1'],env=env,stdout=log,stderr=subprocess.STDOUT,timeout=110)
 rows.append(dict(index=i,variant=name,returncode=result.returncode))
 (R/'rust-loopback-results.json').write_text(json.dumps(dict(plan=plan,results=rows,accepted=False),indent=2)+'\n')
 print(json.dumps(rows[-1]),flush=True)
raise SystemExit(0 if all(r['returncode']==0 for r in rows) else 1)

NATIVE_WEBKIT_DIAG
