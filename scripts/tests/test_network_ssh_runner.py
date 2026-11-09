import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location('runner', Path(__file__).resolve().parents[1] / 'network-ssh-runner.py')
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class NetworkSSHRunnerTest(unittest.TestCase):
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
