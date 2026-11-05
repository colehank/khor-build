"""Inspect the owned builder cache; prune only inactive superseded test programs."""
import argparse
from collections import defaultdict
import json
import os
from pathlib import Path
import re
import subprocess

ROOT = Path('/Users/z13/khor-builder/cache/target')
MAGIC = {b'\xcf\xfa\xed\xfe', b'\xfe\xed\xfa\xcf', b'\x7fELF'}
parser = argparse.ArgumentParser()
parser.add_argument('mode', choices=['inventory', 'prune-test-programs'])
args = parser.parse_args()
report = Path(os.environ['REPORT_DIR']); report.mkdir(parents=True, exist_ok=True)
assert ROOT.is_dir() and not ROOT.is_symlink() and ROOT.resolve() == ROOT

def allocated(path):
    return int(subprocess.check_output(['/usr/bin/du','-sk',str(path)],text=True).split()[0])*1024

def free():
    stat = os.statvfs(ROOT)
    return stat.f_bavail*stat.f_frsize

before = allocated(ROOT)
summary = {'mode':args.mode,'root':str(ROOT),'target_allocated_before':before,'free_before':free(), 'directories':{}}
for path in sorted(ROOT.iterdir()):
    if path.is_dir() and not path.is_symlink(): summary['directories'][path.name]=allocated(path)
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
if args.mode=='prune-test-programs':
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
summary['target_allocated_after']=allocated(ROOT)
summary['free_after']=free()
(report/'storage.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({key:summary[key] for key in ['mode','target_allocated_before','target_allocated_after','free_before','free_after','candidate_count','candidate_logical_bytes']}))
print('removed',len(summary['removed']),'preserved_active',len(summary['preserved_active']))
