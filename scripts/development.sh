#!/bin/bash
set -euo pipefail
source /Users/z13/khor-builder/env.sh
export GIT_CONFIG_GLOBAL=/dev/null
export CARGO_TERM_COLOR=never
export CARGO_HTTP_MULTIPLEXING=false
mkdir -p "$REPORT_DIR"
cp "$SOURCE_PACKAGE/source.json" "$REPORT_DIR/source.json"
test "$(python3 -c 'import json,os; print(json.load(open(os.environ["REPORT_DIR"]+"/source.json"))["snapshot_commit"])')" = "$SOURCE_SHA"
SOURCE_SHA=$(python3 -c 'import json,os; print(json.load(open(os.environ["REPORT_DIR"]+"/source.json"))["build_commit"])')
export SOURCE_SHA
export KHOR_BUILD_SOURCE="$SOURCE_SHA"
printf '%s\n' "$DEV_TASK" > "$REPORT_DIR/task.txt"
if [[ "$DEV_TASK" = native ]]; then
  /bin/bash "$SOURCE_PACKAGE/native.sh"
  exit 0
fi
[[ "$SOURCE_SHA" =~ ^[0-9a-f]{40}$ ]]
test "$(git --git-dir="$SOURCE_PACKAGE/source.git" rev-parse HEAD)" = "$SOURCE_SHA"
source_dir=/Users/z13/khor-builder/work/khor
if [[ ! -d "$source_dir/.git" ]]; then
  git init "$source_dir"
fi
if [[ -n "$(git -C "$source_dir" status --porcelain --untracked-files=no)" ]]; then
  echo 'Preserving remote tracked edits; refusing source replacement' >&2
  exit 2
fi
git -C "$source_dir" -c protocol.file.allow=always fetch "$SOURCE_PACKAGE/source.git" "$SOURCE_SHA"
git -C "$source_dir" checkout --detach "$SOURCE_SHA"
cd "$source_dir"
if [[ "$DEV_TASK" = resolve ]]; then
  trap 'git restore -- Cargo.lock' EXIT
  cargo update --workspace
  git diff --binary -- Cargo.lock > "$REPORT_DIR/resolve.patch"
  exit 0
fi
if [[ "$DEV_TASK" = updates ]]; then
  trap 'git restore -- .' EXIT
  python3 scripts/check-rust-format.py --write
  git diff --binary -- '*.rs' > "$REPORT_DIR/format.patch"
  cargo test --locked -p khor-update -p khor-runtime 2>&1 | tee "$REPORT_DIR/updates.log"
  python3 -m unittest discover -s scripts/tests -p test_install_activation.py 2>&1 | tee "$REPORT_DIR/install-activation.log"
  python3 scripts/check-release-install.py 2>&1 | tee "$REPORT_DIR/release-install.log"
  python3 -m unittest discover -s scripts/tests -p test_onset_build.py 2>&1 | tee "$REPORT_DIR/ssh-release-install.log"
  python3 scripts/check-onset-uninstall.py 2>&1 | tee "$REPORT_DIR/onset-uninstall.log"
  cargo test --locked -p khor-node --lib update_wire -- --test-threads=1 2>&1 | tee "$REPORT_DIR/update-wire.log"
  python3 -m unittest discover -s scripts/tests -p test_release_mirror.py 2>&1 | tee "$REPORT_DIR/release-mirror.log"
  cargo test --locked -p khor-commands --lib 2>&1 | tee "$REPORT_DIR/commands.log"
  cargo test --locked -p khor-gui-core --test command_tables 2>&1 | tee "$REPORT_DIR/native-command-boundary.log"
  cargo test --locked -p khor-cli --test runtime_image activation_check_and_incompatible_local_runtime_never_open_user_data -- --exact --test-threads=1 2>&1 | tee "$REPORT_DIR/update-activation.log"
  if [[ -f apps/gui/src-tauri/src/app_updates.rs ]]; then
    npm ci --prefix apps/gui
    npm run gen --prefix apps/gui
    npm run build --prefix apps/gui
    git diff --binary -- apps/gui/src/gen > "$REPORT_DIR/bindings.patch"
    cargo check --locked -p khor-gui --lib 2>&1 | tee "$REPORT_DIR/desktop-updates.log"
    cargo test --locked -p khor-gui --lib update -- --test-threads=1 2>&1 | tee "$REPORT_DIR/desktop-update-contracts.log"
    export KHOR_TEST_CHROME=1
    npm run chat-drafts --prefix apps/gui
    npm run workspace-memory --prefix apps/gui
    npm run updates-ui --prefix apps/gui
    export WORKSPACE_RESTART_REQUIRE=1
    export WORKSPACE_CONTENT_DIR="$REPORT_DIR/workspace-restart"
    npm run workspace-content-ui --prefix apps/gui
  fi
  exit 0
