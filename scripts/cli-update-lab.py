"""Real CLI upgrade, shared launchers, paired fallback, keeper handoff and occupied GC."""
import gzip
import importlib.util
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import tempfile
import threading
import time
from http.server import ThreadingHTTPServer

spec = importlib.util.spec_from_file_location("update_lab", Path(__file__).with_name("update-lab.py"))
common = importlib.util.module_from_spec(spec)
spec.loader.exec_module(common)
SOURCE, REPORT, SHA = common.SOURCE, common.REPORT, common.SHA
OLD, NEW, FAILED, STABLE = common.OLD, common.NEW, common.FAILED, "v0.2.0"


def execute(program, environment, *arguments, input=None, timeout=90):
    process = subprocess.Popen([str(program), *arguments], env=environment, text=True,
                               stdin=subprocess.PIPE if input is not None else subprocess.DEVNULL,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        out, err = process.communicate(input, timeout=timeout)
    except subprocess.TimeoutExpired as problem:
        # Every timed operation here executes a binary inside this fixture's owned directory.
        prefix = REPORT / ("cli-timeout-" + arguments[0].strip("_").replace("/", "-"))
        subprocess.run(["/usr/bin/sample", str(process.pid), "1", "-file", str(prefix) + ".sample.txt"],
                       check=False, capture_output=True, timeout=10)
        (Path(str(prefix) + ".processes.txt")).write_text(subprocess.run(
            ["/bin/ps", "-axo", "pid=,ppid=,lstart=,stat=,command="], text=True, capture_output=True, check=False).stdout)
        subprocess.run(["/usr/sbin/lsof", "-n", "-P", "-p", str(process.pid)], text=True,
                       stdout=Path(str(prefix) + ".files.txt").open("w"), stderr=subprocess.DEVNULL, check=False, timeout=10)
        process.kill()
        try:
            out, err = process.communicate(timeout=3)
        except subprocess.TimeoutExpired:
            out, err = problem.stdout or b"", problem.stderr or b""
        for name, value in [("stdout", out), ("stderr", err)]:
            Path(str(prefix) + "." + name + ".txt").write_text(value.decode(errors="replace") if isinstance(value, bytes) else value)
        raise RuntimeError(f"owned CLI {arguments[:2]} timed out; native stack/handles/output saved") from problem
    if process.returncode:
        raise RuntimeError(f"CLI {arguments[:2]} exited {process.returncode}: {err[-4000:]}")
    return out


def main():
    REPORT.mkdir(parents=True, exist_ok=True)
    evidence = {"source": SHA, "published": False, "formal_signing_key": "never read or transferred"}
    original_key = (SOURCE / "crates/update/updater.pub").read_bytes()
    original_feed = (SOURCE / "crates/update/src/feed.rs").read_bytes()
    original_wire = (SOURCE / "crates/node/src/update_wire.rs").read_bytes()
    original_link = (SOURCE / "crates/node/src/link.rs").read_bytes()
    last_primary = None
    with tempfile.TemporaryDirectory(prefix="khor-cli-update-lab-") as temporary:
        root = Path(temporary)
        server = ThreadingHTTPServer(("127.0.0.1", 0), common.Feed)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        address = f"http://127.0.0.1:{server.server_port}"
        environment = {k: v for k, v in os.environ.items()
                       if not k.startswith(("KHOR_", "TAURI_SIGNING_"))
                       and k.lower() not in ("http_proxy", "https_proxy", "all_proxy", "no_proxy")}
        environment.update(KHOR_BUILD_SOURCE=SHA, KHOR_AGENT_BUNDLE=os.environ["KHOR_AGENT_BUNDLE"])
        primary = Path(environment["CARGO_TARGET_DIR"]) / "debug/khor"
        signer = SOURCE / "apps/gui/node_modules/.bin/tauri"
        key = root / "fixture.key"
        bin = root / "shared-home/bin"
        bin.mkdir(parents=True)
        launcher = bin / "khor"
        environments = {}
        for name in ("alpha", "beta"):
            profile = root / name / "data"
            (profile / ".khor").mkdir(parents=True)
            (profile / ".khor/relays.json").write_text(json.dumps({"relays": [{"url": "http://127.0.0.1:9"}], "only_mine": True}))
            environments[name] = dict(environment, KHOR_HOME=str(profile), KHOR_RUNTIME_DIR=str(root / name / "runtime"),
                                      KHOR_NAME="update-lab-" + name, KHOR_WEB_PORT="0")
        packages = {}

        def build(tag):
            build_env = dict(environment, KHOR_RELEASE_TAG=tag, KHOR_RELEASE_BUILD=tag.rsplit(".", 1)[1])
            subprocess.run(["cargo", "build", "--locked", "-p", "khor-cli", "--bin", "khor"], env=build_env, check=True)
            directory = root / "packages" / tag
            directory.mkdir(parents=True)
            program = directory / "khor"
            shutil.copyfile(primary, program)
            program.chmod(0o755)
            info = json.loads(execute(program, build_env, "_release_info"))
            assert info["build"]["release"]["tag"] == tag and info["build"]["source"] == SHA
            archive = directory / "khor-macos-arm64.gz"
            with program.open("rb") as source, gzip.open(archive, "wb", compresslevel=1) as output:
                shutil.copyfileobj(source, output)
            asset = dict(component="cli", platform=info["platform"], file=archive.name,
                         url=f"{address}/assets/{tag}/{archive.name}", bytes=archive.stat().st_size,
                         unpacked_bytes=program.stat().st_size, sha256=common.digest(archive),
                         signature=common.sign(signer, key, archive), contract=info["contract"])
            result = dict(program=program, info=info, archive=archive, asset=asset, sha256=common.digest(program))
            packages[tag] = result
            publish(tag, result, tag)
            return common.digest(primary)

        def publish(tag, package, built_from):
            directory = root / "feeds" / tag
            directory.mkdir(parents=True, exist_ok=True)
            manifest = directory / "khor-release.json"
            identity = dict(package["info"]["build"]["release"], tag=tag)
            manifest.write_text(json.dumps(dict(schema=1, identity=identity, built_from=built_from,
                                               notes="Private CLI Lab only", artifacts=[package["asset"]])))
            signature = common.sign(signer, key, manifest)
            signature_file = Path(str(manifest) + ".sig")
            signature_file.write_text(signature)
            prefix = f"/assets/{tag}/"
            common.Feed.routes.update({prefix + manifest.name: manifest, prefix + manifest.name + ".sig": signature.encode(),
                                       f"/assets/{built_from}/{package['archive'].name}": package["archive"]})
            package["manifest"], package["signature_file"] = manifest, signature_file
            common.Feed.routes["/releases"] = json.dumps([dict(tag_name=tag, draft=False, prerelease="-rc." in tag)]).encode()

        def endpoint(name):
            path = Path(environments[name]["KHOR_HOME"]) / ".khor/endpoint.json"
            return json.loads(path.read_text()) if path.is_file() else None

        def start(name):
            log = (root / (name + ".log")).open("w")
            process = subprocess.Popen([str(launcher), "serve"], env=environments[name], stdout=log, stderr=log,
                                       start_new_session=True)
            common.wait_for(lambda: endpoint(name), 45, f"{name} resident did not start")
            return process

        def installed(name, *args, **kwargs):
            return execute(launcher, environments[name], *args, **kwargs)

        try:
            common.run(str(signer), "signer", "generate", "--ci", "--password", "public-lab-fixture", "--write-keys", str(key), stdout=subprocess.DEVNULL)
            (SOURCE / "crates/update/updater.pub").write_bytes(key.with_suffix(".key.pub").read_bytes())
            common.replace("crates/update/src/feed.rs", "https://api.github.com/repos/colehank/khor/releases?per_page=100", address + "/releases")
            common.replace("crates/update/src/feed.rs", "https://github.com/colehank/khor/releases/download/", address + "/assets/")
            common.replace("crates/node/src/update_wire.rs", "write(send, &Response::Upload).await?;",
                           'eprintln!("private CLI Lab receiving verified transfer");\n                    write(send, &Response::Upload).await?;')
            common.replace("crates/node/src/link.rs", "let mut shutdown_reply = None;",
                           'if option_env!("KHOR_RELEASE_TAG") == Some("' + FAILED + '") { return Err("private CLI Lab startup failure".into()); }\n        let mut shutdown_reply = None;')
            (REPORT / "cli-lab-boundary.patch").write_bytes(subprocess.check_output(["git", "diff", "--binary"]))
            last_primary = build(OLD)
            for name in ("alpha", "beta"):
                package = packages[OLD]
                execute(package["program"], environments[name], "_install_cli", str(bin), OLD,
                        str(package["manifest"]), str(package["signature_file"]), str(package["archive"]))
                info = json.loads(installed(name, "_runtime_info"))
                assert info["executable"].startswith(str(root / name / "runtime"))
            assert not (bin / ".khor").exists(), "a new shared installation must not choose a shared executable"
            start("alpha")
            initial_inner = endpoint("alpha")["pid"]
            heartbeat = root / "heartbeat"
            shell = f"while :; do date +%s >> {shlex.quote(str(heartbeat))}; sleep 0.25; done"
            session = installed("alpha", "open", "-d", "--", "/bin/sh", "-c", shell).strip()
            old_hosts = common.wait_for(lambda: common.hosts(root), 30, "old CLI task host did not start")
            common.wait_for(lambda: heartbeat.is_file() and heartbeat.stat().st_size, 15, "old task heartbeat missing")
            old_program = Path(next(iter(old_hosts.values()))["command"].split(" _host ", 1)[0])
            last_primary = build(NEW)
            checked = json.loads(installed("alpha", "upgrade", "--check", "--channel", "beta", "--json"))
            assert checked["target"]["tag"] == NEW and not checked["installed"]
            outcome = json.loads(installed("alpha", "upgrade", "--channel", "beta", "--json"))
            assert outcome["installed"] and outcome["target"]["tag"] == NEW
            common.wait_for(lambda: endpoint("alpha")["pid"] != initial_inner, 45, "keeper did not adopt the new node-local selection")
            assert common.hosts(root) == old_hosts
            size = heartbeat.stat().st_size
            common.wait_for(lambda: heartbeat.stat().st_size > size, 10, "old task stopped during CLI upgrade")
            assert json.loads(installed("beta", "_release_info"))["build"]["release"]["tag"] == OLD
            new_session = installed("alpha", "open", "-d", "--", "/bin/sh", "-c", "sleep 60").strip()
            new_hosts = common.wait_for(lambda: common.hosts(root) if len(common.hosts(root)) == 2 else None, 30, "new task host did not start")
            new_program = Path(next(value["command"].split(" _host ", 1)[0] for pid, value in new_hosts.items() if pid not in old_hosts))
            assert json.loads(execute(new_program, environments["alpha"], "_release_info"))["build"]["release"]["tag"] == NEW
            assert new_program != old_program
            installed("alpha", "close", new_session)
            common.wait_for(lambda: common.hosts(root) == old_hosts, 30, "new temporary task did not close")
            evidence["local_upgrade"] = dict(outcome=outcome, old_hosts_unchanged=True, heartbeat_continues=True, other_node_still_old=True, new_task_uses_new_build=True)

            environments["beta"].update(HTTP_PROXY="http://127.0.0.1:1", http_proxy="http://127.0.0.1:1", NO_PROXY="", no_proxy="")
            start("beta")
            beta_pid = endpoint("beta")["pid"]
            ticket = installed("alpha", "invite").splitlines()[0]
            installed("beta", "pair", "-", input=ticket + "\n")
            paired = json.loads(installed("alpha", "upgrade", "--machine", "update-lab-beta", "--channel", "beta", "--json", timeout=300))
            assert paired["installed"] and paired["target"]["tag"] == NEW
            common.wait_for(lambda: endpoint("beta")["pid"] != beta_pid, 45, "paired target keeper did not adopt the new release")
            assert "private CLI Lab receiving verified transfer" in (root / "beta.log").read_text()
            assert common.hosts(root) == old_hosts
            evidence["paired_fallback"] = dict(outcome=paired, verified_transfer=True, other_tasks_unchanged=True)

            previous_pid = endpoint("alpha")["pid"]
            previous_program = Path(json.loads(installed("alpha", "_runtime_info"))["executable"])
            last_primary = build(FAILED)
            installed("alpha", "upgrade", "--version", FAILED, "--channel", "beta", "--json")
            installation = previous_program.parent.parent.parent
            def rolled_back():
                receipt = json.loads((installation / "update.json").read_text())
                return receipt if receipt["state"] == "rolled-back" else None
            receipt = common.wait_for(rolled_back, 60, "actual resident startup failure did not roll back the node selection")
            common.wait_for(lambda: endpoint("alpha")["pid"] != previous_pid, 45, "previous resident did not return after rollback")
            assert json.loads(installed("alpha", "_release_info"))["build"]["release"]["tag"] == NEW
            assert common.hosts(root) == old_hosts
            size = heartbeat.stat().st_size
            common.wait_for(lambda: heartbeat.stat().st_size > size, 10, "old task stopped during failed resident startup")
            assert "private CLI Lab startup failure" in (root / "alpha.log").read_text()
            evidence["resident_failure_rollback"] = dict(receipt=receipt, actual_previous_build=NEW, old_hosts_unchanged=True, heartbeat_continues=True)

            publish(STABLE, packages[NEW], NEW)
            promoted = json.loads(installed("alpha", "upgrade", "--version", STABLE, "--json"))
            assert promoted["installed"] and promoted["target"]["tag"] == STABLE
            assert old_program.is_file(), "occupied old runtime was collected"
            installed("alpha", "close", session)
            common.wait_for(lambda: not common.hosts(root), 30, "closed Lab task did not leave")
            installed("alpha", "version")
            assert not old_program.parent.exists(), "released old generation was not collected"
            evidence["promotion_and_gc"] = dict(same_rc_archive_promoted=True, occupied_old_retained=True, released_old_collected=True)
            evidence["builds"] = {tag: dict(program_sha256=p["sha256"], info=p["info"]) for tag, p in packages.items()}
        finally:
            for name in ("alpha", "beta"):
                path = root / (name + ".log")
                if path.is_file(): shutil.copyfile(path, REPORT / ("cli-lab-" + name + ".log"))
            for pid in common.own_processes(root):
                try:
                    if pid in common.own_processes(root): os.kill(pid, signal.SIGTERM)
                except ProcessLookupError: pass
            end = time.monotonic() + 5
            while common.own_processes(root) and time.monotonic() < end: time.sleep(0.2)
            for pid in common.own_processes(root):
                try:
                    if pid in common.own_processes(root): os.kill(pid, signal.SIGKILL)
                except ProcessLookupError: pass
            (SOURCE / "crates/update/updater.pub").write_bytes(original_key)
            (SOURCE / "crates/update/src/feed.rs").write_bytes(original_feed)
            (SOURCE / "crates/node/src/update_wire.rs").write_bytes(original_wire)
            (SOURCE / "crates/node/src/link.rs").write_bytes(original_link)
            if last_primary and primary.is_file() and common.digest(primary) == last_primary: primary.unlink()
            server.shutdown()
            evidence["feed_requests"] = common.Feed.requests
            (REPORT / "cli-update-lab.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
