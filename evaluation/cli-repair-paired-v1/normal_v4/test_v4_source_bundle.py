"""The live tagged builder must include and enroll the v4 reviewer modules."""
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))
from native_bundle import SOURCE_DIRS, source_inventory


class V4SourceIncludedTests(unittest.TestCase):
    def test_v4_directory_is_exported_by_native_build_and_enrolled_for_both_arms(self):
        root=Path(__file__).resolve().parents[3]
        relative="evaluation/cli-repair-paired-v1/normal_v4"
        self.assertIn(relative, SOURCE_DIRS)
        files=set(source_inventory(root))
        for item in ("shard_contract.py","tag_gate.py","aggregate.py",
                     "test_shard_operator.py","README.md"):
            self.assertIn(relative+"/"+item,files)


if __name__=="__main__":
    unittest.main()
