import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location('runner', Path(__file__).resolve().parents[1] / 'network-ssh-runner.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class NetworkSSHRunnerTest(unittest.TestCase):
    def test_stable_source_cache_keeps_unchanged_mtimes_and_refuses_tracked_edits(self):
        # Tiny Git data only: no Cargo, compiler, product or network lab runs.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'cache').mkdir()
            (root / 'owner.json').write_text(json.dumps(dict(runner.OWNER, uid=os.getuid())))
            fixture = root / 'fixture'
            fixture.mkdir()
            env = dict(os.environ, GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1',
                       GIT_AUTHOR_NAME='Test', GIT_AUTHOR_EMAIL='test@localhost',
                       GIT_COMMITTER_NAME='Test', GIT_COMMITTER_EMAIL='test@localhost')
            def git(*args):
                return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', '-C', str(fixture), *args],
                                               env=env, text=True).strip()
            git('init', '-q')
            (fixture / 'lib.rs').write_text('unchanged fixture data\n')
            (fixture / 'README.md').write_text('first\n')
            git('add', '.')
            git('commit', '-qm', 'fixture')
            def prepare(name):
                run = root / name
                (run / 'package').mkdir(parents=True)
                (run / 'report').mkdir()
                git('clone', '--bare', '--quiet', str(fixture), str(run / 'package/source.git'))
                expected = dict(build_commit=git('rev-parse', 'HEAD'), tree=git('rev-parse', 'HEAD^{tree}'))
                (run / 'package/source.json').write_text(json.dumps(expected))
                return run
            first = prepare('one')
            runner.cached_source(root, first, env)
            cached = root / 'cache/source'
            self.assertEqual(env['KHOR_NETWORK_SOURCE_DIR'], str(cached))
            self.assertFalse(json.loads((first / 'report/source-cache.json').read_text())['reused'])
            os.utime(cached / 'lib.rs', ns=(1_600_000_000_000_000_000,) * 2)
            stamp = (cached / 'lib.rs').stat().st_mtime_ns
            second = prepare('two')
            runner.cached_source(root, second, env)
            self.assertEqual((cached / 'lib.rs').stat().st_mtime_ns, stamp)
            self.assertTrue(json.loads((second / 'report/source-cache.json').read_text())['reused'])
            (fixture / 'README.md').write_text('second\n')
            git('add', 'README.md')
            git('commit', '-qm', 'documentation only')
            third = prepare('three')
            runner.cached_source(root, third, env)
            self.assertEqual((cached / 'lib.rs').stat().st_mtime_ns, stamp)
            self.assertEqual((cached / 'README.md').read_text(), 'second\n')
            # A format failure must export a patch without modifying the cache.
            # Both executables below are stubs; no Rust or product runs locally.
            binary = root / 'bin'
            binary.mkdir()
            rustc = binary / 'rustc'
            rustc.write_text("#!/bin/sh\nprintf 'rustc stub; no compiler executed\\n'\n")
            rustc.chmod(0o755)
            cargo = binary / 'cargo'
            cargo.write_text('#!' + sys.executable + '\n' + '''
from pathlib import Path
import sys
if sys.argv[1:] == ['fmt', '--all', '--', '--check']:
    sys.exit(1)
if sys.argv[1:] == ['fmt', '--all']:
    Path('lib.rs').write_text('formatted fixture data\\n')
    sys.exit(0)
sys.exit(97)
''')
            cargo.chmod(0o755)
            work = third / 'work'
            work.mkdir()
            outcome = subprocess.run(['bash', str(Path(__file__).resolve().parents[1] / 'network.sh')],
                                     env=dict(env, PATH=str(binary) + os.pathsep + env['PATH'],
                                              DEV_TASK='net-unit', SOURCE_PACKAGE=str(third / 'package'),
                                              REPORT_DIR=str(third / 'report'), RUNNER_TEMP=str(work),
                                              GITHUB_RUN_ID='stub', GITHUB_SHA='control-fixture', CARGO_BUILD_JOBS='2'),
                                     text=True, capture_output=True)
            self.assertEqual(outcome.returncode, 1, outcome.stdout + outcome.stderr)
            self.assertIn('+formatted fixture data', (third / 'report/format.patch').read_text())
            self.assertEqual((cached / 'lib.rs').read_text(), 'unchanged fixture data\n')
            self.assertEqual((cached / 'lib.rs').stat().st_mtime_ns, stamp)
            (cached / 'lib.rs').write_text('preserve this unexpected tracked edit\n')
            with self.assertRaisesRegex(ValueError, 'modified source cache'):
                runner.cached_source(root, third, env)
            self.assertEqual((cached / 'lib.rs').read_text(), 'preserve this unexpected tracked edit\n')

    def test_stable_source_cache_preserves_unowned_directories_and_symlinks(self):
        for linked in [False, True]:
            with self.subTest(linked=linked), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                (root / 'cache').mkdir()
                preserved = root / 'preserved'
                preserved.mkdir()
                (preserved / 'keep').write_text('another owner\n')
                cached = root / 'cache/source'
                if linked:
                    cached.symlink_to(preserved, target_is_directory=True)
                else:
                    cached.mkdir()
                    (cached / 'keep').write_text('another checkout\n')
                with self.assertRaisesRegex(ValueError, 'source cache'):
                    runner.cached_source(root, root / 'unused-run', {})
                self.assertEqual((preserved / 'keep').read_text(), 'another owner\n')
                if not linked:
                    self.assertEqual((cached / 'keep').read_text(), 'another checkout\n')

    def test_build_environment_neither_forwards_auth_nor_writes_cargo_to_home(self):
        with patch.dict(os.environ, {'HOME': '/nfs/user', 'SSH_AUTH_SOCK': '/personal/agent',
                                   'GH_TOKEN': 'private', 'GITHUB_TOKEN': 'private',
                                   'CARGO_HOME': '/personal/cargo'}, clear=True):
            env = runner.environment(Path('/tmp/owned'), Path('/tmp/owned/runs/one'),
                                     dict(task='netlab-recovery', control_commit='a' * 40, run_id='one'))
        self.assertEqual(env['HOME'], '/nfs/user')
        self.assertEqual(env['CARGO_HOME'], '/tmp/owned/cache/cargo')
        self.assertEqual(env['CARGO_TARGET_DIR'], '/tmp/owned/cache/target')
        self.assertEqual(env['NETLAB_TOOLS'], '/tmp/owned/tools')
        self.assertTrue(env['RUSTC'].startswith('/tmp/owned/tools/'))
        self.assertFalse(any(name in env for name in ['GH_TOKEN', 'GITHUB_TOKEN', 'SSH_AUTH_SOCK']))

    def test_checksum_failure_refuses_to_materialize_source(self):
        with tempfile.TemporaryDirectory() as directory:
            run = Path(directory)
            (run / 'input').mkdir()
            (run / 'input/source.bundle').write_bytes(b'wrong private source')
            with patch.object(runner.subprocess, 'run') as git, self.assertRaisesRegex(ValueError, 'checksum'):
                runner.materialize(run, dict(source={}, bundle_sha256='0' * 64), {})
            git.assert_not_called()
            self.assertFalse((run / 'package').exists())

    def test_tool_extraction_rejects_paths_and_links_outside_owned_directory(self):
        for link in [False, True]:
            with self.subTest(link=link), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                archive = root / 'fixture.tar'
                with tarfile.open(archive, 'w') as output:
                    member = tarfile.TarInfo('tool' if link else '../escaped')
                    if link:
                        member.type, member.linkname = tarfile.SYMTYPE, '../../escaped'
                        output.addfile(member)
                    else:
                        member.size = 1
                        output.addfile(member, io.BytesIO(b'x'))
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    runner.extract(archive, root / 'toolchain')
                self.assertFalse((root / 'escaped').exists())

    def test_an_existing_toolchain_without_the_pinned_receipt_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            prefix = root / 'tools/rust195'
            prefix.mkdir(parents=True)
            (prefix / 'keep').write_text('another build')
            with self.assertRaisesRegex(ValueError, 'unverified'):
                runner.toolchain(root, {}, root)
            self.assertEqual((prefix / 'keep').read_text(), 'another build')


if __name__ == '__main__':
    unittest.main()
