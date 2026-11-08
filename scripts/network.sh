#!/bin/bash
# Trusted control driver: test a materialized source snapshot on disposable Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|netlab-smoke|netlab) ;; *) exit 2 ;; esac
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
if [[ "$DEV_TASK" == net-unit ]]; then
  cargo fmt --all -- --check 2>&1 | tee "$REPORT_DIR/format.log"
  cargo test --locked -p khor-net --lib 2>&1 | tee "$REPORT_DIR/net-unit.log"
else
  # Prove the actual namespace, veth, netem and firewall capabilities before compiling.
  unshare -Urn sh -ec 'ip link add probe0 type dummy; ip link set probe0 up; tc qdisc add dev probe0 root netem delay 1ms; nft add table inet probe; nft list ruleset' > "$REPORT_DIR/namespace.log" 2>&1
  export NETLAB_RUN_DIR="$REPORT_DIR/netlab"
  if [[ "$DEV_TASK" == netlab-smoke ]]; then
    bash scripts/netlab.sh paths::p2_ paths::p3_ paths::p6_
  else
    bash scripts/netlab.sh
  fi
fi
