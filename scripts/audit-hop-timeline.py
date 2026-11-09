"""Run the verbatim pure hop reducer on Turing; format copies, never local Rust."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
from datetime import datetime, timezone
import uuid


FILES = ['crates/cli/tests/netlab/hop.rs', 'crates/node/src/link/dial_budget/wire.rs', 'rustfmt.toml']
OPTIONS = ['-o', 'BatchMode=yes', '-o', 'ForwardAgent=no', '-o', 'StrictHostKeyChecking=yes', '-o', 'ConnectTimeout=10']
REMOTE = r'''
from pathlib import Path
import fcntl,hashlib,json,os,subprocess,sys
job=json.load(sys.stdin);root=Path('/tmp/khor-netlab-feat-net-1110')
assert os.getuid()==1110 and json.loads((root/'owner.json').read_text())=={'schema':1,'owner':'khor feat/net netlab','uid':1110}
lock=(root/'build.lock').open('a');fcntl.flock(lock.fileno(),fcntl.LOCK_EX|fcntl.LOCK_NB)
parent=root/'fixture-audits';parent.mkdir(exist_ok=True);run=parent/job['run_id'];run.mkdir(mode=0o700)
for name,text in job['files'].items():
 assert name in ['crates/cli/tests/netlab/hop.rs','crates/node/src/link/dial_budget/wire.rs','rustfmt.toml']
 assert hashlib.sha256(text.encode()).hexdigest()==job['file_sha256'][name]
 p=run/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text)
hop=job['files']['crates/cli/tests/netlab/hop.rs'];fragment=hop.split('// TIMELINE_AUDIT_BEGIN',1)[1].split('// TIMELINE_AUDIT_END',1)[0]
test=run/'timeline.rs';test.write_text(fragment)
tools=root/'tools/rust195/bin';env={k:os.environ[k] for k in ['HOME','USER','LOGNAME','LANG'] if k in os.environ};env['PATH']='/usr/bin:/bin:/usr/sbin:/sbin'
version=subprocess.check_output([str(tools/'rustc'),'--version'],env=env,universal_newlines=True).strip();assert version.startswith('rustc 1.95.0 ')
compiled=subprocess.run([str(tools/'rustc'),'--edition=2024','--test',str(test),'-o',str(run/'timeline-test')],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,universal_newlines=True,timeout=60)
answer={'compiler':version,'compile_returncode':compiled.returncode,'compile_log':compiled.stdout,'remote_run':str(run),'fragment_sha256':hashlib.sha256(fragment.encode()).hexdigest(),'file_sha256':job['file_sha256']}
if compiled.returncode==0:
 tested=subprocess.run([str(run/'timeline-test'),'--test-threads=1','--nocapture'],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,universal_newlines=True,timeout=30)
 answer.update(test_returncode=tested.returncode,test_log=tested.stdout)
formatted=subprocess.run([str(tools/'rustfmt'),'--edition=2024','--config-path',str(run/'rustfmt.toml'),*[str(run/name) for name in job['files'] if name.endswith('.rs')]],env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,universal_newlines=True,timeout=30)
answer.update(format_returncode=formatted.returncode,format_log=formatted.stdout,formatted_files={name:(run/name).read_text() for name in job['files'] if name.endswith('.rs')})
(run/'result.json').write_text(json.dumps(answer,indent=2)+'\n');print(json.dumps(answer))
'''


def main(source, output):
    source, output = Path(source).resolve(), Path(output).resolve()
    if subprocess.check_output(['git', '-C', str(source), 'status', '--porcelain'], text=True).strip():
        raise ValueError('commit the fixture candidate before its standalone audit')
    def git(*args, **kwargs):
        return subprocess.check_output(['git', '-C', str(source), *args], text=True, **kwargs).strip()
    base, tree = git('rev-parse', 'HEAD'), git('rev-parse', 'HEAD^{tree}')
    run = 'turing-' + datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ') + '-' + uuid.uuid4().hex[:8]
    output.mkdir(parents=True, exist_ok=False)
    files = {name: subprocess.check_output(['git', '-C', str(source), 'show', base + ':' + name]).decode() for name in FILES}
    info = dict(schema=1, base_commit=base, tree=tree, dirty=False, created_at=datetime.now(timezone.utc).isoformat())
    container = git('mktree', input='040000 tree ' + tree + '\tsource\n')
    env = dict(os.environ, GIT_AUTHOR_NAME='Khor fixture audit', GIT_AUTHOR_EMAIL='snapshot@localhost',
               GIT_COMMITTER_NAME='Khor fixture audit', GIT_COMMITTER_EMAIL='snapshot@localhost')
    snapshot = git('commit-tree', container, '-m', 'Khor developer snapshot\n\n' + json.dumps(info), env=env)
    ref = 'refs/khor/net-evidence/' + run
    git('update-ref', ref, snapshot, '0' * 40)
    control = subprocess.check_output(['git', '-C', str(Path(__file__).resolve().parent.parent), 'rev-parse', 'HEAD'], text=True).strip()
    job = dict(run_id=run, files=files, file_sha256={name: hashlib.sha256(text.encode()).hexdigest() for name, text in files.items()})
    receipt = dict(run_id=run, scope='verbatim standalone fixture reducer, not product or netlab SLO',
                   source=dict(info, snapshot_commit=snapshot), local_ref=ref, control_commit=control,
                   control_script_sha256=hashlib.sha256(REMOTE.encode()).hexdigest(), file_sha256=job['file_sha256'])
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    index = Path('/Volumes/disk0/Khor/evidence/net/source-refs.json')
    data = json.loads(index.read_text())
    entry = dict(run_id=run, local_ref=ref, snapshot_commit=snapshot, source_tree=tree, control_commit=control,
                 executor='turing-fixture-audit', pending=True, submitted_receipt=str(output / 'receipt.json'), evidence_dir=str(output))
    data['sources'].append(entry)
    index.write_text(json.dumps(data, indent=2) + '\n')
    raw = subprocess.check_output(['ssh', *OPTIONS, 'turing', shlex.join(['python3', '-c', REMOTE])],
                                  input=json.dumps(job), text=True, timeout=150)
    answer = json.loads(raw)
    if answer['file_sha256'] != job['file_sha256']:
        raise ValueError('remote source byte receipt mismatch')
    (output / 'result.json').write_text(json.dumps(answer, indent=2) + '\n')
    for name, text in answer['formatted_files'].items():
        path = output / 'formatted' / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
    entry.update(pending=False, tests_executed=answer['compile_returncode'] == 0, product_tests_executed=False,
                 compile_returncode=answer['compile_returncode'], test_returncode=answer.get('test_returncode'), remote_source_verified=True)
    index.write_text(json.dumps(data, indent=2) + '\n')
    print(json.dumps({key: value for key, value in answer.items() if key != 'formatted_files'}, indent=2))
    print('Evidence: ' + str(output))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    main(args.source, args.output)
