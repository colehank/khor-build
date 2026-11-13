#!/bin/bash
# Trusted control driver: portable network checks on Linux/Mac; netlab requires Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|net-node|net-node-routes|net-vendor|net-wan-build|net-wan-lifetime|net-wan-peer|net-congestion|netlab-workload-policy|netlab-workload-wires|netlab-workload-observer|netlab-workload-prefetch|netlab-workload-high-policy|netlab-workload-high|netlab-workload-high-capacity|netlab-workload-high-queues|netlab-workload-high-burst|netlab-workload-high-long|netlab-workload-high-sockets|netlab-workload-high-calibrate|netlab-workload-calibrate|netlab-workload|netlab-measure-check|netlab-cpu-check|netlab-mobility-calibrate|netlab-mobility-node|netlab-mobility-roam|netlab-peer-mobility|netlab-peer-mobility-samples|netlab-nat-rebind|netlab-nat-rebind-lifecycle|netlab-hop-reference|netlab-hop-reference-lifecycle|netlab-hop-ownership|netlab-hop-lifecycle|netlab-smoke|netlab-dynamic|netlab-compare|netlab-hop|netlab-recovery|netlab-stability|netlab-stalls|netlab-diagnostic|netlab-hop-diagnostic|netlab) ;; *) exit 2 ;; esac
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
if [[ "$DEV_TASK" == net-unit || "$DEV_TASK" == net-node || "$DEV_TASK" == net-node-routes || "$DEV_TASK" == net-vendor || "$DEV_TASK" == net-congestion || "$DEV_TASK" == net-wan-build || "$DEV_TASK" == net-wan-lifetime || "$DEV_TASK" == net-wan-peer ]]; then
  check_format || exit 1
  if [[ "$DEV_TASK" == net-unit ]]; then
    cargo test --locked -p khor-net --lib 2>&1 | tee "$REPORT_DIR/net-unit.log"
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
  elif [[ "$DEV_TASK" == net-node-routes ]]; then
    # Fixed, SSH-runner-only iteration lane. Full net-node remains required
    # before acceptance; neither filters nor expected counts come from input.
    printf '{"layer":"node","slo":false,"suite":"route_contracts_v1","full_regression":false,"test_threads":1,"expected_tests":[6,2]}\n' > "$REPORT_DIR/measurement-mode.json"
    routes_status=0
    cargo test --locked -p khor-node --lib link::dial_budget::initial_custom:: \
      -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/initial-route-contracts.log" || routes_status=$?
    cargo test --locked -p khor-node --lib hop::tests_offer::published::incoming:: \
      -- --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/incoming-route-contracts.log" || routes_status=$?
    if ! python3 - "$REPORT_DIR" <<'PYTHON'
import hashlib, json, pathlib, re, sys
report = pathlib.Path(sys.argv[1])
groups = {}
for name, expected in [('initial-route-contracts.log', 6), ('incoming-route-contracts.log', 2)]:
    data = (report / name).read_bytes()
    rows = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', data.decode(errors='replace'))
    item = dict(expected=expected, sha256=hashlib.sha256(data).hexdigest(), valid=False, passed=False)
    if len(rows) == 1:
        status, good, failed, ignored = rows[0]
        item.update(passed_count=int(good), failed_count=int(failed), ignored_count=int(ignored))
        item['valid'] = int(good) + int(failed) == expected and int(ignored) == 0
        item['passed'] = item['valid'] and status == 'ok' and int(failed) == 0
    groups[name] = item
result = dict(schema=1, suite='route_contracts_v1', full_regression=False,
              source=json.loads((report / 'source.json').read_text()), groups=groups,
              passed=all(item['passed'] for item in groups.values()))
(report / 'route-contracts.json').write_text(json.dumps(result, indent=2) + '\n')
sys.exit(0 if result['passed'] else 1)
PYTHON
    then
      if [[ "$routes_status" == 0 ]]; then routes_status=1; fi
    fi
    exit "$routes_status"
  elif [[ "$DEV_TASK" == net-vendor ]]; then
    printf '{"suite":"tuple_exit_iroh_mac_full_lib_v1","slo":false,"runtime_performance":false,"no_filter":true,"test_threads":1,"noq_algorithm_unchanged":true}\n' > "$REPORT_DIR/measurement-mode.json"
    cargo fmt --manifest-path vendor/iroh/Cargo.toml -- --check 2>&1 | tee "$REPORT_DIR/iroh-format.log"
    cp vendor/noq-proto/Cargo.lock "$REPORT_DIR/noq-proto-Cargo.lock"
    cp vendor/iroh/Cargo.lock "$REPORT_DIR/iroh-Cargo.lock"
    python3 - <<'TUPLE_EXIT_METADATA'
