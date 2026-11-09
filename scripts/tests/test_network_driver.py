import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class NetworkDriverTest(unittest.TestCase):
    def test_congestion_keeps_a_failed_round_and_runs_the_remaining_rounds(self):
        driver = Path(__file__).resolve().parents[1] / "network.sh"
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        env.update(GIT_AUTHOR_NAME="Test", GIT_AUTHOR_EMAIL="test@localhost",
                   GIT_COMMITTER_NAME="Test", GIT_COMMITTER_EMAIL="test@localhost")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source"
            source.mkdir()
            def git(*args):
                return subprocess.check_output(["git", "-C", str(source), *args],
                                               text=True, env=env).strip()
            git("init", "-q")
            (source / "README.md").write_text("driver fixture; no buildable source\n")
            git("add", ".")
            git("commit", "-qm", "fixture")
            sha, tree = git("rev-parse", "HEAD"), git("rev-parse", "HEAD^{tree}")
            package = root / "package"
            package.mkdir()
            git("clone", "--bare", "--quiet", str(source), str(package / "source.git"))
            (package / "source.json").write_text(json.dumps({"build_commit": sha, "tree": tree}))
            binary = root / "bin"
            binary.mkdir()
            # These are Python stubs. No Cargo, Rust toolchain or network lab is executed.
            cargo = binary / "cargo"
            cargo.write_text("#!" + sys.executable + "\n" + r'''
import json, os, sys
from pathlib import Path
trace = Path(os.environ['MOCK_TRACE'])
calls = json.loads(trace.read_text()) if trace.exists() else []
round_index = sum(call['args'][0] == 'test' for call in calls)
calls.append({'args': sys.argv[1:], 'cc_only': os.environ.get('CC_ONLY'),
              'rtt': os.environ.get('CC_RTT_MS')})
trace.write_text(json.dumps(calls))
print('stub round', round_index)
sys.exit(8 if sys.argv[1] == 'test' and round_index == 0 else 0)
''')
            cargo.chmod(0o755)
            rustc = binary / "rustc"
            rustc.write_text("#!/bin/sh\nprintf 'rustc stub; no compiler executed\\n'\n")
            rustc.chmod(0o755)
            temporary = root / "runner"
            temporary.mkdir()
            report, trace = root / "report", root / "calls.json"
            result = subprocess.run(["bash", str(driver)], text=True, capture_output=True,
                                    env=dict(env, PATH=str(binary) + os.pathsep + env['PATH'],
                                             DEV_TASK="net-congestion", SOURCE_PACKAGE=str(package),
                                             RUNNER_TEMP=str(temporary), REPORT_DIR=str(report),
                                             GITHUB_RUN_ID="stub", GITHUB_SHA="control-fixture",
                                             CARGO_BUILD_JOBS="2", MOCK_TRACE=str(trace),
                                             CC_ONLY="inherited-value-must-be-removed"))
            self.assertEqual(result.returncode, 8, result.stderr)
            calls = json.loads(trace.read_text())
            self.assertEqual(calls[0]['args'], ['fmt', '--all', '--', '--check'])
            tests = calls[1:]
            self.assertEqual(len(tests), 3)
            for call in tests:
                self.assertEqual(call['args'], ['test', '--locked', '-p', 'khor-net', '--test',
                                               'congestion', '--', '--ignored', '--nocapture',
                                               '--test-threads=1'])
                self.assertIsNone(call['cc_only'])
                self.assertEqual(call['rtt'], '300')
            self.assertEqual(len(list(report.glob('congestion-*.log'))), 3)
            mode = json.loads((report / "measurement-mode.json").read_text())
            self.assertFalse(mode['slo'])
            self.assertEqual(mode['layer'], 'noq')
            self.assertEqual(mode['repeats'], 3)
            self.assertEqual(json.loads((report / "source.json").read_text())['tree'], tree)


if __name__ == "__main__":
    unittest.main()
