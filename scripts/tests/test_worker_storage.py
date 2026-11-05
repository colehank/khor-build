import importlib.util
import os
from pathlib import Path
import tempfile
import time
import unittest
from unittest.mock import patch
from types import SimpleNamespace

spec = importlib.util.spec_from_file_location("worker_storage", Path(__file__).resolve().parents[1] / "worker-storage.py")
storage = importlib.util.module_from_spec(spec)
spec.loader.exec_module(storage)


class IncrementalRetentionTest(unittest.TestCase):
    def test_admission_policy_change_is_atomic_idempotent_and_preserves_unknown_edits(self):
        original = '\n'.join(['START_RESERVE = 35 * 2**30', 'STOP_RESERVE = 20 * 2**30',
                              'TARGET_LIMIT = 60 * 2**30', 'message = "less than 35 GiB free; target cache reached 60 GiB; free space dropped below 20 GiB"'])
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / 'build_guard.py'
            path.write_text(original)
            result = storage.configure_guard(path)
            self.assertTrue(result['changed'])
            self.assertEqual(Path(result['backup']).read_text(), original)
            self.assertIn('TARGET_LIMIT = 100 * 2**30', path.read_text())
            self.assertIn('STOP_RESERVE = 40 * 2**30', path.read_text())
            self.assertFalse(storage.configure_guard(path)['changed'])
            changed = path.read_text().replace('TARGET_LIMIT = 100', 'TARGET_LIMIT = 101')
            path.write_text(changed)
            with self.assertRaisesRegex(RuntimeError, 'preserve it for review'):
                storage.configure_guard(path)
            self.assertEqual(path.read_text(), changed)

    def test_newest_eight_and_recent_variants_and_unknown_paths_are_preserved(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            now = time.time()
            for i in range(12):
                path = root / ("khor_node-" + str(i).zfill(13))
                path.mkdir()
                os.utime(path, (now - i * 3600, now - i * 3600))
            (root / "unknown").mkdir()
            (root / "khor_node-zzzzzzzzzzzzz").symlink_to(root / "unknown")
            candidates = storage.incremental_candidates(root, now)
            self.assertEqual({row[1].name for row in candidates}, {"khor_node-" + str(i).zfill(13) for i in range(8, 12)})

    def test_open_directory_is_preserved_even_when_lsof_returns_one_with_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            active, idle = root / "active", root / "idle"
            active.mkdir(); idle.mkdir()
            candidates = [(p.stat().st_mtime_ns, p, p.stat().st_dev, p.stat().st_ino) for p in [active, idle]]
            with patch.object(storage.subprocess, "run", return_value=SimpleNamespace(returncode=1, stderr="", stdout="p42\nn" + str(active / "file"))), patch.object(storage, "allocated", return_value=12):
                removed = storage.collect_incremental(root, candidates)
            self.assertTrue(active.exists())
            self.assertFalse(idle.exists())
            self.assertEqual(removed, [{"directory": str(idle), "allocated_bytes": 12}])

    def test_replaced_directory_is_never_collected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "variant"
            path.mkdir()
            stat = path.stat()
            candidate = (stat.st_mtime_ns, path, stat.st_dev, stat.st_ino)
            path.rename(root / "saved")
            path.mkdir()
            with patch.object(storage.subprocess, "run", return_value=SimpleNamespace(returncode=1, stderr="", stdout="")):
                self.assertEqual(storage.collect_incremental(root, [candidate]), [])
            self.assertTrue(path.exists())
            self.assertTrue((root / "saved").exists())


if __name__ == "__main__":
    unittest.main()
