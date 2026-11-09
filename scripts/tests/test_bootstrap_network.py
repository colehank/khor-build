import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


SCRIPTS = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("bootstrap_network", SCRIPTS / "bootstrap-network.py")
bootstrap = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bootstrap)


def blob(content):
    return dict(encoding="base64", size=len(content), content=base64.b64encode(content).decode(),
                sha=hashlib.sha1(b"blob " + str(len(content)).encode() + b"\0" + content).hexdigest())


class BootstrapNetworkTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        environment = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
        environment.update(GIT_AUTHOR_NAME="Test", GIT_AUTHOR_EMAIL="test@localhost",
                           GIT_COMMITTER_NAME="Test", GIT_COMMITTER_EMAIL="test@localhost")
        self.environment = patch.dict(os.environ, environment, clear=True)
        self.environment.start()
        self.addCleanup(self.environment.stop)
        source = self.root / "original"
        source.mkdir()
        def git(*args, **kwargs):
            return subprocess.check_output(["git", "-C", str(source), *args], text=True, **kwargs).strip()
        git("init", "--quiet")
        (source / "test.txt").write_text("exact source\n")
        git("add", "test.txt")
        self.tree = git("write-tree")
        container = git("mktree", input=f"040000 tree {self.tree}\tsource\n")
        metadata = dict(schema=1, base_commit="a" * 40, tree=self.tree, dirty=True)
        self.snapshot = git("commit-tree", container, "-m", "Khor developer snapshot\n\n" + json.dumps(metadata))
        git("update-ref", "refs/heads/snapshot", self.snapshot)
        git("symbolic-ref", "HEAD", "refs/heads/snapshot")
        bundle = self.root / "fixture.bundle"
        git("bundle", "create", str(bundle), "--all", "HEAD")
        self.bundle = blob(bundle.read_bytes())
        self.package = self.root / "package"
        os.environ.update(SNAPSHOT_SHA=self.snapshot, GITHUB_SHA="b" * 40,
                          SOURCE_BUNDLE_BLOB_SHA=self.bundle["sha"],
                          SOURCE_BUNDLE_SHA256=hashlib.sha256(bundle.read_bytes()).hexdigest(),
                          GITHUB_REPOSITORY="private/build", RUNNER_TEMP=str(self.root),
                          SOURCE_PACKAGE=str(self.package), REPORT_DIR=str(self.root / "report"),
                          GITHUB_ENV=str(self.root / "github-env"), GH_TOKEN="test-step-token",
                          GITHUB_TOKEN="test-token-must-not-enter-git")

    def api(self, path, maximum):
        if path == "git/blobs/" + self.bundle["sha"]:
            return self.bundle
        prefix = "contents/scripts/"
        self.assertTrue(path.startswith(prefix))
        name, ref = path[len(prefix):].split("?ref=")
        self.assertEqual(ref, os.environ["GITHUB_SHA"])
        self.assertIn(name, bootstrap.CONTROL_FILES)
        return blob((SCRIPTS / name).read_bytes())

    def test_exact_bundle_materializes_without_network_git_or_inherited_token(self):
        original = subprocess.run
        calls = []
        def run(args, **kwargs):
            calls.append(args)
            self.assertNotIn("GH_TOKEN", kwargs["env"])
            self.assertNotIn("GITHUB_TOKEN", kwargs["env"])
            return original(args, **kwargs)
        with patch.object(bootstrap, "api", side_effect=self.api), patch.object(bootstrap.subprocess, "run", side_effect=run):
            bootstrap.main()
        self.assertTrue(any(args[0] == "git" and "clone" in args for args in calls))
        self.assertFalse(any("https://" in arg for args in calls for arg in args))
        actual = json.loads((self.package / "source.json").read_text())
        self.assertEqual(actual["snapshot_commit"], self.snapshot)
        self.assertEqual(actual["tree"], self.tree)
        git_dir = "--git-dir=" + str(self.package / "source.git")
        content = subprocess.check_output(["git", git_dir, "show", "HEAD:test.txt"], text=True)
        self.assertEqual(content, "exact source\n")
        config = (self.package / "source.git/config").read_text().lower()
        self.assertNotIn("extraheader", config)
        self.assertNotIn("[remote", config)
        report = json.loads((self.root / "report/bundle-transport.json").read_text())
        self.assertEqual(report["control_commit"], "b" * 40)
        self.assertEqual(set(report["control_files"]), set(bootstrap.CONTROL_FILES))
        self.assertIn("NETWORK_CONTROL_DIR=", (self.root / "github-env").read_text())

    def test_wrong_checksum_stops_before_git_or_source_materialization(self):
        os.environ["SOURCE_BUNDLE_SHA256"] = "0" * 64
        with patch.object(bootstrap, "api", side_effect=self.api), patch.object(bootstrap.subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "checksum mismatch"):
                bootstrap.main()
        run.assert_not_called()
        self.assertFalse(self.package.exists())

    def test_wrong_snapshot_rejects_an_otherwise_valid_bundle(self):
        os.environ["SNAPSHOT_SHA"] = "c" * 40
        with patch.object(bootstrap, "api", side_effect=self.api):
            with self.assertRaisesRegex(ValueError, "different snapshot"):
                bootstrap.main()
        self.assertFalse(self.package.exists())


if __name__ == "__main__":
    unittest.main()
