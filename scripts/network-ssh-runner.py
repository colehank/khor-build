"""Trusted, per-user Linux netlab job; source and caches stay on local /tmp."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import subprocess
import sys
import tarfile
import time
from urllib.request import urlopen


OWNER = {'schema': 1, 'owner': 'khor feat/net netlab'}
TASKS = {'net-unit', 'net-node', 'net-node-routes', 'net-vendor', 'net-wan-build', 'netlab-workload-policy', 'netlab-workload-wires', 'netlab-workload-observer', 'netlab-workload-high-calibrate', 'netlab-workload-calibrate', 'netlab-workload', 'netlab-measure-check', 'netlab-cpu-check', 'netlab-mobility-calibrate', 'netlab-mobility-node', 'netlab-mobility-roam', 'netlab-peer-mobility', 'netlab-peer-mobility-samples', 'netlab-nat-rebind', 'netlab-nat-rebind-lifecycle', 'netlab-hop-reference', 'netlab-hop-reference-lifecycle', 'netlab-hop-ownership', 'netlab-hop-lifecycle', 'netlab-smoke', 'netlab-recovery', 'netlab-dynamic', 'netlab-compare',
         'netlab-hop', 'netlab-stability', 'netlab-stalls', 'netlab-diagnostic',
         'netlab-hop-diagnostic', 'netlab'}
RUST_COMPONENTS = {
    'rustc': '8426a3d170a5879f5682f5fbdd024a1779b3951e7baba685af2d6dc32a6dfc15',
    'cargo': 'e74edd2cf7d0f1f1383b4f00eb90c843750bc489e2ccf7214e6476678a907425',
    'rust-std': '047ea7098803d3500fa1072e9cee5392697e21525559e4458128a2bf874aa382',
    'rustfmt': 'f1b2a7301513ffdd95ebf22ebbdd932e4d17fc806f748d93924740d0297b1396',
}


def record(path, value):
    temporary = path.with_suffix(path.suffix + '.new')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)


def sha256(path):
    h = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def extract(archive, destination):
    destination.mkdir()
    root = str(destination.resolve())
    with tarfile.open(archive) as source:
        for member in source:
            target = (destination / member.name).resolve()
            if os.path.commonpath([root, str(target)]) != root:
                raise ValueError('tool archive escapes its owned directory')
            if member.issym() or member.islnk():
                parent = target.parent if member.issym() else destination
                if os.path.commonpath([root, str((parent / member.linkname).resolve())]) != root:
                    raise ValueError('tool archive link escapes its owned directory')
            elif not (member.isfile() or member.isdir()):
                raise ValueError('unsupported tool archive entry')
            member.mode &= 0o777
            source.extract(member, path=str(destination))


def environment(root, run, job):
    # Keep the real HOME; override Cargo's writable locations and the tools
    # netlab.sh prepends, so neither rustup nor a cache writes to the NFS home.
    env = {k: os.environ[k] for k in ['HOME', 'USER', 'LOGNAME', 'LANG'] if k in os.environ}
    env.update(PATH=str(root / 'tools/bin') + ':/usr/bin:/bin:/usr/sbin:/sbin',
               GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1',
               GIT_AUTHOR_NAME='Khor netlab snapshot', GIT_AUTHOR_EMAIL='snapshot@localhost',
               GIT_COMMITTER_NAME='Khor netlab snapshot', GIT_COMMITTER_EMAIL='snapshot@localhost',
               CARGO_HOME=str(root / 'cache/cargo'), CARGO_TARGET_DIR=str(root / 'cache/target'),
               CARGO_BUILD_JOBS='16', CARGO_INCREMENTAL='0', CARGO_TERM_COLOR='never',
               CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0', CARGO_HTTP_MULTIPLEXING='false',
               RUSTC=str(root / 'tools/rust195/bin/rustc'), RUSTFMT=str(root / 'tools/rust195/bin/rustfmt'),
               RUSTDOC=str(root / 'tools/rust195/bin/rustdoc'),
               DEV_TASK=job['task'], GITHUB_SHA=job['control_commit'], GITHUB_RUN_ID=job['run_id'],
               RUNNER_TEMP=str(run / 'work'), SOURCE_PACKAGE=str(run / 'package'), REPORT_DIR=str(run / 'report'),
               NETLAB_TOOLS=str(root / 'tools'))
    return env


def toolchain(root, env, report):
    tools = root / 'tools'
    prefix = tools / 'rust195'
    marker = prefix / 'khor-components.json'
    tools.mkdir(exist_ok=True)
    if prefix.exists():
        if prefix.is_symlink() or not marker.is_file() or json.loads(marker.read_text()) != RUST_COMPONENTS:
            raise ValueError('preserving an unverified toolchain directory')
    else:
        with (report / 'toolchain.log').open('w') as log:
            for name, expected in RUST_COMPONENTS.items():
                stem = name + '-1.95.0-x86_64-unknown-linux-gnu'
                archive = tools / (stem + '.tar.xz')
                with urlopen('https://static.rust-lang.org/dist/2026-04-16/' + archive.name, timeout=120) as response, archive.open('xb') as output:
                    count = 0
                    while True:
                        block = response.read(1024 * 1024)
                        if not block:
                            break
                        count += len(block)
                        if count > 256 * 1024**2:
                            raise ValueError('tool download exceeds fixed component limit')
                        output.write(block)
                if sha256(archive) != expected:
                    raise ValueError('tool checksum mismatch; executable not used')
                directory = tools / (name + '-install')
                extract(archive, directory)
                subprocess.run(['/bin/sh', str(directory / stem / 'install.sh'),
                                '--prefix=' + str(prefix), '--disable-ldconfig'],
                               check=True, env=env, stdout=log, stderr=subprocess.STDOUT)
                shutil.rmtree(str(directory))
                archive.unlink()
        record(marker, RUST_COMPONENTS)
    binary = tools / 'bin'
    binary.mkdir(exist_ok=True)
    for original in (prefix / 'bin').iterdir():
        link = binary / original.name
        if not link.exists():
            link.symlink_to(original)
    for name in ['nft', 'conntrack']:
        link = binary / name
        if not link.exists():
            original = Path.home() / 'khor-netlab/tools/bin' / name
            if not original.is_file():
                raise ValueError('existing unprivileged netlab tool is missing: ' + name)
            link.symlink_to(original)
    versions = {name: subprocess.check_output([str(binary / name), '--version'], env=env, universal_newlines=True).strip()
                for name in ['rustc', 'cargo', 'rustfmt', 'nft', 'conntrack']}
    if not versions['rustc'].startswith('rustc 1.95.0 ') or not versions['cargo'].startswith('cargo 1.95.0 '):
        raise ValueError('unexpected pinned Linux toolchain')
    record(report / 'toolchain.json', dict(versions=versions, components=RUST_COMPONENTS))


def materialize(run, job, env):
    source = dict(job['source'])
    bundle = run / 'input/source.bundle'
    if bundle.stat().st_size > 64 * 1024**2 or sha256(bundle) != job['bundle_sha256']:
        raise ValueError('source bundle checksum/size mismatch')
    package = run / 'package'
    package.mkdir()
    bare = package / 'source.git'
    subprocess.run(['git', 'clone', '--bare', str(bundle), str(bare)], check=True, env=env)
    def git(*args):
        return subprocess.check_output(['git', '--git-dir=' + str(bare), *args], env=env, universal_newlines=True).strip()
    snapshot = source['snapshot_commit']
    if git('rev-parse', 'HEAD') != snapshot or git('ls-tree', '--name-only', snapshot) != 'source':
        raise ValueError('snapshot container identity mismatch')
    if git('rev-list', '--parents', '-n', '1', snapshot) != snapshot:
        raise ValueError('snapshot must be parentless')
    message = git('show', '-s', '--format=%B', snapshot)
    metadata = json.loads(message.split('\n\n', 1)[1])
    if not message.startswith('Khor developer snapshot\n\n') or metadata['tree'] != source['tree'] or metadata['base_commit'] != source['base_commit']:
        raise ValueError('snapshot metadata mismatch')
    if git('rev-parse', snapshot + ':source') != source['tree']:
        raise ValueError('source tree mismatch')
    source['build_commit'] = git('commit-tree', source['tree'], '-m', 'Khor netlab source ' + snapshot)
    git('update-ref', 'refs/heads/source', source['build_commit'])
    git('symbolic-ref', 'HEAD', 'refs/heads/source')
    git('remote', 'remove', 'origin')
    record(package / 'source.json', source)
    record(run / 'report/source.json', source)


def cached_source(root, run, env):
    """Keep unchanged source paths/mtimes under the exclusive build slot.

    Never reset, clean, or remove an existing checkout. Unknown ownership,
    symlinks and tracked edits fail before checkout or product execution.
    """
    cache = root / 'cache'
    source = cache / 'source'
    marker = cache / 'source-owner.json'
    owner = dict(OWNER, uid=os.getuid(), purpose='stable source cache v1')
    if cache.is_symlink() or cache.resolve() != cache or source.is_symlink() or marker.is_symlink():
        raise ValueError('preserving an unsafe source cache path')
    reused = source.exists()
    if reused != marker.exists() or (marker.exists() and json.loads(marker.read_text()) != owner):
        raise ValueError('preserving an unowned source cache')
    if json.loads((root / 'owner.json').read_text()) != dict(OWNER, uid=os.getuid()):
        raise ValueError('unowned executor root')
    expected = json.loads((run / 'package/source.json').read_text())
    package = run / 'package/source.git'
    started = time.monotonic()
    def git(*args):
        return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', '-C', str(source), *args],
                                       env=env, universal_newlines=True).strip()
    previous = None
    if reused:
        if not (source / '.git').is_dir() or (source / '.git').is_symlink():
            raise ValueError('preserving a non-standalone source cache')
        if git('rev-parse', '--show-toplevel') != str(source) or git('status', '--porcelain', '--untracked-files=no'):
            raise ValueError('preserving a modified source cache')
        previous = git('rev-parse', 'HEAD')
        git('fetch', '--no-tags', str(package), expected['build_commit'])
    else:
        subprocess.run(['git', 'clone', '--no-hardlinks', '--no-checkout', str(package), str(source)],
                       check=True, env=env)
        git('remote', 'remove', 'origin')
    git('checkout', '--detach', expected['build_commit'])
    if git('rev-parse', 'HEAD^{tree}') != expected['tree'] or git('status', '--porcelain', '--untracked-files=no'):
        raise ValueError('cached source identity mismatch')
    if not reused:
        record(marker, owner)
    record(run / 'report/source-cache.json', dict(schema=1, path=str(source), reused=reused,
           previous_commit=previous, build_commit=expected['build_commit'], tree=expected['tree'],
           tracked_clean=True, elapsed_seconds=time.monotonic() - started))
    env['KHOR_NETWORK_SOURCE_DIR'] = str(source)


def main():
    run = Path(__file__).resolve().parent.parent
    root = run.parent.parent
    if platform.system() != 'Linux' or platform.machine() != 'x86_64' or os.geteuid() == 0:
        raise ValueError('this executor requires the authorized non-root x86_64 Linux host')
    if str(root) != '/tmp/khor-netlab-feat-net-' + str(os.getuid()) or root.is_symlink():
        raise ValueError('unexpected per-user local executor root')
    if json.loads((root / 'owner.json').read_text()) != dict(OWNER, uid=os.getuid()):
        raise ValueError('unowned executor root')
    job = json.loads((run / 'input/job.json').read_text())
    if not re.fullmatch(r'turing-[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}', run.name) or job['run_id'] != run.name or job['task'] not in TASKS:
        raise ValueError('unexpected task or run identity')
    for name, expected in job['control_files'].items():
        if name not in ['network.sh', 'network-ssh-runner.py'] or sha256(run / 'input' / name) != expected:
            raise ValueError('trusted driver identity mismatch')
    report = run / 'report'
    report.mkdir()
    result = dict(schema=1, run_id=run.name, task=job['task'], source_tree=job['source']['tree'],
                  control_commit=job['control_commit'], executor='turing-ssh',
                  status='running', phase='admission', build_driver_started=False, pid=os.getpid())
    lock = (root / 'build.lock').open('a')
    child = None
    started = time.monotonic()
    def interrupted(_signum, _frame):
        raise KeyboardInterrupt('executor interrupted')
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    try:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        record(report / 'execution.json', result)
        if shutil.disk_usage(root).free < 50 * 1024**3:
            raise ValueError('less than 50 GiB free on executor local disk')
        env = environment(root, run, job)
        for path in [root / 'cache/cargo', root / 'cache/target', run / 'work']:
            path.mkdir(parents=True, exist_ok=True)
        result['phase'] = 'materialize'
        materialize(run, job, env)
        cached_source(root, run, env)
        result['phase'] = 'toolchain'
        record(report / 'execution.json', result)
        toolchain(root, env, report)
        with (report / 'namespace.log').open('w') as log:
            subprocess.run(['unshare', '-Urn', 'sh', '-ec', 'ip link add probe0 type veth peer name probe1; ip link set probe0 up; tc qdisc add dev probe0 root netem delay 1ms; nft add table inet probe; nft list ruleset; conntrack -L'],
                           check=True, env=env, stdout=log, stderr=subprocess.STDOUT)
        record(report / 'host.json', dict(system=platform.platform(), cpus=os.cpu_count(), load=os.getloadavg(),
               meminfo=Path('/proc/meminfo').read_text(), disk_free_bytes=shutil.disk_usage(root).free,
               jobs=16, measurement_threads='as frozen by task profile', writable_cache=str(root / 'cache')))
        result.update(phase='network-driver', build_driver_started=True)
        record(report / 'execution.json', result)
        with (report / 'task.log').open('w') as output:
            child = subprocess.Popen(['/bin/bash', str(run / 'input/network.sh')], env=env,
                                     stdin=subprocess.DEVNULL, stdout=output, stderr=subprocess.STDOUT,
                                     start_new_session=True, pass_fds=(lock.fileno(),))
            result['child_pid'] = child.pid
            record(report / 'execution.json', result)
            deadline = time.monotonic() + 5400
            while child.poll() is None:
                if time.monotonic() >= deadline:
                    raise TimeoutError('network driver exceeded 90 minute job budget')
                if shutil.disk_usage(root).free < 40 * 1024**3:
                    raise OSError('executor local disk below 40 GiB stop reserve')
                time.sleep(2)
            result['returncode'] = child.returncode
        result['phase'] = 'finished'
    except BaseException as error:
        result.update(returncode=1, error=type(error).__name__ + ': ' + str(error))
    finally:
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        signal.signal(signal.SIGINT, signal.SIG_IGN)
        if child is not None and child.poll() is None:
            os.killpg(child.pid, signal.SIGTERM)
            try:
                child.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                child.wait()
        result.update(status='completed', elapsed_seconds=time.monotonic() - started,
                      load_after=os.getloadavg(), disk_free_after=shutil.disk_usage(root).free)
        record(report / 'execution.json', result)
        print(json.dumps(result, indent=2), flush=True)
        lock.close()
    return result['returncode']


if __name__ == '__main__':
    raise SystemExit(main())
