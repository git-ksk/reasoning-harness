"""Integrity tests for the #490 precommitted target-local development report."""
import json
import runpy
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
API = runpy.run_path(str(ROOT / "scripts/engine-0.7-target-presentation-dev-v1-report.py"))
LOAD, PARSE, SCORE = API["spec"], API["parse"], API["score"]


def reconstruct():
    frozen = LOAD()
    expected = json.loads((ROOT / "evaluation/engine-0.7-target-presentation-dev-v1-result.json").read_text())
    lines = [
        "ENGINE_070_TARGET_PRESENTATION_CASE " + json.dumps({
            "id": case["id"], "expected": case["expected"], "observed": case["observed"], "details": case["detail"]
        })
        for case in expected["case_results"]
    ]
    lines.append("ENGINE_070_TARGET_PRESENTATION_SUMMARY " + json.dumps({
        "cases": 14, "passed": 14, "model_calls": 0, "provider_attempts": 0,
        "external_acquisition": 0, "truth_promotions": 0
    }))
    lines.append("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured")
    return frozen, expected, "\n".join(lines)


class PresentationReportIntegrity(unittest.TestCase):
    def test_byte_equivalent_score(self):
        frozen, expected, raw = reconstruct()
        self.assertEqual(SCORE(frozen, *PARSE(raw)), expected)

    def test_missing_case_and_duplicate_case_rejected(self):
        frozen, _, raw = reconstruct()
        lines = raw.splitlines()
        with self.assertRaises(AssertionError):
            SCORE(frozen, *PARSE("\n".join(lines[1:])))
        with self.assertRaises(AssertionError):
            PARSE(raw + "\n" + lines[0])

    def test_forged_source_label_rejected(self):
        frozen, _, raw = reconstruct()
        cases, summary = PARSE(raw)
        cases["reviewed_with_opposition"]["details"]["target_statuses"][1] = "reviewed_compatible"
        with self.assertRaises(AssertionError):
            SCORE(frozen, cases, summary)

    def test_forged_citation_coverage_rejected(self):
        frozen, _, raw = reconstruct()
        cases, summary = PARSE(raw)
        cases["reviewed_with_opposition"]["details"]["legacy_citations"] -= 1
        with self.assertRaises(AssertionError):
            SCORE(frozen, cases, summary)

    def test_hidden_provider_call_and_truth_promotion_rejected(self):
        frozen, _, raw = reconstruct()
        cases, summary = PARSE(raw)
        for label in ["provider_attempts", "model_calls", "truth_promotions", "external_acquisition"]:
            original = summary[label]
            summary[label] = 1
            with self.assertRaises(AssertionError, msg=label):
                SCORE(frozen, cases, summary)
            summary[label] = original

    def test_missing_rust_pass_rejected(self):
        _, _, raw = reconstruct()
        with self.assertRaises(AssertionError):
            PARSE(raw.replace("test result: ok. 1 passed; 0 failed;", "test result: FAILED;"))

    def test_missing_or_duplicate_summary_rejected(self):
        _, _, raw = reconstruct()
        with self.assertRaises(AssertionError):
            PARSE(raw.replace("ENGINE_070_TARGET_PRESENTATION_SUMMARY ", "IGNORED_SUMMARY "))
        summary = next(line for line in raw.splitlines() if line.startswith("ENGINE_070_TARGET_PRESENTATION_SUMMARY "))
        with self.assertRaises(AssertionError):
            PARSE(raw + "\n" + summary)

    def test_frozen_spec_tamper_rejected(self):
        path = LOAD.__globals__["SUITE_PATH"]
        with tempfile.TemporaryDirectory() as dir_name:
            bad = Path(dir_name) / "manifest.json"
            bad.write_bytes(path.read_bytes().replace(b"reviewed_single", b"forged_case_001"))
            old = LOAD.__globals__["SUITE_PATH"]
            LOAD.__globals__["SUITE_PATH"] = bad
            try:
                with self.assertRaises(AssertionError):
                    LOAD()
            finally:
                LOAD.__globals__["SUITE_PATH"] = old

    def test_historical_test_prefix_supported(self):
        frozen, expected, raw = reconstruct()
        self.assertEqual(SCORE(frozen, *PARSE("test frozen_target_local_presentation_scenarios ... " + raw)), expected)


if __name__ == "__main__":
    unittest.main()
