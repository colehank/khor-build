"""Owned VZ capability probe; no product build, host mounts or global installation."""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import re
import select
import shutil
import signal
import subprocess
import sys
import tarfile
import time
from urllib.request import Request, urlopen


BASE = Path('/Users/z13/khor-builder/netlab')
GUARD = Path('/Users/z13/khor-builder/bin/build_guard.py')
LOCK = Path('/Users/z13/khor-builder/cache/build.lock')
GUARD_SHA = '998f5723eb447a1634411397c18a524e5ed0d50bd6dbd81001a16626219f64eb'
LIMA_SHA = 'b7af444bd8f91d3c307477e562f2cf3bd7595d62b9429185e4fa841563578cfc'
LIMA_URL = 'https://api.github.com/repos/lima-vm/lima/releases/assets/717479690'
IMAGE_SHA = '50e24e1fb79e89116e939d90aa7738f7af48c442fa1bc2510131e0b39c50348e'
IMAGE_URL = 'https://cloud-images.ubuntu.com/releases/noble/release-20261017/ubuntu-24.04-server-cloudimg-arm64.img'
OWNER = {'schema': 1, 'owner': 'colehank/khor-build development netlab'}

GUEST_PROBE = r'''
set -euo pipefail
test "$(uname -s)" = Linux
sudo -n apt-get update
sudo -n env DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends iproute2 nftables conntrack
if test -e /proc/sys/kernel/apparmor_restrict_unprivileged_userns; then
  sudo -n sysctl -w kernel.apparmor_restrict_unprivileged_userns=0
fi
sudo -n modprobe nf_conntrack_netlink
unshare -Urn sh -ec '
  ip link set lo up
  ip link add probe0 type veth peer name probe1
  ip link set probe0 up
  ip link set probe1 up
  tc qdisc add dev probe0 root netem delay 1ms
  tc qdisc show dev probe0
  nft add table inet probe
  nft list ruleset
  conntrack -L
  ip -details link show probe0
'
python3 - <<'PY'
import json, os, platform
print('KHOR_NETLAB_CAPS ' + json.dumps(dict(system=platform.system(), arch=platform.machine(), kernel=platform.release(), user_uid=os.getuid(), namespace=True, veth=True, netem=True, nft=True, conntrack=True)))
PY
'''


def record(path, value):
    temporary = path.with_suffix(path.suffix + '.new')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            result.update(block)
    return result.hexdigest()


def download(url, expected, maximum, destination):
    if destination.exists():
        if destination.is_symlink() or digest(destination) != expected:
            raise ValueError('preserving an unverified cached download')
        return
    temporary = destination.with_suffix(destination.suffix + '.partial')
    if temporary.exists():
        raise ValueError('preserving an earlier partial download')
    request = Request(url, headers={'Accept': 'application/octet-stream', 'User-Agent': 'Khor netlab capability probe'})
    size = 0
    with urlopen(request, timeout=120) as source, temporary.open('xb') as output:
        while True:
            block = source.read(1024 * 1024)
            if not block:
                break
            size += len(block)
            if size > maximum:
                raise ValueError('download exceeds the pinned asset limit')
            output.write(block)
    if digest(temporary) != expected:
        raise ValueError('download checksum mismatch; executable not used')
    temporary.chmod(0o444)
    temporary.replace(destination)


def extract_owned(archive, destination):
    destination.mkdir(mode=0o700)
    root = destination.resolve()
    with tarfile.open(archive) as source:
        for member in source:
            name = PurePosixPath(member.name)
            if name.is_absolute() or '..' in name.parts:
                raise ValueError('archive path escapes the owned tool directory')
            target = root.joinpath(*name.parts)
            if not target.resolve().is_relative_to(root):
                raise ValueError('archive path follows an escaping link')
            if member.issym():
                linked = PurePosixPath(member.linkname)
                if linked.is_absolute() or not (target.parent / member.linkname).resolve().is_relative_to(root):
                    raise ValueError('archive link escapes the owned tool directory')
            elif not (member.isfile() or member.isdir()):
                raise ValueError('unsupported archive entry type')
            member.mode &= 0o777
            source.extract(member, path=root)


