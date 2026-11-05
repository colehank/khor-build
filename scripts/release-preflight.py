"""Run the original release build adapters without signing, uploading or installing."""
import hashlib
import json
import os
from pathlib import Path
import plistlib
import subprocess
import tomllib


SOURCE = Path.cwd()
REPORT = Path(os.environ["REPORT_DIR"])
CACHE = Path(os.environ["CARGO_TARGET_DIR"]).resolve(strict=True)
SHA = os.environ["SOURCE_SHA"]
TARGET = "aarch64-apple-darwin"
VERSION = tomllib.loads((SOURCE / "Cargo.toml").read_text())["workspace"]["package"]["version"]
TAG = f"v{VERSION}-rc.900099"


def run(*arguments, environment):
    subprocess.run(arguments, env=environment, stdin=subprocess.DEVNULL, check=True)


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
    environment = {key: value for key, value in os.environ.items()
                   if not key.startswith(("APPLE_", "IOS_CERTIFICATE", "TAURI_SIGNING_", "MAC_CERTIFICATE"))
                   and key not in {"IOS_PROFILE", "GH_TOKEN", "GITHUB_TOKEN"}}
    environment.update(KHOR_BUILD_SOURCE=SHA, KHOR_RELEASE_TAG=TAG,
                       KHOR_RELEASE_BUILD=os.environ["GITHUB_RUN_ID"])
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
    evidence = {"source": SHA, "profile": "release", "fixture_tag": TAG,
                "signed": False, "uploaded": False, "installed": False}
    try:
        run("rustup", "target", "add", "aarch64-apple-ios", environment=environment)
        run("npm", "ci", "--prefix", "apps/agent", environment=environment)
        run("node", "apps/agent/build.mjs", TARGET, environment["KHOR_AGENT_BUNDLE"], environment=environment)
        run("npm", "ci", "--prefix", "apps/gui", environment=environment)
        run("npm", "run", "build", "--prefix", "apps/gui", environment=environment)
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
        if created and link.is_symlink() and link.lstat().st_ino == inode and link.resolve() == CACHE:
            link.unlink()
        (REPORT / "unsigned-release.json").write_text(json.dumps(evidence, indent=2) + "\n")


if __name__ == "__main__":
    main()
