"""A quiet witness needs stable counters, stable ownership and a real positive control."""
import copy
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('stream_witness', Path(__file__).resolve().parents[1]/'netlab-first-action-streams.py')
streams = importlib.util.module_from_spec(spec)
spec.loader.exec_module(streams)


class StreamWitness(unittest.TestCase):
    def test_stable_and_counter_registry_clock_failures(self):
        row = dict(quiet_started_at_unix_us=1000000000, restore_started_at_unix_us=1012000000,
                   quiet_started_us=5000, restore_started_us=12005000,
                   request_ended_at_unix_us=1012500000)
        for mutation in ['green', 'background_stream', 'registry_change', 'actor_restart',
                         'no_positive', 'sparse', 'short_witness', 'wrong_peer',
                         'clock_jump', 'malformed', 'duplicate_connection']:
            with self.subTest(mutation=mutation):
                lines = []
                for index, offset in enumerate(range(-250000, 13500001, 250000)):
                    counters = [(7, 10, 20), (8, 30, 40)]
                    actor, registry = 1, 2
                    if offset >= 12500000 and mutation != 'no_positive':
                        counters[0] = (7, 11, 21)
                    if mutation == 'background_stream' and offset >= 5000000:
                        counters[1] = (8, 31, 40)
                    if mutation == 'registry_change' and offset >= 5000000:
                        registry = 4
                    if mutation == 'actor_restart' and offset >= 5000000:
                        actor = 2
                    if mutation == 'duplicate_connection':
                        counters[1] = counters[0]
                    if mutation == 'sparse' and 4000000 <= offset <= 6000000:
                        continue
                    if mutation == 'short_witness' and 500000 <= offset <= 1750000:
                        continue
                    peer = 'someone-else' if mutation == 'wrong_peer' else 'peer-a'
                    wall = 1000000000+offset+(200000 if mutation == 'clock_jump' and offset >= 5000000 else 0)
                    value = 'broken' if mutation == 'malformed' else repr(counters)
                    lines.append(f'DEBUG khor_net_samples: khor_stream_snapshot_v1 peer={peer} '
                                 f'monotonic_us={2000000+offset} unix_us={wall} '
                                 f'actor_serial={actor} registry_epoch={registry} counters={value}')
                raw = '\n'.join(lines)
                if mutation in ['wrong_peer', 'clock_jump', 'malformed', 'duplicate_connection']:
                    with self.assertRaises(ValueError):
                        streams.witness(raw, 'peer-a', copy.deepcopy(row))
                else:
                    result = streams.witness(raw, 'peer-a', copy.deepcopy(row))
                    self.assertEqual(result['passed'], mutation == 'green', result)
                    if mutation == 'green':
                        self.assertGreaterEqual(result['quiet_witness_us'], 10000000)
                        self.assertTrue(result['positive_request_control'])
                        self.assertEqual(result['connection_ids'], [7, 8])
