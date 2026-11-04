"""Isolated native updater Lab. Its fixture trust/feed patch is never a release artifact."""
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
from pathlib import Path
import plistlib
import shutil
import shlex
import subprocess
import tarfile
import tempfile
import threading
import time

SOURCE = Path.cwd()
REPORT = Path(os.environ["REPORT_DIR"])
SHA = os.environ["SOURCE_SHA"]
OLD = "v0.2.0-rc.900001"
NEW = "v0.2.0-rc.900002"
FAILED = "v0.2.0-rc.900003"


def run(*args, env=None, stdout=None):
    return subprocess.run(args, cwd=SOURCE, env=env, check=True, text=True, stdout=stdout)


def replace(path, before, after):
    path = SOURCE / path
    text = path.read_text()
    if text.count(before) != 1:
        raise ValueError(f"Lab boundary changed: {path.name}")
    path.write_text(text.replace(before, after))


class Feed(BaseHTTPRequestHandler):
    routes = {}
    requests = []

    def log_message(self, *_):
        pass

    def do_GET(self):
        path = self.path.split("?", 1)[0]
        self.requests.append(path)
        body = self.routes.get(path)
        self.send_response(200 if body is not None else 404)
        self.send_header("Content-Type", "application/octet-stream")
        size = body.stat().st_size if isinstance(body, Path) else len(body or b"")
        self.send_header("Content-Length", str(size))
        self.end_headers()
        try:
            if isinstance(body, Path):
                with body.open("rb") as stream:
                    shutil.copyfileobj(stream, self.wfile)
            elif body is not None:
                self.wfile.write(body)
        except (BrokenPipeError, ConnectionResetError):
            pass


LAB_CONTROLLER = r'''
fn update_lab_start(app: tauri::AppHandle) {
    static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    let Ok(path) = std::env::var("KHOR_UPDATE_LAB_CONTROL") else { return; };
    if STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) { return; }
    tauri::async_runtime::spawn(async move {
        let control = std::path::PathBuf::from(path);
        let identity = khor_update::build_identity().unwrap().release.unwrap();
        let tag = identity.tag.as_str();
        let root = control.parent().unwrap();
        std::fs::write(root.join(format!("ready-{tag}-{}.json", std::process::id())),
            serde_json::to_vec(&serde_json::json!({"tag":tag,"pid":std::process::id()})).unwrap()).unwrap();
        let mut last = 0;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            let Some(request) = std::fs::read(&control).ok().and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok()) else { continue; };
            let id = request["id"].as_u64().unwrap_or(0);
            if id <= last || request["tag"].as_str() != Some(tag) { continue; }
            last = id;
            let state = app.state::<crate::app_updates::AppUpdates>();
            let result: Result<serde_json::Value,String> = match request["action"].as_str().unwrap_or("") {
                "status" => state.status().and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                "preferences" => match serde_json::from_value(request["preferences"].clone()) {
                    Ok(p) => state.preferences(&app,p).await.and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                    Err(e) => Err(e.to_string()),
                },
                "check" => state.check(&app).await.and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                "download" => state.download(&app).await.and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                "install" => state.install(&app,false).await.and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                "shell" => async {
                    let machine = crate::off_main(|| khor_node::Node::open(khor_node::Node::root_from_env()).map(|n| n.name().to_owned())).await?;
                    let session = crate::open_shell(machine, root.to_string_lossy().into_owned(), request["command"].as_str().unwrap().into(), Some("Updater Lab heartbeat".into()), Some(80), Some(24), request["token"].as_str().unwrap().into()).await?;
                    crate::off_main(move || {
                        let node = khor_node::Node::open(khor_node::Node::root_from_env())?;
                        let host = node.terminal_address(&khor_node::SessionId(session.clone()))?;
                        let mut stream = khor_node::host::connect_terminal(&format!("127.0.0.1:{}",host.port), &host.cookie,80,24).map_err(|e|e.to_string())?;
                        khor_node::host::write_frame(&mut stream,&khor_node::host::ClientOp::Ready).map_err(|e|e.to_string())?;
                        Ok(serde_json::Value::String(session))
                    }).await
                }.await,
                "sessions" => crate::sessions("recent".into()).await.and_then(|s| serde_json::to_value(s).map_err(|e| e.to_string())),
                _ => Err("unknown private Lab action".into()),
            };
            let bytes = serde_json::to_vec(&result).unwrap();
            std::fs::write(root.join(format!("reply-{id}.json")), bytes).unwrap();
        }
    });
}
'''


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def ready(root, tag):
    paths = sorted(root.glob(f"ready-{tag}-*.json"), key=lambda p: p.stat().st_mtime_ns)
    return json.loads(paths[-1].read_text()) if paths else None