fi
install_dependencies() {
  local app=$1
  local expected
  expected=$({ shasum -a 256 "apps/$app/package-lock.json" "apps/$app/package.json"; node --version; npm --version; } | shasum -a 256 | cut -d ' ' -f 1)
  local stamp="apps/$app/node_modules/.khor-dev-lock"
  if [[ ! -f "$stamp" || "$(cat "$stamp")" != "$expected" ]]; then
    npm ci --prefix "apps/$app"
    printf '%s\n' "$expected" > "$stamp"
  fi
}
install_dependencies agent
install_dependencies gui
case "$DEV_TASK" in
  fast)
    printf 'Runner ProcessType: '
    /usr/libexec/PlistBuddy -c 'Print :ProcessType' /Library/LaunchDaemons/com.khor.github-runner.macmini.plist
    /usr/sbin/taskpolicy -B -t 0 -l 0 -p $$
    if [[ ! -x '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' ]]; then
      (cd apps/gui && node node_modules/playwright-core/cli.js install chrome)
    fi
    npm run gen --prefix apps/gui
    python3 scripts/local_checks.py gui
    export KHOR_TEST_CHROME=1
    npm run list-scroll-ui --prefix apps/gui
    npm run khor-conversations-ui --prefix apps/gui
    python3 scripts/local_checks.py agent
    file_failed=0
    for file_gate in file-handoff quicklook; do
      export FILE_HANDOFF_OUT="$REPORT_DIR/$file_gate"
      if ! npm run "$file_gate" --prefix apps/gui; then file_failed=1; fi
    done
    if node -e 'process.exit(JSON.parse(require("fs").readFileSync("apps/gui/package.json")).scripts["file-pdf-ui"]?0:1)'; then
      export FILE_HANDOFF_OUT="$REPORT_DIR/pdf"
      if ! npm run file-pdf-ui --prefix apps/gui; then file_failed=1; fi
    fi
    if [[ "$file_failed" = 1 ]]; then exit 1; fi
    ;;
  lab)
    mkdir -p /Users/z13/khor-builder/cache/agent
    export KHOR_AGENT_BUNDLE=/Users/z13/khor-builder/cache/agent/khor-agent-native
    (cd apps/agent && node build.mjs aarch64-apple-darwin "$KHOR_AGENT_BUNDLE")
    npm run build --prefix apps/gui
    export TAURI_CONFIG='{"identifier":"io.github.colehank.khor.lab","productName":"KhorLab"}'
    cargo build --locked -p khor-gui
    python3 "$SOURCE_PACKAGE/package-lab.py" "$CARGO_TARGET_DIR/debug/khor-gui" "$LAB_DIR" "$REPORT_DIR/source.json"
    ;;
  *) echo 'Unknown development task' >&2; exit 2 ;;
esac
printf '%s\n' 'Development task passed. This is NOT the trusted GitHub Quality or release gate.' | tee "$REPORT_DIR/result.txt"
{
  echo "Development task: $DEV_TASK"
  echo "Snapshot: $SOURCE_SHA"
  echo 'Source tree and public base: see dev-evidence/source.json.'
  echo 'Native device gestures and IME still require human acceptance. No production signing or installation.'
} >> "$GITHUB_STEP_SUMMARY"
