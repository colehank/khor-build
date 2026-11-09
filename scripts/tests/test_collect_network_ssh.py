import importlib.util
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest


spec = importlib.util.spec_from_file_location('collector', Path(__file__).resolve().parents[1] / 'collect-network-ssh.py')
collector = importlib.util.module_from_spec(spec)
spec.loader.exec_module(collector)


class CollectNetworkSSHTest(unittest.TestCase):
    def test_archive_cannot_follow_a_report_link_or_write_outside_evidence(self):
        for link in [True, False]:
            with self.subTest(link=link), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                archive = root / 'report.tar'
                with tarfile.open(archive, 'w') as output:
                    member = tarfile.TarInfo('link' if link else '../escaped')
                    if link:
                        member.type, member.linkname = tarfile.SYMTYPE, '/nfs/personal/credentials'
                        output.addfile(member)
                    else:
                        member.size = 1
                        output.addfile(member, io.BytesIO(b'x'))
                with self.assertRaisesRegex(ValueError, 'escaping'):
                    collector.extract_report(archive, root / 'report')
                self.assertFalse((root / 'escaped').exists())

    def test_wrong_source_receipt_is_rejected_before_interpreting_a_green_status(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory)
            source = dict(base_commit='a' * 40, tree='b' * 40, snapshot_commit='c' * 40)
            (output / 'source.json').write_text(json.dumps(dict(source, tree='d' * 40)))
            (output / 'execution.json').write_text('{"status":"completed","returncode":0}')
            with self.assertRaisesRegex(ValueError, 'source does not match'):
                collector.verify({'source': source}, output, {})


if __name__ == '__main__':
    unittest.main()
