"""Run the original release build adapters without signing, uploading or installing."""
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shutil
import time
import subprocess
import tomllib


SOURCE = Path.cwd()
REPORT = Path(os.environ["REPORT_DIR"])
CACHE = Path(os.environ["CARGO_TARGET_DIR"]).resolve(strict=True)
SHA = os.environ["SOURCE_SHA"]
TARGET = "aarch64-apple-darwin"
VERSION = tomllib.loads((SOURCE / "Cargo.toml").read_text())["workspace"]["package"]["version"]
TAG = f"v{VERSION}-rc.900099"


def run(*arguments, environment, timeout=None):
    started = time.monotonic()
    print(f"Preflight command started: {' '.join(arguments[:3])}", flush=True)
    try:
        subprocess.run(arguments, env=environment, stdin=subprocess.DEVNULL, check=True, timeout=timeout)
    finally:
        print(f"Preflight command finished after {time.monotonic()-started:.1f}s: {' '.join(arguments[:3])}", flush=True)


def ios_prerequisites(environment, evidence):
    system_environment = {key: value for key, value in environment.items() if key != "DEVELOPER_DIR"}
    selected = subprocess.check_output(["/usr/bin/xcode-select", "-p"],
                                       env=system_environment, text=True, timeout=30).strip()
    if Path(selected).resolve() != Path(environment.get("DEVELOPER_DIR", selected)).resolve():
        raise RuntimeError("global xcode-select must match the builder Xcode before Tauri archives")
    evidence["xcode_developer_directory"] = selected
    sdk = subprocess.check_output(["xcrun", "--sdk", "iphoneos", "--show-sdk-path"],
                                  env=environment, text=True, timeout=30).strip()
    if not sdk or not Path(sdk).is_dir():
        raise RuntimeError("the selected Xcode has no usable iPhoneOS SDK")
    evidence["iphoneos_sdk"] = sdk

    def runtimes():
        result = json.loads(subprocess.check_output(
            ["xcrun", "simctl", "list", "runtimes", "--json"], env=environment, text=True, timeout=30))
        return [item for item in result.get("runtimes", [])
                if item.get("name", "").startswith("iOS") and item.get("isAvailable") is True]

    installed = runtimes()
    if not installed:
        free = shutil.disk_usage(CACHE).free
        evidence["free_before_runtime_install"] = free
        if free < 60 * 1024 ** 3:
            raise RuntimeError("preserving cache: runtime installation requires60GiB free, including40GiB reserve")
        # Tauri2.12.1 checks simctl even for a device-only archive. Only the renter gets this public runtime.
        developer = environment.get("DEVELOPER_DIR") or subprocess.check_output(
            ["xcode-select", "-p"], env=environment, text=True, timeout=30).strip()
        if not Path(developer).is_absolute() or not Path(developer).is_dir():
            raise RuntimeError("runtime installation requires the selected Xcode developer directory")
        # sudo drops DEVELOPER_DIR; forward this one selection, not the job's credential environment.
        run("sudo", "-n", "/usr/bin/env", f"DEVELOPER_DIR={developer}", "/usr/bin/xcodebuild",
            "-downloadPlatform", "iOS", "-architectureVariant", "arm64",
            environment=environment, timeout=2700)
        installed = runtimes()
    if not installed:
        raise RuntimeError("Tauri requires an available iOS simulator runtime before starting any compilation")
    evidence["ios_runtimes"] = [{key: item.get(key) for key in ("name", "identifier", "version", "isAvailable")}
                                for item in installed]
    print("iOS device SDK and Tauri runtime prerequisites confirmed", flush=True)


def identity(program, component, environment):
    info = json.loads(subprocess.check_output(
        [str(program), "_release_info"], env=environment, stdin=subprocess.DEVNULL,
        text=True, timeout=30))
    if (info["schema"] != 1 or info["component"] != component
            or info["platform"] != {"os": "macos", "arch": "aarch64"}
            or info["build"]["source"] != SHA
            or info["build"]["release"]["tag"] != TAG):
        raise RuntimeError("unsigned release program differs from the pinned source and platform")
    return info


