"""Validate kernel calibration evidence with synthetic logs, not a product."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class KernelCalibration(unittest.TestCase):
    def test_missing_population_or_forged_metrics_cannot_pass(self):
        driver = (Path(__file__).resolve().parents[1] / 'network.sh').read_text()
        script = driver.split('# kernel TCP calibration raw validator\n', 1)[1].split('\nPYTHON', 1)[0]
        script = 'import hashlib,json,pathlib,re,sys\n' + script
        for mode in ['success', 'harness_prefix', 'missing_echo', 'duplicate_echo', 'wrong_warmup', 'wrong_bytes',
                     'forged_rtt', 'forged_rate', 'missing_capture', 'missing_round', 'two_summaries', 'premature_acceptance']:
            with self.subTest(mode=mode), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                (root / 'source.json').write_text(json.dumps({'tree': 'a' * 40}))
                for round_id in range(2 if mode == 'missing_round' else 3):
                    rows = [{'event': 'manifest', 'warmup': 4, 'echoes': 64, 'echo_bytes': 64,
                             'bulk_bytes': 8388608, 'min_rtt_us_bounds': [35000, 80000],
                             'goodput_mbit_bounds': [6.4, 9.6], 'qdisc_units_review_required': True},
                            {'event': 'connect', 'ok': True, 'error': None}]
                    for i in range(68):
                        rows.append({'event': 'echo', 'index': i, 'warmup': i < 4, 'bytes': 64,
                                     'started_us': i * 40000, 'ended_us': (i + 1) * 40000,
                                     'elapsed_us': 40000, 'outcome': 'ok', 'error': None})
                    for direction in ['upload', 'download']:
                        rows.append({'event': 'bulk', 'direction': direction, 'bytes': 8388608,
                                     'started_us': 0, 'ended_us': 9000000, 'elapsed_us': 9000000,
                                     'outcome': 'ok', 'error': None, 'exact_bytes': True})
                    for name in ['x_before', 'gateway_before', 'x_after', 'gateway_after']:
                        rows.append({'event': 'capture', 'name': name, 'capture': {
                            'qdisc_json': {'ok': True, 'stdout': json.dumps([{'kind': 'netem'}, {'kind': 'tbf'}])},
                            'qdisc_text': {'ok': True, 'stdout': 'raw readback needs review'},
                            'interfaces': {'ok': True, 'stdout': '[]'}}})
                    rows.append({'event': 'complete', 'samples': 70, 'min_rtt_us': 40000,
                                 'goodput_mbit': [8388608 * 8 / 9000000] * 2, 'bounds_ok': True,
                                 'calibrated': False, 'qdisc_units_review_required': True,
                                 'functional_and_bounds_ok': True, 'server_ok': True, 'qdiscs_present': True})
                    if round_id == 0:
                        if mode == 'missing_echo': rows.pop(2)
                        elif mode == 'duplicate_echo': rows[3] = dict(rows[2])
                        elif mode == 'wrong_warmup': rows[2]['warmup'] = False
                        elif mode == 'wrong_bytes': rows[70]['exact_bytes'] = False
                        elif mode == 'forged_rtt': rows[-1]['min_rtt_us'] = 1
                        elif mode == 'forged_rate': rows[-1]['goodput_mbit'] = [8, 8]
                        elif mode == 'missing_capture': rows.pop(-2)
                        elif mode == 'premature_acceptance': rows[-1]['calibrated'] = True
                    for row in rows:
                        row.update(schema=1, revision='workload_kernel_tcp_calibration_v1')
                    text = ''.join('[netlab-workload-calibration] ' + json.dumps(row) + '\n' for row in rows)
                    if mode == 'harness_prefix':
                        text = 'test workload::calibration::w0_kernel_tcp_confirms ... ' + text
                    summary = 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 30 filtered out; finished in 20s\n'
                    p = root / 'netlab' / str(round_id)
                    p.mkdir(parents=True)
                    (p / 'scenarios.log').write_text(text + summary * (2 if mode == 'two_summaries' else 1))
                result = subprocess.run([sys.executable, '-B', '-c', script, str(root)], capture_output=True, text=True)
                evidence = json.loads((root / 'workload-calibration-contracts.json').read_text())
                self.assertEqual(evidence['passed'], mode in ['success', 'harness_prefix'], result.stderr)
                self.assertEqual(result.returncode == 0, mode in ['success', 'harness_prefix'], result.stderr)
                self.assertFalse(evidence['calibrated'])
                self.assertTrue(evidence['qdisc_units_review_required'])
