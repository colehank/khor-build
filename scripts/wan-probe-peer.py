"""Trusted Mac controller for one isolated, leased Mac/Turing WAN experiment.

Only this parent receives an ephemeral Actions capability and status token.
Private RPC bytes stay in pipes; reports contain public identities and outcomes.
"""
import importlib.util
import json
import os
from pathlib import Path
import re
import select
import shutil
import subprocess
import sys
import time
from urllib.request import Request, urlopen


def child_environment():
    allowed = ['HOME', 'USER', 'LOGNAME', 'LANG', 'LC_ALL', 'TMPDIR']
    return dict({key: os.environ[key] for key in allowed if key in os.environ},
                PATH='/usr/bin:/bin:/usr/sbin:/sbin')


class ProbeClient:
    def __init__(self, process):
        self.process, self.sequence = process, 0

    def response(self, sequence, seconds):
        if not select.select([self.process.stdout], [], [], seconds)[0]:
            raise TimeoutError('private RPC deadline')
        line = self.process.stdout.readline(65537)
        if not line or len(line) > 65536:
            raise RuntimeError('private RPC transport ended')
        value = json.loads(line)
        if value.get('seq') != sequence or value.get('ok') is not True:
            # Never include response bytes in an exception or an ordinary log.
            raise RuntimeError('probe operation failed')
        return value['data']

    def call(self, operation, **fields):
        self.sequence += 1
        command = dict(fields, seq=self.sequence, op=operation)
        self.process.stdin.write((json.dumps(command)+'\n').encode())
        self.process.stdin.flush()
        return self.response(self.sequence, 40 if operation in ('read', 'read_content') else 100)


