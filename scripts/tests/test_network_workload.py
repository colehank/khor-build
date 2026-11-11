"""Exercise the pinned log validator with synthetic evidence, never a product."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class WorkloadEvidence(unittest.TestCase):
    def test_raw_population_and_actual_overlap_are_required(self):
        driver = (Path(__file__).resolve().parents[1] / 'network.sh').read_text()
        script = driver.split("import collections,hashlib,json,pathlib,re,sys\n", 1)[1].split('\nPYTHON', 1)[0]
        script = 'import collections,hashlib,json,pathlib,re,sys\n' + script
        for mode in ['success', 'missing_sample', 'duplicate_sample', 'timeout', 'forged_overlap', 'missing_round', 'two_summaries']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                report = Path(tmp)
                (report / 'source.json').write_text(json.dumps({'tree': 'a' * 40}))
                for round_id in range(2 if mode == 'missing_round' else 3):
                    rows = [{'schema': 1, 'revision': 'resident_mixed_workload_v2', 'event': 'manifest',
                             'samples_per_operation_per_phase': 120, 'period_us': 100000,
                             'caller_us': 2000000, 'slo': False, 'calibrated': False}]
                    for phase_id, phase in enumerate(['idle_before', 'bulk', 'idle_after']):
                        for op in ['directory', 'terminal']:
                            for index in range(120):
                                started = phase_id * 20000000 + index * 100000
                                rows.append({'schema': 1, 'revision': 'resident_mixed_workload_v2',
                                             'event': 'sample', 'phase': phase, 'operation': op, 'index': index,
                                             'scheduled_us': started, 'started_us': started, 'ended_us': started + 100,
                                             'elapsed_us': 100, 'caller_us': 2000000, 'outcome': 'ok', 'error': None})
                    rows.extend([
                        {'schema': 1, 'revision': 'resident_mixed_workload_v2', 'event': 'bulk', 'phase': 'bulk',
                         'started_us': 20000000, 'ended_us': 35000000, 'moved': 16 * 1024 * 1024,
                         'expected': 16 * 1024 * 1024, 'outcome': 'ok'},
                        {'schema': 1, 'revision': 'resident_mixed_workload_v2', 'event': 'complete', 'samples': 720,
                         'directory_overlap': 120, 'terminal_overlap': 120, 'functional_ok': True,
                         'all_answers': True, 'exact_bytes': True, 'closed': True, 'direct_after': True},
                    ])
                    if round_id == 0:
                        if mode == 'missing_sample':
                            rows.pop(1)
                        elif mode == 'duplicate_sample':
                            rows[2] = dict(rows[1])
                        elif mode == 'timeout':
                            rows[1].update(outcome='timeout', error='bounded caller deadline')
                        elif mode == 'forged_overlap':
                            rows[-2]['ended_us'] = 20000001
                    folder = report / 'netlab' / str(round_id)
                    folder.mkdir(parents=True)
                    text = ''.join('[netlab-workload] ' + json.dumps(row) + '\n' for row in rows)
                    summary = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 20 filtered out; finished in 60s\n'
                    (folder / 'scenarios.log').write_text(text + summary * (2 if mode == 'two_summaries' else 1))
                result = subprocess.run([sys.executable, '-B', '-c', script, str(report)], capture_output=True, text=True)
                evidence = json.loads((report / 'workload-contracts.json').read_text())
                self.assertEqual(evidence['passed'], mode == 'success', result.stderr)
                self.assertEqual(result.returncode == 0, mode == 'success', result.stderr)
                self.assertFalse(evidence['slo'])
                self.assertFalse(evidence['calibrated'])
                if mode == 'timeout':
                    self.assertTrue(evidence['results'][0]['valid'])
                    self.assertEqual(evidence['results'][0]['groups']['idle_before/directory']['outcomes']['timeout'], 1)
