#!/bin/bash
# Trusted control driver: portable network checks on Linux/Mac; netlab requires Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|net-node|net-vendor|net-congestion|netlab-smoke|netlab-dynamic|netlab-compare|netlab-hop|netlab-recovery|netlab-stability|netlab-stalls|netlab-diagnostic|netlab-hop-diagnostic|netlab) ;; *) exit 2 ;; esac
mkdir -p "$REPORT_DIR"
cp "$SOURCE_PACKAGE/source.json" "$REPORT_DIR/source.json"
source_dir="$RUNNER_TEMP/network-source"
git clone --no-hardlinks "$SOURCE_PACKAGE/source.git" "$source_dir"
cd "$source_dir"
python3 - <<'PY'
import json, os, platform, subprocess
from pathlib import Path
report = Path(os.environ['REPORT_DIR'])
source = json.loads((report / 'source.json').read_text())
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == source['build_commit']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], text=True).strip() == source['tree']
(report / 'environment.json').write_text(json.dumps(dict(task=os.environ['DEV_TASK'], system=platform.platform(),
    rust=subprocess.check_output(['rustc', '--version'], text=True).strip(), run_id=os.environ['GITHUB_RUN_ID'],
    control_commit=os.environ['GITHUB_SHA'], jobs=os.environ['CARGO_BUILD_JOBS']), indent=2) + '\n')
PY
# Netlab does not serve the face; same isolated placeholder as scripts/netlab.sh.
mkdir -p apps/gui/dist/assets
printf '// network test fixture\n' > apps/gui/dist/assets/index-netlab.js
printf '<!doctype html><script type="module" src="/assets/index-netlab.js"></script>\n' > apps/gui/dist/index.html
repeat_netlab() {
  local status=0 repeat
  for repeat in 1 2 3; do
    printf '[netlab-repeat] %s/3\n' "$repeat"
    NETLAB_TEST_THREADS=1 bash scripts/netlab.sh "$@" || status=$?
  done
  return "$status"
}
if [[ "$DEV_TASK" == net-unit || "$DEV_TASK" == net-node || "$DEV_TASK" == net-vendor || "$DEV_TASK" == net-congestion ]]; then
  if ! cargo fmt --all -- --check 2>&1 | tee "$REPORT_DIR/format.log"; then
    cargo fmt --all
    git diff --binary > "$REPORT_DIR/format.patch"
    exit 1
  fi
  if [[ "$DEV_TASK" == net-unit ]]; then
    cargo test --locked -p khor-net --lib 2>&1 | tee "$REPORT_DIR/net-unit.log"
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
  fi
else
  # Prove the actual namespace, veth, netem and firewall capabilities before compiling.
  unshare -Urn sh -ec 'ip link add probe0 type dummy; ip link set probe0 up; tc qdisc add dev probe0 root netem delay 1ms; nft add table inet probe; nft list ruleset' > "$REPORT_DIR/namespace.log" 2>&1
  export NETLAB_RUN_DIR="$REPORT_DIR/netlab"
  if [[ "$DEV_TASK" == netlab-smoke ]]; then
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
