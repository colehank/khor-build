#!/bin/bash
set -euo pipefail
source /Users/z13/khor-builder/env.sh
python3 -c 'import hashlib,sys; assert sys.version_info >= (3,11) and hasattr(hashlib,"file_digest"), "Builder needs Python 3.11 or newer"'
export GIT_CONFIG_GLOBAL=/dev/null
export CARGO_TERM_COLOR=never
export CARGO_HTTP_MULTIPLEXING=false
if [[ ! "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]; then
  echo 'source_sha must be a full lowercase Git commit SHA' >&2
  exit 2
fi
source_dir=/Users/z13/khor-builder/work/khor
source_repo="$SOURCE_PACKAGE/source.git"
test "$(git --git-dir="$source_repo" rev-parse HEAD)" = "$SOURCE_SHA"
if [[ ! -d "$source_dir/.git" ]]; then
  git init "$source_dir"
  git -C "$source_dir" remote add origin https://github.com/colehank/khor.git
fi
if [[ -n "$(git -C "$source_dir" status --porcelain --untracked-files=no)" ]]; then
  echo 'Tracked source edits exist; preserving them and refusing checkout' >&2
  exit 2
fi
git -C "$source_dir" -c protocol.file.allow=always fetch --depth=1 --update-shallow "$source_repo" "$SOURCE_SHA"
git -C "$source_dir" checkout --detach "$SOURCE_SHA"
test "$(git -C "$source_dir" rev-parse HEAD)" = "$SOURCE_SHA"
cd "$source_dir"
export KHOR_BUILD_SOURCE="$SOURCE_SHA"
mkdir -p "$REPORT_DIR" /Users/z13/khor-builder/cache/agent
export KHOR_AGENT_BUNDLE=/Users/z13/khor-builder/cache/agent/khor-agent-native
export KHOR_TEST_AGENT_BIN="$KHOR_AGENT_BUNDLE"
export KHOR_TEST_MCP_EXECUTABLE="$CARGO_TARGET_DIR/debug/khor"
export KHOR_RUNTIME_DIR="$RUNNER_TEMP/runtime-images-$GITHUB_RUN_ID"
printf '%s\n' "$SOURCE_SHA" > "$REPORT_DIR/source-sha.txt"
run_tests() {
  local name=$1
  shift
  cargo test --locked "$@" -- --test-threads=1 2>&1 | tee "$REPORT_DIR/$name.log"
  grep -Eq '^test result: ok\. [1-9][0-9]* passed;' "$REPORT_DIR/$name.log"
}
sync_swift_mirror() {
  local prepared="$SOURCE_PACKAGE/swift-rs.git"
  local cached=/Users/z13/khor-builder/cache/git-mirrors/swift-rs.git
  local config=/Users/z13/.gitconfig
  test "$(git --git-dir="$prepared" rev-parse --is-bare-repository)" = true
  mkdir -p "$(dirname "$cached")"
  if [[ -d "$cached" ]]; then
    git -c protocol.file.allow=always --git-dir="$cached" fetch --force --prune "$prepared" '+refs/*:refs/*'
  else
    git -c protocol.file.allow=always clone --mirror "$prepared" "$cached"
  fi
  git --git-dir="$cached" symbolic-ref HEAD "$(git --git-dir="$prepared" symbolic-ref HEAD)"
  test "$(git --git-dir="$cached" rev-parse HEAD)" = "$(cat "$SOURCE_PACKAGE/swift-rs-sha.txt")"
  if [[ -L "$config" || ( -e "$config" && ( ! -f "$config" || ! -O "$config" ) ) ]]; then
    echo 'Preserving an unexpected global Git configuration' >&2
    exit 2
  fi
  git config --file "$config" "url.file://$cached.insteadOf" https://github.com/Brendonovich/swift-rs
  env -u GIT_CONFIG_GLOBAL -u GIT_CONFIG_COUNT -u GIT_CONFIG_KEY_0 -u GIT_CONFIG_VALUE_0 GIT_ALLOW_PROTOCOL=file \
    git -c credential.helper= ls-remote https://github.com/Brendonovich/swift-rs HEAD > "$REPORT_DIR/swift-global-mirror.txt"
  grep -q "$(cat "$SOURCE_PACKAGE/swift-rs-sha.txt")" "$REPORT_DIR/swift-global-mirror.txt"
  cp "$SOURCE_PACKAGE/swift-rs-sha.txt" "$REPORT_DIR/swift-rs-sha.txt"
}
case "${CHECK:-full}" in
  borrow)
    rustfmt --edition 2024 --emit stdout apps/gui/core/src/web.rs > "$REPORT_DIR/formatted-web.rs"
    rustfmt --edition 2024 --emit stdout apps/gui/src-tauri/src/phone_page.rs > "$REPORT_DIR/formatted-phone_page.rs"
    rustfmt --edition 2024 --config skip_children=true --emit stdout apps/gui/src-tauri/src/lib.rs > "$REPORT_DIR/formatted-lib.rs"
    run_tests borrowed-loopback -p khor-gui-core --lib web::loopback_tests
    webkit_fixture=apps/gui/core/tests/fixtures/webkit_loopback.swift
    if [[ ! -f "$webkit_fixture" ]]; then
      webkit_fixture="$SOURCE_PACKAGE/webkit-loopback.swift"
    fi
    xcrun swiftc -swift-version 5 "$webkit_fixture" -o "$REPORT_DIR/webkit-probe"
    export KHOR_WEBKIT_PROBE="$REPORT_DIR/webkit-probe"
    cargo test --locked -p khor-gui-core --lib web::loopback_tests::native_webkit_reaches_the_borrowed_loopback_origin -- --ignored --test-threads=1 2>&1 | tee "$REPORT_DIR/borrowed-native-webkit.log"
    grep -Eq '^test result: ok\. 1 passed;' "$REPORT_DIR/borrowed-native-webkit.log"
    npm ci --prefix apps/gui
    npm run build --prefix apps/gui
    cargo check --locked -p khor-gui --lib
    sync_swift_mirror
    rustup target add aarch64-apple-ios
    export GIT_CONFIG_COUNT=1
    export GIT_CONFIG_KEY_0="url.file://$SOURCE_PACKAGE/swift-rs.git.insteadOf"
    export GIT_CONFIG_VALUE_0=https://github.com/Brendonovich/swift-rs
    cargo check --locked -p khor-gui --lib --target aarch64-apple-ios
    exit 0
    ;;
  session-startup)
    python3 "$SOURCE_PACKAGE/session-startup.py"
    exit 0
    ;;
  dependencies)
    sync_swift_mirror
    exit 0
    ;;
  release-preflight|ios-release-preflight)
    if [[ "$CHECK" = ios-release-preflight ]]; then
      export KHOR_PREFLIGHT_PHASE=ios
    fi
    test "$(git --git-dir="$SOURCE_PACKAGE/swift-rs.git" rev-parse --is-bare-repository)" = true
    sync_swift_mirror
    export GIT_CONFIG_COUNT=1
    export GIT_CONFIG_KEY_0="url.file://$SOURCE_PACKAGE/swift-rs.git.insteadOf"
    export GIT_CONFIG_VALUE_0=https://github.com/Brendonovich/swift-rs
    python3 "$SOURCE_PACKAGE/release-preflight.py"
    exit 0
    ;;
  cli-update-lab)
    npm ci --prefix apps/agent
    node apps/agent/build.mjs aarch64-apple-darwin "$KHOR_AGENT_BUNDLE"
    npm ci --prefix apps/gui
    python3 "$SOURCE_PACKAGE/cli-update-lab.py"
    exit 0
    ;;
  update-lab|update-lab-interrupt)
    if [[ "$CHECK" = update-lab-interrupt ]]; then
      export KHOR_UPDATE_LAB_CASE=interrupt
    fi
    npm ci --prefix apps/agent
    node apps/agent/build.mjs aarch64-apple-darwin "$KHOR_AGENT_BUNDLE"
    npm ci --prefix apps/gui
    npm run build --prefix apps/gui
    cargo test --locked -p khor-gui --lib updater_package::tests -- --test-threads=1 2>&1 | tee "$REPORT_DIR/updater-initialization.log"
    python3 "$SOURCE_PACKAGE/update-lab.py"
    exit 0
    ;;
  files)
    test "$(git --git-dir="$SOURCE_PACKAGE/swift-rs.git" rev-parse --is-bare-repository)" = true
    cp "$SOURCE_PACKAGE/swift-rs-sha.txt" "$REPORT_DIR/swift-rs-sha.txt"
    run_tests original-file-handoff -p khor-gui-core --lib file_handoff::tests
    npm ci --prefix apps/gui
    npm run build --prefix apps/gui
    cargo check --locked -p khor-gui --lib
    rustup target add aarch64-apple-ios
    export GIT_CONFIG_COUNT=1
    export GIT_CONFIG_KEY_0="url.file://$SOURCE_PACKAGE/swift-rs.git.insteadOf"
    export GIT_CONFIG_VALUE_0=https://github.com/Brendonovich/swift-rs
    cargo check --locked -p khor-gui --lib --target aarch64-apple-ios
    exit 0
    ;;
  ssh-evidence)
    python3 - "$CARGO_TARGET_DIR/tmp" "$REPORT_DIR" <<'PYSSH'