def measure(client, peer, fixture, count=30, machine='wan-linux', operations=('read',)):
    assert operations and all(op in ('read', 'read_content') for op in operations)
    plan = [(op, index) for op in operations for index in range(count)]
    outcomes = []
    for position, (operation, index) in enumerate(plan):
        began = time.monotonic_ns()
        try:
            value = client.call(operation, expected_id=peer, machine=machine, fixture=fixture)
            assert value['peer']==peer
            if operation=='read_content':
                assert value['marker_content_verified'] is True and value['bytes']==29
            else:
                assert value['marker_verified'] is True
            row = dict(operation=operation, index=index, outcome='ok', elapsed_us=value['elapsed_us'],
                       controller_elapsed_us=(time.monotonic_ns()-began)//1000)
            if operation=='read_content':
                row.update(bytes=29, marker_content_verified=True)
            outcomes.append(row)
        except Exception as error:
            outcomes.append(dict(operation=operation, index=index, outcome='error', error_type=type(error).__name__,
                                 controller_elapsed_us=(time.monotonic_ns()-began)//1000))
            if client.process.poll() is not None or isinstance(error, (TimeoutError, BrokenPipeError)):
                outcomes.extend(dict(operation=op, index=n, outcome='cancelled', reason='control transport unavailable')
                                for op, n in plan[position+1:])
                break
    return outcomes


def main():
    # Remove capabilities before constructing any child environment.
    capability = json.loads(os.environ.pop('KHOR_WAN_CAPABILITY'))
    token = os.environ.pop('GH_TOKEN')
    run_id = os.environ['WAN_RUN_ID']
    repository, commit = os.environ['GITHUB_REPOSITORY'], os.environ['GITHUB_SHA']
    workflow_run = os.environ['GITHUB_RUN_ID']
    assert re.fullmatch('[0-9a-f]{16}', run_id)
    assert re.fullmatch('[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository)
    assert re.fullmatch('[0-9a-f]{40}', commit) and workflow_run.isdecimal()
    assert capability['run_id']==run_id
    assert set(capability)=={'schema','run_id','expected_id','ticket','fixture','source_tree','snapshot_commit'}
    assert capability['schema']==1 and re.fullmatch('[0-9a-f]{64}', capability['expected_id'])
    assert capability['fixture']==('/tmp/khor-netlab-feat-net-1110/wan/'+run_id+'-linux/home/fixture')
    report = Path(os.environ['REPORT_DIR'])
    manifest = json.loads((report/'wan-binary.json').read_text())
    assert manifest['source']['tree']==capability['source_tree']
    assert manifest['source']['snapshot_commit']==capability['snapshot_commit']
    guardian_path = Path(__file__).with_name('wan-probe-lease.py').resolve()
    spec = importlib.util.spec_from_file_location('wan_guardian', guardian_path)
    guardian = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(guardian)

    def api(suffix, body=None):
        headers = {'Authorization':'Bearer '+token, 'Accept':'application/vnd.github+json',
                   'User-Agent':'Khor isolated WAN controller'}
        request = Request('https://api.github.com/repos/'+repository+'/'+suffix,
                          data=None if body is None else json.dumps(body).encode(), headers=headers)
        with urlopen(request, timeout=15) as response:
            raw = response.read(256*1024+1)
        assert len(raw)<=256*1024
        return json.loads(raw)

    target = 'https://github.com/'+repository+'/actions/runs/'+workflow_run
    def status(phase, description, state='success'):
        api('statuses/'+commit, dict(state=state, context='khor-wan/'+run_id+'/'+phase,
                                    description=description, target_url=target))

    result = dict(schema=2, measurement_revision='wan_paired_directory_and_content_v2',
                  run_id=run_id, role='mac', source=manifest['source'],
                  binary_sha256=manifest['sha256'], guardian_sha256=guardian.digest(guardian_path),
                  outcomes=[], passed=False, limits=['Paired warm directory and fixed 29-byte content requests; not cold connection, throughput, mobility, phone or energy proof'])
    process = None
    with guardian.admission('mac') as (fd, base, _target):
        guardian.owned_base(base)
        inputs = base.parent/'wan-input'
        owner = dict(schema=1, owner='khor feat/net wan inputs', uid=os.getuid())
        if not inputs.exists():
            inputs.mkdir(mode=0o700)
            guardian.record(inputs/'owner.json', owner)
        assert not inputs.is_symlink() and inputs.stat().st_uid==os.getuid()
        assert json.loads((inputs/'owner.json').read_text())==owner
        config = dict(schema=1, run_id=run_id, role='mac', binary=manifest,
                      source_tree=capability['source_tree'], snapshot_commit=capability['snapshot_commit'],
                      guardian_commit=commit, guardian_sha256=guardian.digest(guardian_path))
        configuration = inputs/(run_id+'-mac.json')
        guardian.record(configuration, config)
        root = base/(run_id+'-mac')
        assert not root.exists()
        with (inputs/(run_id+'-controller.log')).open('xb') as stderr:
            try:
                process = subprocess.Popen([sys.executable, str(guardian_path), str(configuration)],
                    stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                    env=child_environment(), pass_fds=(fd,))
                client = ProbeClient(process)
                ready = client.response(0, 110)
                assert ready['run_id']==run_id and ready['role']=='mac'
                assert re.fullmatch('[0-9a-f]{64}', ready['id'])
                result['ready'] = ready
                paired = client.call('pair', ticket=capability.pop('ticket'), expected_id=capability['expected_id'])
                result['paired_peer'] = paired['peer']
                result['devices'] = client.call('devices')
                result['links_before'] = client.call('links')
                status('ready', ready['id'])
                result['outcomes'] = measure(client, capability['expected_id'], capability['fixture'],
                                             operations=('read', 'read_content'))
                result['links_after'] = client.call('links')
                reads_ok = len(result['outcomes'])==60 and all(x['outcome']=='ok' for x in result['outcomes'])
                status('reads', 'directory/content '+str(sum(x['outcome']=='ok' for x in result['outcomes']))+'/60',
                       'success' if reads_ok else 'failure')
                until = time.monotonic()+180
                result['controller_done'] = False
                while time.monotonic()<until:
                    statuses = api('commits/'+commit+'/statuses?per_page=100')
                    done = next((s for s in statuses if s['context']=='khor-wan/'+run_id+'/done' and s['target_url']==target), None)
                    if done:
                        result['controller_done'] = done['state']=='success'
                        break
                    time.sleep(2)
                result['stop'] = client.call('stop')
                process.stdin.close()
                process.stdin = None
                assert process.wait(timeout=65)==0
                result['business_passed'] = reads_ok and result['controller_done']
            except BaseException as error:
                result['error_type'] = type(error).__name__
                # Do not print exception values: an HTTP/RPC error can echo secrets.
            finally:
                if process is not None:
                    if process.stdin is not None:
                        process.stdin.close()
                        process.stdin = None
                    try:
                        process.wait(timeout=75)
                    except subprocess.TimeoutExpired:
                        result['controller_wait_timeout'] = True
                        process.terminate()
                        process.wait(timeout=60)
                    process.stdout.close()
                result['lifecycle'] = json.loads((root/'lifecycle.json').read_text()) if (root/'lifecycle.json').exists() else {}
                result['remaining_owned_processes'] = guardian.owned_processes(root) if root.exists() else []
                life = result['lifecycle']
                clean = (life.get('resident_joined') and life.get('worker_returncode')==0
                         and not life.get('forced_worker_stop') and not life.get('error')
                         and not result['remaining_owned_processes'] and not (root/'lease.sock').exists())
                result['passed'] = bool(result.get('business_passed') and clean and not result.get('error_type'))
                guardian.record(report/'wan-peer.json', result)
                if clean:
                    assert json.loads((root/'owner.json').read_text())==dict(schema=1,owner='khor feat/net wan',uid=os.getuid(),run_id=run_id,role='mac')
                    for name in ['home','tmux']:
                        shutil.rmtree(root/name)
                    (root/'probe').unlink()
                    (root/'worker.log').unlink()
                manifest.update(executed=process is not None, execution_scope='isolated paired WAN directory and fixed content requests')
                guardian.record(report/'wan-binary.json', manifest)
    print(json.dumps(dict(run_id=run_id, passed=result['passed'], outcomes=len(result['outcomes']),
                          resident_joined=result['lifecycle'].get('resident_joined'))), flush=True)
    return 0 if result['passed'] else 1


if __name__=='__main__':
    try:
        raise SystemExit(main())
    except Exception as error:
        print('WAN controller failed: '+type(error).__name__, file=sys.stderr)
        raise SystemExit(1)