def command(root, tag, action, wait_reply=True, **args):
    identifier = time.time_ns()
    temporary = root / "control.tmp"
    temporary.write_text(json.dumps(dict(id=identifier, tag=tag, action=action, **args)))
    temporary.replace(root / "control.json")
    if not wait_reply:
        return identifier
    wait_for(lambda: (root / f"reply-{identifier}.json").exists(), 90,
             f"native Lab action did not reply: {action}")
    reply = json.loads((root / f"reply-{identifier}.json").read_text())
    if "Err" in reply:
        raise RuntimeError(f"native Lab {action}: {reply['Err']}")
    return reply["Ok"]


def sign(signer, key, path):
    run(str(signer), "signer", "sign", "--private-key-path", str(key),
        "--password", "public-lab-fixture", str(path), stdout=subprocess.DEVNULL)
    return Path(str(path) + ".sig").read_text().strip()


def publish_fixture(root, signer, key, app, info, address):
    tag = info["build"]["release"]["tag"]
    directory = root / tag
    directory.mkdir()
    archive = directory / "KhorUpdateLab.app.tar.gz"
    with tarfile.open(archive, "w:gz", compresslevel=1, format=tarfile.GNU_FORMAT) as stream:
        stream.add(app, arcname=app.name)
    signature = sign(signer, key, archive)
    url = f"{address}/assets/{tag}/{archive.name}"
    asset = dict(component="desktop-app", platform=info["platform"], file=archive.name,
                 url=url, bytes=archive.stat().st_size,
                 unpacked_bytes=sum(p.stat().st_size for p in app.rglob("*") if p.is_file()),
                 sha256=digest(archive), signature=signature, contract=info["contract"])
    manifest = directory / "khor-release.json"
    manifest.write_text(json.dumps(dict(schema=1, identity=info["build"]["release"],
        built_from=tag, notes="Private native updater Lab; never publish", artifacts=[asset])))
    manifest_signature = sign(signer, key, manifest)
    prefix = f"/assets/{tag}/"
    Feed.routes.update({prefix + archive.name: archive, prefix + manifest.name: manifest,
                       prefix + manifest.name + ".sig": manifest_signature.encode(),
                       prefix + "khor-tauri-update.json": json.dumps(dict(version=tag[1:],
                           platforms={"darwin-aarch64": dict(url=url, signature=signature)})).encode()})
    return dict(tag=tag, archive_sha256=asset["sha256"], bytes=asset["bytes"])


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


def hosts(root):
    output = subprocess.check_output(["/bin/ps", "-axo", "pid=,command="], text=True)
    found = {}
    for row in output.splitlines():
        fields = row.strip().split(None, 1)
        if len(fields) == 2 and fields[1].startswith(str(root) + "/") and "_host" in fields[1].split():
            pid = int(fields[0])
            started = subprocess.check_output(["/bin/ps", "-p", str(pid), "-o", "lstart="], text=True).strip()
            found[pid] = dict(command=fields[1], started=started)
    return found


