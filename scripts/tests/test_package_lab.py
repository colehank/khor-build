"""The generated Lab launcher must never reuse an ambient production runtime."""
import json
import os
from pathlib import Path
import runpy
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


class PackageLabTests(unittest.TestCase):
    @unittest.skipUnless(os.name == "posix", "Mac Lab launches through /bin/sh")
    def test_generated_launcher_isolates_runtime_before_starting_its_program(self):
        package = Path(__file__).resolve().parents[1] / "package-lab.py"
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            home = root / "home with spaces"
            home.mkdir()
            program = root / "environment-probe"
            program.write_text(
                f"#!{sys.executable}\n"
                "import json, os, pathlib, sys\n"
                "pathlib.Path(os.environ['LAB_PROBE_OUTPUT']).write_text(json.dumps({"
                "key: os.environ.get(key) for key in "
                "['KHOR_HOME', 'KHOR_RUNTIME_DIR', 'KHOR_RUNTIME_INSTALL_SOURCE', 'KHOR_SESSION']}))\n"
            )
            receipt = root / "receipt.json"
            receipt.write_text(json.dumps({"snapshot_commit": "a" * 40}))
            output = root / "artifacts"
            observed = root / "observed.json"
            inherited = dict(os.environ, HOME=str(home), KHOR_RUNTIME_DIR="/ambient-runtime",
                             KHOR_RUNTIME_INSTALL_SOURCE="/ambient-install-source",
                             KHOR_SESSION="ambient-session", LAB_PROBE_OUTPUT=str(observed))
            real_run = subprocess.run
            signed = []

            def package_command(command, *, check):
                self.assertTrue(check)
                if command[0] == "codesign":
                    signed.append(command)
                elif command[0] == "ditto":
                    launcher = output / "KhorLab.app/Contents/MacOS/KhorLab"
                    # Execute only the generated isolation wrapper and this
                    # test's environment probe; no native App is launched.
                    real_run([str(launcher)], env=inherited, check=True)
                    Path(command[-1]).write_bytes(b"fixture archive")
                else:
                    self.fail(f"Unexpected package command: {command}")
                return subprocess.CompletedProcess(command, 0)

            with patch.object(sys, "argv", [str(package), str(program), str(output), str(receipt)]), \
                    patch("subprocess.run", side_effect=package_command):
                runpy.run_path(str(package), run_name="__main__")
            store = home / "Library/Caches/KhorLab" / ("a" * 40)
            self.assertEqual(json.loads(observed.read_text()), {
                "KHOR_HOME": str(store), "KHOR_RUNTIME_DIR": str(store / "runtime"),
                "KHOR_RUNTIME_INSTALL_SOURCE": None, "KHOR_SESSION": None,
            })
            self.assertTrue(store.is_dir())
            self.assertEqual(inherited["KHOR_RUNTIME_DIR"], "/ambient-runtime")
            self.assertEqual(inherited["KHOR_RUNTIME_INSTALL_SOURCE"], "/ambient-install-source")
            self.assertEqual(len(signed), 3)


if __name__ == "__main__":
    unittest.main()
