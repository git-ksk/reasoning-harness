import json
import tempfile
import unittest
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import engine_0_5_final_composite as composite


class Engine050CompositeTests(unittest.TestCase):
    def reports(self, final_ok=True, material_ok=True):
        td = tempfile.TemporaryDirectory()
        root = Path(td.name)
        f = root / "final.json"
        m = root / "material.json"
        f.write_text(json.dumps({"acceptance_passed": final_ok, "aggregate": {"total_cases": 3}}))
        m.write_text(json.dumps({
            "acceptance_passed": material_ok,
            "checks": {},
            "summary": {},
            "observed_model_identities": {"test-model": 1},
        }))
        return td, f, m

    def test_required_pass(self):
        td, f, m = self.reports()
        try:
            report = composite.build("mistral-8b", f, 0, m, 0)
            self.assertEqual(report["classification"], "PASS")
            self.assertTrue(report["release_gate_passed"])
        finally:
            td.cleanup()

    def test_required_failure_is_release_failure(self):
        td, f, m = self.reports(material_ok=False)
        try:
            report = composite.build("mistral-8b", f, 0, m, 3)
            self.assertEqual(report["classification"], "FAIL")
            self.assertFalse(report["release_gate_passed"])
        finally:
            td.cleanup()

    def test_observed_failure_is_not_release_gate(self):
        td, f, m = self.reports(final_ok=False)
        try:
            report = composite.build("mistral-14b", f, 1, m, 0)
            self.assertEqual(report["classification"], "FAIL")
            self.assertIsNone(report["release_gate_passed"])
        finally:
            td.cleanup()

    def test_missing_report_is_incomplete(self):
        td, f, m = self.reports()
        missing = Path(td.name) / "missing.json"
        try:
            report = composite.build("nvidia-nemotron", missing, 2, m, 3)
            self.assertEqual(report["classification"], "INCOMPLETE")
            self.assertIsNone(report["release_gate_passed"])
        finally:
            td.cleanup()


if __name__ == "__main__":
    unittest.main()
