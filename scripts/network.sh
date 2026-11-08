#!/bin/bash
# Trusted control driver: test a materialized source snapshot on disposable Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|net-node|net-vendor|netlab-smoke|netlab-dynamic|netlab-compare|netlab) ;; *) exit 2 ;; esac
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
if [[ "$DEV_TASK" == net-unit || "$DEV_TASK" == net-node || "$DEV_TASK" == net-vendor ]]; then
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
  else
    cargo test --locked -p khor-node --lib hop::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/hop-unit.log"
    cargo test --locked -p khor-node --lib wires::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/wires-unit.log"
  fi
else
  # Prove the actual namespace, veth, netem and firewall capabilities before compiling.
  unshare -Urn sh -ec 'ip link add probe0 type dummy; ip link set probe0 up; tc qdisc add dev probe0 root netem delay 1ms; nft add table inet probe; nft list ruleset' > "$REPORT_DIR/namespace.log" 2>&1
  export NETLAB_RUN_DIR="$REPORT_DIR/netlab"
  if [[ "$DEV_TASK" == netlab-smoke ]]; then
    printf '{"test_threads":2,"repeats":1,"filters":["paths::p2_","paths::p3_","paths::p6_","hop::f4_","hop::f4s_","hop::f4i_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    NETLAB_TEST_THREADS=2 bash scripts/netlab.sh paths::p2_ paths::p3_ paths::p6_ hop::f4_ hop::f4s_ hop::f4i_
  elif [[ "$DEV_TASK" == netlab-dynamic ]]; then
    printf '{"test_threads":1,"repeats":3,"filters":["paths::p7_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab paths::p7_
  elif [[ "$DEV_TASK" == netlab-compare ]]; then
    # Performance comparisons run sequentially; retain all three outcomes
    # even when one iteration exposes the baseline's expected product red.
    printf '{"test_threads":1,"repeats":3,"filters":["paths::p2_","paths::p3_","paths::p6_"]}\n' > "$REPORT_DIR/measurement-mode.json"
    repeat_netlab paths::p2_ paths::p3_ paths::p6_
  else
    bash scripts/netlab.sh
  fi
fi
