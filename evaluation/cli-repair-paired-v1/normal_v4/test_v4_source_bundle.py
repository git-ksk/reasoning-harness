"""The live tagged builder must include and enroll the v4 reviewer modules."""
from pathlib import Path
import os
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))
from native_bundle import SOURCE_DIRS, source_inventory


class V4SourceIncludedTests(unittest.TestCase):
    def test_full_frozen_export_supports_v5_tag_gate_import_without_checkout_fallback(self):
        # Copy ONLY operator-enrolled source bytes; a real hosted runner does
        # not have a root-level module from the checkout in this frozen tree.
        root = Path(__file__).resolve().parents[3]
        enrolled = set(source_inventory(root))
        needed = [n for n in enrolled
                  if n == "evaluation/cli-repair-paired-v1/tag_admission.py"
                  or n.startswith("evaluation/cli-repair-paired-v1/normal_v4/")]
        self.assertIn("evaluation/cli-repair-paired-v1/tag_admission.py",needed)
        with tempfile.TemporaryDirectory() as tmp:
            private_root = Path(tmp)
            for name in needed:
                source = root / name
                destination = private_root / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                destination.write_bytes(source.read_bytes())
            script = str(private_root/"evaluation/cli-repair-paired-v1/normal_v4/tag_gate.py")
            cmd = [sys.executable,"-B","-c",
                "import importlib.util,sys;"
                "p=sys.argv[1];"
                "sys.path.insert(0,p.rsplit('/',1)[0]);"
                "spec=importlib.util.spec_from_file_location('v5_tag_gate_check',p);"
                "m=importlib.util.module_from_spec(spec);"
                "spec.loader.exec_module(m);"
                "assert m.TAG=='cli-normal-real24-development-v5';"
                "print('ISOLATED_SOURCE_CLOSURE_PASS')",script]
            done = subprocess.run(cmd, cwd=private_root, text=True, capture_output=True,
                                  timeout=20, env={**os.environ,"PYTHONPATH":""})
            self.assertEqual(done.returncode,0,done.stderr)
            self.assertIn("ISOLATED_SOURCE_CLOSURE_PASS",done.stdout)

    def test_v4_directory_is_exported_by_native_build_and_enrolled_for_both_arms(self):
        root=Path(__file__).resolve().parents[3]
        relative="evaluation/cli-repair-paired-v1/normal_v4"
        self.assertIn("evaluation/cli-repair-paired-v1", SOURCE_DIRS)
        files=set(source_inventory(root))
        self.assertIn("evaluation/cli-repair-paired-v1/tag_admission.py",files)
        for item in ("shard_contract.py","tag_gate.py","aggregate.py",
                     "test_shard_operator.py","README.md"):
            self.assertIn(relative+"/"+item,files)


if __name__=="__main__":
    unittest.main()
