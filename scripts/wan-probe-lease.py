"""Private SSH control channel for an isolated WAN worker; never log its RPC bytes.

Run this guardian under the installed Mac admission guard, or the owned Turing
flock. Its bridge owns the worker's Unix lease. EOF or a killed bridge closes
that lease while the guardian and inherited admission descriptor stay alive.
"""
import contextlib
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import select
import secrets
import shutil
import signal
import socket
import subprocess
import sys
import time

GUARD_SHA = '998f5723eb447a1634411397c18a524e5ed0d50bd6dbd81001a16626219f64eb'


def digest(path):
    out = hashlib.sha256()
    with path.open('rb') as source:
        for part in iter(lambda: source.read(1024 * 1024), b''):
            out.update(part)
    return out.hexdigest()


def record(path, value):
    temporary = path.with_suffix(path.suffix + '.new')
    with temporary.open('x') as out:
        os.chmod(temporary, 0o600)
        json.dump(value, out, indent=2)
        out.write('\n')
    temporary.replace(path)


@contextlib.contextmanager
def admission(role):
    lock = None
    if role == 'mac':
        assert platform.system() == 'Darwin'
        builder = Path.home() / 'khor-builder'
        assert digest(builder / 'bin/build_guard.py') == GUARD_SHA
        expected = (builder / 'cache/build.lock').stat()
        descriptors = []
        for entry in os.listdir('/dev/fd'):
            try:
                fd = int(entry)
                actual = os.fstat(fd)
            except (ValueError, OSError):
                continue
            if fd > 2 and (actual.st_dev, actual.st_ino) == (expected.st_dev, expected.st_ino):
                descriptors.append(fd)
        assert descriptors, 'installed admission descriptor was not inherited'
        fd = descriptors[0]
        base = builder / 'netlab/feat-net-wan'
        target = builder / 'cache/target'
    else:
        assert role == 'linux' and platform.system() == 'Linux' and os.getuid() == 1110
        owned = Path('/tmp/khor-netlab-feat-net-1110')
        assert not owned.is_symlink()
        assert json.loads((owned/'owner.json').read_text()) == dict(schema=1, owner='khor feat/net netlab', uid=os.getuid())
        lock = (owned/'build.lock').open('a')
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        fd = lock.fileno()
        base, target = owned/'wan', owned/'cache/target'
    try:
        assert shutil.disk_usage(base.parent).free >= 50 * 2**30
        yield fd, base, target
    finally:
        if lock is not None:
            lock.close()


def owned_base(base):
    expected = dict(schema=1, owner='khor feat/net wan base', uid=os.getuid())
    if not base.exists():
        base.mkdir(mode=0o700)
        record(base/'owner.json', expected)
    assert base.is_dir() and not base.is_symlink() and base.stat().st_uid == os.getuid()
    assert base.stat().st_mode & 0o777 == 0o700
    assert json.loads((base/'owner.json').read_text()) == expected


def bridge(root, fd):
    os.fstat(fd)  # This child must retain its guardian's admission lease.
    stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    stream.settimeout(15)
    stream.connect(str(root/'lease.sock'))
    stream.settimeout(None)
    first, buffered = True, b''
    try:
        while True:
            ready, _, _ = select.select([sys.stdin.buffer, stream], [], [], 1)
            if sys.stdin.buffer in ready:
                data = os.read(sys.stdin.fileno(), 65536)
                if not data:
                    return 0
                stream.sendall(data)
            if stream in ready:
                data = stream.recv(65536)
                if not data:
                    return 0
                if first:
                    buffered += data
                    assert len(buffered) <= 65536
                    if b'\n' in buffered:
                        message = json.loads(buffered.split(b'\n', 1)[0])
                        public = message['data']
                        assert message['seq'] == 0 and message['ok'] is True
                        assert set(public) == {'id', 'name', 'fixture', 'run_id', 'role'}
                        record(root/'bridge-ready.json', dict(ready=True, bridge_pid=os.getpid(), **public))
                        first = False
                # This is a private RPC transport, not a diagnostic log/artifact.
                sys.stdout.buffer.write(data)
                sys.stdout.buffer.flush()
    finally:
        stream.close()