def main():
    REPORT.mkdir(parents=True, exist_ok=True)
    if shutil.disk_usage(SOURCE).free < 20 * 1024 ** 3:
        raise RuntimeError("Lab requires at least 20 GiB of free builder space")
    with tempfile.TemporaryDirectory(prefix="khor-updater-lab-") as temporary:
        root = Path(temporary)
        root.chmod(0o700)
        server = ThreadingHTTPServer(("127.0.0.1", 0), Feed)
        Feed.routes["/releases"] = b"[]"
        threading.Thread(target=server.serve_forever, daemon=True).start()
        address = f"http://127.0.0.1:{server.server_port}"
        label = f"io.github.colehank.khor.update-lab.{os.environ['GITHUB_RUN_ID']}"
        domain = f"gui/{os.getuid()}"
        bootstrapped = False
        lab_binary_digest = None
        originals = {}
        evidence = {"source": SHA, "mode": "native SDK installation and health rollback", "published": False,
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
            replace("apps/gui/src-tauri/src/updater_commands.rs", "    crate::app_updates::start(&app);",
                    "    crate::app_updates::start(&app);\n    update_lab_start(app.clone());")
            replace("apps/gui/src-tauri/src/updater_commands.rs",
                    '    crate::off_main(crate::app_update_install::ui_ready).await?;', r'''    {
        let tag = khor_update::build_identity()?.release.unwrap().tag;
        if std::env::var("KHOR_UPDATE_LAB_FAIL_TAG").ok().as_deref() == Some(tag.as_str())
            && std::env::var_os("KHOR_APP_UPDATE_RECORD").is_some()
        {
            let root = std::path::PathBuf::from(std::env::var("KHOR_UPDATE_LAB_CONTROL").unwrap());
            std::fs::write(root.with_file_name("failed-ui.json"), serde_json::to_vec(&serde_json::json!({"tag":tag,"pid":std::process::id()})).unwrap()).unwrap();
            app.exit(89);
            return Err("private Lab UI health failure".into());
        }
        crate::off_main(crate::app_update_install::ui_ready).await?;
    }''')
            commands = SOURCE / "apps/gui/src-tauri/src/updater_commands.rs"
            commands.write_text(commands.read_text() + LAB_CONTROLLER)
            diff = subprocess.check_output(["git", "diff", "--binary"], cwd=SOURCE)
            (REPORT / "lab-boundary.patch").write_bytes(diff)
            evidence["boundary_patch_sha256"] = hashlib.sha256(diff).hexdigest()
            environment = os.environ.copy()
            environment.update(KHOR_BUILD_SOURCE=SHA, KHOR_RELEASE_TAG=OLD, KHOR_RELEASE_BUILD="900001",
                TAURI_CONFIG=json.dumps({"identifier": label, "productName": "Khor Update Lab",
                    "plugins": {"updater": {"dangerousInsecureTransportProtocol": True}}}))
            for name in ["TAURI_SIGNING_PRIVATE_KEY", "TAURI_SIGNING_PRIVATE_KEY_PATH", "TAURI_SIGNING_PRIVATE_KEY_PASSWORD"]:
                environment.pop(name, None)
            app = root / "KhorUpdateLab.app"
            program = app / "Contents/MacOS/khor-gui"
            program.parent.mkdir(parents=True)
            evidence["builds"] = []
            for tag in [NEW, FAILED, OLD]:
                number = tag.rsplit(".", 1)[1]
                environment.update(KHOR_RELEASE_TAG=tag, KHOR_RELEASE_BUILD=number)
                run("cargo", "build", "--locked", "-p", "khor-gui", "--bin", "khor-gui", env=environment)
                primary = Path(environment["CARGO_TARGET_DIR"]) / "debug/khor-gui"
                lab_binary_digest = digest(primary)
                shutil.copyfile(primary, program)
                program.chmod(0o755)
                (app / "Contents/Info.plist").write_bytes(plistlib.dumps({"CFBundleName": "Khor Update Lab",
                    "CFBundleDisplayName": "Khor Update Lab", "CFBundleIdentifier": label,
                    "CFBundleExecutable": "khor-gui", "CFBundlePackageType": "APPL",
                    "CFBundleVersion": number, "CFBundleShortVersionString": "0.2.0", "NSHighResolutionCapable": True}))
                run("codesign", "--force", "--deep", "--sign", "-", str(app))
                info = json.loads(subprocess.check_output([str(program), "_release_info"], env=environment))
                if info["build"]["release"]["tag"] != tag or info["build"]["source"] != SHA:
                    raise AssertionError("Lab executable has a different source or release identity")
                record = dict(program=info, program_sha256=digest(program))
                if tag != OLD:
                    record.update(publish_fixture(root, signer, key, app, info, address))
                evidence["builds"].append(record)
            evidence["program"] = info
            identity = evidence["program"]["build"]["release"]
            if identity["tag"] != OLD or identity["source"] != SHA:
                raise AssertionError("Lab executable has a different source or release identity")
            with program.open("rb") as stream:
                evidence["program_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
            lab_binary_digest = evidence["program_sha256"]
            home = root / "data"
            runtime = root / "runtime"
            log = root / "app.log"
            variables = {"KHOR_HOME": str(home), "KHOR_RUNTIME_DIR": str(runtime), "KHOR_NAME": "Khor Update Lab",
                         "KHOR_WEB_PORT": "0", "KHOR_UPDATE_LAB_CONTROL": str(root / "control.json"),
                         "KHOR_UPDATE_LAB_FAIL_TAG": FAILED, "PATH": environment["PATH"]}
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
            if code != 0 or home.exists() or ready(root, OLD):
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
            evidence["initial_ui"] = wait_for(lambda: ready(root, OLD), 90, "real native App did not restore its React UI")
            wait_for(lambda: command(root, OLD, "status")["phase"] not in ["checking", "downloading"], 30,
                     "initial automatic check did not finish")
            preferences = dict(channel="beta", automatic_check=False, automatic_download=False, install_on_start=True)
            command(root, OLD, "preferences", preferences=preferences)
            heartbeat = root / "heartbeat"
            shell = "while :; do date +%s >> " + shlex.quote(str(heartbeat)) + "; sleep 0.25; done"
            session = command(root, OLD, "shell", command=shell, token="native-update-lab-old")
            before_hosts = wait_for(lambda: hosts(root), 30, "native Shell host did not start")
            wait_for(lambda: heartbeat.exists() and heartbeat.stat().st_size > 0, 30, "Shell heartbeat did not start")
            evidence["old_host"] = dict(session=session, processes=before_hosts)
            Feed.routes["/releases"] = json.dumps([dict(tag_name=NEW, draft=False, prerelease=True)]).encode()
            checked = command(root, OLD, "check")
            if checked["available"] != NEW:
                raise AssertionError(f"manual native discovery failed: {checked}")
            downloaded = command(root, OLD, "download")
            if downloaded["phase"] != "ready":
                raise AssertionError(f"actual SDK download did not become ready: {downloaded}")
            time.sleep(2)
            if ready(root, NEW):
                raise AssertionError("a fresh download restarted the App during this launch")
            command(root, OLD, "install", wait_reply=False)
            new_ui = wait_for(lambda: ready(root, NEW), 90, "installed native App did not restore its React UI")
            if new_ui["pid"] == evidence["initial_ui"]["pid"]:
                raise AssertionError("SDK installation did not relaunch the native App")
            wait_for(lambda: any(json.loads(p.read_text())["phase"] == "Complete"
                                 for p in runtime.glob("app-updates/install-*/receipt.json")), 30,
                     "independent updater monitor did not confirm actual UI health")
            if hosts(root) != before_hosts:
                raise AssertionError("the original Shell host changed across native SDK update")
            heartbeat_before = heartbeat.stat().st_size
            wait_for(lambda: heartbeat.stat().st_size > heartbeat_before, 10, "original task stopped after update")
            rows = command(root, NEW, "sessions")
            if session not in json.dumps(rows):
                raise AssertionError("updated native App lost its existing Shell session")
            if digest(program) != evidence["builds"][0]["program_sha256"]:
                raise AssertionError("installed App bytes do not match the actual new build")
            evidence["sdk_update"] = dict(ui=new_ui, phase="Complete", host_unchanged=True, task_continues=True)
            next_session = command(root, NEW, "shell", command=shell, token="native-update-lab-new")
            after_hosts = wait_for(lambda: hosts(root) if len(hosts(root)) > len(before_hosts) else None, 30,
                                   "new Shell did not start after update")
            old_commands = {value["command"].split()[0] for value in before_hosts.values()}
            new_commands = {value["command"].split()[0] for pid, value in after_hosts.items() if pid not in before_hosts}
            if not new_commands or old_commands & new_commands:
                raise AssertionError("new Shell reused the previous runtime executable")
            evidence["new_host"] = dict(session=next_session, processes=after_hosts, new_runtime=True)
            Feed.routes["/releases"] = json.dumps([dict(tag_name=FAILED, draft=False, prerelease=True)]).encode()
            command(root, NEW, "check")
            downloaded = command(root, NEW, "download")
            if downloaded["phase"] != "ready":
                raise AssertionError(f"failure-case package was not ready: {downloaded}")
            command(root, NEW, "install", wait_reply=False)
            wait_for(lambda: (root / "failed-ui.json").is_file(), 90, "fault-injected build never reached React UI")
            restored = wait_for(lambda: ready(root, NEW) if ready(root, NEW) and ready(root, NEW)["pid"] != new_ui["pid"] else None,
                                90, "previous native App did not relaunch after UI health failure")
            time.sleep(3)
            if digest(program) != evidence["builds"][0]["program_sha256"] or hosts(root) != after_hosts:
                raise AssertionError("native rollback changed the retained program or existing task hosts")
            deferred = runtime / "app-updates/deferred.json"
            if not deferred.is_file() or FAILED not in deferred.read_text():
                raise AssertionError("failed update was not deferred against automatic install loops")
            evidence["health_failure_rollback"] = dict(ui=restored, rejected=json.loads((root / "failed-ui.json").read_text()),
                old_bytes_restored=True, hosts_unchanged=True, failed_offer_deferred=True)
            print("Native updater Lab passed: actual SDK install, restored React UI, continuous old Shell, new runtime and health-failure rollback")
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
            for path in (root / "runtime").glob("app-updates/install-*/*"):
                if path.name in ["worker.log", "receipt.json", "healthy.json"] and path.is_file():
                    shutil.copyfile(path, REPORT / (path.parent.name + "-" + path.name))
            evidence["feed_requests"] = Feed.requests
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
