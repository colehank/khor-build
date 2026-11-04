"""Create a separate, ad-hoc signed Mac Lab with an isolated store and browser profile."""
import hashlib
import json
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys


binary, output, receipt = map(Path, sys.argv[1:])
info = json.loads(receipt.read_text())
output.mkdir(parents=True, exist_ok=True)
app = output / "KhorLab.app"
contents = app / "Contents"
programs = contents / "MacOS"
programs.mkdir(parents=True)
program = programs / "khor-gui-bin"
shutil.copyfile(binary, program)
program.chmod(0o755)
launcher = programs / "KhorLab"
launcher.write_text('''#!/bin/sh
set -eu
export KHOR_HOME="$HOME/Library/Caches/KhorLab/SNAPSHOT"
export KHOR_NAME="Khor Lab"
export KHOR_WEB_PORT=0
unset KHOR_SESSION
mkdir -p "$KHOR_HOME"
exec "$(dirname "$0")/khor-gui-bin" "$@"
'''.replace("SNAPSHOT", info["snapshot_commit"]))
launcher.chmod(0o755)
plist = {"CFBundleName": "KhorLab", "CFBundleDisplayName": "Khor Lab",
         "CFBundleIdentifier": "io.github.colehank.khor.lab", "CFBundleExecutable": "KhorLab",
         "CFBundlePackageType": "APPL", "CFBundleVersion": "1", "CFBundleShortVersionString": "0.0.0",
         "NSHighResolutionCapable": True, "LSMinimumSystemVersion": "13.0"}
(contents / "Info.plist").write_bytes(plistlib.dumps(plist))
subprocess.run(["codesign", "--force", "--sign", "-", str(program)], check=True)
subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app)], check=True)
subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True)
archive = output / "KhorLab.zip"
subprocess.run(["ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(app), str(archive)], check=True)
with archive.open("rb") as stream:
    info["archive_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
(output / "source.json").write_text(json.dumps(info, indent=2) + "\n")
shutil.rmtree(app)
