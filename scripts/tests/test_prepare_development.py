import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


class PrepareDevelopmentTest(unittest.TestCase):
    def test_artifact_roundtrip_retains_a_valid_repo_without_empty_directories(self):
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        env.update(GIT_AUTHOR_NAME="Test", GIT_AUTHOR_EMAIL="test@localhost",
                   GIT_COMMITTER_NAME="Test", GIT_COMMITTER_EMAIL="test@localhost")
        script = Path(__file__).resolve().parents[1] / "prepare-development.py"
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source"
            source.mkdir()
            def git(*args):
                return subprocess.check_output(["git", "-C", str(source), *args], env=env, text=True).strip()
            git("init", "-q")
            (source / "test.txt").write_text("source\n")
            git("add", ".")
            tree = git("write-tree")
            info = {"schema": 1, "base_commit": "a" * 40, "tree": tree, "dirty": True}
            container = subprocess.check_output(["git", "-C", str(source), "mktree"],
                                                input=f"040000 tree {tree}\tsource\n", text=True, env=env).strip()
            sha = git("commit-tree", container, "-m", "Khor developer snapshot\n\n" + json.dumps(info))
            git("checkout", "--detach", sha)
            package = Path(directory) / "package"
            subprocess.run([sys.executable, str(script), str(source), str(package)],
                           env=dict(env, SNAPSHOT_SHA=sha), check=True, capture_output=True)
            restored = Path(directory) / "download"
            # upload-artifact retains files, not empty directories or ordinary hidden files.
            for path in package.rglob("*"):
                relative = path.relative_to(package)
                if path.is_file() and not any(part.startswith(".") for part in relative.parts):
                    target = restored / relative
                    target.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(path, target)
            result = subprocess.check_output(["git", "--git-dir=" + str(restored / "source.git"), "rev-parse", "HEAD"], env=env, text=True).strip()
            actual = json.loads((restored / "source.json").read_text())
            self.assertEqual(result, actual["build_commit"])
            self.assertEqual(actual["snapshot_commit"], sha)
            self.assertEqual(actual["tree"], tree)
            materialized = subprocess.check_output(["git", "--git-dir=" + str(restored / "source.git"), "ls-tree", "--name-only", "HEAD"], env=env, text=True).strip()
            self.assertEqual(materialized, "test.txt")
            config = (restored / "source.git/config").read_text()
            self.assertNotIn("extraheader", config.lower())
            self.assertNotIn("[remote", config)


if __name__ == "__main__":
    unittest.main()
