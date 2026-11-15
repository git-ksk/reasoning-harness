"""Positive and tamper-negative tests for real paired native request audit."""
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parent))
import verify_paired_first_requests as pair

class NativeParityContract(unittest.TestCase):
    def test_bad_or_unbuilt_pair_refused(self):
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'bad.json'
            path.write_text(json.dumps({'contract':'bad','built':False}))
            with self.assertRaisesRegex(ValueError,'unverified_corrected_cli_pair'):
                pair.check_build(path)

    def test_paired_path_must_not_overwrite_previous_measurements(self):
        with tempfile.TemporaryDirectory() as root:
            root=Path(root)
            with patch.object(pair,'check_build',return_value={'arms':{}}):
                existing=root/'already'
                existing.mkdir()
                with self.assertRaisesRegex(ValueError,'refuse_to_reuse'):
                    pair.verify(pair.ROOT,root/'fake.json',existing)

if __name__=='__main__':unittest.main()