import json, shutil, sys
from pathlib import Path
root, report = map(Path, sys.argv[1:])
cases = sorted(root.glob("add-ssh-*"), key=lambda p: p.stat().st_mtime, reverse=True)
assert cases, "No retained SSH fixture"
case = cases[0]
for name in ["origin.log", "sponsor.log"]:
    path = case / name
    if path.is_file():
        shutil.copyfile(path, report / ("ssh-" + name))
summary = {"fixture": str(case), "endpoints": {}}
for node in ["origin", "sponsor"]:
    path = case / node / ".khor/endpoint.json"
    if path.is_file():
        value = json.loads(path.read_text())
        summary["endpoints"][node] = {k: value.get(k) for k in ["pid", "addrs", "app"]}
(report / "ssh-fixture-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
PYSSH
    exit 0
    ;;
  ssh)
    test -x "$KHOR_AGENT_BUNDLE"
    npm ci --prefix apps/gui
    python3 scripts/check-ssh-install.py --output "$REPORT_DIR/ssh-native"
    exit 0
    ;;
  startup)
    python3 - "$CARGO_TARGET_DIR/debug/khor" "$REPORT_DIR" <<'PYSTARTUP'
import json, os, shutil, signal, subprocess, sys, tempfile, time
from pathlib import Path
program, report = sys.argv[1], Path(sys.argv[2])
with tempfile.TemporaryDirectory(prefix="khor-startup-profile-") as home:
    timings = []
    for index in range(3):
        candidate = Path(home) / f"cold-candidate-{index}"
        shutil.copyfile(program, candidate)
        candidate.chmod(0o700)
        before_spawn = time.monotonic()
        cold = subprocess.Popen([str(candidate), "_runtime_info"], env={}, start_new_session=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        spawned = time.monotonic()
        try:
            cold_out, cold_err = cold.communicate(timeout=20)
        finally:
            if cold.poll() is None:
                cold.kill()
                cold.wait()
        timings.append({"spawn_seconds": spawned - before_spawn, "execution_seconds": time.monotonic() - spawned, "returncode": cold.returncode, "stdout_bytes": len(cold_out), "stderr": cold_err.decode()})
        candidate.unlink()
    (report / "cold-candidate-timing.json").write_text(json.dumps(timings, indent=2) + "\n")
    copied = Path(home) / "khor-copy"
    shutil.copyfile(program, copied)
    copied.chmod(0o700)
    started = time.monotonic()
    probe = subprocess.Popen([str(copied), "_runtime_info"], env={}, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        time.sleep(0.5)
        if probe.poll() is None:
            subprocess.run(["/usr/bin/sample", str(probe.pid), "2", "-file", str(report / "copied-probe-sample.txt")], timeout=10, check=False)
        out, err = probe.communicate(timeout=20)
    finally:
        if probe.poll() is None:
            probe.kill()
            probe.wait()
    (report / "copied-probe.json").write_text(json.dumps({"seconds": time.monotonic() - started, "returncode": probe.returncode, "stdout": out.decode(), "stderr": err.decode()}, indent=2) + "\n")
    env = dict(os.environ, KHOR_HOME=home, KHOR_NAME="startup-profile", KHOR_WEB_PORT="0", KHOR_RUNTIME_DIR=str(Path(home) / "runtime"))
    env.pop("KHOR_SESSION", None)
    process = subprocess.Popen([program, "serve"], env=env, start_new_session=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        time.sleep(5)
        if process.poll() is None:
            subprocess.run(["/usr/bin/sample", str(process.pid), "2", "-file", str(report / "startup-sample.txt")], timeout=10, check=False)
    finally:
        if process.poll() is None:
            os.killpg(process.pid, signal.SIGTERM)
        try:
            out, err = process.communicate(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            out, err = process.communicate(timeout=10)
        (report / "startup-stdout.txt").write_bytes(out)
        (report / "startup-stderr.txt").write_bytes(err)
print("Startup profiling completed; this diagnostic does not replace keeper acceptance.")
PYSTARTUP
    exit 0
    ;;
  approval|close|pair)
    test -x "$KHOR_AGENT_BUNDLE"
    # Targeted checks do not replace full provenance, frontend or release gates.
    cargo test --locked -p khor-node --lib --no-run --message-format=json > "$REPORT_DIR/targeted-artifacts.json"
    python3 - "$CHECK" "$REPORT_DIR" <<'PYTEST'
import json, runpy, sys
from pathlib import Path
sys.path.insert(0, str(Path.cwd() / "scripts"))
supervisor = runpy.run_path("scripts/run-node-acceptance.py")
check, report = sys.argv[1], Path(sys.argv[2])
name = {
    "approval": "khor_agent::conversations::tests::approval_answers_reach_only_the_conversation_that_owns_the_run",
    "close": "khor_agent::runtime_close_tests::running_token_retry_is_idempotent_and_its_record_writer_blocks_close",
    "pair": "link::pairing::tests::unfinished_bootstrap_is_progress_and_never_a_false_completion",
}[check]
binary, _ = supervisor["select_binary"](report / "targeted-artifacts.json", Path.cwd())
arguments = [name, "--exact", "--nocapture", "--test-threads=1"] + (["--ignored"] if check == "close" else [])
result, raw = supervisor["run"](binary, arguments, report / f"{check}.log", Path.cwd())
supervisor["require_test_success"](raw, name, set())
(report / f"{check}-result.json").write_text(json.dumps(result, indent=2) + "\n")
print(raw)
PYTEST
    exit 0
    ;;
  contracts)
    test -x "$KHOR_AGENT_BUNDLE"
    run_tests acp-setup -p khor-node --lib acp_management::tests
    run_tests acp-process -p khor-node --lib acp_process::tests
    run_tests runtime-image -p khor-cli --test runtime_image
    run_tests agent-hook-retirement -p khor-cli --test run
    run_tests agent-recognition -p khor-node --lib recognition::tests
    run_tests agent-native-status -p khor-node --lib adaptor::claude::
    run_tests agent-native-unread -p khor-node --lib native_claude_state_supersedes_retired_hook_reports
    run_tests agent-terminal-seat -p khor-node --lib a_khor_terminal_wears_the_agent_inside_it
    run_tests agent-native-identity -p khor-node --lib a_native_status_file_identifies_the_vendor
    run_tests agent-roster -p khor-node --lib what_is_on_the_list_is_the_users_choice
    run_tests conversations -p khor-node --lib khor_agent::conversations::tests
    run_tests conversation-commands -p khor-gui-core --lib console::tests::slash_receipts
    run_tests pairing-progress -p khor-node --lib unfinished_bootstrap_is_progress
    exit 0
    ;;
  keeper)
    test -x "$KHOR_AGENT_BUNDLE"
    python3 - "$CARGO_TARGET_DIR/debug/khor" "$REPORT_DIR" <<'PYPROFILE'
import json, subprocess, sys, time
from pathlib import Path
program, report = sys.argv[1], Path(sys.argv[2])
started = time.monotonic()
with subprocess.Popen([program, "_runtime_check", program], stdout=subprocess.PIPE, stderr=subprocess.PIPE) as process:
    try:
        time.sleep(1)
        if process.poll() is None:
            subprocess.run(["/usr/bin/sample", str(process.pid), "2", "-file", str(report / "activation-sample.txt")], timeout=10, check=False)
        out, err = process.communicate(timeout=120)
    except BaseException:
        process.kill()
        process.wait()
        raise
result = {"seconds": time.monotonic() - started, "returncode": process.returncode, "stdout": out.decode(), "stderr": err.decode()}
(report / "activation-timing.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result))
PYPROFILE
    run_tests keeper -p khor-cli --test serve_keeper
    exit 0
    ;;
  full) ;;
  *) echo 'Unknown validation scope' >&2; exit 2 ;;
