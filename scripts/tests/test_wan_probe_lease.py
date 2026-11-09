"""Real pipe/socket/process ownership with Python leaf stubs; no Rust or Node."""
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import unittest

WORKER = r'''
import json,os,pathlib,socket,sys
root=pathlib.Path(sys.argv[1]);os.fstat(int(sys.argv[2]))
listener=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM);listener.bind(str(root/'lease.sock'));listener.listen();listener.settimeout(5)
stream,_=listener.accept();listener.close();reader=stream.makefile('rb')
stream.sendall((json.dumps(dict(seq=0,ok=True,data=dict(id='a'*64,name='stub',fixture=str(root/'fixture'),run_id='a'*16,role='mac')))+'\n').encode())
if os.environ.get('STUB_CRASH'):
    os._exit(3)
seq=None
for line in reader:
    command=json.loads(line)
    if command['op']=='stop':seq=command['seq'];break
(root/'lease.sock').unlink()
print('[khor-wan-final] '+json.dumps(dict(run_id='a'*16,role='mac',resident_joined=True)),file=sys.stderr,flush=True)
if seq is not None:stream.sendall((json.dumps(dict(seq=seq,ok=True,data=dict(resident_joined=True)))+'\n').encode())
reader.close();stream.close()
'''

WRAPPER = r'''
import fcntl,importlib.util,os,pathlib,sys,types
spec=importlib.util.spec_from_file_location('guardian',sys.argv[1]);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
root=pathlib.Path(sys.argv[2]);mode=sys.argv[3]
# Unit-only resource fixture. Production main still enforces real 50/40 GiB.
module.shutil.disk_usage=lambda path:types.SimpleNamespace(free=(35 if mode=='reserve' else 100)*2**30)
with (root/'guard.lock').open('a') as lock:
    fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    env=dict(os.environ)
    if mode=='crash':env['STUB_CRASH']='1'
    raise SystemExit(module.monitor(root,[sys.executable,str(root/'worker.py'),str(root),str(lock.fileno())],env,lock.fileno(),kill_bridge=mode=='kill'))
'''


class WanLeaseTest(unittest.TestCase):
    def test_controller_stop_eof_and_sigkill_leave_no_owned_process(self):
        guardian = Path(__file__).resolve().parents[1] / 'wan-probe-lease.py'
        for mode in ['stop', 'eof', 'kill', 'crash', 'reserve']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(prefix='kw-') as directory:
                root = Path(directory)
                (root/'worker.py').write_text(WORKER)
                (root/'wrapper.py').write_text(WRAPPER)
                process = subprocess.Popen([sys.executable,str(root/'wrapper.py'),str(guardian),str(root),mode],
                    stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                try:
                    if mode in ['stop', 'eof']:
                        self.assertTrue(select.select([process.stdout],[],[],10)[0], 'worker readiness')
                        ready=json.loads(process.stdout.readline())
                        self.assertEqual(ready['seq'],0)
                        if mode=='stop':
                            process.stdin.write(b'{"seq":1,"op":"stop"}\n');process.stdin.flush()
                            self.assertTrue(select.select([process.stdout],[],[],10)[0], 'joined stop response')
                            self.assertTrue(json.loads(process.stdout.readline())['data']['resident_joined'])
                        process.stdin.close();process.stdin=None
                    status=process.wait(timeout=15)
                    stdout,stderr=process.communicate(timeout=2)
                    self.assertEqual(status,0 if mode in ['stop','eof','kill'] else 1,stderr.decode()+stdout.decode())
                    result=json.loads((root/'lifecycle.json').read_text())
                    self.assertEqual(result['remaining_owned_processes'],[])
                    self.assertFalse(result.get('forced_worker_stop',False))
                    if mode in ['stop','eof','kill']:
                        self.assertTrue(result['resident_joined'])
                        self.assertEqual(result['worker_group'],result['group'])
                        self.assertEqual(result['bridge_group'],result['group'])
                        self.assertFalse((root/'lease.sock').exists())
                    if mode=='kill':
                        self.assertTrue(result['bridge_killed'])
                        self.assertEqual(result['bridge_returncode'],-9)
                    with (root/'guard.lock').open('a') as lock:
                        import fcntl
                        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
                finally:
                    if process.poll() is None:
                        process.kill();process.wait()
                    for stream in [process.stdin,process.stdout,process.stderr]:
                        if stream is not None:stream.close()
