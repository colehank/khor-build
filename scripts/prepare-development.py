"""Package an exact source snapshot without forwarding repository credentials."""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys


def git(directory, *args):
    return subprocess.check_output(["git", "-C", str(directory), *args], text=True).strip()


source, output = map(Path, sys.argv[1:])
sha = os.environ["SNAPSHOT_SHA"]
assert re.fullmatch(r"[0-9a-f]{40}", sha)
assert git(source, "rev-parse", "HEAD") == sha
assert len(git(source, "cat-file", "-p", sha).split("\n\n", 1)[0].splitlines()) >= 3
parents = [line for line in git(source, "cat-file", "-p", sha).splitlines() if line.startswith("parent ")]
assert not parents, "Only a parentless source snapshot is accepted"
message = git(source, "show", "-s", "--format=%B", sha)
assert message.startswith("Khor developer snapshot\n\n")
info = json.loads(message.split("\n\n", 1)[1])
assert info["schema"] == 1
assert re.fullmatch(r"[0-9a-f]{40}", info["base_commit"])
assert git(source, "ls-tree", "--name-only", "HEAD") == "source", "Snapshot workflows must not exist at the repository root"
assert info["tree"] == git(source, "rev-parse", "HEAD:source")
info["snapshot_commit"] = sha
output.mkdir(parents=True, exist_ok=False)
subprocess.run(["git", "clone", "--bare", "--no-hardlinks", str(source.resolve()), str(output / "source.git")], check=True)
environment = dict(os.environ, GIT_AUTHOR_NAME="Khor development materialization", GIT_AUTHOR_EMAIL="snapshot@localhost",
                   GIT_COMMITTER_NAME="Khor development materialization", GIT_COMMITTER_EMAIL="snapshot@localhost")
build_commit = subprocess.check_output(["git", "--git-dir=" + str(output / "source.git"), "commit-tree", info["tree"],
                                        "-m", "Khor source from private snapshot " + sha], env=environment, text=True).strip()
info["build_commit"] = build_commit
subprocess.run(["git", "--git-dir=" + str(output / "source.git"), "update-ref", "refs/heads/source", build_commit], check=True)
subprocess.run(["git", "--git-dir=" + str(output / "source.git"), "symbolic-ref", "HEAD", "refs/heads/source"], check=True)
# checkout's auth header belongs to its working repository; clone --bare does not copy it.
subprocess.run(["git", "--git-dir=" + str(output / "source.git"), "remote", "remove", "origin"], check=True)
for name in ["native.sh", "development.sh", "package-lab.py", "update-lab.py", "cli-update-lab.py"]:
    shutil.copyfile(Path(__file__).with_name(name), output / name)
(output / "source.json").write_text(json.dumps(info, indent=2) + "\n")