def config(image):
    return dict(vmType='vz', arch='aarch64', os='Linux', cpus=2, memory='6GiB', disk='30GiB',
                images=[dict(location=str(image), arch='aarch64', digest='sha256:' + IMAGE_SHA)],
                mounts=[], containerd=dict(system=False, user=False),
                ssh=dict(loadDotSSHPubKeys=False, forwardAgent=False, forwardX11=False, forwardX11Trusted=False),
                portForwards=[dict(guestIP='0.0.0.0', proto='any', guestPortRange=[1, 65535], ignore=True)],
                propagateProxyEnv=False, rosetta=dict(enabled=False, binfmt=False),
                networkOpts=dict(hostLoopback=False), vmOpts=dict(vz=dict(diskImageFormat='raw')))


def clean_env(run):
    allowed = ['HOME', 'USER', 'LOGNAME', 'LANG', 'LC_ALL', 'TMPDIR', 'RUNNER_TRACKING_ID']
    result = {name: os.environ[name] for name in allowed if name in os.environ}
    result.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', LIMA_HOME=str(run / 'state'))
    return result


def inherited_lock_fd():
    if digest(GUARD) != GUARD_SHA:
        raise ValueError('installed admission changed; lifecycle assumptions require re-audit')
    expected = LOCK.stat()
    for entry in os.listdir('/dev/fd'):
        try:
            fd = int(entry)
            actual = os.fstat(fd)
        except (ValueError, OSError):
            continue
        if fd > 2 and (actual.st_dev, actual.st_ino) == (expected.st_dev, expected.st_ino):
            return fd
    raise ValueError('the installed admission lock descriptor was not inherited')


def command(run, *args, timeout=30):
    return subprocess.run([str(run / 'lima/bin/limactl'), '--tty=false', *args],
                          env=clean_env(run), capture_output=True, text=True, timeout=timeout)


def instance(run):
    return 'v' + hashlib.sha256(run.name.encode()).hexdigest()[:8]


def status(run):
    result = command(run, 'list', '--json', instance(run), timeout=15)
    if result.returncode:
        return None
    rows = [json.loads(line) for line in result.stdout.splitlines() if line.strip()]
    rows = [row for row in rows if row.get('name') == instance(run)]
    if len(rows) != 1:
        return None
    return rows[0]


def assert_owned(run):
    if run.is_symlink() or run.parent.resolve() != (BASE / 'runs').resolve():
        raise ValueError('instance is not in the owned run root')
    if not re.fullmatch(r'[0-9]+-[0-9]+', run.name):
        raise ValueError('invalid owned run key')
    owner = json.loads((run / 'owner.json').read_text())
    if any(owner.get(key) != value for key, value in OWNER.items()) or owner.get('run_key') != run.name:
        raise ValueError('instance owner mismatch; preserving state')
    return owner


def owned_processes(run, exclude=()):
    rows = subprocess.check_output(['/bin/ps', '-axo', 'pid=,ppid=,pgid=,command='], text=True)
    selected = []
    for line in rows.splitlines():
        fields = line.strip().split(None, 3)
        if len(fields) != 4:
            continue
        pid, parent, group = map(int, fields[:3])
        if pid not in exclude and (str(run) in fields[3] or instance(run) in fields[3]):
            selected.append(dict(pid=pid, parent=parent, group=group, command=fields[3]))
    return selected


def interrupt(_signal, _frame):
    raise KeyboardInterrupt


