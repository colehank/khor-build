import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class NetworkDriverTest(unittest.TestCase):
    def test_wan_build_records_identity_without_running_or_exporting_the_binary(self):
        import hashlib
        driver = Path(__file__).resolve().parents[1] / 'network.sh'
        env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
        env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1',
                   GIT_AUTHOR_NAME='Test', GIT_AUTHOR_EMAIL='test@localhost',
                   GIT_COMMITTER_NAME='Test', GIT_COMMITTER_EMAIL='test@localhost')
        for mode in ['normal', 'missing', 'symlink']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source = root / 'source'
                source.mkdir()
                def git(*args):
                    return subprocess.check_output(['git', '-C', str(source), *args], env=env, text=True).strip()
                git('init', '-q')
                (source / 'README.md').write_text('no buildable product; Python stubs only\n')
                git('add', '.')
                git('-c', 'core.hooksPath=/dev/null', 'commit', '-qm', 'fixture')
                package = root / 'package'
                package.mkdir()
                git('clone', '--bare', '--quiet', str(source), str(package / 'source.git'))
                identity = dict(build_commit=git('rev-parse', 'HEAD'), tree=git('rev-parse', 'HEAD^{tree}'))
                (package / 'source.json').write_text(json.dumps(identity))
                binary = root / 'bin'
                binary.mkdir()
                cargo = binary / 'cargo'
                cargo.write_text('#!' + sys.executable + '\n' + r'''
import json,os,pathlib,sys
trace=pathlib.Path(os.environ['MOCK_TRACE'])
calls=json.loads(trace.read_text()) if trace.exists() else []
calls.append(sys.argv[1:]);trace.write_text(json.dumps(calls))
if sys.argv[1]=='build' and os.environ['MOCK_BINARY_MODE']!='missing':
    target=pathlib.Path(os.environ['CARGO_TARGET_DIR'])/'debug/examples/net_wan_probe'
    target.parent.mkdir(parents=True)
    contents=b'not an executable product; identity fixture only\n'
    if os.environ['MOCK_BINARY_MODE']=='symlink':
        other=target.with_name('other');other.write_bytes(contents);other.chmod(0o700);target.symlink_to(other)
    else:
        target.write_bytes(contents);target.chmod(0o700)
''')
                cargo.chmod(0o700)
                rustc = binary / 'rustc'
                rustc.write_text('#!/bin/sh\nprintf "stub; no Rust executed\\n"\n')
                rustc.chmod(0o700)
                report, runner, target = root / 'report', root / 'runner', root / 'target'
                runner.mkdir()
                outcome = subprocess.run(['bash', str(driver)], text=True, capture_output=True,
                    env=dict(env, PATH=str(binary)+os.pathsep+env['PATH'], DEV_TASK='net-wan-build',
                        SOURCE_PACKAGE=str(package), RUNNER_TEMP=str(runner), REPORT_DIR=str(report),
                        CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS='2', GITHUB_RUN_ID='stub',
                        GITHUB_SHA='control-fixture', MOCK_TRACE=str(root/'calls.json'), MOCK_BINARY_MODE=mode))
                self.assertEqual(json.loads((root/'calls.json').read_text()),
                    [['fmt','--all','--','--check'], ['build','--locked','-p','khor-node','--example','net_wan_probe']])
                if mode != 'normal':
                    self.assertNotEqual(outcome.returncode, 0)
                    self.assertFalse((report/'wan-binary.json').exists())
                    continue
                self.assertEqual(outcome.returncode, 0, outcome.stdout+outcome.stderr)
                manifest = json.loads((report/'wan-binary.json').read_text())
                actual = target/'debug/examples/net_wan_probe'
                self.assertEqual(manifest['source']['tree'], identity['tree'])
                self.assertEqual(manifest['path'], str(actual))
                self.assertEqual(manifest['sha256'], hashlib.sha256(actual.read_bytes()).hexdigest())
                self.assertFalse(manifest['executed'])
                self.assertEqual(sorted(p.name for p in report.iterdir()),
                    ['environment.json','format.log','source.json','wan-binary.json','wan-build.log'])

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
round_index = sum('--ignored' in call['args'] for call in calls)
calls.append({'args': sys.argv[1:], 'cc_only': os.environ.get('CC_ONLY'),
              'rtt': os.environ.get('CC_RTT_MS')})