def main():
    phase = os.environ.get("KHOR_PREFLIGHT_PHASE", "all")
    if phase not in {"all", "ios"}:
        raise RuntimeError("unknown unsigned preflight phase")
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith(("APPLE_", "IOS_CERTIFICATE", "TAURI_SIGNING_", "MAC_CERTIFICATE"))
                   and key not in {"IOS_PROFILE", "GH_TOKEN", "GITHUB_TOKEN"}}
    environment.update(KHOR_BUILD_SOURCE=SHA, KHOR_RELEASE_TAG=TAG,
                       KHOR_RELEASE_BUILD=os.environ["GITHUB_RUN_ID"], CARGO_INCREMENTAL="0")
    if CACHE != Path("/Users/z13/khor-builder/cache/target"):
        raise RuntimeError("release preflight requires the admitted shared target cache")
    link = SOURCE / "target"
    created = False
    if link.is_symlink():
        if link.resolve(strict=True) != CACHE:
            raise RuntimeError("preserving a target link owned by another build")
    elif link.exists():
        raise RuntimeError("preserving an unexpected source target directory")
    else:
        link.symlink_to(CACHE, target_is_directory=True)
        created = True
    inode = link.lstat().st_ino
    evidence = {"source": SHA, "profile": "release", "fixture_tag": TAG, "phase": phase,
                "signed": False, "uploaded": False, "installed": False}
    apple_inputs = [SOURCE / "apps/gui/src-tauri/gen/apple/khor-gui.xcodeproj/project.pbxproj",
                    SOURCE / "apps/gui/src-tauri/gen/apple/khor-gui_iOS/Info.plist"]
    originals = {path: path.read_bytes() for path in apple_inputs if path.is_file()}
    try:
        ios_prerequisites(environment, evidence)
        run("rustup", "target", "add", "aarch64-apple-ios", environment=environment)
        if phase == "all":
            run("npm", "ci", "--prefix", "apps/agent", environment=environment)
            run("node", "apps/agent/build.mjs", TARGET, environment["KHOR_AGENT_BUNDLE"], environment=environment)
        run("npm", "ci", "--prefix", "apps/gui", environment=environment)
        run("npm", "run", "build", "--prefix", "apps/gui", environment=environment)
        if phase == "all":
            print("Unsigned release preflight: original release profile CLI", flush=True)
            run("cargo", "build", "--locked", "--release", "--target", TARGET,
                "-p", "khor-cli", "--bin", "khor", environment=environment)
            evidence["cli"] = identity(CACHE / TARGET / "release/khor", "cli", environment)
            print("Unsigned release preflight: original Mac App adapter and layered icon", flush=True)
            run("python3", "scripts/apple_release.py", "mac-build", "--target", TARGET,
                "--signed", "false", environment=environment)
            app = CACHE / TARGET / "release/bundle/macos/Khor.app"
            evidence["app"] = identity(app / "Contents/MacOS/khor-gui", "desktop-app", environment)
        print("Unsigned release preflight: original no-sign iOS archive adapter", flush=True)
        run("python3", "scripts/apple_release.py", "ios-build", "--signed", "false", environment=environment)
        phone = SOURCE / "apps/gui/src-tauri/gen/apple/build/khor-gui_iOS.xcarchive/Products/Applications/Khor.app"
        info = plistlib.loads((phone / "Info.plist").read_bytes())
        if (info["CFBundleIdentifier"] != "io.github.colehank.khor"
                or info["CFBundleShortVersionString"] != VERSION
                or (phone / "embedded.mobileprovision").exists()):
            raise RuntimeError("unsigned iOS archive has another application identity")
        program = phone / "Khor"
        with program.open("rb") as stream:
            digest = hashlib.file_digest(stream, "sha256").hexdigest()
        evidence["ios"] = {"version": info["CFBundleShortVersionString"],
                           "build": info["CFBundleVersion"], "program_sha256": digest,
                           "program_bytes": program.stat().st_size}
        evidence["complete"] = True
    finally:
        changes = subprocess.check_output(["git", "diff", "--binary"], cwd=SOURCE)
        (REPORT / "ios-generated-inputs.patch").write_bytes(changes)
        for path, original in originals.items():
            if path.read_bytes() != original:
                path.write_bytes(original)  # This exclusive worker owns Tauri's generated project/version edits.
        if created and link.is_symlink() and link.lstat().st_ino == inode and link.resolve() == CACHE:
            link.unlink()
        (REPORT / "unsigned-release.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
