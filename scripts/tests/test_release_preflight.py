"""Control tests for early runtime admission; no SDK download or compilation."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

SCRIPT = Path(__file__).resolve().parents[1] / "release-preflight.py"


class RuntimeAdmission(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        (self.root / "Cargo.toml").write_text('[workspace.package]\nversion="0.2.0"\n')
        self.sdk = self.root / "iPhoneOS.sdk"
        self.sdk.mkdir()
        original = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, original)
        environment = patch.dict(os.environ, REPORT_DIR=str(self.root), SOURCE_SHA="a"*40,
                                 CARGO_TARGET_DIR=str(self.root))
        environment.start()
        self.addCleanup(environment.stop)
        spec = importlib.util.spec_from_file_location("release_preflight_control", SCRIPT)
        self.module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.module)
        self.available = {"name": "iOS26.5", "identifier": "public-test", "version": "26.5", "isAvailable": True}

    def check(self, lists, free=80):
        replies = [str(self.sdk)] + [json.dumps({"runtimes": rows}) for rows in lists]
        with patch.object(self.module.subprocess, "check_output", side_effect=replies), \
             patch.object(self.module.shutil, "disk_usage", return_value=SimpleNamespace(free=free*1024**3)), \
             patch.object(self.module, "run") as execute:
            evidence = {}
            self.executed = execute
            self.module.ios_prerequisites({}, evidence)
            return execute.call_args_list, evidence

    def test_available_runtime_does_not_download_again(self):
        calls, evidence = self.check([[self.available]])
        self.assertEqual(calls, [])
        self.assertEqual(evidence["ios_runtimes"][0]["identifier"], "public-test")

    def test_missing_runtime_downloads_only_arm64_and_checks_registration(self):
        calls, evidence = self.check([[], [self.available]])
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0].args,
                         ("sudo", "-n", "xcodebuild", "-downloadPlatform", "iOS", "-architectureVariant", "arm64"))
        self.assertEqual(calls[0].kwargs["timeout"], 2700)
        self.assertTrue(evidence["ios_runtimes"][0]["isAvailable"])

    def test_low_space_refuses_download_without_clearing_cache(self):
        with self.assertRaisesRegex(RuntimeError, "preserving cache"):
            self.check([[]], free=59)
        self.executed.assert_not_called()

    def test_a_successful_download_without_available_runtime_cannot_start_building(self):
        unavailable = dict(self.available, isAvailable=False)
        with self.assertRaisesRegex(RuntimeError, "before starting any compilation"):
            self.check([[], [unavailable]])


if __name__ == "__main__":
    unittest.main()