esac
npm ci --prefix apps/agent
(cd apps/agent && node build.mjs aarch64-apple-darwin "$KHOR_AGENT_BUNDLE")
export KHOR_TEST_AGENT_BIN="$KHOR_AGENT_BUNDLE"
run_tests runtime-update-domain -p khor-runtime -p khor-update
run_tests acp-setup -p khor-node --lib acp_management::tests
run_tests acp-process -p khor-node --lib acp_process::tests
cargo test --locked -p khor-runtime actual_standalone_agent -- --ignored --nocapture --test-threads=1 2>&1 | tee "$REPORT_DIR/actual-agent-runtime.log"
run_tests semantic-media -p khor-node --lib adaptor::media::tests
run_tests semantic-claude -p khor-node --lib adaptor::claude::tests
run_tests semantic-codex -p khor-node --lib adaptor::codex::tests
run_tests setting-receipts -p khor-node --lib gui_host::setting_receipt_tests
run_tests full-khor-history -p khor-node --lib khor_agent::history::tests
run_tests redacted-summary -p khor-node --lib khor_agent::updates::tests
run_tests native-file-handoff -p khor-gui-core --lib file_handoff::
run_tests native-history-export -p khor-gui-core --lib history_export::
npm ci --prefix apps/gui
npm run build --prefix apps/gui
cargo test --locked -p khor-gui --lib update -- --test-threads=1 2>&1 | tee "$REPORT_DIR/desktop-update-contracts.log"
if [[ -f apps/gui/src-tauri/src/app_companions.rs ]]; then
  cargo test --locked -p khor-gui --lib app_companions -- --test-threads=1 2>&1 | tee "$REPORT_DIR/desktop-component-contracts.log"