def supervise(run, phase):
    owner = assert_owned(run)
    report = Path(owner['report_dir'])
    fd = int(os.environ['KHOR_NETLAB_GUARD_FD'])
    if inherited_lock_fd() != fd:
        raise ValueError('supervisor lost its shared admission descriptor')
    result = dict(phase=phase, parent_pid=os.getppid(), supervisor_pid=os.getpid(),
                  group=os.getpgrp(), lease_eof=False, stopped=False)
    foreground = None
    signal.signal(signal.SIGTERM, interrupt)
    signal.signal(signal.SIGINT, interrupt)
    try:
        args = ['start', '--foreground']
        if (run / 'state' / instance(run)).exists():
            args.append(instance(run))
        else:
            args.extend(['--name', instance(run), str(run / 'config.yaml')])
        with (report / ('foreground-' + phase + '.log')).open('w') as output:
            foreground = subprocess.Popen([str(run / 'lima/bin/limactl'), '--tty=false', *args],
                                          env=clean_env(run), stdin=subprocess.DEVNULL,
                                          stdout=output, stderr=subprocess.STDOUT, pass_fds=(fd,))
            result.update(foreground_pid=foreground.pid, foreground_group=os.getpgid(foreground.pid))
            record(report / ('lifecycle-' + phase + '.json'), result)
            if result['foreground_group'] != result['group']:
                raise ValueError('foreground hostagent escaped the admitted process group')
            while foreground.poll() is None:
                if select.select([sys.stdin.buffer], [], [], 0.5)[0]:
                    if os.read(sys.stdin.fileno(), 1) == b'':
                        result['lease_eof'] = True
                        break
            if not result['lease_eof']:
                raise RuntimeError('foreground hostagent exited before its lease ended')
    except BaseException as error:
        result['error'] = type(error).__name__ + ': ' + str(error)
    finally:
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        try:
            stopped = command(run, 'stop', '--force', instance(run), timeout=15)
            result['stop_returncode'] = stopped.returncode
            result['stop_stderr'] = stopped.stderr[-8192:]
        except BaseException as error:
            result['stop_error'] = type(error).__name__ + ': ' + str(error)
        # A failed/timed-out stop CLI must never bypass hostagent termination.
        if foreground is not None:
            try:
                foreground.wait(timeout=10)
            except subprocess.TimeoutExpired:
                foreground.kill()
                foreground.wait(timeout=5)
                result['foreground_forced_kill'] = True
            result['foreground_returncode'] = foreground.returncode
        try:
            observed = status(run)
            result['instance_status'] = observed.get('status') if observed else None
            result['stopped'] = result['instance_status'] == 'Stopped' and (foreground is None or foreground.poll() is not None)
        except BaseException as error:
            result['cleanup_error'] = type(error).__name__ + ': ' + str(error)
        record(report / ('lifecycle-' + phase + '.json'), result)
    return 0 if result['stopped'] and not any(key in result for key in ['error', 'stop_error', 'cleanup_error']) else 1


class Lease:
    def __init__(self, run, phase):
        self.run, self.phase = run, phase
        self.owner = assert_owned(run)
        fd = inherited_lock_fd()
        environment = dict(clean_env(run), KHOR_NETLAB_GUARD_FD=str(fd))
        self.process = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--supervise', str(run), phase],
                                        stdin=subprocess.PIPE, env=environment, pass_fds=(fd,))

    def ready(self):
        deadline = time.monotonic() + 420
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError('foreground supervisor exited before guest readiness')
            observed = status(self.run)
            if observed and observed.get('status') == 'Running':
                try:
                    result = command(self.run, 'shell', '--workdir', '/', instance(self.run),
                                     '/bin/sh', '-c', 'test "$(uname -s)" = Linux', timeout=20)
                    if result.returncode == 0:
                        return
                except subprocess.TimeoutExpired:
                    pass  # Running may precede the first usable SSH session.
            time.sleep(2)
        raise TimeoutError('guest did not become ready inside its startup budget')

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=55)
        report = Path(self.owner['report_dir']) / ('lifecycle-' + self.phase + '.json')
        observed = json.loads(report.read_text())
        if self.process.returncode or not observed.get('stopped'):
            raise RuntimeError('owned VM cleanup failed')
        return observed


def hold(run):
    owner = assert_owned(run)
    lease = Lease(run, 'parent-death')
    try:
        lease.ready()
        record(Path(owner['report_dir']) / 'hold-ready.json', dict(pid=os.getpid()))
        while True:
            time.sleep(1)
    finally:
        lease.close()