import json,os,pathlib,subprocess
root=pathlib.Path.cwd();report=pathlib.Path(os.environ['REPORT_DIR'])
source=json.loads((report/'source.json').read_text())
assert source['tree'] in {'cf9d172bb92778ccfd97899ddea27bdf0e6d2f6e','2d702718ff8f664cc1e2fa5db2352e2d8e91ee8d'}
verbose=subprocess.check_output(['rustc','-vV'],text=True)
host=next(line.removeprefix('host: ') for line in verbose.splitlines() if line.startswith('host: '))
metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1','--filter-platform',host,'--manifest-path','vendor/iroh/Cargo.toml','--config','patch.crates-io.noq-proto.path="'+str(root/'vendor/noq-proto')+'"']))
for name,version in [('iroh','1.4.1'),('noq','1.4.0'),('noq-proto','1.4.0')]:
 rows=[p for p in metadata['packages'] if p['name']==name]
 assert len(rows)==1 and rows[0]['version']==version
 if name in {'iroh','noq-proto'}:assert pathlib.Path(rows[0]['manifest_path'])==root/'vendor'/name/'Cargo.toml'
(report/'iroh-metadata.json').write_text(json.dumps(metadata,indent=2)+'\n')
(report/'iroh-toolchain.json').write_text(json.dumps({'rustc_verbose':verbose,'host':host,'source':source,'root':str(root)},indent=2)+'\n')
TUPLE_EXIT_METADATA
    cargo test --locked --manifest-path vendor/iroh/Cargo.toml \
      --config "patch.crates-io.noq-proto.path=\"$source_dir/vendor/noq-proto\"" \
      --lib -- --test-threads=1 2>&1 | tee "$REPORT_DIR/iroh-full-unit.log"
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
  if [[ "$DEV_TASK" == netlab-workload ]]; then
    check_format || exit 1
    printf '{"layer":"node","slo":false,"calibrated":false,"measurement_revision":"resident_mixed_workload_v4","test_threads":1,"repeats":3,"ignored":true,"filters":["workload::w1_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    workload_status=0
    repeat_netlab workload::w1_ --ignored || workload_status=$?
    python3 - "$REPORT_DIR" <<'PYTHON' || workload_status=$?