def owned_processes(root):
    rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,pgid=,command='], text=True)
    matches = []
    for line in rows.splitlines():
        fields = line.strip().split(None, 2)
        if len(fields) == 3 and int(fields[0]) != os.getpid() and str(root) in fields[2]:
            matches.append(dict(pid=int(fields[0]), group=int(fields[1]), command=fields[2]))
    return matches


def interrupted(_signum, _frame):
    raise KeyboardInterrupt


def monitor(root, command, environment, fd, kill_bridge=False):
    result = dict(schema=1, guardian_pid=os.getpid(), group=os.getpgrp(), resident_joined=False)
    worker = controller = None
    began = time.monotonic()
    old_handlers = {sig: signal.signal(sig, interrupted) for sig in [signal.SIGTERM, signal.SIGINT]}
    try:
        with (root/'worker.log').open('xb') as output:
            worker = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=output,
                stderr=subprocess.STDOUT, env=environment, pass_fds=(fd,))
            result.update(worker_pid=worker.pid, worker_group=os.getpgid(worker.pid))
            assert result['worker_group'] == result['group']
            until = time.monotonic() + 15
            while not (root/'lease.sock').exists():
                assert worker.poll() is None and time.monotonic() < until, 'worker did not open its private lease'
                time.sleep(0.05)
            controller = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--bridge', str(root), str(fd)],
                env=environment, pass_fds=(fd,))
            result.update(bridge_pid=controller.pid, bridge_group=os.getpgid(controller.pid))
            assert result['bridge_group'] == result['group']
            record(root/'lifecycle.json', result)
            ended_at = None
            while worker.poll() is None:
                if kill_bridge and not result.get('bridge_killed') and (root/'bridge-ready.json').exists():
                    os.kill(controller.pid, signal.SIGKILL)
                    result['bridge_killed'] = True
                if controller.poll() is not None:
                    ended_at = ended_at or time.monotonic()
                    assert time.monotonic()-ended_at < 55, 'worker outlived its closed controller lease'
                assert time.monotonic()-began < 660, 'guardian absolute resource deadline'
                assert shutil.disk_usage(root).free >= 40 * 2**30, 'disk reserve reached'
                time.sleep(0.1)
    except BaseException as error:
        result['error'] = type(error).__name__ + ': ' + str(error)
    finally:
        for sig in old_handlers:
            signal.signal(sig, signal.SIG_IGN)
        if controller is not None:
            if worker is not None and worker.poll() is not None:
                try:
                    controller.wait(timeout=2)
                except subprocess.TimeoutExpired:
                    pass
            if controller.poll() is None:
                controller.terminate()
            try:
                controller.wait(timeout=3)
            except subprocess.TimeoutExpired:
                controller.kill()
                controller.wait(timeout=3)
            result['bridge_returncode'] = controller.returncode
        if worker is not None:
            try:
                worker.wait(timeout=50)
            except subprocess.TimeoutExpired:
                worker.terminate()
                try:
                    worker.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    worker.kill()
                    worker.wait(timeout=5)
                result['forced_worker_stop'] = True
            result['worker_returncode'] = worker.returncode
        finals = []
        for line in (root/'worker.log').read_text(errors='replace').splitlines():
            if line.startswith('[khor-wan-final] '):
                finals.append(json.loads(line.split(' ', 1)[1]))
        result['worker_final'] = finals[-1] if finals else None
        result['resident_joined'] = bool(finals and finals[-1]['resident_joined'])
        result['remaining_owned_processes'] = owned_processes(root)
        result['elapsed_seconds'] = time.monotonic()-began
        record(root/'lifecycle.json', result)
        for sig, handler in old_handlers.items():
            signal.signal(sig, handler)
    bridge_ok = result.get('bridge_returncode') == (-signal.SIGKILL if kill_bridge else 0)
    return 0 if (result.get('worker_returncode') == 0 and bridge_ok and result['resident_joined']
        and not result['remaining_owned_processes'] and 'error' not in result and not result.get('forced_worker_stop')) else 1


