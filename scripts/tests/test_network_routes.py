"""Control-driver contracts using Python stubs; no Rust or product execution."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class NetworkRoutesTest(unittest.TestCase):
    def test_focused_lane_rejects_empty_results_and_keeps_both_outcomes(self):
        driver = Path(__file__).resolve().parents[1] / 'network.sh'
        env = {key: value for key, value in os.environ.items() if not key.startswith('GIT_')}
        env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1',
                   GIT_AUTHOR_NAME='Fixture', GIT_AUTHOR_EMAIL='fixture@localhost',
                   GIT_COMMITTER_NAME='Fixture', GIT_COMMITTER_EMAIL='fixture@localhost')
        for mode in ['success', 'first_failed', 'second_failed', 'zero', 'missing']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source = root / 'source'
                source.mkdir()

                def git(*args):
                    return subprocess.check_output(
                        ['git', '-c', 'core.hooksPath=/dev/null', '-C', str(source), *args],
                        env=env, text=True).strip()

                git('init', '-q')
                (source / 'README.md').write_text('stub fixture, no buildable product\n')
                git('add', '.')
                git('commit', '-qm', 'fixture')
                package = root / 'package'
                package.mkdir()
                git('clone', '--bare', '--quiet', str(source), str(package / 'source.git'))
                identity = dict(build_commit=git('rev-parse', 'HEAD'), tree=git('rev-parse', 'HEAD^{tree}'))
                (package / 'source.json').write_text(json.dumps(identity))
                binary = root / 'bin'
                binary.mkdir()
                cargo = binary / 'cargo'
                cargo.write_text('#!' + sys.executable + '\n' + r'''
import json, os, pathlib, sys
path = pathlib.Path(os.environ['MOCK_TRACE'])
calls = json.loads(path.read_text()) if path.exists() else []
calls.append(sys.argv[1:]); path.write_text(json.dumps(calls))
if sys.argv[1] == 'fmt':
    sys.exit(0)
first = 'link::dial_budget::initial_custom::' in sys.argv
second = 'hop::tests_offer::published::incoming::' in sys.argv
if not (first or second):
    sys.exit(98)
mode = os.environ['MOCK_MODE']
if first and mode == 'missing':
    print('no summary supplied')
    sys.exit(0)
count = 6 if first else 2
if first and mode == 'zero':
    count = 0
failed = 2 if first and mode == 'first_failed' else 1 if second and mode == 'second_failed' else 0
print(f'test result: {"FAILED" if failed else "ok"}. {count-failed} passed; {failed} failed; 0 ignored;')
sys.exit(73 if failed else 0)
''')
                cargo.chmod(0o700)
                rustc = binary / 'rustc'
                rustc.write_text('#!' + sys.executable + '\nprint("stub; no compiler executed")\n')
                rustc.chmod(0o700)
                report, temporary, trace = root / 'report', root / 'temporary', root / 'calls.json'
                temporary.mkdir()
                result = subprocess.run(['bash', str(driver)], capture_output=True, text=True,
                    env=dict(env, PATH=str(binary) + os.pathsep + env['PATH'],
                             DEV_TASK='net-node-routes', SOURCE_PACKAGE=str(package),
                             RUNNER_TEMP=str(temporary), REPORT_DIR=str(report),
                             CARGO_BUILD_JOBS='2', GITHUB_RUN_ID='stub', GITHUB_SHA='control-fixture',
                             MOCK_TRACE=str(trace), MOCK_MODE=mode))
                calls = json.loads(trace.read_text())
                self.assertEqual(len(calls), 3)
                self.assertEqual(calls[0], ['fmt', '--all', '--', '--check'])
                self.assertIn('link::dial_budget::initial_custom::', calls[1])
                self.assertIn('hop::tests_offer::published::incoming::', calls[2])
                for call in calls[1:]:
                    self.assertIn('--locked', call)
                    self.assertIn('--nocapture', call)
                    self.assertIn('--test-threads=1', call)
                summary = json.loads((report / 'route-contracts.json').read_text())
                self.assertEqual(summary['source'], identity)
                self.assertFalse(summary['full_regression'])
                self.assertEqual(summary['passed'], mode == 'success')
                self.assertEqual(result.returncode == 0, mode == 'success', result.stdout + result.stderr)
                self.assertTrue(summary['groups']['incoming-route-contracts.log']['valid'])
                if mode in ['zero', 'missing']:
                    self.assertFalse(summary['groups']['initial-route-contracts.log']['valid'])
                elif mode != 'success':
                    self.assertEqual(result.returncode, 73)


if __name__ == '__main__':
    unittest.main()
