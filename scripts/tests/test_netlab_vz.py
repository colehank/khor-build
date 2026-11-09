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


spec = importlib.util.spec_from_file_location('netlab_vz', Path(__file__).resolve().parents[1] / 'netlab-vz.py')
vz = importlib.util.module_from_spec(spec)
spec.loader.exec_module(vz)


class Foreground:
    pid = 4242
    returncode = None

    def __init__(self, timeout=False):
        self.timeout = timeout
        self.killed = False

    def poll(self):
        return self.returncode

    def wait(self, timeout):
        if self.timeout and not self.killed:
            raise subprocess.TimeoutExpired('fake-foreground', timeout)
        self.returncode = -9 if self.killed else 0
        return self.returncode

    def kill(self):
        self.killed = True


class NetlabVZTest(unittest.TestCase):
    def test_config_has_no_host_mounts_personal_keys_agent_or_automatic_ports(self):
        cfg = vz.config(Path('/owned/pinned.img'))
        self.assertEqual(cfg['mounts'], [])
        self.assertFalse(any(cfg['ssh'].values()))
        self.assertFalse(any(cfg['containerd'].values()))
        self.assertFalse(cfg['propagateProxyEnv'])
        self.assertFalse(cfg['networkOpts']['hostLoopback'])
        self.assertTrue(cfg['portForwards'][0]['ignore'])
        self.assertEqual(cfg['portForwards'][0]['guestIP'], '0.0.0.0')
        self.assertEqual(cfg['portForwards'][0]['proto'], 'any')
        self.assertEqual(cfg['portForwards'][0]['guestPortRange'], [1, 65535])
        self.assertEqual(len(cfg['images']), 1)
        self.assertEqual(cfg['images'][0]['digest'], 'sha256:' + vz.IMAGE_SHA)
        self.assertEqual((cfg['cpus'], cfg['memory'], cfg['disk']), (2, '6GiB', '30GiB'))

    def test_unverified_cache_is_preserved_and_never_extracted_or_executed(self):
        with tempfile.TemporaryDirectory() as directory:
            cached = Path(directory) / 'wrong.tar.gz'
            cached.write_bytes(b'unverified executable cache')
            with patch.object(vz, 'urlopen') as network, self.assertRaisesRegex(ValueError, 'unverified cached'):
                vz.download('https://example.invalid', '0' * 64, 1024, cached)
            network.assert_not_called()
            self.assertEqual(cached.read_bytes(), b'unverified executable cache')

    def test_archive_cannot_write_outside_its_owned_directory(self):
        for link in [False, True]:
            with self.subTest(link=link), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                archive = root / 'fixture.tar'
                with tarfile.open(archive, 'w') as output:
                    member = tarfile.TarInfo('tool' if link else '../outside')
                    if link:
                        member.type = tarfile.SYMTYPE
                        member.linkname = '../../outside'
                        output.addfile(member)
                    else:
                        member.size = 1
                        output.addfile(member, io.BytesIO(b'x'))
                with self.assertRaisesRegex(ValueError, 'escapes'):
                    vz.extract_owned(archive, root / 'tools')
                self.assertFalse((root / 'outside').exists())

    def test_existing_unowned_root_is_never_claimed_or_removed(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'owner.json').write_text('{"owner":"another task"}')
            (root / 'keep').write_text('unchanged')
            with self.assertRaisesRegex(ValueError, 'unowned'):
                vz.create_run(root, '123-1', {'tree': 'a' * 40}, root / 'report')
            self.assertEqual((root / 'keep').read_text(), 'unchanged')
            self.assertFalse((root / 'runs').exists())

    def supervise_fixture(self, *, stop_timeout=False, wrong_group=False):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            report = root / 'report'
            report.mkdir()
            foreground = Foreground(timeout=stop_timeout)
            process_group = 11 if wrong_group else 10
            stop = subprocess.CompletedProcess([], 0, stdout='', stderr='')
            effect = subprocess.TimeoutExpired('stop', 15) if stop_timeout else None
            with patch.object(vz, 'assert_owned', return_value={'report_dir': str(report)}), \
                 patch.object(vz, 'inherited_lock_fd', return_value=3), \
                 patch.dict(os.environ, KHOR_NETLAB_GUARD_FD='3'), \
                 patch.object(vz.os, 'getpgrp', return_value=10), \
                 patch.object(vz.os, 'getpgid', return_value=process_group), \
                 patch.object(vz.signal, 'signal'), \
                 patch.object(vz.subprocess, 'Popen', return_value=foreground) as popen, \
                 patch.object(vz.select, 'select', return_value=([1], [], [])), \
                 patch.object(vz.os, 'read', return_value=b''), \
                 patch.object(vz, 'command', return_value=stop, side_effect=effect) as command, \
                 patch.object(vz, 'status', return_value={'status': 'Stopped'}):
                code = vz.supervise(root / '123-1', 'fixture')
            state = json.loads((report / 'lifecycle-fixture.json').read_text())
            self.assertFalse(popen.call_args.kwargs.get('start_new_session', False))
            self.assertEqual(popen.call_args.kwargs['pass_fds'], (3,))
            self.assertEqual(command.call_args.args[1:4], ('stop', '--force', 'khor-netlab-123-1'))
            return code, state, foreground

    def test_parent_lease_eof_stops_and_joins_foreground_in_the_guard_group(self):
        code, state, foreground = self.supervise_fixture()
        self.assertEqual(code, 0)
        self.assertTrue(state['lease_eof'])
        self.assertTrue(state['stopped'])
        self.assertEqual(foreground.returncode, 0)

    def test_failed_stop_still_kills_foreground_and_reports_failure(self):
        code, state, foreground = self.supervise_fixture(stop_timeout=True)
        self.assertEqual(code, 1)
        self.assertTrue(foreground.killed)
        self.assertTrue(state['stopped'])
        self.assertIn('stop_error', state)

    def test_a_foreground_that_escapes_the_guard_group_is_stopped_and_rejected(self):
        code, state, _ = self.supervise_fixture(wrong_group=True)
        self.assertEqual(code, 1)
        self.assertTrue(state['stopped'])
        self.assertIn('escaped', state['error'])


if __name__ == '__main__':
    unittest.main()