def run(configuration):
    meta = configuration.lstat()
    assert configuration.is_file() and not configuration.is_symlink() and meta.st_uid == os.getuid()
    assert meta.st_mode & 0o777 == 0o600
    config = json.loads(configuration.read_text())
    run_id, role, binary = config['run_id'], config['role'], config['binary']
    assert re.fullmatch('[0-9a-f]{16}', run_id) and role in ['mac', 'linux']
    with admission(role) as (fd, base, target):
        owned_base(base)
        source = Path(binary['path'])
        assert source.is_absolute()
        source.resolve().relative_to(target.resolve())
        assert source.is_file() and not source.is_symlink() and source.stat().st_uid == os.getuid()
        assert digest(source) == binary['sha256'] and source.stat().st_size == binary['bytes']
        assert binary['source']['tree'] == config['source_tree']
        assert binary['source']['snapshot_commit'] == config['snapshot_commit']
        root = base/(run_id+'-'+role)
        root.mkdir(mode=0o700)
        record(root/'owner.json', dict(schema=1, owner='khor feat/net wan', uid=os.getuid(), run_id=run_id, role=role))
        record(root/'inputs.json', dict(configuration=config, guardian_sha256=digest(Path(__file__).resolve())))
        executable = root/'probe'
        shutil.copyfile(source, executable)
        executable.chmod(0o700)
        assert digest(executable) == binary['sha256']
        (root/'tmux').mkdir(mode=0o700)
        allowed = ['HOME', 'USER', 'LOGNAME', 'LANG', 'LC_ALL', 'TMPDIR']
        environment = {k: os.environ[k] for k in allowed if k in os.environ}
        environment.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', KHOR_HOME=str(root/'home'),
            TMUX_TMPDIR=str(root/'tmux'), KHOR_NO_MDNS='1')
        return monitor(root, [str(executable), str(root), run_id, role], environment, fd,
            kill_bridge=config.get('kill_bridge_after_ready', False))


