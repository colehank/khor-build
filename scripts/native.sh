#!/bin/bash
set -euo pipefail
source /Users/z13/khor-builder/env.sh
export GIT_CONFIG_GLOBAL=/dev/null
export CARGO_TERM_COLOR=never
export CARGO_HTTP_MULTIPLEXING=false
if [[ ! "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]; then
  echo 'source_sha must be a full lowercase Git commit SHA' >&2
  exit 2
fi
source_dir=/Users/z13/khor-builder/work/khor
if [[ ! -d "$source_dir/.git" ]]; then
  git init "$source_dir"
  git -C "$source_dir" remote add origin https://github.com/colehank/khor.git
fi
if [[ -n "$(git -C "$source_dir" status --porcelain --untracked-files=no)" ]]; then
  echo 'Tracked source edits exist; preserving them and refusing checkout' >&2
  exit 2
fi
git -C "$source_dir" -c credential.helper= -c http.version=HTTP/1.1 fetch --depth=1 origin "$SOURCE_SHA"
git -C "$source_dir" checkout --detach "$SOURCE_SHA"
test "$(git -C "$source_dir" rev-parse HEAD)" = "$SOURCE_SHA"
cd "$source_dir"
mkdir -p "$REPORT_DIR" /Users/z13/khor-builder/cache/agent
export KHOR_AGENT_BUNDLE=/Users/z13/khor-builder/cache/agent/khor-agent-native
export KHOR_TEST_AGENT_BIN="$KHOR_AGENT_BUNDLE"
export KHOR_TEST_MCP_EXECUTABLE="$CARGO_TARGET_DIR/debug/khor"
export KHOR_RUNTIME_DIR="$RUNNER_TEMP/runtime-images-$GITHUB_RUN_ID"
printf '%s\n' "$SOURCE_SHA" > "$REPORT_DIR/source-sha.txt"
npm ci --prefix apps/agent
npm test --prefix apps/agent
(cd apps/agent && node build.mjs aarch64-apple-darwin "$KHOR_AGENT_BUNDLE")
npm ci --prefix apps/gui
npm run build --prefix apps/gui
native_host=$(rustc -vV | sed -n 's/^host: //p')
test -n "$native_host"
cargo metadata --locked --format-version 1 --filter-platform "$native_host" > "$REPORT_DIR/node-metadata.json"
python3 scripts/native_provenance.py prepare --metadata "$REPORT_DIR/node-metadata.json" --target-dir "$CARGO_TARGET_DIR" --package khor-cli --name khor --kind bin --tool "$(rustup which cargo)" --tool "$(rustup which rustc)" --apply-refresh --output "$REPORT_DIR/cli-prepared.json"
cargo build --locked -p khor-cli --bin khor --message-format=json > "$REPORT_DIR/cli-artifacts.json"
python3 scripts/native_provenance.py verify --prepared "$REPORT_DIR/cli-prepared.json" --artifacts "$REPORT_DIR/cli-artifacts.json" --output "$REPORT_DIR/cli-provenance.json"
python3 scripts/native_provenance.py prepare --metadata "$REPORT_DIR/node-metadata.json" --target-dir "$CARGO_TARGET_DIR" --package khor-node --name khor_node --kind lib --test --tool "$(rustup which cargo)" --tool "$(rustup which rustc)" --prior "$REPORT_DIR/cli-provenance.json" --apply-refresh --output "$REPORT_DIR/node-prepared.json"
cargo test --locked -p khor-node --lib --no-run --message-format=json > "$REPORT_DIR/node-artifacts.json"
python3 scripts/native_provenance.py verify --prepared "$REPORT_DIR/node-prepared.json" --artifacts "$REPORT_DIR/node-artifacts.json" --output "$REPORT_DIR/node-provenance.json"
python3 scripts/run-node-acceptance.py --suite runtime-close --artifacts "$REPORT_DIR/node-artifacts.json" --provenance "$REPORT_DIR/node-provenance.json" --worker-provenance "$REPORT_DIR/cli-provenance.json" --output "$REPORT_DIR/node-native-close"
run_tests() {
  local name=$1
  shift
  cargo test --locked "$@" -- --test-threads=1 2>&1 | tee "$REPORT_DIR/$name.log"
  grep -Eq '^test result: ok\. [1-9][0-9]* passed;' "$REPORT_DIR/$name.log"
}
run_tests runtime-image -p khor-cli --test runtime_image
run_tests pi-usage -p khor-node --lib accounts::pi_usage
run_tests observer -p khor-node --lib observer_tests
run_tests transfer-owner -p khor-node --lib doc_transfer_state::io::tests
run_tests attachment-failure -p khor-node --lib connection_failure_does_not_claim_the_host_ended
run_tests handshake-recovery -p khor-node --lib interrupted_and_invalid_handshakes_have_different_recovery
run_tests explicit-refusal -p khor-node --lib a_cookie_refusal_keeps_the_hosts_reason
run_tests gui-rejoin -p khor-gui-core --lib first_rejoin_is_safe_inside_a_runtime_worker
run_tests terminal-io -p khor-gui-core --lib term::io_tests::
run_tests terminal-current -p khor-gui-core --test terminal_current
run_tests account-registration -p khor-gui-core --test account_registration
run_tests keeper -p khor-cli --test serve_keeper
KHOR_TEST_KEEP=1 run_tests ssh-install -p khor-cli --test add_ssh
printf '%s\n' 'Khor native development checks passed.' >> "$GITHUB_STEP_SUMMARY"