trace.write_text(json.dumps(calls))
print('stub round', round_index)
if 'road_directions_do_not_wait_for_each_others_future_packets' in sys.argv:
    print('test result: ok. 1 passed; 0 failed;')
    sys.exit(9 if os.environ.get('MOCK_CALIBRATION_RED') else 0)
sys.exit(8 if '--ignored' in sys.argv and round_index == 0 else 0)
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
            self.assertIn('road_directions_do_not_wait_for_each_others_future_packets', calls[1]['args'])
            tests = calls[2:]
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
            self.assertTrue(mode['calibration_required'])
            self.assertEqual(mode['road_model'], 'independent_fifo_v2')
            self.assertEqual(json.loads((report / "source.json").read_text())['tree'], tree)
            # A failed calibration must stop before any timed measurement.
            failed_runner = root / "failed-runner"
            failed_runner.mkdir()
            failed_report = root / "failed-report"
            failed_trace = root / "failed-calls.json"
            failed = subprocess.run(["bash", str(driver)], text=True, capture_output=True,
                                    env=dict(env, PATH=str(binary) + os.pathsep + env['PATH'],
                                             DEV_TASK="net-congestion", SOURCE_PACKAGE=str(package),
                                             RUNNER_TEMP=str(failed_runner), REPORT_DIR=str(failed_report),
                                             GITHUB_RUN_ID="stub", GITHUB_SHA="control-fixture",
                                             CARGO_BUILD_JOBS="2", MOCK_TRACE=str(failed_trace),
                                             MOCK_CALIBRATION_RED="1"))
            self.assertEqual(failed.returncode, 9, failed.stderr)
            self.assertEqual(len(json.loads(failed_trace.read_text())), 2)
            self.assertEqual(len(list(failed_report.glob('congestion-*.log'))), 0)


    def test_short_cpu_calibration_and_hop_diagnostics_are_separate_from_timed_matrix(self):
        driver = Path(__file__).resolve().parents[1] / "network.sh"
        env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        env.update(GIT_CONFIG_GLOBAL="/dev/null", GIT_CONFIG_NOSYSTEM="1",
                   GIT_AUTHOR_NAME="Test", GIT_AUTHOR_EMAIL="test@localhost",
                   GIT_COMMITTER_NAME="Test", GIT_COMMITTER_EMAIL="test@localhost")
        for task, expected_args, repeats, diagnostic, expected_status, verify_failure in [
            ("netlab-cpu-check", ["paths::cpu::", "--ignored"], 1, None, 0, False),
            ("netlab-mobility-calibrate", ["paths::mobility::", "--ignored"], 1, None, 0, False),
            ("netlab-mobility-node", ["paths::mobile::mv6_overlap_", "--ignored"], 1, None, 7, False),
            ("netlab-mobility-node", ["paths::mobile::mv6_overlap_", "--ignored"], 1, None, 6, True),
            ("netlab-mobility-roam", ["paths::roam::roam_", "--ignored"], 1, None, 7, False),
            ("netlab-mobility-roam", ["paths::roam::roam_", "--ignored"], 1, None, 6, True),
            ("netlab-peer-mobility", ["hop::mobility::pm_", "--ignored"], 1, "lifecycle", 7, False),
            ("netlab-peer-mobility-samples", ["hop::mobility::pm_", "--ignored"], 1, "1", 7, False),
            ("netlab-peer-mobility-samples", ["hop::mobility::pm_", "--ignored"], 1, "1", 6, True),
            ("netlab-peer-mobility", ["hop::mobility::pm_", "--ignored"], 1, "lifecycle", 6, True),
            ("netlab-nat-rebind", [], 1, None, 7, False),
            ("netlab-nat-rebind", [], 2, None, 6, True),
            ("netlab-nat-rebind", [], 2, None, 0, False),
            ("netlab-nat-rebind-lifecycle", [], 1, "lifecycle", 7, False),
            ("netlab-nat-rebind-lifecycle", [], 2, "lifecycle", 6, True),
            ("netlab-nat-rebind-lifecycle", [], 2, "lifecycle", 0, False),
            ("netlab-hop-lifecycle", ["hop::f4i_"], 3, "1", 7, False),
            ("netlab-hop-ownership", ["hop::f4_"], 3, "lifecycle", 7, False),
            ("netlab-hop-ownership", ["hop::f4_"], 3, "lifecycle", 6, True),
            ("netlab-hop-reference", ["hop::reference::fr_", "--ignored"], 3, None, 7, False),
            ("netlab-hop-reference", ["hop::reference::fr_", "--ignored"], 3, None, 6, True),
            ("netlab-hop-reference-lifecycle", ["hop::reference::fr_automatic_", "--ignored"], 3, "lifecycle", 7, False),
            ("netlab-hop-reference-lifecycle", ["hop::reference::fr_automatic_", "--ignored"], 3, "lifecycle", 6, True),
            ("netlab-measure-check", ["paths::attempts::", "--ignored"], 1, None, 0, False),
            ("netlab-measure-check", ["paths::attempts::", "--ignored"], 1, None, 6, True),
        ]:
            with self.subTest(task=task), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source = root / "source"
                (source / "scripts").mkdir(parents=True)
                stub = r"""#!/bin/bash
python3 - "$@" <<'STUB'
import json, os, sys
from pathlib import Path
path = Path(os.environ['MOCK_TRACE'])
rows = json.loads(path.read_text()) if path.exists() else []
rows.append(dict(args=sys.argv[1:], threads=os.environ.get('NETLAB_TEST_THREADS'), diagnostic=os.environ.get('NETLAB_DIAGNOSTICS')))
path.write_text(json.dumps(rows))
if sys.argv[1:]==['paths::rebind::r0_', '--ignored']:
    calibration=Path(os.environ['NETLAB_RUN_DIR'])/'calibration'
    calibration.mkdir(parents=True)
    (calibration/'scenarios.log').write_text('[netlab-rebind-calibration] stub only\n'+'test result: ok. 1 passed; 0 failed;\n')
print('[netlab-result] cpu-or-lifecycle-stub | green | no real compiler or network lab')
sys.exit(7 if os.environ.get('MOCK_FAIL_FIRST') and len(rows) == 1 else 0)
STUB
"""
                (source / "scripts/netlab.sh").write_text(stub)
                (source / "scripts/netlab-attempts.py").write_text("""import json, os, sys
from pathlib import Path
Path(os.environ['MOCK_VERIFY']).write_text(json.dumps(sys.argv[1:]))
print(json.dumps({'valid': not bool(os.environ.get('MOCK_VERIFY_FAIL'))}))
sys.exit(6 if os.environ.get('MOCK_VERIFY_FAIL') else 0)
""")
                def git(*args):
                    return subprocess.check_output(["git", "-C", str(source), *args], text=True, env=env).strip()
                git("init", "-q")
                git("add", ".")
                git("-c", "core.hooksPath=/dev/null", "commit", "-qm", "fixture")
                package = root / "package"
                package.mkdir()
                git("clone", "--bare", "--quiet", str(source), str(package / "source.git"))
                (package / "source.json").write_text(json.dumps(dict(build_commit=git("rev-parse", "HEAD"), tree=git("rev-parse", "HEAD^{tree}"))))
                binary = root / "bin"
                binary.mkdir()
                for name, contents in {
                    "rustc": "#!/bin/sh\nprintf 'rustc stub; no compiler executed\\n'\n",
                    "unshare": "#!/bin/sh\nexit 0\n",
                    "cargo": "#!/bin/sh\nif [ \"$1\" = fmt ]; then exit 0; fi\nexit 97\n",
                }.items():
                    path = binary / name
                    path.write_text(contents)
                    path.chmod(0o755)
                trace, report = root / "calls.json", root / "report"
                runner = root / "runner"
                runner.mkdir()
                run_env = dict(env, PATH=str(binary) + os.pathsep + env['PATH'],
                               DEV_TASK=task, SOURCE_PACKAGE=str(package), RUNNER_TEMP=str(runner), REPORT_DIR=str(report),
                               GITHUB_RUN_ID="stub", GITHUB_SHA="control-fixture", CARGO_BUILD_JOBS="2", MOCK_TRACE=str(trace),
                               MOCK_VERIFY=str(root / 'verify.json'))
                run_env.pop("NETLAB_DIAGNOSTICS", None)
                if verify_failure:
                    run_env['MOCK_VERIFY_FAIL'] = "1"
                elif expected_status:
                    run_env['MOCK_FAIL_FIRST'] = "1"
                outcome = subprocess.run(["bash", str(driver)], text=True, capture_output=True, env=run_env)
                self.assertEqual(outcome.returncode, expected_status, outcome.stdout + outcome.stderr)
                rows = json.loads(trace.read_text())
                self.assertEqual(len(rows), repeats)
                for index, row in enumerate(rows):
                    wanted = ['paths::rebind::r0_' if index == 0 else 'paths::rebind::r1_', '--ignored'] if task.startswith('netlab-nat-rebind') else expected_args
                    self.assertEqual(row['args'], wanted)
                    self.assertEqual(row['threads'], '1')
                    self.assertEqual(row['diagnostic'], diagnostic)
                mode = json.loads((report / 'measurement-mode.json').read_text())
                self.assertFalse(mode['slo'])
                self.assertEqual(mode['repeats'], 1 if task.startswith('netlab-nat-rebind') else repeats)
                if task.startswith('netlab-nat-rebind'):
                    self.assertTrue(mode['calibration_required'])
                    self.assertEqual(mode['calibration_revision'], 'nat_port_rebind_calibration_v1')
                    self.assertEqual(mode['diagnostic'], diagnostic or 'off')
                    if expected_status == 7 and not verify_failure:
                        self.assertFalse((root / 'verify.json').exists())
                        continue
                if task == 'netlab-mobility-calibrate':
                    self.assertTrue((report / 'format.log').is_file())
                    self.assertEqual(mode['measurement_revision'], 'ipv6_renumber_v1')
                if task == 'netlab-measure-check':
                    self.assertEqual(json.loads((root / 'verify.json').read_text()),
                                     [str(report / 'netlab'), '--calibration'])
                    self.assertEqual(mode['measurement_revision'], 'attempt_outcomes_v4')
                    self.assertTrue((report / 'attempt-calibration.json').is_file())
                elif task in {'netlab-hop-ownership', 'netlab-hop-reference', 'netlab-hop-reference-lifecycle', 'netlab-mobility-node', 'netlab-mobility-roam', 'netlab-peer-mobility', 'netlab-peer-mobility-samples', 'netlab-nat-rebind', 'netlab-nat-rebind-lifecycle'}:
                    self.assertEqual(json.loads((root / 'verify.json').read_text()),
                                     [str(report / 'netlab')])
                    self.assertEqual(mode['measurement_revision'], 'attempt_outcomes_v4')
                    if task == 'netlab-mobility-node':
                        self.assertEqual(mode['scenario_revision'], 'ipv6_node_overlap_pair_v1')
                        self.assertEqual(mode['scenario_revisions'], ['ipv6_node_overlap_v1', 'ipv6_node_overlap_remote_first_v1'])
                        self.assertTrue(mode['ignored'])
                    elif task == 'netlab-mobility-roam':
                        self.assertEqual(mode['scenario_revision'], 'node_replug_triad_v1')
                        self.assertEqual(mode['scenario_revisions'], ['node_v4_nat_replug_v1', 'node_dual_stack_replug_v1', 'node_v4_to_v6_replug_v1'])
                        self.assertTrue(mode['ignored'])
                    elif task.startswith('netlab-peer-mobility'):
                        self.assertEqual(mode['scenario_revision'], 'peer_relay_replug_v1')
                        self.assertEqual(mode['diagnostic'], diagnostic)
                        self.assertTrue(mode['ignored'])
                    elif task.startswith('netlab-nat-rebind'):
                        self.assertEqual(mode['scenario_revision'], 'node_nat_port_rebind_v1')
                        self.assertTrue(mode['ignored'])
                    elif task == 'netlab-hop-ownership':
                        self.assertEqual(mode['diagnostic'], 'lifecycle')
                    else:
                        self.assertEqual(mode['reference'], 'hop_reference_v2')
                        self.assertTrue(mode['ignored'])
                        if task == 'netlab-hop-reference-lifecycle':
                            self.assertEqual(mode['diagnostic'], 'lifecycle')
                    self.assertTrue((report / 'attempts.json').is_file())


if __name__ == "__main__":
    unittest.main()
