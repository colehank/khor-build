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
# Preserve the mtime of the same isolated web placeholder across cached builds.
for name, content in {
    'apps/gui/dist/assets/index-netlab.js': '// network test fixture\n',
    'apps/gui/dist/index.html': '<!doctype html><script type="module" src="/assets/index-netlab.js"></script>\n',
}.items():
    path = Path(name)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text() != content:
        path.write_text(content)
PY
repeat_netlab() {
  local status=0 repeat
  for repeat in 1 2 3; do
    printf '[netlab-repeat] %s/3\n' "$repeat"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh "$@" || status=$?
  done
  return "$status"
}
check_format() {
  if cargo fmt --all -- --check 2>&1 | tee "$REPORT_DIR/format.log"; then
    return 0
  fi
  local formatting_source="$source_dir"
  if [[ -n "${KHOR_NETWORK_SOURCE_DIR:-}" ]]; then
    formatting_source="$RUNNER_TEMP/network-format"
    git clone --no-hardlinks "$SOURCE_PACKAGE/source.git" "$formatting_source" || return 1
  fi
  # Export a formatter patch from a disposable copy, preserving cached inputs.
  (
    cd "$formatting_source" || exit 1
    cargo fmt --all >&2 || exit 1
    git diff --binary
  ) > "$REPORT_DIR/format.patch" || return 1
  return 1
}
if [[ "$DEV_TASK" == net-unit || "$DEV_TASK" == net-node || "$DEV_TASK" == net-vendor || "$DEV_TASK" == net-congestion || "$DEV_TASK" == net-wan-build || "$DEV_TASK" == net-wan-lifetime || "$DEV_TASK" == net-wan-peer ]]; then
  check_format || exit 1
  if [[ "$DEV_TASK" == net-unit ]]; then
    cargo test --locked -p khor-net --lib 2>&1 | tee "$REPORT_DIR/net-unit.log"
    if [[ -f vendor/noq-udp/KHOR.md ]]; then
      cargo test --locked --manifest-path vendor/noq-udp/Cargo.toml \
        --features fast-apple-datapath --test tests 2>&1 | tee "$REPORT_DIR/noq-udp-tests.log"
    fi
  elif [[ "$DEV_TASK" == net-wan-build || "$DEV_TASK" == net-wan-lifetime || "$DEV_TASK" == net-wan-peer ]]; then
    cargo build --locked -p khor-node --example net_wan_probe 2>&1 | tee "$REPORT_DIR/wan-build.log"
    python3 - "$CARGO_TARGET_DIR/debug/examples/net_wan_probe" "$REPORT_DIR" <<'PYTHON'
import hashlib,json,os,pathlib,sys
binary=pathlib.Path(sys.argv[1]);report=pathlib.Path(sys.argv[2]);stat=binary.lstat()
assert binary.is_absolute() and binary.is_file() and not binary.is_symlink() and stat.st_uid==os.getuid() and stat.st_mode & 0o111
source=json.loads((report/'source.json').read_text())
manifest=dict(schema=1,path=str(binary),sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),bytes=stat.st_size,source=source,executed=False,storage='mutable build cache; reverify hash under guard before owned runtime copy')
(report/'wan-binary.json').write_text(json.dumps(manifest,indent=2)+'\n')
PYTHON
    if [[ "$DEV_TASK" == net-wan-lifetime ]]; then
      wan_status=0
      python3 "$(dirname "$0")/wan-probe-lease.py" --check "$REPORT_DIR/wan-binary.json" "$REPORT_DIR" || wan_status=$?
      python3 - "$REPORT_DIR" <<'PYTHON'
import json,pathlib,sys
report=pathlib.Path(sys.argv[1]);path=report/'wan-binary.json';manifest=json.loads(path.read_text());contracts=report/'wan-lifetime-contracts.json'
manifest['executed']=any(r.get('lifecycle',{}).get('worker_pid') for r in json.loads(contracts.read_text())['results']) if contracts.exists() else None
manifest['execution_scope']='isolated lifetime contracts; no mint, pair or WAN business'
path.write_text(json.dumps(manifest,indent=2)+'\n')
PYTHON
      exit "$wan_status"
    fi
  elif [[ "$DEV_TASK" == net-node ]]; then
    printf '{"suite":"final_net_release_candidate_v2","slo":false,"compile_only":true,"runtime_tests":0}\n' > "$REPORT_DIR/measurement-mode.json"
    /usr/bin/python3 - <<'VERIFY_PYTHON'
import pathlib,hashlib
p=pathlib.Path('/nfs/t1/userhome/zzl-zgh/miniconda3/bin/python')
assert hashlib.sha256(p.read_bytes()).hexdigest()=='c46ed00dd978dbb20e32eb34d9e16660775b2ed4fc84395a73120804e3e66379'
VERIFY_PYTHON
    fixture_bin="$RUNNER_TEMP/fixture-tools"
    mkdir "$fixture_bin"
    cat > "$fixture_bin/python3" <<'PYTHON_WRAPPER'
