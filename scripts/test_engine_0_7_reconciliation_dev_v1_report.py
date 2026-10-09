"""Fail-closed report-consistency controls for #490 development-only observations."""

import json
import runpy
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PARSER = runpy.run_path(str(ROOT / "scripts/engine-0.7-reconciliation-dev-v1-report.py"))
SPEC = PARSER["load_spec"]
SCORE = PARSER["score"]
OBSERVE = PARSER["observe"]


def synthetic_log():
    spec = SPEC()
    stored = json.loads(
        (ROOT / "evaluation/engine-0.7-reconciliation-dev-v1-result.json").read_text()
    )
    lines = []
    for item in stored["case_results"]:
        lines.append("ENGINE_070_RECONCILIATION_CASE " + json.dumps(item))
    lines.append("ENGINE_070_RECONCILIATION_SUMMARY " + json.dumps({
        "cases": 22, "passed": 22, "model_calls": 0, "provider_attempts": 0,
        "external_calls": 0, "hard_authority_promotions": 0,
        "source_state_mutations": 0,
    }))
    lines.append("test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out")
    return spec, stored, "\n".join(lines)


class ReportIntegrityTests(unittest.TestCase):
    def test_full_report_reconstructs_exactly(self):
        spec, stored, log = synthetic_log()
        self.assertEqual(SCORE(spec, *OBSERVE(log)), stored)

    def test_missing_case_is_not_assumed_success(self):
        spec, _, log = synthetic_log()
        lines = [line for line in log.splitlines() if '"id": "equiv-01"' not in line]
        with self.assertRaises(AssertionError):
            SCORE(spec, *OBSERVE("\n".join(lines)))

    def test_duplicate_case_is_rejected(self):
        _, _, log = synthetic_log()
        first = next(line for line in log.splitlines() if '"id": "equiv-01"' in line)
        with self.assertRaises(AssertionError):
            OBSERVE(log + "\n" + first)

    def test_fake_promotion_activity_is_rejected(self):
        spec, _, log = synthetic_log()
        cases, summary = OBSERVE(log)
        summary["hard_authority_promotions"] = 1
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)

    def test_untrusted_status_relabel_is_rejected(self):
        spec, _, log = synthetic_log()
        cases, summary = OBSERVE(log)
        cases["safe-04"]["observed"] = "reviewed_compatible"
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)

    def test_dropped_citation_is_rejected(self):
        spec, _, log = synthetic_log()
        cases, summary = OBSERVE(log)
        cases["equiv-01"]["details"]["citations"] = 1
        with self.assertRaises(AssertionError):
            SCORE(spec, cases, summary)

    def test_missing_test_completion_is_rejected(self):
        _, _, log = synthetic_log()
        with self.assertRaises(AssertionError):
            OBSERVE(log.split("test result: ok.")[0])

    def test_libtest_prefix_is_supported_without_rescoring(self):
        spec, stored, log = synthetic_log()
        lines = log.splitlines()
        lines[0] = "test frozen_control ... ok " + lines[0]
        self.assertEqual(SCORE(spec, *OBSERVE("\n".join(lines))), stored)


if __name__ == "__main__":
    unittest.main()