def create_run(base, run_key, source, report):
    if not re.fullmatch(r'[0-9]+-[0-9]+', run_key):
        raise ValueError('invalid workflow run key')
    if base.exists():
        if base.is_symlink() or json.loads((base / 'owner.json').read_text()) != OWNER:
            raise ValueError('preserving an unowned netlab root')
    else:
        base.mkdir(mode=0o700)
        record(base / 'owner.json', OWNER)
    runs = base / 'runs'
    runs.mkdir(exist_ok=True)
    if runs.is_symlink():
        raise ValueError('runs root must not be a symlink')
    run = runs / run_key
    if len(str(run / 'state' / instance(run) / 'ssh.sock.1234567890123456').encode()) >= 104:
        raise ValueError('owned instance path exceeds the macOS UNIX socket limit')
    run.mkdir(mode=0o700)
    record(run / 'owner.json', dict(OWNER, run_key=run_key, report_dir=str(report), source_tree=source['tree'], root_pid=os.getpid()))
    return run


def main():
    report = Path(os.environ['REPORT_DIR']).resolve()
    report.mkdir(parents=True, exist_ok=True)
    source = json.loads((Path(os.environ['SOURCE_PACKAGE']) / 'source.json').read_text())
    record(report / 'source.json', source)
    record(report / 'environment.json', dict(task='netlab-vz-probe', system=platform.platform(),
           control_commit=os.environ['GITHUB_SHA'], run_id=os.environ['GITHUB_RUN_ID'], jobs='2', probe_only=True))
    result = dict(schema=1, layer='linux-vz-capabilities', product_tests_executed=False,
                  control_commit=os.environ['GITHUB_SHA'], run_id=os.environ['GITHUB_RUN_ID'],
                  source_tree=source['tree'], vm_booted=False, supervisor_started=False,
                  normal_cleanup=False, parent_death_cleanup=False)
    run, lease, controller = None, None, None
    signal.signal(signal.SIGTERM, interrupt)
    signal.signal(signal.SIGINT, interrupt)
    try:
        if platform.system() != 'Darwin' or platform.machine() not in ('arm64', 'aarch64') or os.geteuid() == 0:
            raise ValueError('this probe requires the admitted non-root Apple Silicon Mac worker')
        fd = inherited_lock_fd()
        run_key = os.environ['GITHUB_RUN_ID'] + '-' + os.environ.get('GITHUB_RUN_ATTEMPT', '1')
        run = create_run(BASE, run_key, source, report)
        cached = BASE / 'downloads'
        cached.mkdir(exist_ok=True)
        if cached.is_symlink():
            raise ValueError('download root must not be a symlink')
        archive, image = cached / (LIMA_SHA + '.tar.gz'), cached / (IMAGE_SHA + '.img')
        download(LIMA_URL, LIMA_SHA, 50317981, archive)
        download(IMAGE_URL, IMAGE_SHA, 2 * 1024**3, image)
        extract_owned(archive, run / 'lima')
        record(run / 'config.yaml', config(image))
        record(report / 'vm-config.json', config(image))
        version = command(run, '--version')
        if version.returncode or not re.search(r'\b2\.3\.0\b', version.stdout):
            raise ValueError('unexpected pinned Lima version')
        result.update(lima_version=version.stdout.strip(), guard_sha256=GUARD_SHA,
                      lima_sha256=LIMA_SHA, image_sha256=IMAGE_SHA,
                      host_free_before=shutil.disk_usage(BASE).free, group=os.getpgrp())
        lease = Lease(run, 'normal')
        result['supervisor_started'] = True
        try:
            lease.ready()
            result['vm_booted'] = True
            completed = command(run, 'shell', '--workdir', '/', instance(run), '/bin/bash', '-lc', GUEST_PROBE, timeout=600)
            (report / 'guest-probe.log').write_text(completed.stdout + '\n' + completed.stderr)
            if completed.returncode:
                raise RuntimeError('guest capability probe failed')
            caps = [json.loads(line.removeprefix('KHOR_NETLAB_CAPS ')) for line in completed.stdout.splitlines() if line.startswith('KHOR_NETLAB_CAPS ')]
            if len(caps) != 1 or caps[0].get('system') != 'Linux':
                raise ValueError('guest capability receipt missing')
            result['guest'] = caps[0]
        finally:
            normal = lease.close()
            lease = None
            result['normal_cleanup'] = normal['stopped']
        controller = subprocess.Popen([sys.executable, str(Path(__file__).resolve()), '--hold', str(run)],
                                      env=dict(clean_env(run), KHOR_NETLAB_GUARD_FD=str(fd)), pass_fds=(fd,))
        deadline = time.monotonic() + 420
        while not (report / 'hold-ready.json').exists():
            if controller.poll() is not None or time.monotonic() >= deadline:
                raise RuntimeError('parent-death fixture did not reach guest readiness')
            time.sleep(1)
        controller.kill()
        controller.wait(timeout=10)
        result['killed_controller_returncode'] = controller.returncode
        lifecycle = report / 'lifecycle-parent-death.json'
        deadline = time.monotonic() + 45
        while time.monotonic() < deadline:
            state = json.loads(lifecycle.read_text()) if lifecycle.exists() else {}
            if state.get('stopped') and state.get('lease_eof'):
                result['parent_death_cleanup'] = True
                break
            time.sleep(1)
        if not result['parent_death_cleanup']:
            raise RuntimeError('VM survived its killed controller lease')
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            result['remaining_processes'] = owned_processes(run, exclude=(os.getpid(),))
            if not result['remaining_processes']:
                break
            time.sleep(1)
        if result['remaining_processes']:
            raise RuntimeError('owned VM helper processes remain after cleanup')
        result['success'] = True
    except BaseException as error:
        result.update(success=False, error=type(error).__name__ + ': ' + str(error))
    finally:
        if controller is not None and controller.poll() is None:
            controller.terminate()
            try:
                controller.wait(timeout=35)
            except subprocess.TimeoutExpired:
                controller.kill()
                controller.wait()
        if lease is not None:
            try:
                lease.close()
            except BaseException as error:
                result['lease_cleanup_error'] = str(error)
        if run is not None:
            try:
                vm_dir = run / 'state' / instance(run)
                for log in vm_dir.glob('*.log'):
                    if log.is_file() and not log.is_symlink():
                        with log.open('rb') as original:
                            original.seek(max(0, log.stat().st_size - 2 * 1024**2))
                            (report / ('lima-' + log.name)).write_bytes(original.read())
                observed = status(run) if result['supervisor_started'] else None
                stopped = not observed or observed.get('status') == 'Stopped'
                remaining = owned_processes(run, exclude=(os.getpid(),))
                result.update(final_instance_status=observed.get('status') if observed else None,
                              remaining_processes=remaining, host_free_after=shutil.disk_usage(BASE).free)
                if not stopped or remaining:
                    result['success'] = False
                    result['cleanup_incomplete'] = True
                if stopped and not remaining:
                    for name in ['state', 'lima']:
                        path = run / name
                        if path.exists() and not path.is_symlink():
                            shutil.rmtree(path)
                    result['ephemeral_state_removed'] = True
            except BaseException as error:
                result.update(success=False, cleanup_error=str(error))
        record(report / 'vm-capabilities.json', result)
        print(json.dumps(result, indent=2))
    return 0 if result.get('success') else 1


if __name__ == '__main__':
    if len(sys.argv) == 4 and sys.argv[1] == '--supervise':
        raise SystemExit(supervise(Path(sys.argv[2]), sys.argv[3]))
    if len(sys.argv) == 3 and sys.argv[1] == '--hold':
        hold(Path(sys.argv[2]))
    elif len(sys.argv) == 1:
        raise SystemExit(main())
    else:
        raise SystemExit('invalid internal capability controller arguments')