#!/bin/sh
exec /nfs/t1/userhome/zzl-zgh/miniconda3/bin/python -E -S "$@"
PYTHON_WRAPPER
    chmod 700 "$fixture_bin/python3"
    export PATH="$fixture_bin:$PATH"
    export KHOR_RUNTIME_DIR="$RUNNER_TEMP/runtime-images"
    export KHOR_TEST_MCP_EXECUTABLE="$CARGO_TARGET_DIR/debug/khor"
    python3 - <<'TOOLS_PROOF'
import pathlib,json,hashlib,os,sys
p=pathlib.Path('/nfs/t1/userhome/zzl-zgh/miniconda3/bin/python')
assert sys.version_info >= (3,11)
(pathlib.Path(os.environ['REPORT_DIR'])/'fixture-python.json').write_text(json.dumps(dict(path=str(p),version=sys.version,sha256=hashlib.sha256(p.read_bytes()).hexdigest(),wrapper=str(pathlib.Path(os.environ['RUNNER_TEMP'])/'fixture-tools/python3')),indent=2)+'\n')
TOOLS_PROOF
    python3 - <<'FINAL_NET_RELEASE_COMPILE'
FROZEN={'role': 'candidate', 'source': {'schema': 1, 'base_commit': '1b20eee94f0f4d17ee332b88282de899bf46a95f', 'tree': '3073248f692b79ba97ee734a2f01fd338997ca66', 'dirty': False, 'created_at': '2026-11-17T01:36:04.930023+00:00', 'excluded_untracked_count': 0, 'snapshot_commit': '896a486a7a118b62df5f9d8ca26c5de467a57ff2'}, 'prior': 'turing-20261116T183237Z-373944af', 'prior_tree': 'ba2579934d682c18732b31837231c58bf014dbc9', 'prior_inputs_sha256': {'release-compile.json': 'ab4d6d1447b8acd8d9a94cb20393d9fc16119e8175be07f6cae0131e863bf688', 'release-provenance.json': '6cf47a440df30063743c94d7c8db922afcaeeadd72bdfaa0d708f62c917622e0', 'release-cli-provenance.json': 'a768bb30c9d70414dce0981c24da6f2f944643c45bdcfa561ae4f72ad1901018', 'release-artifacts.json': 'c8b293fcaa67ba1b6b4edae06a77c7cc799b762b3e0fc843de9792955ae8f4f8'}, 'changes': [['M', 'apps/gui/core/src/web.rs'], ['M', 'apps/gui/core/tests/fixtures/webkit_loopback.swift']], 'versions': {'iroh': '1.4.1', 'noq': '1.4.0', 'noq-proto': '1.4.0'}, 'target': 'netlab', 'main': '84e75c57cf87d0dfaffda14149d0d726f15dfe9e', 'production_overlay': None, 'common_fixture_sha256': {'crates/cli/tests/netlab/attempts.rs': '2c41147fca3f12ca38e8119498ea7bfaf563818b52b5eccd4299c08070c2e542', 'crates/cli/tests/netlab/lab.rs': '7fa2e2002f91762968fa9fd3a812799d313732af5a5f5747c3f7860d9a55a60a', 'crates/cli/tests/netlab/workload.rs': '71ce97ff47cf9b1611a9ed23bb69c3e6838af691b0a548f3db5dfdd68c9365bc', 'crates/cli/tests/netlab/workload/calibration.rs': '163327fb95e56f16cbd7d29514479e3e7130b44387d000108f699dddd2e8ddae', 'crates/cli/tests/netlab/workload/calibration_burst.rs': 'f07cce45f6c5625909ae68c409f90c0b0ac7556b485e431cbed34918fca19f4d', 'crates/cli/tests/netlab/workload/calibration_capacity.rs': '528dd72e1462dccdda5b2636f46162ff3273eb0c40e3419b8b7936dfecaae930', 'crates/cli/tests/netlab/workload/calibration_datagrams.rs': '85701e1a191656e9ba43eeb09a1d8fd13d2107171d9249a33ab6ca13072028c8', 'crates/cli/tests/netlab/workload/calibration_queues.rs': '50a03d9530b2b6d5400f953cee355333691a257b1fd05a0534aefc8cca63ca3c', 'crates/cli/tests/netlab/workload/calibration_sockets.rs': '0905def0c34d2f9259c91628d18b1e3241d026b9ef0ec119691c21a61f59d4f6', 'crates/cli/tests/netlab/workload/observer.rs': '0dfa5bbb8332f0bb39c0e3e5d5094270e910bdb08845220346629e85a6c66f49', 'crates/cli/tests/netlab/workload/profile.rs': '8fb2c072f9af0997045fd01f4ffad395e4617456dc9decbd5f1eb30d344ea71f'}, 'validation_source_sha256': {'scripts/native_provenance.py': '7dbd2c0f4048f8cc96a3c9303565b6298c4999daccdd4fe360825d21d225ac7b', 'scripts/tests/test_native_provenance.py': '3512a1c1a555179db128b335bf17a66464b3d063c0c36d3cd4e177b84f6fca29'}, 'expected_validation_tests': 46}
import copy,hashlib,importlib.util,json,os,shutil,subprocess,sys,tomllib,unittest,time,tempfile,types
from pathlib import Path
REPORT=Path(os.environ['REPORT_DIR']);SOURCE=json.loads((REPORT/'source.json').read_text())
assert os.getuid()==1110 and SOURCE['tree']==FROZEN['source']['tree']
arm=Path('/tmp/khor-netlab-feat-net-1110/cache/sole-peer-release-comparison-v1')/FROZEN['role'];root=arm/'source';target=arm/'target'
assert not arm.is_symlink() and not root.is_symlink() and arm.stat().st_uid==root.stat().st_uid==1110
wrapper_run='turing-20261113T030746Z-850f325b';wrapper=Path('/tmp/khor-netlab-feat-net-1110/runs')/wrapper_run/'work/fixture-tools/python3'
assert wrapper.read_text()=='#!/bin/sh\nexec /nfs/t1/userhome/zzl-zgh/miniconda3/bin/python -E -S "$@"\n'
parts=os.environ['PATH'].split(':');assert Path(parts[0])==Path(os.environ['RUNNER_TEMP'])/'fixture-tools'
os.environ.update(PATH=':'.join([str(wrapper.parent),*parts[1:]]),CARGO_TARGET_DIR=str(target),KHOR_NETWORK_SOURCE_DIR=str(root),KHOR_RUNTIME_DIR=str(arm/'runtime-images'),KHOR_TEST_MCP_EXECUTABLE=str(target/'release/khor'))
os.chdir(root)
current_tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip()
assert not subprocess.check_output(['git','status','--porcelain'])
if current_tree!=FROZEN['prior_tree']:
    failed=Path('/tmp/khor-netlab-feat-net-1110/runs')/FROZEN['prior_failed_migration']['run']/'report'
    migration=failed/'release-source-migration.json'
    assert current_tree==FROZEN['prior_failed_migration']['source_tree']
    assert hashlib.sha256(migration.read_bytes()).hexdigest()==FROZEN['prior_failed_migration']['migration_sha256']
    assert json.loads((failed/'execution.json').read_text())['returncode']==FROZEN['prior_failed_migration']['original_returncode']
    assert not (failed/'release-artifacts.json').exists()
    original=json.loads((Path('/tmp/khor-netlab-feat-net-1110/runs')/FROZEN['prior']/'report/release-compile.json').read_text())
    subprocess.run(['git','checkout','--quiet','--detach',original['source']['build_commit']],check=True)
    assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip()==FROZEN['prior_tree'] and not subprocess.check_output(['git','status','--porcelain'])
    p=REPORT/'release-prior-source-restored.json';assert not p.exists();p.write_text(json.dumps(dict(prior_failed_run=FROZEN['prior_failed_migration']['run'],from_tree=current_tree,to_tree=FROZEN['prior_tree'],owned_clean_source_only=True))+'\n')
