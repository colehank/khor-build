"""Inspect the owned builder cache; prune only inactive superseded test programs."""
import argparse
import ast
from collections import defaultdict
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import shutil
import time
import tempfile

ROOT = Path('/Users/z13/khor-builder/cache/target')
CACHE_BUDGET_GIB = 100
CACHE_MAINTENANCE_GIB = 90
MINIMUM_FREE_GIB = 40
MAGIC = {b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\x7fELF'}

def configure_guard(path):
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 64 * 1024:
        raise RuntimeError('unexpected installed admission implementation')
    original = path.read_text()
    replacements = [
        ('START_RESERVE = 35 * 2**30', f'START_RESERVE = {MINIMUM_FREE_GIB + 10} * 2**30'),
        ('STOP_RESERVE = 20 * 2**30', f'STOP_RESERVE = {MINIMUM_FREE_GIB} * 2**30'),
        ('TARGET_LIMIT = 60 * 2**30', f'TARGET_LIMIT = {CACHE_BUDGET_GIB} * 2**30'),
        ('less than 35 GiB free', f'less than {MINIMUM_FREE_GIB + 10} GiB free'),
        ('target cache reached 60 GiB', f'target cache reached {CACHE_BUDGET_GIB} GiB'),
        ('free space dropped below 20 GiB', f'free space dropped below {MINIMUM_FREE_GIB} GiB'),
    ]
    if all(new in original and old not in original for old, new in replacements):
        return {'changed': False, 'sha256': hashlib.sha256(original.encode()).hexdigest()}
    if not all(original.count(old) == 1 and new not in original for old, new in replacements):
        raise RuntimeError('installed admission changed; preserve it for review before changing the policy')
    updated = original
    for old, new in replacements:
        updated = updated.replace(old, new)
    ast.parse(updated)
    before = hashlib.sha256(original.encode()).hexdigest()
    backup = path.with_name('build_guard.before-policy-' + before[:12] + '.py')
    if backup.is_symlink() or (backup.exists() and backup.read_text() != original):
        raise RuntimeError('preserving an unexpected admission-policy backup')
    if not backup.exists():
        with backup.open('x') as stream:
            stream.write(original)
        backup.chmod(0o600)
    with tempfile.NamedTemporaryFile(mode='w', dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(updated)
        stream.flush(); os.fsync(stream.fileno())
    try:
        temporary.chmod(path.stat().st_mode & 0o777)
        temporary.replace(path)
    finally:
        temporary.unlink(missing_ok=True)
    return {'changed': True, 'before_sha256': before, 'sha256': hashlib.sha256(updated.encode()).hexdigest(), 'backup': str(backup)}

def allocated(path):
    return int(subprocess.check_output(['/usr/bin/du','-sk',str(path)],text=True).split()[0])*1024

def free():
    stat = os.statvfs(ROOT)
    return stat.f_bavail*stat.f_frsize

def incremental_candidates(root, now):
    groups = defaultdict(list)
    for path in root.iterdir():
        match = re.fullmatch(r'([A-Za-z0-9_]+)-[a-z0-9]{13}', path.name)
        if not match or path.is_symlink() or not path.is_dir():
            continue
        stat = path.stat()
        groups[match[1]].append((stat.st_mtime_ns, path, stat.st_dev, stat.st_ino))
    candidates = []
    for entries in groups.values():
        entries.sort(reverse=True)
        candidates.extend(row for row in entries[8:] if now - row[0] / 1e9 >= 2 * 3600)
    return candidates


def collect_incremental(root, candidates):
    if not candidates:
        return []
    if not shutil.rmtree.avoids_symlink_attacks:
        raise RuntimeError('incremental collection requires descriptor-safe directory removal')
    opened = subprocess.run(['/usr/sbin/lsof', '-n', '-P', '-F', 'n', '+D', str(root)],
                            capture_output=True, text=True, timeout=30)
    if opened.returncode not in (0, 1) or opened.stderr.strip():
        raise RuntimeError('cannot establish incremental cache occupancy')
    active = [Path(line[1:]) for line in opened.stdout.splitlines() if line.startswith('n')]
    removed = []
    for modified, path, device, inode in candidates:
        if any(file == path or path in file.parents for file in active):
            continue
        stat = path.lstat()
        if path.is_symlink() or not path.is_dir() or (stat.st_dev, stat.st_ino, stat.st_mtime_ns) != (device, inode, modified):
            continue
        size = allocated(path)
        shutil.rmtree(path)
        removed.append({'directory': str(path), 'allocated_bytes': size})
    return removed


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['inventory', 'prune-test-programs', 'ensure-budget'])
    args = parser.parse_args()
    assert ROOT.is_dir() and not ROOT.is_symlink() and ROOT.resolve() == ROOT
    report = Path(os.environ['REPORT_DIR']); report.mkdir(parents=True, exist_ok=True)
    admission = None
    if args.mode != 'inventory':
        lock = (ROOT.parent/'build.lock').open('a')
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise RuntimeError('another build is active; preserve its caches and policy')
        if args.mode == 'ensure-budget':
            admission = configure_guard(ROOT.parents[1]/'bin/build_guard.py')
    before = allocated(ROOT)
    summary = {'mode':args.mode,'root':str(ROOT),'target_allocated_before':before,'free_before':free(), 'directories':{}}
    summary['policy'] = {'cache_budget_gib': CACHE_BUDGET_GIB, 'maintenance_gib': CACHE_MAINTENANCE_GIB, 'minimum_free_gib': MINIMUM_FREE_GIB}
    summary['admission'] = admission
    if args.mode == 'ensure-budget' and before < CACHE_MAINTENANCE_GIB * 1024**3 and free() >= (MINIMUM_FREE_GIB + 10) * 1024**3:
        summary.update(target_allocated_after=before, free_after=summary['free_before'], removed=[], reason='below maintenance threshold')
        (report/'storage.json').write_text(json.dumps(summary,indent=2)+'\n')
        print('Cache below maintenance threshold; preserving all build artifacts')
        raise SystemExit(0)
    for path in sorted(ROOT.iterdir()):
        if path.is_dir() and not path.is_symlink(): summary['directories'][path.name]=allocated(path)
    if args.mode == 'inventory':
        guard = Path('/Users/z13/khor-builder/bin/run-build')
        if guard.stat().st_size > 64 * 1024:
            raise RuntimeError('unexpected builder admission script size')
        (report/'run-build.txt').write_bytes(guard.read_bytes())
        admission = guard.with_name('build_guard.py')
        if admission.stat().st_size > 64 * 1024:
            raise RuntimeError('unexpected builder admission implementation size')
        (report/'build_guard.py.txt').write_bytes(admission.read_bytes())
        summary['directory_depth_two'] = subprocess.check_output(['/usr/bin/du','-k','-d','2',str(ROOT)],text=True).splitlines()
        summary['largest_regular_files'] = sorted(
            ({'file':str(path),'bytes':path.stat().st_size,'links':path.stat().st_nlink}
             for path in (ROOT/'debug/deps').iterdir() if path.is_file() and not path.is_symlink()),
            key=lambda item:item['bytes'], reverse=True)[:30]
        summary['incremental_directories'] = [
            {'directory':str(path),'bytes':allocated(path),'modified':path.stat().st_mtime}
            for path in sorted((ROOT/'debug/incremental').iterdir()) if path.is_dir() and not path.is_symlink()]
    deps=ROOT/'debug/deps'
    assert deps.is_dir() and not deps.is_symlink()
    groups=defaultdict(list)
    for path in deps.iterdir():
        match=re.fullmatch(r'([a-zA-Z0-9_]+)-[0-9a-f]{16}',path.name)
        if not match or path.is_symlink() or not path.is_file():continue
        if match[1].startswith('lib'):continue
        stat=path.stat()
        if stat.st_nlink != 1:continue
        with path.open('rb') as file:
            if file.read(4) not in MAGIC:continue
        groups[match[1]].append((stat.st_mtime_ns,path,stat.st_size,stat.st_ino))
    candidates=[]
    for name, entries in groups.items():
        entries.sort(reverse=True)
        candidates.extend(entries[2:])
    summary['candidate_count']=len(candidates)
    summary['candidate_logical_bytes']=sum(row[2] for row in candidates)
    summary['largest_candidates']=[{'file':str(row[1]),'bytes':row[2]} for row in sorted(candidates,key=lambda row:row[2],reverse=True)[:40]]
    summary['groups']={name:len(entries) for name,entries in sorted(groups.items()) if len(entries)>2}
    summary['removed']=[];summary['preserved_active']=[]
    if args.mode in ('prune-test-programs', 'ensure-budget'):
        for offset in range(0,len(candidates),20):
            batch=candidates[offset:offset+20]
            opened=subprocess.run(['/usr/sbin/lsof','-n','-P','-F','n','--',*[str(row[1]) for row in batch]],text=True,capture_output=True)
            if opened.returncode not in (0,1) or opened.stderr.strip(): raise RuntimeError('cannot establish active cache-file users: '+opened.stderr[:400])
            active={line[1:] for line in opened.stdout.splitlines() if line.startswith('n')}
            for _,path,size,inode in batch:
                if str(path) in active:
                    summary['preserved_active'].append(str(path));continue
                stat=path.stat()
                if stat.st_ino!=inode or stat.st_nlink!=1:continue
                with path.open('rb') as file:
                    if file.read(4) not in MAGIC:continue
                path.unlink();summary['removed'].append({'file':str(path),'logical_bytes':size})
    incremental = ROOT/'debug/incremental'
    if incremental.is_dir() and not incremental.is_symlink():
        candidates = incremental_candidates(incremental, time.time())
        summary['incremental_candidate_count'] = len(candidates)
        summary['incremental_retention'] = 'newest 8 variants per crate; preserve changes within 2 hours and all open directories'
        if args.mode == 'ensure-budget':
            summary['incremental_removed'] = collect_incremental(incremental, candidates)
    summary['target_allocated_after']=allocated(ROOT)
    summary['free_after']=free()
    (report/'storage.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps({key:summary[key] for key in ['mode','target_allocated_before','target_allocated_after','free_before','free_after','candidate_count','candidate_logical_bytes']}))
    print('removed',len(summary['removed']),'preserved_active',len(summary['preserved_active']))


if __name__ == '__main__':
    main()
