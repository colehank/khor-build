"""Collect one completed Turing job without following report links or merging runs."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shlex
import shutil
import subprocess
import tarfile


OPTIONS = ['-o', 'BatchMode=yes', '-o', 'ForwardAgent=no',
           '-o', 'StrictHostKeyChecking=yes', '-o', 'ConnectTimeout=10']


def extract_report(archive, output):
    output.mkdir()
    expanded = 0
    with tarfile.open(archive) as source:
        for member in source:
            name = PurePosixPath(member.name)
            if name.is_absolute() or '..' in name.parts or not (member.isfile() or member.isdir()):
                raise ValueError('report archive contains an escaping path or link')
            target = output.joinpath(*name.parts)
            if os.path.commonpath([str(output.resolve()), str(target.resolve())]) != str(output.resolve()):
                raise ValueError('report archive escapes output')
            expanded += member.size
            if expanded > 1024**3:
                raise ValueError('report exceeds expanded evidence limit')
            if member.isdir():
                target.mkdir(parents=True, exist_ok=True)
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                with source.extractfile(member) as original, target.open('xb') as destination:
                    shutil.copyfileobj(original, destination)


def verify(receipt, output, observed):
    source = json.loads((output / 'source.json').read_text())
    for key in ['base_commit', 'tree', 'snapshot_commit']:
        if source.get(key) != receipt['source'][key]:
            raise ValueError('collected source does not match receipt: ' + key)
    execution = json.loads((output / 'execution.json').read_text())
    if (execution['run_id'] != receipt['run_id'] or execution['task'] != receipt['task'] or execution['source_tree'] != source['tree'] or
            execution['control_commit'] != receipt['control_commit'] or execution['status'] != 'completed'):
        raise ValueError('collected execution identity/terminal state mismatch')
    if observed['control_files'] != receipt['control_files']:
        raise ValueError('remote trusted control bytes do not match submitted receipt')
    if observed['bundle_sha256'] != receipt['bundle_sha256']:
        raise ValueError('remote source bundle does not match submitted receipt')
    env = output / 'environment.json'
    if env.exists():
        data = json.loads(env.read_text())
        if data['task'] != receipt['task'] or data['control_commit'] != receipt['control_commit'] or data['run_id'] != receipt['run_id']:
            raise ValueError('collected build environment identity mismatch')
    return execution


def collect(receipt_path):
    receipt_path = Path(receipt_path).resolve()
    receipt = json.loads(receipt_path.read_text())
    if not re.fullmatch(r'turing-[0-9]{8}T[0-9]{6}Z-[a-f0-9]{8}', receipt['run_id']) or receipt['host'] != 'turing':
        raise ValueError('receipt must name the authorized Turing run')
    remote = '/tmp/khor-netlab-feat-net-1110/runs/' + receipt['run_id']
    if receipt['remote_run'] != remote:
        raise ValueError('unexpected Turing job directory')
    inspect = '''from pathlib import Path
import json,hashlib,sys,subprocess
p=Path(sys.argv[1]);d=json.loads((p/'report/execution.json').read_text())
group=d.get('child_pid');rows=subprocess.check_output(['ps','-eo','pid=,pgid=,stat=,comm='],universal_newlines=True).splitlines()
live=[]
for row in rows:
 fields=row.split(None,3)
 if len(fields)==4 and group and int(fields[1])==group and not fields[2].startswith('Z'):live.append(row.strip())
print(json.dumps(dict(execution=d,remaining_original_group=live,control_files={name:hashlib.sha256((p/'input'/name).read_bytes()).hexdigest() for name in ['network.sh','network-ssh-runner.py']},bundle_sha256=hashlib.sha256((p/'input/source.bundle').read_bytes()).hexdigest())))
'''
    observed = json.loads(subprocess.check_output(['ssh', *OPTIONS, 'turing',
                          shlex.join(['python3', '-c', inspect, remote])], text=True, timeout=30))
    if observed['execution']['status'] != 'completed':
        raise ValueError('run is still active; poll the same job before collecting')
    if observed['remaining_original_group']:
        raise ValueError('original job process group still contains live processes')
    output = receipt_path.parent / 'report'
    if output.exists():
        raise ValueError('preserving already collected report; do not merge evidence')
    archive = receipt_path.parent / 'report.tar.gz'
    with archive.open('xb') as destination:
        subprocess.run(['ssh', *OPTIONS, 'turing',
                        shlex.join(['tar', '-C', remote + '/report', '-czf', '-', '.'])],
                       stdout=destination, check=True, timeout=120)
    if archive.stat().st_size > 512 * 1024**2:
        raise ValueError('compressed report exceeds evidence limit')
    extract_report(archive, output)
    (receipt_path.parent / 'collection.json').write_text(json.dumps(observed, indent=2) + '\n')
    execution = verify(receipt, output, observed)
    (receipt_path.parent / 'report-sha256.json').write_text(json.dumps({str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
        for p in output.rglob('*') if p.is_file()}, indent=2) + '\n')
    print(json.dumps(dict(run_id=receipt['run_id'], source_tree=receipt['source']['tree'],
                         returncode=execution['returncode'], evidence=str(output)), indent=2))
    return execution['returncode']


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('receipt')
    raise SystemExit(collect(parser.parse_args().receipt))