import collections,hashlib,json,pathlib,re,sys
report=pathlib.Path(sys.argv[1]);results=[]
logs=sorted((report/'netlab').glob('*/scenarios.log'))
for path in logs:
    result={'log':str(path.relative_to(report)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'valid':False,'functional_ok':False}
    try:
        text=path.read_text();rows=[json.loads(line[len('[netlab-workload] '):]) for line in text.splitlines() if line.startswith('[netlab-workload] ')]
        assert rows and all(r['schema']==1 and r['revision']=='resident_mixed_workload_v4' for r in rows)
        one=lambda kind: [r for r in rows if r['event']==kind]
        manifests=one('manifest');completes=one('complete');bulks=one('bulk');samples=one('sample')
        assert len(manifests)==len(completes)==len(bulks)==1
        manifest,complete,bulk=manifests[0],completes[0],bulks[0]
        assert manifest['samples_per_operation_per_phase']==120 and manifest['period_us']==100000 and manifest['caller_us']==2000000 and manifest['slo'] is False and manifest['calibrated'] is False
        assert bulk['phase']=='bulk' and bulk['expected']==16*1024*1024 and bulk['ended_us']>bulk['started_us']
        expected={(phase,op) for phase in ['idle_before','bulk','idle_after'] for op in ['directory','terminal']}
        assert {(r['phase'],r['operation']) for r in samples}==expected and len(samples)==720
        groups={}
        for phase,op in sorted(expected):
            group=[r for r in samples if r['phase']==phase and r['operation']==op]
            assert len(group)==120 and {r['index'] for r in group}==set(range(120))
            assert all(r['ended_us']>=r['started_us']>=r['scheduled_us'] and r['elapsed_us']==r['ended_us']-r['started_us'] and r['outcome'] in ['ok','error','timeout','blocked'] for r in group)
            ordered=sorted(group,key=lambda r:r['index']);base=ordered[0]['scheduled_us']
            assert all(r['scheduled_us']==base+r['index']*100000 and r['caller_us']==2000000 for r in ordered)
            times=sorted(r['elapsed_us'] for r in group if r['outcome']=='ok')
            groups[phase+'/'+op]={'outcomes':dict(collections.Counter(r['outcome'] for r in group)),'all_samples':120,'successful_samples':len(times),'success_p50_us':times[(len(times)-1)//2] if times else None,'success_p95_us':times[(len(times)*95+99)//100-1] if times else None,'success_max_us':max(times) if times else None,'p99_claimed':False}
        overlap={op:sum(r['phase']=='bulk' and r['operation']==op and bulk['started_us']<=r['started_us']<=r['ended_us']<=bulk['ended_us'] for r in samples) for op in ['directory','terminal']}
        assert all(complete[op+'_overlap']==n for op,n in overlap.items()) and complete['samples']==720
        summaries=re.findall(r'^test result: (.*)$',text,re.M)
        assert len(summaries)==1 and summaries[0].startswith('ok. 1 passed; 0 failed; 0 ignored;')
        functional=all(r['outcome']=='ok' and r['error'] is None for r in samples) and bulk['outcome']=='ok' and bulk['moved']==16*1024*1024 and all(n>=30 for n in overlap.values()) and all(complete[k] is True for k in ['functional_ok','all_answers','exact_bytes','closed','direct_after'])
        result.update(valid=True,functional_ok=functional,groups=groups,overlap=overlap,bulk=bulk)
    except (AssertionError,KeyError,IndexError,ValueError,TypeError) as error:
        result['error']=type(error).__name__+': '+str(error)
    results.append(result)
passed=len(results)==3 and all(r['valid'] and r['functional_ok'] for r in results)
(report/'workload-contracts.json').write_text(json.dumps({'schema':1,'revision':'resident_mixed_workload_v4','source':json.loads((report/'source.json').read_text()),'results':results,'passed':passed,'slo':False,'calibrated':False,'limits':['Diagnostic baseline only; no capacity calibration, policy comparison, CPU, energy or p99 acceptance']},indent=2)+'\n')
sys.exit(0 if passed else 1)
PYTHON
    exit "$workload_status"
  elif [[ "$DEV_TASK" == netlab-workload-policy ]]; then
    check_format || exit 1
    printf '{"layer":"node_policy_comparison","slo":false,"calibrated":false,"diagnostic":"off","measurement_revision":"resident_mixed_workload_v4","comparison_revision":"workload_response_priority_v1","test_threads":1,"repeats":3,"ignored":true,"guard_filter":"workload::observer::w3_","filters":["workload::w2_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    policy_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::observer::w3_ --ignored || policy_status=$?
    if [[ "$policy_status" == 0 ]]; then
      for repeat in 1 2 3; do
        next_status=0
        NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::w2_ --ignored || next_status=$?
        if [[ "$policy_status" == 0 && "$next_status" != 0 ]]; then policy_status=$next_status; fi
      done
    fi
    next_status=0
    python3 scripts/netlab-workload-policy.py "$REPORT_DIR" || next_status=$?
    if [[ "$policy_status" == 0 && "$next_status" != 0 ]]; then policy_status=$next_status; fi
    exit "$policy_status"
  elif [[ "$DEV_TASK" == netlab-workload-wires ]]; then
    check_format || exit 1
    printf '{"layer":"node_diagnostic","slo":false,"diagnostic":"lifecycle","measurement_revision":"resident_mixed_workload_v4","wire_revision":"workload_wire_stages_v1","test_threads":1,"repeats":3,"ignored":true,"guard_filter":"workload::observer::w3_","filters":["workload::w2_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    wire_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::observer::w3_ --ignored || wire_status=$?
    if [[ "$wire_status" == 0 ]]; then
      for repeat in 1 2 3; do
        next_status=0
        NETLAB_DIAGNOSTICS=lifecycle NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::w2_ --ignored || next_status=$?
        if [[ "$wire_status" == 0 && "$next_status" != 0 ]]; then wire_status=$next_status; fi
      done
    fi
    next_status=0
    python3 scripts/netlab-workload-wires.py "$REPORT_DIR" || next_status=$?
    if [[ "$wire_status" == 0 && "$next_status" != 0 ]]; then wire_status=$next_status; fi
    exit "$wire_status"
  elif [[ "$DEV_TASK" == netlab-workload-observer ]]; then
    check_format || exit 1
    printf '{"layer":"node_observer","slo":false,"calibrated":false,"measurement_revision":"resident_mixed_workload_v4","test_threads":1,"paired_repeats":3,"expected_order":["streaming","buffered","buffered","streaming","streaming","buffered"],"ignored":true,"guard_filter":"workload::observer::w3_"}\n' > "$REPORT_DIR/measurement-mode.json"
    observer_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 \
      bash scripts/netlab.sh workload::observer::w3_ --ignored || observer_status=$?
    if [[ "$observer_status" == 0 ]]; then
      # Freeze the order before observing results; preserve every case failure.
      for observer_filter in workload::w1_ workload::w2_ workload::w2_ workload::w1_ workload::w1_ workload::w2_; do
        observer_case_status=0
        NETLAB_TEST_THREADS=1 bash scripts/netlab.sh "$observer_filter" --ignored || observer_case_status=$?
        if [[ "$observer_status" == 0 && "$observer_case_status" != 0 ]]; then observer_status=$observer_case_status; fi
      done
    fi
    observer_validation_status=0
    python3 scripts/netlab-observer.py "$REPORT_DIR" || observer_validation_status=$?
    if [[ "$observer_status" == 0 ]]; then observer_status=$observer_validation_status; fi
    exit "$observer_status"
  elif [[ "$DEV_TASK" == netlab-workload-prefetch ]]; then
    check_format || exit 1
    printf '{"layer":"node_prefetch_comparison","slo":false,"calibrated":false,"diagnostic":"off","comparison_revision":"workload_prefetch_v1","measurement_revisions":["resident_mixed_workload_v4","resident_mixed_workload_high_v3"],"profiles":["low","high"],"cases_per_profile":1,"test_threads":1,"ignored":true,"guard_filter":"workload::observer::w3_"}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    prefetch_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 \
      bash scripts/netlab.sh workload::observer::w3_ --ignored || prefetch_status=$?
    if [[ "$prefetch_status" == 0 ]]; then
      next_status=0
      NETLAB_RUN_DIR="$REPORT_DIR/low-prefetch" NETLAB_TEST_THREADS=1 \
        bash scripts/netlab.sh workload::w2_ --ignored || next_status=$?
      if [[ "$prefetch_status" == 0 && "$next_status" != 0 ]]; then prefetch_status=$next_status; fi
      next_status=0
      NETLAB_RUN_DIR="$REPORT_DIR/high-prefetch" NETLAB_TEST_THREADS=1 \
        bash scripts/netlab.sh workload::wh2_ --ignored || next_status=$?
      if [[ "$prefetch_status" == 0 && "$next_status" != 0 ]]; then prefetch_status=$next_status; fi
    fi
    next_status=0
    python3 scripts/netlab-workload-prefetch.py "$REPORT_DIR" || next_status=$?
    if [[ "$prefetch_status" == 0 && "$next_status" != 0 ]]; then prefetch_status=$next_status; fi
    exit "$prefetch_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-policy ]]; then
    check_format || exit 1
    printf '{"layer":"node_high_policy_comparison","slo":false,"calibrated":false,"diagnostic":"off","measurement_revision":"resident_mixed_workload_high_v3","comparison_revision":"workload_high_response_priority_v1","test_threads":1,"repeats":3,"ignored":true,"guard_filter":"workload::observer::w3_","filters":["workload::wh2_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    policy_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 \
      bash scripts/netlab.sh workload::observer::w3_ --ignored || policy_status=$?
    if [[ "$policy_status" == 0 ]]; then
      for repeat in 1 2 3; do
        next_status=0
        NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::wh2_ --ignored || next_status=$?
        if [[ "$policy_status" == 0 && "$next_status" != 0 ]]; then policy_status=$next_status; fi
      done
    fi
    next_status=0
    python3 scripts/netlab-workload-high-policy.py "$REPORT_DIR" || next_status=$?
    if [[ "$policy_status" == 0 && "$next_status" != 0 ]]; then policy_status=$next_status; fi
    exit "$policy_status"
  elif [[ "$DEV_TASK" == netlab-workload-high ]]; then
    check_format || exit 1
    printf '{"layer":"node_high_preflight","slo":false,"calibrated":false,"measurement_revision":"resident_mixed_workload_high_v3","test_threads":1,"guard_filter":"workload::observer::w3_","low_repeats":3,"high_repeats":3,"ignored":true}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    NETLAB_RUN_DIR="$REPORT_DIR/observer-guards" NETLAB_TEST_THREADS=1 \
      bash scripts/netlab.sh workload::observer::w3_ --ignored || high_status=$?
    if [[ "$high_status" == 0 ]]; then
      for repeat in 1 2 3; do
        next_status=0
        NETLAB_RUN_DIR="$REPORT_DIR/low-regression" NETLAB_TEST_THREADS=1 \
          bash scripts/netlab.sh workload::w2_ --ignored || next_status=$?
        if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
      done
    fi
    if [[ "$high_status" == 0 ]]; then
      for repeat in 1 2 3; do
        next_status=0
        NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::wh2_ --ignored || next_status=$?
        if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
      done
    fi
    next_status=0
    python3 scripts/netlab-workload-high.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-capacity ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_UDP","slo":false,"calibrated":false,"diagnostic":"paced_datagrams","measurement_revision":"workload_kernel_udp_capacity_v2","test_threads":1,"repeats":3,"ignored":true,"filters":["workload::calibration::uq0_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for repeat in 1 2 3; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::calibration::uq0_ --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-capacity.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-queues ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"diagnostic":"forwarding_queue_topology","measurement_revision":"workload_kernel_tcp_queues_v1","test_threads":1,"repeats":4,"ignored":true,"qdisc_units_review_required":true,"expected_layout_order":["combined","separated","separated","combined"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for high_filter in workload::calibration::hb0_ workload::calibration::hq0_ workload::calibration::hq0_ workload::calibration::hb0_; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh "$high_filter" --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-queues.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-burst ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"diagnostic":"burst_fixed_latency","measurement_revision":"workload_kernel_tcp_burst_v1","test_threads":1,"repeats":4,"ignored":true,"qdisc_units_review_required":true,"requested_burst_order":[4096,65536,65536,4096],"derived_fifo_limit_changes":true}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for high_filter in workload::calibration::ha0_ workload::calibration::hb0_ workload::calibration::hb0_ workload::calibration::ha0_; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh "$high_filter" --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-burst.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-long ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"diagnostic":"tcp_long_sockets","measurement_revision":"workload_kernel_tcp_long_v1","test_threads":1,"repeats":1,"ignored":true,"qdisc_units_review_required":true,"filters":["workload::calibration::hl0_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for repeat in 1; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::calibration::hl0_ --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-long.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-sockets ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"diagnostic":"tcp_sockets","measurement_revision":"workload_kernel_tcp_sockets_v1","test_threads":1,"repeats":3,"ignored":true,"qdisc_units_review_required":true,"filters":["workload::calibration::hd0_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for repeat in 1 2 3; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::calibration::hd0_ --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-sockets.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-high-calibrate ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"diagnostic":"off","measurement_revision":"workload_kernel_tcp_calibration_high_v1","test_threads":1,"repeats":3,"ignored":true,"qdisc_units_review_required":true,"filters":["workload::calibration::h0_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    export NETLAB_DIAGNOSTICS=0
    unset KHOR_NET_DIAGNOSTICS
    high_status=0
    for repeat in 1 2 3; do
      next_status=0
      NETLAB_TEST_THREADS=1 bash scripts/netlab.sh workload::calibration::h0_ --ignored || next_status=$?
      if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    done
    next_status=0
    python3 scripts/netlab-kernel-high.py "$REPORT_DIR" || next_status=$?
    if [[ "$high_status" == 0 && "$next_status" != 0 ]]; then high_status=$next_status; fi
    exit "$high_status"
  elif [[ "$DEV_TASK" == netlab-workload-calibrate ]]; then
    check_format || exit 1
    printf '{"layer":"kernel_TCP","slo":false,"calibrated":false,"measurement_revision":"workload_kernel_tcp_calibration_v1","test_threads":1,"repeats":3,"ignored":true,"qdisc_units_review_required":true,"filters":["workload::calibration::w0_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    calibration_status=0
    repeat_netlab workload::calibration::w0_ --ignored || calibration_status=$?
    python3 - "$REPORT_DIR" <<'PYTHON' || calibration_status=$?
import hashlib,json,pathlib,re,sys
# kernel TCP calibration raw validator
report=pathlib.Path(sys.argv[1]);results=[]
for path in sorted((report/'netlab').glob('*/scenarios.log')):
    item={'log':str(path.relative_to(report)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'valid':False,'functional_and_bounds_ok':False}
    try:
        text=path.read_text();marker='[netlab-workload-calibration] '
        records=[line for line in text.splitlines() if marker in line];assert all(line.count(marker)==1 for line in records)
        rows=[json.loads(line.split(marker,1)[1]) for line in records]
        assert rows and all(r['schema']==1 and r['revision']=='workload_kernel_tcp_calibration_v1' for r in rows)
        def one(kind):
            found=[r for r in rows if r['event']==kind];assert len(found)==1;return found[0]
        manifest=one('manifest');complete=one('complete');connection=one('connect')
        assert manifest['warmup']==4 and manifest['echoes']==64 and manifest['echo_bytes']==64 and manifest['bulk_bytes']==8388608
        assert manifest['min_rtt_us_bounds']==[35000,80000] and manifest['goodput_mbit_bounds']==[6.4,9.6] and manifest['qdisc_units_review_required'] is True
        echoes=[r for r in rows if r['event']=='echo'];bulks=[r for r in rows if r['event']=='bulk'];captures=[r for r in rows if r['event']=='capture']
        assert len(echoes)==68 and {r['index'] for r in echoes}==set(range(68))
        assert all(r['warmup']==(r['index']<4) and r['bytes']==64 and r['ended_us']>=r['started_us'] and r['elapsed_us']==r['ended_us']-r['started_us'] for r in echoes)
        assert len(bulks)==2 and {r['direction'] for r in bulks}=={'upload','download'}
        assert all(r['bytes']==8388608 and r['elapsed_us']==r['ended_us']-r['started_us'] and r['elapsed_us']>0 for r in bulks)
        assert len(captures)==4 and {r['name'] for r in captures}=={'x_before','gateway_before','x_after','gateway_after'}
        for capture in captures:
            commands=capture['capture'];assert all(commands[k]['ok'] is True for k in ['qdisc_json','qdisc_text','interfaces'])
            qdiscs=json.loads(commands['qdisc_json']['stdout']);assert all(any(r['kind']==kind for r in qdiscs) for kind in ['netem','tbf'])
        successes=[r['elapsed_us'] for r in echoes if not r['warmup'] and r['outcome']=='ok'];minimum=min(successes) if successes else None
        rates=[8388608*8/r['elapsed_us'] for r in bulks]
        bounds=minimum is not None and 35000<=minimum<=80000 and all(6.4<=v<=9.6 for v in rates)
        assert complete['samples']==70 and complete['min_rtt_us']==minimum and complete['goodput_mbit']==rates and complete['bounds_ok']==bounds
        assert complete['calibrated'] is False and complete['qdisc_units_review_required'] is True
        summaries=re.findall(r'^test result: (.*)$',text,re.M);assert len(summaries)==1 and summaries[0].startswith('ok. 1 passed; 0 failed; 0 ignored;')
        functional=bounds and connection['ok'] is True and all(r['outcome']=='ok' and r['error'] is None for r in echoes+bulks) and all(r['exact_bytes'] is True for r in bulks) and all(complete[k] is True for k in ['functional_and_bounds_ok','server_ok','qdiscs_present'])
        item.update(valid=True,functional_and_bounds_ok=functional,min_rtt_us=minimum,goodput_mbit=rates,echo_samples=68,bulk_bytes=16777216)
    except (AssertionError,ValueError,KeyError,TypeError,IndexError,ZeroDivisionError) as error:
        item['error']=type(error).__name__+': '+str(error)
    results.append(item)
passed=len(results)==3 and all(r['valid'] and r['functional_and_bounds_ok'] for r in results)
(report/'workload-calibration-contracts.json').write_text(json.dumps({'schema':1,'revision':'workload_kernel_tcp_calibration_v1','source':json.loads((report/'source.json').read_text()),'results':results,'passed':passed,'calibrated':False,'slo':False,'qdisc_units_review_required':True},indent=2)+'\n')
sys.exit(0 if passed else 1)
PYTHON
    exit "$calibration_status"
  elif [[ "$DEV_TASK" == netlab-measure-check ]]; then
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
