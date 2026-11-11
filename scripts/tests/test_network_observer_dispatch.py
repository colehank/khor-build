"""Fixed paired-lane control dispatch with stubs, never Rust or namespaces."""
import json
import itertools
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class ObserverDispatch(unittest.TestCase):
    def test_order_guard_and_failure_preservation(self):
        driver = Path(__file__).resolve().parents[1] / 'network.sh'
        env = {k:v for k,v in os.environ.items() if not k.startswith('GIT_')}
        env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1', GIT_AUTHOR_NAME='Fixture',
                   GIT_AUTHOR_EMAIL='fixture@localhost', GIT_COMMITTER_NAME='Fixture', GIT_COMMITTER_EMAIL='fixture@localhost')
        for task, mode in itertools.product(['netlab-workload-observer', 'netlab-workload-wires', 'netlab-workload-policy', 'netlab-workload-high-calibrate', 'netlab-workload-high-sockets', 'netlab-workload-high-long', 'netlab-workload-high-burst', 'netlab-workload-high-queues'], ['success', 'case_failed', 'guard_failed']):
            with self.subTest(task=task, mode=mode), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root/'source'
                (source/'scripts').mkdir(parents=True)
                def git(*args):
                    return subprocess.check_output(['git', '-c', 'core.hooksPath=/dev/null', '-C', str(source), *args], env=env, text=True).strip()
                (source/'scripts/netlab.sh').write_text("#!/bin/sh\npython3 - \"$@\" <<'PY'\n"+r'''
import json,os,pathlib,sys
p=pathlib.Path(os.environ['MOCK_TRACE']);rows=json.loads(p.read_text()) if p.exists() else []
rows.append(dict(args=sys.argv[1:],directory=os.environ['NETLAB_RUN_DIR'],threads=os.environ['NETLAB_TEST_THREADS'],diagnostic=os.environ.get('NETLAB_DIAGNOSTICS')))
p.write_text(json.dumps(rows));mode=os.environ['MOCK_MODE']
failure_index=1 if os.environ['DEV_TASK']=='netlab-workload-high-long' else 3
sys.exit(73 if mode=='guard_failed' and len(rows)==1 or mode=='case_failed' and len(rows)==failure_index else 0)
'''+"\nPY\n")
                (source/'scripts/netlab-observer.py').write_text("import json,os,pathlib\np=pathlib.Path(os.environ['MOCK_TRACE']);r=json.loads(p.read_text());r.append({'validation':True});p.write_text(json.dumps(r))\n")
                (source/'scripts/netlab-workload-wires.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-workload-policy.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-kernel-high.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-kernel-sockets.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-kernel-long.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-kernel-burst.py').write_text((source/'scripts/netlab-observer.py').read_text())
                (source/'scripts/netlab-kernel-queues.py').write_text((source/'scripts/netlab-observer.py').read_text())
                git('init','-q');git('add','.');git('commit','-qm','stub fixture')
                package=root/'package';package.mkdir();git('clone','--bare','--quiet',str(source),str(package/'source.git'))
                (package/'source.json').write_text(json.dumps(dict(build_commit=git('rev-parse','HEAD'),tree=git('rev-parse','HEAD^{tree}'))))
                binary=root/'bin';binary.mkdir()
                for name,body in [('cargo','import sys\nassert sys.argv[1]=="fmt"\n'),('rustc','print("stub; no compiler")\n'),('unshare','pass\n')]:
                    p=binary/name;p.write_text('#!'+sys.executable+'\n'+body);p.chmod(0o700)
                report=root/'report';temporary=root/'temporary';temporary.mkdir();trace=root/'calls.json'
                run=subprocess.run(['bash',str(driver)],capture_output=True,text=True,env=dict(env,
                    PATH=str(binary)+os.pathsep+env['PATH'],DEV_TASK=task,SOURCE_PACKAGE=str(package),
                    RUNNER_TEMP=str(temporary),REPORT_DIR=str(report),CARGO_BUILD_JOBS='2',GITHUB_RUN_ID='stub',
                    GITHUB_SHA='control-fixture',MOCK_TRACE=str(trace),MOCK_MODE=mode))
                calls=json.loads(trace.read_text());self.assertEqual(calls[-1],{'validation':True})
                cases=calls[:-1]
                if task in ['netlab-workload-high-burst', 'netlab-workload-high-queues']:
                    expected = ['ha0_','hb0_','hb0_','ha0_'] if task == 'netlab-workload-high-burst' else ['hb0_','hq0_','hq0_','hb0_']
                    self.assertEqual([r['args'] for r in cases], [['workload::calibration::'+x,'--ignored'] for x in expected])
                    self.assertTrue(all(r['directory']==str(report/'netlab') and r['threads']=='1' and r['diagnostic']=='0' for r in cases))
                    self.assertEqual(run.returncode,0 if mode=='success' else 73,run.stdout+run.stderr)
                    continue
                if task in ['netlab-workload-high-calibrate','netlab-workload-high-sockets','netlab-workload-high-long']:
                    self.assertEqual(len(cases),1 if task=='netlab-workload-high-long' else 3)
                    selected={'netlab-workload-high-calibrate':'h0_','netlab-workload-high-sockets':'hd0_','netlab-workload-high-long':'hl0_'}[task]
                    self.assertTrue(all(r['args']==['workload::calibration::'+selected,'--ignored'] for r in cases))
                    self.assertTrue(all(r['directory']==str(report/'netlab') and r['threads']=='1' and r['diagnostic']=='0' for r in cases))
                    self.assertEqual(run.returncode,0 if mode=='success' else 73,run.stdout+run.stderr)
                    continue
                self.assertEqual(len(cases),1 if mode=='guard_failed' else (7 if task=='netlab-workload-observer' else 4),run.stdout+run.stderr)
                self.assertEqual(cases[0]['args'],['workload::observer::w3_','--ignored'])
                self.assertEqual(cases[0]['directory'],str(report/'observer-guards'))
                if mode!='guard_failed':
                    expected=['workload::w1_','workload::w2_','workload::w2_','workload::w1_','workload::w1_','workload::w2_'] if task=='netlab-workload-observer' else ['workload::w2_']*3
                    self.assertEqual([r['args'][0] for r in cases[1:]],expected)
                    self.assertTrue(all(r['directory']==str(report/'netlab') and r['threads']=='1' for r in cases[1:]))
                    diagnostic={'netlab-workload-wires':'lifecycle','netlab-workload-policy':'0'}.get(task)
                    self.assertTrue(all(r['diagnostic']==diagnostic for r in cases[1:]))
                self.assertEqual(run.returncode,0 if mode=='success' else 73,run.stdout+run.stderr)