fi
native_host=$(rustc -vV | sed -n 's/^host: //p')
test -n "$native_host"
cargo metadata --locked --format-version 1 --filter-platform "$native_host" > "$REPORT_DIR/node-metadata.json"
python3 scripts/native_provenance.py prepare --metadata "$REPORT_DIR/node-metadata.json" --target-dir "$CARGO_TARGET_DIR" --package khor-cli --name khor --kind bin --tool "$(rustup which cargo)" --tool "$(rustup which rustc)" --apply-refresh --output "$REPORT_DIR/cli-prepared.json"
cargo build --locked -p khor-cli --bin khor --message-format=json > "$REPORT_DIR/cli-artifacts.json"
python3 scripts/native_provenance.py verify --prepared "$REPORT_DIR/cli-prepared.json" --artifacts "$REPORT_DIR/cli-artifacts.json" --output "$REPORT_DIR/cli-provenance.json"
KHOR_TEST_CLI_BIN="$CARGO_TARGET_DIR/debug/khor" npm test --prefix apps/agent 2>&1 | tee "$REPORT_DIR/agent-runtime-and-cli.log"
python3 scripts/native_provenance.py prepare --metadata "$REPORT_DIR/node-metadata.json" --target-dir "$CARGO_TARGET_DIR" --package khor-node --name khor_node --kind lib --test --tool "$(rustup which cargo)" --tool "$(rustup which rustc)" --prior "$REPORT_DIR/cli-provenance.json" --apply-refresh --output "$REPORT_DIR/node-prepared.json"
cargo test --locked -p khor-node --lib --no-run --message-format=json > "$REPORT_DIR/node-artifacts.json"
python3 scripts/native_provenance.py verify --prepared "$REPORT_DIR/node-prepared.json" --artifacts "$REPORT_DIR/node-artifacts.json" --output "$REPORT_DIR/node-provenance.json"
python3 scripts/run-node-acceptance.py --suite runtime-close --artifacts "$REPORT_DIR/node-artifacts.json" --provenance "$REPORT_DIR/node-provenance.json" --worker-provenance "$REPORT_DIR/cli-provenance.json" --output "$REPORT_DIR/node-native-close"
run_tests runtime-image -p khor-cli --test runtime_image
run_tests agent-hook-retirement -p khor-cli --test run
run_tests agent-recognition -p khor-node --lib recognition::tests
run_tests agent-native-status -p khor-node --lib adaptor::claude::
run_tests agent-native-unread -p khor-node --lib native_claude_state_supersedes_retired_hook_reports
run_tests agent-terminal-seat -p khor-node --lib a_khor_terminal_wears_the_agent_inside_it
run_tests agent-native-identity -p khor-node --lib a_native_status_file_identifies_the_vendor
run_tests agent-roster -p khor-node --lib what_is_on_the_list_is_the_users_choice
run_tests conversations -p khor-node --lib khor_agent::conversations::tests
run_tests conversation-commands -p khor-gui-core --lib console::tests::slash_receipts
run_tests pi-usage -p khor-node --lib accounts::pi_usage
run_tests observer -p khor-node --lib observer_tests
run_tests transfer-owner -p khor-node --lib doc_transfer_state::io::tests
run_tests attachment-failure -p khor-node --lib connection_failure_does_not_claim_the_host_ended
run_tests handshake-recovery -p khor-node --lib interrupted_and_invalid_handshakes_have_different_recovery
run_tests explicit-refusal -p khor-node --lib a_cookie_refusal_keeps_the_hosts_reason
run_tests pairing-progress -p khor-node --lib unfinished_bootstrap_is_progress
run_tests gui-rejoin -p khor-gui-core --lib first_rejoin_is_safe_inside_a_runtime_worker
run_tests terminal-io -p khor-gui-core --lib term::io_tests::
run_tests terminal-current -p khor-gui-core --test terminal_current
run_tests account-registration -p khor-gui-core --test account_registration
run_tests keeper -p khor-cli --test serve_keeper
npm ci --prefix apps/gui
python3 scripts/check-ssh-install.py --output "$REPORT_DIR/ssh-native"
printf '%s\n' 'Khor native development checks passed.' >> "$GITHUB_STEP_SUMMARY"