def check_contracts(manifest_file, report):
    """Mac CI fallback: no mint/pair RPC, only real resident lifetime contracts."""
    assert platform.system() == 'Darwin'
    manifest = json.loads(manifest_file.read_text())
    results = []
    with admission('mac') as (fd, base, _target):
        owned_base(base)
        inputs = base.parent/'wan-input'
        expected = dict(schema=1, owner='khor feat/net wan inputs', uid=os.getuid())
        if not inputs.exists():
            inputs.mkdir(mode=0o700)
            record(inputs/'owner.json', expected)
        assert inputs.is_dir() and not inputs.is_symlink() and inputs.stat().st_uid == os.getuid()
        assert json.loads((inputs/'owner.json').read_text()) == expected
        for mode in ['stop', 'eof', 'kill']:
            run_id = secrets.token_hex(8)
            root = base/(run_id+'-mac')
            config = dict(schema=1, run_id=run_id, role='mac', binary=manifest,
                source_tree=manifest['source']['tree'], snapshot_commit=manifest['source']['snapshot_commit'],
                guardian_commit=os.environ.get('GITHUB_SHA'), guardian_sha256=digest(Path(__file__).resolve()),
                kill_bridge_after_ready=mode=='kill')
            path = inputs/(run_id+'-mac.json')
            record(path, config)
            error = None
            with (inputs/(run_id+'-controller.log')).open('xb') as stderr:
                process = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), str(path)],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr, pass_fds=(fd,))
                try:
                    if mode != 'kill':
                        assert select.select([process.stdout], [], [], 110)[0], 'readiness deadline'
                        ready = json.loads(process.stdout.readline())
                        assert ready['seq']==0 and ready['ok'] and ready['data']['run_id']==run_id
                        if mode == 'stop':
                            process.stdin.write(b'{"seq":1,"op":"stop"}\n')
                            process.stdin.flush()
                            assert select.select([process.stdout], [], [], 65)[0], 'stop reply deadline'
                            stopped = json.loads(process.stdout.readline())
                            assert stopped['seq']==1 and stopped['ok'] and stopped['data']['resident_joined']
                        process.stdin.close()
                        process.stdin = None
                    assert process.wait(timeout=165 if mode=='kill' else 75)==0, 'guardian failed'
                except BaseException as failure:
                    error = type(failure).__name__ + ': ' + str(failure)
                finally:
                    if process.stdin is not None:
                        process.stdin.close()
                        process.stdin = None
                    if process.poll() is None:
                        process.terminate()
                        try:
                            process.wait(timeout=60)
                        except subprocess.TimeoutExpired:
                            process.kill()
                            process.wait(timeout=5)
                    process.stdout.close()
            lifecycle = json.loads((root/'lifecycle.json').read_text()) if (root/'lifecycle.json').exists() else {}
            remaining = owned_processes(root) if root.exists() else []
            forced = bool(remaining)
            if remaining:
                for row in remaining:
                    os.kill(row['pid'], signal.SIGTERM)
                until = time.monotonic()+5
                while owned_processes(root) and time.monotonic()<until:
                    time.sleep(0.1)
                for row in owned_processes(root):
                    os.kill(row['pid'], signal.SIGKILL)
                remaining = owned_processes(root)
            final = lifecycle.get('worker_final') or {}
            passed = (not error and not forced and not remaining and lifecycle.get('resident_joined')
                and lifecycle.get('worker_returncode')==0 and final.get('run_id')==run_id and final.get('role')=='mac'
                and not (root/'lease.sock').exists())
            result = dict(schema=1, run_id=run_id, role='mac', mode=mode, passed=bool(passed),
                source=manifest['source'], binary_sha256=manifest['sha256'], guardian_sha256=config['guardian_sha256'],
                lifecycle=lifecycle, remaining_owned_processes=remaining, forced_cleanup=forced, error=error,
                limits=['Isolated real Node lifetime; no pairing, WAN business, phone or energy proof'])
            record(report/('wan-lease-'+mode+'.json'), result)
            results.append(result)
            print(json.dumps(dict(mode=mode, passed=bool(passed), resident_joined=lifecycle.get('resident_joined'),
                remaining=len(remaining), error=error)), flush=True)
            if passed:
                assert json.loads((root/'owner.json').read_text()) == dict(schema=1, owner='khor feat/net wan', uid=os.getuid(), run_id=run_id, role='mac')
                for name in ['home', 'tmux']:
                    shutil.rmtree(root/name)
                (root/'probe').unlink()
                (root/'worker.log').unlink()
            else:
                break  # Keep this owned diagnostic state; do not launch more residents.
    complete = len(results)==3 and all(result['passed'] for result in results)
    record(report/'wan-lifetime-contracts.json', dict(schema=1, passed=complete, results=results))
    return 0 if complete else 1


if __name__ == '__main__':
    try:
        if len(sys.argv) == 4 and sys.argv[1] == '--check':
            raise SystemExit(check_contracts(Path(sys.argv[2]), Path(sys.argv[3])))
        if len(sys.argv) == 4 and sys.argv[1] == '--bridge':
            raise SystemExit(bridge(Path(sys.argv[2]), int(sys.argv[3])))
        if len(sys.argv) == 2:
            raise SystemExit(run(Path(sys.argv[1])))
        raise ValueError('usage: wan-probe-lease.py <private-config>')
    except Exception as error:
        print(type(error).__name__ + ': ' + str(error), file=sys.stderr)
        raise SystemExit(1)
