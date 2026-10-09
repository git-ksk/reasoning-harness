"""Integrity tests for frozen local reviewer development report."""
import json
import runpy
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
API = runpy.run_path(str(ROOT / "scripts/engine-0.7-reviewer-host-dev-v1-report.py"))
SPEC = API["load_spec"]
PARSE = API["parse"]
SCORE = API["score"]

def reconstruct():
    spec = SPEC()
    expected = json.loads(
        (ROOT / "evaluation/engine-0.7-reviewer-host-dev-v1-result.json").read_text()
    )
    lines = [
        "ENGINE_070_HOST_REVIEW_CASE " + json.dumps({
            "id": row["id"], "expected": row["expected"], "observed": row["observed"]
        })
        for row in expected["case_results"]
    ]
    lines.append("ENGINE_070_HOST_REVIEW_SUMMARY " + json.dumps({
        "cases": 18, "passed": 18, "model_calls": 0, "provider_attempts": 0,
        "external_acquisition": 0, "truth_promotions": 0
    }))
    lines.append("test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
    return spec, expected, "\n".join(lines)

class ReviewerHostReportTests(unittest.TestCase):
    def test_report_is_byte_equivalent(self):
        spec, expected, text = reconstruct()
        self.assertEqual(SCORE(spec, *PARSE(text)), expected)

    def test_missing_or_duplicated_case_fails(self):
        spec, _, text = reconstruct()
        lines = [x for x in text.splitlines() if '"id": "human-equivalent"' not in x]
        with self.assertRaises(AssertionError):
            SCORE(spec, *PARSE("\n".join(lines)))
        first = next(x for x in text.splitlines() if '"id": "human-equivalent"' in x)
        with self.assertRaises(AssertionError):
            PARSE(text + "\n" + first)

    def test_fake_success_on_opposition_fails(self):
        spec, _, text = reconstruct()
        cases, summary = PARSE(text)
        cases["opposed-claims"]["observed"] = "reviewed_compatible"
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)

    def test_fake_enrollment_or_provider_activity_fails(self):
        spec, _, text = reconstruct()
        cases, summary = PARSE(text)
        summary["truth_promotions"] = 1
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)
        summary["truth_promotions"] = 0
        summary["provider_attempts"] = 1
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)

    def test_missing_test_completion_fails(self):
        _, _, text = reconstruct()
        with self.assertRaises(AssertionError):
            PARSE(text.split("test result: ok.")[0])

    def test_libtest_prefix_is_supported(self):
        spec, expected, text = reconstruct()
        lines = text.splitlines()
        lines[0] = "test precommitted_host_workflow_scenarios ... " + lines[0]
        self.assertEqual(SCORE(spec, *PARSE("\n".join(lines))), expected)

if __name__ == "__main__":
    unittest.main()
