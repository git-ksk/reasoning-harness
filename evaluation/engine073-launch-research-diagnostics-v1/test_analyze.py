import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location("archive_diagnostic", Path(__file__).with_name("analyze.py"))
DIAGNOSTIC = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(DIAGNOSTIC)
REPO = Path(__file__).resolve().parents[2]


class ArchiveDiagnosticTests(unittest.TestCase):
    def test_preserved_observation_and_pair_losses_are_reported_without_execution(self):
        report = DIAGNOSTIC.analyze(REPO)
        self.assertEqual(report["original_first_outcome"], "FAIL")
        self.assertEqual(report["verified_archive_files"], 444)
        self.assertEqual(report["saved_counts"]["passed"], 49)
        self.assertEqual(report["saved_counts"]["negative_v30_relevant_violations"], 0)
        self.assertEqual(report["pair_patterns_from_saved_grades"], {
            "both_checks_pass": 19, "positive_only_fail": 11,
        })
        self.assertEqual(len(report["failed_positive_ids"]), 11)
        self.assertEqual(report["new_execution"], {"engine": 0, "scorer": 0, "provider": 0})
        self.assertFalse(report["trace_limits"]["implementation_root_causes_established"])

    def copy_archive(self, destination):
        for folder in (REPO / "evaluation").glob("engine073-launch-independent-v*"):
            shutil.copytree(folder, destination / "evaluation" / folder.name,
                            copy_function=shutil.copyfile)

    def test_changed_first_output_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.copy_archive(root)
            path = root / "evaluation" / DIAGNOSTIC.ARCHIVE / "first-results.json"
            path.write_bytes(path.read_bytes() + b"\n")
            with self.assertRaisesRegex(DIAGNOSTIC.ArchiveError, "frozen artifact changed"):
                DIAGNOSTIC.analyze(root)

    def test_unrecorded_current_file_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.copy_archive(root)
            (root / "evaluation" / DIAGNOSTIC.ARCHIVE / "unrecorded.txt").write_text("extra")
            with self.assertRaisesRegex(DIAGNOSTIC.ArchiveError, "inventory differs"):
                DIAGNOSTIC.analyze(root)

    def test_changed_predecessor_evidence_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.copy_archive(root)
            path = root / "evaluation" / "engine073-launch-independent-v2" / "README.md"
            path.write_bytes(path.read_bytes() + b"\n")
            with self.assertRaisesRegex(DIAGNOSTIC.ArchiveError, "v2/README.md"):
                DIAGNOSTIC.analyze(root)

    def test_different_final_anchor_is_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.copy_archive(root)
            path = root / "evaluation" / DIAGNOSTIC.ARCHIVE / "final-artifact-manifest.json"
            data = json.loads(path.read_text())
            data["identity"] = "replacement"
            path.write_text(json.dumps(data))
            with self.assertRaisesRegex(DIAGNOSTIC.ArchiveError, "pinned v19 anchor"):
                DIAGNOSTIC.analyze(root)


if __name__ == "__main__":
    unittest.main()
