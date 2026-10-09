"""Fail-closed unit tests for the precommitted Engine 0.7 baseline report parser."""

import json
import runpy
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOOLS = runpy.run_path(str(ROOT / "scripts/engine-0.7-baseline-v1-report.py"))
SPEC = TOOLS["verified_spec"]
PARSE = TOOLS["measured_cases"]
SUMMARIZE = TOOLS["summarize"]


def frozen_log():
    spec = SPEC()
    report = json.loads(
        (ROOT / "evaluation/engine-0.7-baseline-v1-result.json").read_text()
    )
    fixture_map = {case["id"]: case for case in spec["cases"]}
    observations = []
    for case in report["case_results"]:
        observations.append({
            **case,
            "title": fixture_map[case["id"]]["title"],
            "predicted_baseline": fixture_map[case["id"]]["expected_baseline"],
            "baseline_prediction_matches": True,
        })
    header = "\n".join(
        "ENGINE_070_BASELINE_CASE " + json.dumps(item) for item in observations
    )
    footer = (
        "ENGINE_070_BASELINE_SUMMARY "
        + json.dumps({
            "engine_version": "0.6.1",
            "compared_to_frozen_0_6_1": True,
            "cases": 29,
            "matches_predicted_baseline": 29,
            "model_calls": 0,
            "provider_attempts": 0,
            "external_acquisition_calls": 0,
            "provider_failures": 0,
        })
    )
    return spec, observations, header + "\n" + footer + "\n"


class FrozenBaselineReportContractTests(unittest.TestCase):
    def test_snapshot_baseline_reconstructs_exact_report(self):
        spec, _, text = frozen_log()
        observations, summary = PARSE(text)
        result = SUMMARIZE(spec, observations, summary)
        expected = json.loads(
            (ROOT / "evaluation/engine-0.7-baseline-v1-result.json").read_text()
        )
        self.assertEqual(result, expected)

    def test_rejects_missing_or_duplicate_measurements(self):
        spec, observations, text = frozen_log()
        first_id = observations[0]["id"]
        omitted = "\n".join(
            line
            for line in text.splitlines()
            if not (line.startswith("ENGINE_070_BASELINE_CASE ") and f'"id": "{first_id}"' in line)
        )
        seen, summary = PARSE(omitted)
        with self.assertRaises(AssertionError):
            SUMMARIZE(spec, seen, summary)
        first_line = next(
            line for line in text.splitlines() if line.startswith("ENGINE_070_BASELINE_CASE ")
        )
        with self.assertRaises(AssertionError):
            PARSE(text + first_line + "\n")

    def test_rejects_oracle_leak_into_engine(self):
        spec, _, text = frozen_log()
        seen, summary = PARSE(text)
        seen["src-02"]["diagnostics"]["origin_oracle_passed_to_engine"] = True
        with self.assertRaises(AssertionError):
            SUMMARIZE(spec, seen, summary)

    def test_rejects_fabricated_model_or_provider_activity(self):
        spec, _, text = frozen_log()
        seen, summary = PARSE(text)
        summary["provider_attempts"] = 1
        with self.assertRaises(AssertionError):
            SUMMARIZE(spec, seen, summary)

    def test_handles_concurrent_libtest_prefix_without_relaxing_identity(self):
        spec, _, text = frozen_log()
        lines = text.splitlines()
        lines[0] = "test order_invariance ... ok" + lines[0]
        cases, summary = PARSE("\n".join(lines))
        self.assertEqual(SUMMARIZE(spec, cases, summary)["scorecard"]["cases"], 29)


if __name__ == "__main__":
    unittest.main()
