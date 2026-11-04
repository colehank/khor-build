"""Isolated native updater Lab. Its fixture trust/feed patch is never a release artifact."""
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
import threading
import time

SOURCE = Path.cwd()
REPORT = Path(os.environ["REPORT_DIR"])
SHA = os.environ["SOURCE_SHA"]
OLD = "v0.2.0-rc.900001"


def run(*args, env=None, stdout=None):
    return subprocess.run(args, cwd=SOURCE, env=env, check=True, text=True, stdout=stdout)


def replace(path, before, after):
    path = SOURCE / path
    text = path.read_text()
    if text.count(before) != 1:
        raise ValueError(f"Lab boundary changed: {path.name}")
    path.write_text(text.replace(before, after))


class Feed(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(b"[]")


def wait_for(check, seconds, detail):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        result = check()
        if result:
            return result
        time.sleep(0.2)
    raise TimeoutError(detail)


def own_processes(root):
    output = subprocess.check_output(["/bin/ps", "-axo", "pid=,command="], text=True)
    result = []
    for row in output.splitlines():
        fields = row.strip().split(None, 1)
        if len(fields) == 2 and fields[1].startswith(str(root) + "/"):
            result.append(int(fields[0]))
    return result


def main():
    REPORT.mkdir(parents=True, exist_ok=True)
    if shutil.disk_usage(SOURCE).free < 20 * 1024 ** 3:
        raise RuntimeError("Lab requires at least 20 GiB of free builder space")
    with tempfile.TemporaryDirectory(prefix="khor-updater-lab-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        server = ThreadingHTTPServer(("127.0.0.1", 0), Feed)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        address = f"http://127.0.0.1:{server.server_port}"
        label = f"io.github.colehank.khor.update-lab.{os.environ['GITHUB_RUN_ID']}"
        domain = f"gui/{os.getuid()}"
        bootstrapped = False
        lab_binary_digest = None
        originals = {}
        evidence = {"source": SHA, "mode": "native UI smoke only", "published": False,
                    "formal_signing_key": "never read or transferred"}
        try:
            # npm's locked, official Tauri signer creates an ephemeral Lab key only.
            signer = SOURCE / "apps/gui/node_modules/.bin/tauri"
            key = root / "fixture.key"
            run(str(signer), "signer", "generate", "--ci", "--password", "public-lab-fixture",
                "--write-keys", str(key), stdout=subprocess.DEVNULL)
            for name in ["crates/update/updater.pub", "crates/update/src/feed.rs", "apps/gui/src-tauri/src/updater_package.rs", "apps/gui/src-tauri/src/updater_commands.rs"]:
                originals[name] = (SOURCE / name).read_bytes()
            (SOURCE / "crates/update/updater.pub").write_bytes(key.with_suffix(".key.pub").read_bytes())
            replace("crates/update/src/feed.rs", "https://api.github.com/repos/colehank/khor/releases?per_page=100", address + "/releases")
            # This prefix occurs once in each production adapter; package signatures still remain mandatory.
            replace("crates/update/src/feed.rs", "https://github.com/colehank/khor/releases/download/", address + "/assets/")
            replace("apps/gui/src-tauri/src/updater_package.rs", "https://github.com/colehank/khor/releases/download/", address + "/assets/")
            replace("apps/gui/src-tauri/src/updater_commands.rs", "    crate::app_updates::start(&app);", """    crate::app_updates::start(&app);
    if let Ok(path)=std::env::var("KHOR_UPDATE_LAB_READY") {
        std::fs::write(path,khor_update::build_identity()?.release.ok_or("Lab build identity missing")?.tag.as_str()).map_err(|e|e.to_string())?;
    }""")
            diff = subprocess.check_output(["git", "diff", "--binary"], cwd=SOURCE)
            (REPORT / "lab-boundary.patch").write_bytes(diff)
            evidence["boundary_patch_sha256"] = hashlib.sha256(diff).hexdigest()
            environment = os.environ.copy()
            environment.update(KHOR_BUILD_SOURCE=SHA, KHOR_RELEASE_TAG=OLD, KHOR_RELEASE_BUILD="900001",
                TAURI_CONFIG=json.dumps({"identifier": label, "productName": "Khor Update Lab",
                    "plugins": {"updater": {"dangerousInsecureTransportProtocol": True}}}))
            for name in ["TAURI_SIGNING_PRIVATE_KEY", "TAURI_SIGNING_PRIVATE_KEY_PATH", "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"]:
                environment.pop(name, None)
            run("cargo", "build", "--locked", "-p", "khor-gui", "--bin", "khor-gui", env=environment)
            app = root / "KhorUpdateLab.app"
            program = app / "Contents/MacOS/khor-gui"
            program.parent.mkdir(parents=True)
            shutil.copyfile(Path(environment["CARGO_TARGET_DIR"]) / "debug/khor-gui", program)
            program.chmod(0o755)
            (app / "Contents/Info.plist").write_bytes(plistlib.dumps({"CFBundleName": "Khor Update Lab",
                "CFBundleDisplayName": "Khor Update Lab", "CFBundleIdentifier": label,
                "CFBundleExecutable": "khor-gui", "CFBundlePackageType": "APPL",
                "CFBundleVersion": "900001", "CFBundleShortVersionString": "0.2.0", "NSHighResolutionCapable": True}))
            run("codesign", "--force", "--deep", "--sign", "-", str(app))
            evidence["program"] = json.loads(subprocess.check_output([str(program), "_release_info"], env=environment))
            identity = evidence["program"]["build"]["release"]
            if identity["tag"] != OLD or identity["source"] != SHA:
                raise AssertionError("Lab executable has a different source or release identity")
            with program.open("rb") as stream:
                evidence["program_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
            lab_binary_digest = evidence["program_sha256"]
            home = root / "data"
            runtime = root / "runtime"
            ready = root / "ui.ready"
            log = root / "app.log"
            variables = {"KHOR_HOME": str(home), "KHOR_RUNTIME_DIR": str(runtime), "KHOR_NAME": "Khor Update Lab",
                         "KHOR_WEB_PORT": "0", "KHOR_UPDATE_LAB_READY": str(ready), "PATH": environment["PATH"]}
            hook_environment = dict(environment, **variables)
            before = time.monotonic()
            hook = subprocess.Popen([str(program), "state", "--hook"], env=hook_environment,
                                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            try:
                code = hook.wait(timeout=3)
            finally:
                if hook.poll() is None:
                    hook.kill()
                    hook.wait()
                hook.stdin.close()
            if code != 0 or home.exists() or ready.exists():
                raise AssertionError("retired Hook initialized the native App or its profile")
            evidence["retired_hook"] = {"code": code, "elapsed_ms": round((time.monotonic()-before)*1000),
                                        "stdin": "kept open until process exit", "profile_created": False}
            # A short-lived LaunchAgent gives a daemon runner the logged-in user's Aqua bootstrap.
            agent = root / "launch.plist"
            agent.write_bytes(plistlib.dumps({"Label": label, "ProgramArguments": [str(program)],
                "EnvironmentVariables": variables, "RunAtLoad": True, "KeepAlive": False,
                "StandardOutPath": str(log), "StandardErrorPath": str(log), "WorkingDirectory": str(root)}))
            run("launchctl", "bootstrap", domain, str(agent))
            bootstrapped = True
            wait_for(lambda: ready.exists() and ready.read_text() == OLD, 90, "real native App did not restore its React UI")
            evidence["native_ui_ready"] = ready.read_text()
            evidence["processes"] = own_processes(root)
            print("Native updater Lab smoke passed: actual signed-bundle entry, isolated profile and restored React UI")
        finally:
            if bootstrapped:
                subprocess.run(["launchctl", "bootout", domain + "/" + label], check=False, capture_output=True)
            for pid in own_processes(root):
                try:
                    # Recheck the exact ephemeral executable prefix immediately before termination.
                    if pid in own_processes(root):
                        os.kill(pid, 15)
                except ProcessLookupError:
                    pass
            for _ in range(20):
                if not own_processes(root):
                    break
                time.sleep(0.2)
            for pid in own_processes(root):
                try:
                    if pid in own_processes(root):
                        os.kill(pid, 9)
                except ProcessLookupError:
                    pass
            log = root / "app.log"
            if log.is_file():
                shutil.copyfile(log, REPORT / "update-lab-app.log")
            for name, contents in originals.items():
                (SOURCE / name).write_bytes(contents)
            primary = Path(os.environ["CARGO_TARGET_DIR"]) / "debug/khor-gui"
            if lab_binary_digest and primary.is_file():
                with primary.open("rb") as stream:
                    if hashlib.file_digest(stream, "sha256").hexdigest() == lab_binary_digest:
                        primary.unlink()  # Keep dependency caches, never a fake-root primary executable.
            server.shutdown()
            (REPORT / "update-lab.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
