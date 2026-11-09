"""Credential isolation and actual private pipe responses; no product process."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('wan_peer', Path(__file__).resolve().parents[1]/'wan-probe-peer.py')
peer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(peer)


class WanPeerTest(unittest.TestCase):
    def test_child_environment_excludes_capabilities_and_credentials(self):
        with patch.dict(os.environ, dict(HOME='/fixture', GH_TOKEN='private',
                KHOR_WAN_CAPABILITY='private', SSH_AUTH_SOCK='/private',
                OPENAI_API_KEY='private', GITHUB_TOKEN='private'), clear=True):
            self.assertEqual(peer.child_environment(), dict(HOME='/fixture', PATH='/usr/bin:/bin:/usr/sbin:/sbin'))

    def test_rpc_failure_never_echoes_private_response(self):
        process = subprocess.Popen([sys.executable, '-c',
            'import sys; sys.stdin.readline(); print(\'{"seq":1,"ok":false,"error":"private-ticket"}\',flush=True)'],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        try:
            client = peer.ProbeClient(process)
            with self.assertRaises(RuntimeError) as failure:
                client.call('pair', ticket='private-ticket')
            self.assertNotIn('private-ticket', str(failure.exception))
            process.wait(timeout=5)
        finally:
            process.stdin.close()
            process.stdout.close()
            if process.poll() is None:
                process.kill()
                process.wait()

    def test_all_planned_reads_retained_after_transport_loss(self):
        class Lost:
            process = type('Ended', (), dict(poll=lambda self: 1))()
            def call(self, *args, **kwargs):
                raise BrokenPipeError('private bytes must not be copied')
        outcomes = peer.measure(Lost(), 'a'*64, '/fixture', 30)
        self.assertEqual(len(outcomes), 30)
        self.assertEqual(outcomes[0]['outcome'], 'error')
        self.assertTrue(all(row['outcome']=='cancelled' for row in outcomes[1:]))
        self.assertNotIn('private bytes', str(outcomes))