sys.path.insert(0,str(root/'scripts'));import native_provenance as provenance
def save(name,value):
    p=REPORT/name;assert not p.exists();p.write_text(json.dumps(value,indent=2)+'\n')
previous=Path('/tmp/khor-netlab-feat-net-1110/runs')/FROZEN['prior']/'report'
assert json.loads((previous/'execution.json').read_text())['returncode']==0
for name,digest in FROZEN['prior_inputs_sha256'].items(): assert provenance.digest(previous/name)==digest
prior=json.loads((previous/'release-provenance.json').read_text());prior_cli=json.loads((previous/'release-cli-provenance.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip()==FROZEN['prior_tree'] and not subprocess.check_output(['git','status','--porcelain'])
historical_context_deltas=[]
for name,old_proof in [('test',prior),('cli',prior_cli)]:
    try:
        provenance.require_provenance(old_proof,root,old_proof['selected']['row'])
    except ValueError as error:
        # The unmodified verifier has already checked source, linked bytes and
        # generated inputs before this exact context rejection. Archive these
        # closed historical bytes; never admit or reuse them for the new build.
        if str(error)!='Build environment/config/toolchain changed before native acceptance':raise
        current=provenance.context(root,old_proof['context']['tools']);current['vendored_sources']={n:v for n,v in old_proof['source'].items() if n.startswith('vendor/')}
        delta={section:sorted(k for k in set(current[section])|set(old_proof['context'][section]) if current[section].get(k)!=old_proof['context'][section].get(k)) for section in current if current[section]!=old_proof['context'][section]}
        historical_context_deltas.append(dict(artifact=name,changed_context_keys=delta,old_context=old_proof['context'],current_context=current))
effective_prior=[] if historical_context_deltas else [prior,prior_cli]
save('release-historical-context.json',dict(prior_compile_run=FROZEN['prior'],archive_only=True,deltas=historical_context_deltas,prior_receipts_admitted_for_reuse=len(effective_prior),all_first_party_refreshed_when_context_differs=bool(historical_context_deltas)))
expected={**prior['selected']['files'],**prior_cli['selected']['files']};directory=Path('/tmp/khor-netlab-feat-net-1110/cache/closed-selected-release-v1')/FROZEN['prior'];manifest=directory/'selected-files.json'
if manifest.exists():
    assert not directory.is_symlink() and not manifest.is_symlink() and directory.stat().st_uid==1110
    retained=json.loads(manifest.read_text());assert retained['compile_run']==FROZEN['prior'] and set(retained['files'])==set(expected)
else:
    directory.mkdir(mode=0o700,parents=True,exist_ok=False);stored={}
    for original,stamp in expected.items():
        p=Path(original);assert not p.is_symlink() and p.stat().st_uid==1110 and provenance.same_files({original:stamp});saved=directory/(stamp['sha256']+'-'+p.name)
        with p.open('rb') as src,saved.open('xb') as dst:shutil.copyfileobj(src,dst)
        saved.chmod(0o400);assert provenance.same_files({str(saved):stamp});stored[original]=dict(saved=str(saved),sha256=stamp['sha256'],size=stamp['size'])
    retained=dict(schema=1,compile_run=FROZEN['prior'],files=stored);manifest.write_text(json.dumps(retained,indent=2)+'\n')
for original,stamp in expected.items():
    saved=Path(retained['files'][original]['saved']);assert saved.parent==directory and not saved.is_symlink() and saved.stat().st_uid==1110 and provenance.same_files({str(saved):stamp})
save('release-prior-selected-retained.json',retained)
ref='refs/khor/full-netlab-compile/'+os.environ['GITHUB_RUN_ID'];subprocess.run(['git','fetch','--quiet','--no-tags',str(Path(os.environ['SOURCE_PACKAGE'])/'source.git'),'HEAD:'+ref],check=True);subprocess.run(['git','checkout','--quiet','--detach',ref],check=True)
assert subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip()==SOURCE['tree'] and not subprocess.check_output(['git','status','--porcelain'])
changes=[x.split('\t') for x in subprocess.check_output(['git','diff','--name-status',FROZEN['prior_tree'],SOURCE['tree']],text=True).splitlines()];assert changes==FROZEN['changes']
# Historical admission used its historical helper. New preparation/acceptance
# must execute the exact helper now present in the migrated source, not an
# imported module retained from the previous source.
helper=root/'scripts/native_provenance.py';fresh=types.ModuleType('native_provenance');fresh.__file__=str(helper);sys.modules['native_provenance']=fresh
exec(compile(helper.read_bytes(),str(helper),'exec'),fresh.__dict__);provenance=fresh
for name,digest in FROZEN['validation_source_sha256'].items():assert provenance.digest(root/name)==digest
for name,digest in FROZEN['common_fixture_sha256'].items():assert provenance.digest(root/name)==digest
save('release-source-migration.json',dict(source=SOURCE,prior_compile_run=FROZEN['prior'],changed_only=changes,role=FROZEN['role'],production_overlay=FROZEN['production_overlay'],frozen_main=FROZEN['main'],common_fixture_sha256=FROZEN['common_fixture_sha256']))
dep=root/'vendor/iroh/src/lib.rs';before=provenance.digest(dep);os.utime(dep,None);assert provenance.digest(dep)==before
save('release-dependency-refresh.json',dict(source_unchanged=True,dependency='iroh',source_sha256=before,method='Owned same-byte mtime refresh; require fresh artifact in actual workspace graph'))
subprocess.run(['cargo','fmt','--all','--','--check'],check=True)
if FROZEN['role']=='candidate':
    subprocess.run(['cargo','fmt','--manifest-path','vendor/iroh/Cargo.toml','--','--check'],check=True)
else:
    save('release-vendor-format-policy.json',dict(role='baseline',policy='Keep current main vendor literal; original first-party format gate only. Do not format or alter baseline product code to satisfy a candidate-only added standalone check.'))
# Keep the original synthetic assertions. Their positive output fixtures
# must be written after the filesystem's coarse clock passes prepared_ns.
# No production verifier time/threshold/logic is patched.
validation_temp=Path(os.environ['RUNNER_TEMP'])/'validation-temp';validation_temp.mkdir(mode=0o700)
original_temp=tempfile.tempdir;tempfile.tempdir=str(validation_temp)
probe=REPORT/'validation-clock-probe';probe.write_bytes(b'0');clock_observations=[]
suite=unittest.defaultTestLoader.discover(str(root/'scripts/tests'),pattern='test_native_provenance.py');expected_tests=suite.countTestCases();assert expected_tests==FROZEN['expected_validation_tests']
case=sys.modules['test_native_provenance'].ProvenanceTests;original_prepare=case.prepare
def prepared_after_filesystem_clock(self,*args,**kwargs):
    value=original_prepare(self,*args,**kwargs);began=time.monotonic_ns();until=time.monotonic()+0.5;attempts=0
    while True:
        probe.write_bytes(b'1');stamp=probe.stat().st_ctime_ns;attempts+=1
        if stamp>=value['prepared_ns']:break
        if time.monotonic()>=until:raise AssertionError('Synthetic fixture filesystem clock did not pass the unchanged native prepare timestamp')
        time.sleep(0.001)
    clock_observations.append(dict(prepared_ns=value['prepared_ns'],observed_ctime_ns=stamp,wait_ns=time.monotonic_ns()-began,probes=attempts))
    return value
case.prepare=prepared_after_filesystem_clock
try:
    with (REPORT/'release-validation.log').open('x') as out:result=unittest.TextTestRunner(stream=out,verbosity=2).run(suite)
finally:
    case.prepare=original_prepare;tempfile.tempdir=original_temp
save('release-validation-clock-barrier.json',dict(original_test_files_and_assertions_unchanged=True,production_verifier_unchanged=True,scope='Synthetic positive fixture output preparation only',observations=clock_observations))
assert result.wasSuccessful() and result.testsRun==expected_tests and not result.skipped
save('release-validation.json',dict(tests=result.testsRun,failures=0,errors=0,skipped=0,source_sha256=FROZEN['validation_source_sha256'],filesystem_clock_barrier='Original positive fixtures are written only after actual same-filesystem ctime reaches unchanged prepared_ns'))
config=tomllib.loads((root/'Cargo.toml').read_text())['profile']['release'];assert config==dict(strip=True,lto='thin',**{'codegen-units':1})
host=next(x.removeprefix('host: ') for x in subprocess.check_output([os.environ['RUSTC'],'-vV'],text=True).splitlines() if x.startswith('host: '));metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',host]));save('release-metadata.json',metadata)
for name,version in FROZEN['versions'].items():assert next(p for p in metadata['packages'] if p['name']==name)['version']==version
tools=[Path(shutil.which('cargo')),Path(os.environ['RUSTC'])]
cp=provenance.prepare_build(root,target,metadata,dict(package='khor-cli',name='khor',kind=['bin'],test=False),tools,effective_prior,apply_refresh=True);save('release-cli-prepared.json',cp);assert 'crates/cli/src/main.rs' in cp['refresh_plan']
prepared=provenance.prepare_build(root,target,metadata,dict(package='khor-cli',name=FROZEN['target'],kind=['test'],test=True),tools,effective_prior,apply_refresh=True);save('release-prepared.json',prepared)
command=['cargo','test','--release','--locked','-p','khor-cli','--test',FROZEN['target'],'--no-run','--message-format=json']
with (REPORT/'release-artifacts.json').open('xb') as out:subprocess.run(command,stdout=out,check=True)
messages=provenance.read_messages(REPORT/'release-artifacts.json');iroh_rows=[m for m in messages if m.get('reason')=='compiler-artifact' and m['target']['name']=='iroh'];assert len(iroh_rows)==1 and iroh_rows[0]['fresh'] is False
proof=provenance.verify_build(prepared,messages);save('release-provenance.json',proof);provenance.require_provenance(proof,root,proof['selected']['row']);units=[u for u in proof['units'] if u['variant']['name']=='khor' and u['variant']['kind']==['bin'] and u['variant']['test'] is False];assert len(units)==1
cli=copy.deepcopy(proof);cli['selected']=copy.deepcopy(units[0]);cli['derived_admission']=dict(original_proof_sha256=provenance.digest(REPORT/'release-provenance.json'),reason='Actual fresh ordinary Release CLI compiler unit from same producer');provenance.require_provenance(cli,root,cli['selected']['row']);save('release-cli-provenance.json',cli)
for p in [proof,cli]:assert p['selected']['variant']['features']==[] and p['selected']['row']['profile']['opt_level']=='3' and p['selected']['row']['profile']['debug_assertions'] is False and p['selected']['row']['fresh'] is False
assert all('qlog' not in u['variant']['features'] and 'net-diagnostics' not in u['variant']['features'] for u in proof['units'])
old_compile=json.loads((previous/'release-compile.json').read_text())
save('release-prior-byte-comparison.json',dict(prior_compile_run=FROZEN['prior'],current_source=SOURCE,prior_source=old_compile['source'],cli_byte_identical=provenance.digest(Path(cli['selected']['row']['executable']))==old_compile['cli_sha256'],test_byte_identical=provenance.digest(Path(proof['selected']['row']['executable']))==old_compile['test_sha256'],fresh_provenance_required=True,changed_bytes_not_assumed=True))
assert not subprocess.check_output(['git','status','--porcelain'])
save('release-compile.json',dict(schema=1,role=FROZEN['role'],source=SOURCE,profile='release',features=[],diagnostic=None,compile_only=True,runtime_executions=0,command=command,release_config=config,stable_root=str(root),target_dir=str(target),runtime_image_root=str(arm/'runtime-images'),environment_wrapper_run=wrapper_run,test_executable=proof['selected']['row']['executable'],test_sha256=provenance.digest(Path(proof['selected']['row']['executable'])),cli_executable=cli['selected']['row']['executable'],cli_sha256=provenance.digest(Path(cli['selected']['row']['executable'])),cli_actual_row_fresh=False,cli_main_forced_refreshed=True,main=FROZEN['main'],versions=FROZEN['versions'],common_fixture_sha256=FROZEN['common_fixture_sha256']))
FINAL_NET_RELEASE_COMPILE
  elif [[ "$DEV_TASK" == net-vendor ]]; then
    # Excluded dependencies cannot run their dev-dependencies from the root
    # workspace. Use each committed package lock, and make iroh test our noq
    # patch rather than the registry copy. Keep the exact resolution as evidence.
    cp vendor/noq-proto/Cargo.lock "$REPORT_DIR/noq-proto-Cargo.lock"
    cp vendor/iroh/Cargo.lock "$REPORT_DIR/iroh-Cargo.lock"
    cargo test --locked --manifest-path vendor/noq-proto/Cargo.toml --lib 2>&1 | tee "$REPORT_DIR/noq-proto-unit.log"
    cargo test --locked --manifest-path vendor/iroh/Cargo.toml \
      --config "patch.crates-io.noq-proto.path=\"$source_dir/vendor/noq-proto\"" \
      --lib socket::remote_map::remote_state::tests 2>&1 | tee "$REPORT_DIR/iroh-path-unit.log"
    # The path filter omits our relay-home, QAD and interface regressions.
    cargo test --locked --manifest-path vendor/iroh/Cargo.toml \
      --config "patch.crates-io.noq-proto.path=\"$source_dir/vendor/noq-proto\"" \
      --lib khor_ 2>&1 | tee "$REPORT_DIR/iroh-home-unit.log"
  elif [[ "$DEV_TASK" == net-congestion ]]; then
    # Lower-layer baseline, not a Node/SSH/hop SLO. Preserve every outcome
    # and repeat sequentially, without another timed case sharing the runtime.
    printf '{"measurement_version":2,"road_model":"independent_fifo_v2","calibration_required":true,"layer":"noq","slo":false,"test_threads":1,"repeats":3,"rtt_ms":300,"loss_pct_per_direction":2,"mbit_per_direction":20,"queue_ms":200,"bulk_seconds":20,"seeds":["9e3779b97f4a7c15","2545f4914f6cdd1d"]}\n' > "$REPORT_DIR/measurement-mode.json"
    cargo test --locked -p khor-net --test congestion \
      road_directions_do_not_wait_for_each_others_future_packets \
      -- --exact --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/road-calibration.log"
    grep -Fq 'test result: ok. 1 passed; 0 failed;' "$REPORT_DIR/road-calibration.log"
    congestion_status=0
    for congestion_repeat in 1 2 3; do
      printf '[congestion-repeat] %s/3\n' "$congestion_repeat"
      env -u CC_ONLY CC_RTT_MS=300 CC_LOSS_PCT=2 CC_MBIT=20 CC_SECS=20 \
        cargo test --locked -p khor-net --test congestion -- --ignored --nocapture --test-threads=1 \
        2>&1 | tee "$REPORT_DIR/congestion-$congestion_repeat.log" || congestion_status=$?
    done
    exit "$congestion_status"
  else
    cargo test --locked -p khor-net --lib 2>&1 | tee "$REPORT_DIR/net-unit.log"
    cargo test --locked -p khor-node --lib hop::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/hop-unit.log"
    cargo test --locked -p khor-node --lib wires::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/wires-unit.log"
    cargo test --locked -p khor-node --lib journal::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/journal-unit.log"
    cargo test --locked -p khor-node --lib link::dial_budget -- --test-threads=1 2>&1 | tee "$REPORT_DIR/dial-budget-unit.log"
    cargo test --locked -p khor-node --lib ipc:: -- --test-threads=1 2>&1 | tee "$REPORT_DIR/ipc-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::leased_ipc_callers_release_all_held_response_permits_when_dropped -- --test-threads=1 2>&1 | tee "$REPORT_DIR/ipc-response-lease-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::process_contracts::independent_process_budget_releases_a_reply_without_caller_eof_or_manual_release -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/ipc-process-lease-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::quic_cancel_contracts::leased_ipc_cancellation_releases_all_remote_encoded_replies -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/quic-response-lease-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::resume_admission_contracts::local_resume_accepts_a_new_connection_before_the_next_pause_poll -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/resume-admission-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::canonical_content_routes_without_metadata_owner_and_cancels_through_busy_real_wires -- --test-threads=1 2>&1 | tee "$REPORT_DIR/content-wire-unit.log"
    cargo test --locked -p khor-node --lib session_content::wire_tests::quic_stop_contracts::all_cancelled_reply_streams_observe_stop_and_keep_the_same_wire -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/quic-stop-wire-unit.log"
  fi
else
  # Prove the actual namespace, veth, netem and firewall capabilities before compiling.
  unshare -Urn sh -ec 'ip link add probe0 type dummy; ip link set probe0 up; tc qdisc add dev probe0 root netem delay 1ms; nft add table inet probe; nft list ruleset' > "$REPORT_DIR/namespace.log" 2>&1
  export NETLAB_RUN_DIR="$REPORT_DIR/netlab"
  if [[ "$DEV_TASK" == netlab-measure-check ]]; then
    printf '{"layer":"fixture","slo":false,"measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::attempts::"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::attempts:: --ignored
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" --calibration > "$REPORT_DIR/attempt-calibration.json"
  elif [[ "$DEV_TASK" == netlab-cpu-check ]]; then
    printf '{"layer":"fixture","slo":false,"measurement_revision":"checked_affinity_v3","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::cpu::"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::cpu:: --ignored
  elif [[ "$DEV_TASK" == netlab-mobility-calibrate ]]; then
    check_format || exit 1
    printf '{"layer":"fixture","slo":false,"measurement_revision":"ipv6_renumber_v1","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::mobility::"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::mobility:: --ignored
  elif [[ "$DEV_TASK" == netlab-nat-rebind || "$DEV_TASK" == netlab-nat-rebind-lifecycle ]]; then
    check_format || exit 1
    diagnostic=off
    if [[ "$DEV_TASK" == netlab-nat-rebind-lifecycle ]]; then
      diagnostic=lifecycle
      export NETLAB_DIAGNOSTICS=lifecycle
    fi
    printf '{"layer":"node","slo":false,"scenario_revision":"node_nat_port_rebind_v1","calibration_revision":"nat_port_rebind_calibration_v1","calibration_required":true,"measurement_revision":"attempt_outcomes_v4","diagnostic":"%s","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::rebind::r0_","paths::rebind::r1_"]}\n' "$diagnostic" > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::rebind::r0_ --ignored
    grep -RFq '[netlab-rebind-calibration]' "$REPORT_DIR/netlab"
    grep -RFq 'test result: ok. 1 passed; 0 failed;' "$REPORT_DIR/netlab"
    mobility_status=0
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::rebind::r1_ --ignored || mobility_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    if [[ "$diagnostic" == lifecycle && "$mobility_status" == 0 ]]; then
      python3 scripts/netlab-nat-selection.py "$REPORT_DIR" > "$REPORT_DIR/nat-selection-contract.json" || mobility_status=$?
    fi
    exit "$mobility_status"
  elif [[ "$DEV_TASK" == netlab-peer-mobility || "$DEV_TASK" == netlab-peer-mobility-samples ]]; then
    check_format || exit 1
    diagnostic=lifecycle
    if [[ "$DEV_TASK" == netlab-peer-mobility-samples ]]; then diagnostic=1; fi
    printf '{"layer":"node","slo":false,"scenario_revision":"peer_relay_replug_v1","measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":1,"ignored":true,"diagnostic":"%s","filters":["hop::mobility::pm_"]}\n' "$diagnostic" > "$REPORT_DIR/measurement-mode.json"
    mobility_status=0
    NETLAB_DIAGNOSTICS="$diagnostic" NETLAB_TEST_THREADS=1 bash scripts/netlab.sh hop::mobility::pm_ --ignored || mobility_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$mobility_status"
  elif [[ "$DEV_TASK" == netlab-mobility-roam ]]; then
    check_format || exit 1
    printf '{"layer":"node","slo":false,"scenario_revision":"node_replug_triad_v1","scenario_revisions":["node_v4_nat_replug_v1","node_dual_stack_replug_v1","node_v4_to_v6_replug_v1"],"measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::roam::roam_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    mobility_status=0
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::roam::roam_ --ignored || mobility_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$mobility_status"
  elif [[ "$DEV_TASK" == netlab-mobility-node ]]; then
    check_format || exit 1
    printf '{"layer":"node","slo":false,"scenario_revision":"ipv6_node_overlap_pair_v1","scenario_revisions":["ipv6_node_overlap_v1","ipv6_node_overlap_remote_first_v1"],"measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":1,"ignored":true,"filters":["paths::mobile::mv6_overlap_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    mobility_status=0
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::mobile::mv6_overlap_ --ignored || mobility_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$mobility_status"
  elif [[ "$DEV_TASK" == netlab-hop-reference ]]; then
    printf '{"layer":"node","slo":false,"reference":"hop_reference_v2","measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":3,"ignored":true,"filters":["hop::reference::fr_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    reference_status=0
    repeat_netlab hop::reference::fr_ --ignored || reference_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$reference_status"
  elif [[ "$DEV_TASK" == netlab-hop-reference-lifecycle ]]; then
    printf '{"layer":"node","slo":false,"reference":"hop_reference_v2","diagnostic":"lifecycle","measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":3,"ignored":true,"filters":["hop::reference::fr_automatic_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    reference_status=0
    NETLAB_DIAGNOSTICS=lifecycle repeat_netlab hop::reference::fr_automatic_ --ignored || reference_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$reference_status"
  elif [[ "$DEV_TASK" == netlab-hop-ownership ]]; then
    printf '{"layer":"node","slo":false,"diagnostic":"lifecycle","measurement_revision":"attempt_outcomes_v4","test_threads":1,"repeats":3,"filters":["hop::f4_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    ownership_status=0
    NETLAB_DIAGNOSTICS=lifecycle repeat_netlab hop::f4_ || ownership_status=$?
    python3 scripts/netlab-attempts.py "$REPORT_DIR/netlab" > "$REPORT_DIR/attempts.json"
    exit "$ownership_status"
  elif [[ "$DEV_TASK" == netlab-hop-lifecycle ]]; then
    printf '{"layer":"node","slo":false,"diagnostic":true,"test_threads":1,"repeats":3,"filters":["hop::f4i_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_DIAGNOSTICS=1 repeat_netlab hop::f4i_
  elif [[ "$DEV_TASK" == netlab-smoke ]]; then
    printf '{"test_threads":2,"repeats":1,"filters":["paths::p2_","paths::p3_","paths::p6_","hop::f4_","hop::f4s_","hop::f4i_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=2 bash scripts/netlab.sh paths::p2_ paths::p3_ paths::p6_ hop::f4_ hop::f4s_ hop::f4i_
  elif [[ "$DEV_TASK" == netlab-dynamic ]]; then
    printf '{"test_threads":1,"repeats":3,"filters":["paths::p2_","paths::p3_","paths::p7_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab paths::p2_ paths::p3_ paths::p7_
  elif [[ "$DEV_TASK" == netlab-recovery ]]; then
    printf '{"test_threads":1,"repeats":3,"filters":["paths::p2_","paths::p3_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab paths::p2_ paths::p3_
  elif [[ "$DEV_TASK" == netlab-stability ]]; then
    printf '{"test_threads":1,"repeats":1,"filters":["paths::p1_","paths::p1j_","paths::p1s_","paths::p5_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::p1_ paths::p1j_ paths::p1s_ paths::p5_
  elif [[ "$DEV_TASK" == netlab-stalls ]]; then
    printf '{"test_threads":1,"repeats":1,"filters":["paths::p1s_","paths::p5_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::p1s_ paths::p5_
  elif [[ "$DEV_TASK" == netlab-diagnostic ]]; then
    printf '{"test_threads":1,"repeats":1,"diagnostic":true,"filters":["paths::p1s_","hop::f4s_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_DIAGNOSTICS=1 NETLAB_TEST_THREADS=1 bash scripts/netlab.sh paths::p1s_ hop::f4s_
  elif [[ "$DEV_TASK" == netlab-hop-diagnostic ]]; then
    printf '{"test_threads":1,"repeats":1,"diagnostic":true,"filters":["hop::f4s_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_DIAGNOSTICS=1 NETLAB_TEST_THREADS=1 bash scripts/netlab.sh hop::f4s_
  elif [[ "$DEV_TASK" == netlab-hop ]]; then
    printf '{"test_threads":1,"repeats":3,"filters":["hop::f4_","hop::f4s_","hop::f4i_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab hop::f4_ hop::f4s_ hop::f4i_
  elif [[ "$DEV_TASK" == netlab-compare ]]; then
    # Performance comparisons run sequentially; retain all three outcomes
    # even when one iteration exposes the baseline's expected product red.
    printf '{"test_threads":1,"repeats":3,"filters":["paths::p2_","paths::p3_","paths::p6_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab paths::p2_ paths::p3_ paths::p6_
  else
    printf '{"test_threads":8,"repeats":1,"filters":[]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=8 bash scripts/netlab.sh
  fi
fi
