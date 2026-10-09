"""Frozen scorecard integrity tests; use only fabricated runner data."""
import copy
import json
import runpy
import tempfile
import unittest
from pathlib import Path

S = runpy.run_path(str(Path(__file__).with_name("engine_0_7_independent_v1_score.py")))
score_one = S["score_one"]
summarize = S["summarize"]
protocol = S["load_protocol"]
models = S["MODELS"]
sha256 = S["sha256"]


def make_mock(provider="mistral"):
    categories = ["compatible"] * 4 + ["opposed"] * 3 + ["unknown"] * 2 + ["context_mismatch"] + ["identical"] * 2
    cases, observations = [], []
    for i, relation in enumerate(categories):
        cid = f"dev-mock-{i}"
        tid = f"dev-target-{i}"
        is_identical = relation == "identical"
        is_gain = i == 0
        advisory = None if is_identical else is_gain
        baseline = "qualified" if is_identical else "conflict"
        obs_status = "reviewed_compatible" if is_gain else baseline
        cases.append({
            "id": cid,
            "family": "synthetic-test",
            "targets": [{"target_id": tid, "relation": relation}],
        })
        observations.append({
            "id": cid,
            "family": "synthetic-test",
            "baseline_global_status": baseline,
            "candidate_global_status": baseline,
            "original_text_preserved": True,
            "original_citations_preserved": True,
            "replay_identical": True,
            "hard_gate_violations": [],
            "targets": [{
                "target_id": tid, "oracle_relation": relation,
                "baseline_status": baseline,
                "candidate_status": obs_status,
                "advisory": advisory,
                "model_calls": 0 if is_identical else 1,
                "provider_attempts": 0 if is_identical else 1,
                "input_tokens": None if is_identical else 18,
                "output_tokens": None if is_identical else 4,
                "latency_ms": 0 if is_identical else 125,
                "operational_failure": None,
                "baseline_citation_count": 2,
                "candidate_citation_count": 2,
                "gained_source_qualified_compatibility": is_gain,
                "hard_gate_violations": [],
            }],
        })
    source = {
        "suite_id": "engine-0.7-independent-source-holdout-v1",
        "candidate_commit": "4cae9326bcc3e210c3250f05772d84f00e41b645",
        "cases": cases,
    }
    report = {
        "schema": "engine-0.7-independent-holdout-provider-v1-result",
        "suite_id": source["suite_id"],
        "candidate_commit": source["candidate_commit"],
        "corpus_sha256": "f" * 64,
        "provider": provider,
        "model": models[provider],
        "status": "PASS_FROZEN_PROVIDER_GATE",
        "planned_cases": 12, "completed_cases": 12,
        "compatible_targets": 4, "negative_targets": 6,
        "net_useful_answer_gain": 1,
        "model_calls": 10, "provider_attempts": 10,
        "input_tokens": 180, "output_tokens": 40,
        "operational_failures": 0,
        "hard_gate_violations": [],
        "independent_first_observation": True,
        "no_external_acquisition": True,
        "no_truth_promotion": True,
        "observations": observations,
    }
    return source, report


class ScoreContract(unittest.TestCase):
    def test_known_good_first_result_scored(self):
        spec = protocol()
        source, report = make_mock()
        result = score_one(source, report, "f" * 64, spec, "mistral")
        self.assertTrue(result["passed"])
        self.assertEqual(result["net_useful_answer_gain"], 1)

    def test_incorrect_status_or_fabricated_citation_rejected(self):
        source, report = make_mock()
        report["observations"][4]["targets"][0]["candidate_status"] = "reviewed_compatible"
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")
        source, report = make_mock()
        report["observations"][2]["targets"][0]["candidate_citation_count"] = 3
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")

    def test_fake_candidate_or_extra_provider_call_rejected(self):
        source, report = make_mock()
        report["candidate_commit"] = "changed"
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")
        source, report = make_mock()
        report["observations"][0]["targets"][0]["provider_attempts"] = 3
        report["provider_attempts"] += 2
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")

    def test_different_origin_or_reordered_case_rejected(self):
        source, report = make_mock()
        report["observations"].reverse()
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")
        source, report = make_mock()
        report["corpus_sha256"] = "a" * 64
        with self.assertRaises(AssertionError):
            score_one(source, report, "f" * 64, protocol(), "mistral")

    def test_no_utility_gain_is_frozen_failure(self):
        source, report = make_mock()
        target = report["observations"][0]["targets"][0]
        target["advisory"] = False
        target["candidate_status"] = "conflict"
        target["gained_source_qualified_compatibility"] = False
        report["net_useful_answer_gain"] = 0
        report["status"] = "FAIL_FROZEN_PROVIDER_GATE"
        result = score_one(source, report, "f" * 64, protocol(), "mistral")
        self.assertFalse(result["passed"])

    def test_provider_failure_is_not_semantic_unknown(self):
        source, report = make_mock()
        target = report["observations"][1]["targets"][0]
        target["operational_failure"] = "provider_Quota"
        report["operational_failures"] = 1
        report["status"] = "FAIL_FROZEN_PROVIDER_GATE"
        result = score_one(source, report, "f" * 64, protocol(), "mistral")
        self.assertFalse(result["passed"])
        self.assertEqual(result["operational_failures"], 1)

    def test_false_positive_advisory_is_vetoed_and_counted(self):
        source, report = make_mock()
        negative = report["observations"][4]["targets"][0]
        negative["advisory"] = True
        # Trusted precommitted negative oracle prevents candidate promotion.
        negative["candidate_status"] = "conflict"
        result = score_one(source, report, "f" * 64, protocol(), "mistral")
        self.assertTrue(result["passed"])
        self.assertEqual(result["negative_or_unknown_false_positive_advisories"], 1)
        self.assertEqual(result["compatible_advisory_misses"], 3)

    def test_missing_provider_always_fails_overall_gate(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            source, report = make_mock()
            source_path = p / "manifest.json"
            source_path.write_text(json.dumps(source))
            for provider in ("mistral", "google"):
                _, result = make_mock(provider)
                result["corpus_sha256"] = sha256(source_path)
                (p / f"engine-070-independent-{provider}-result.json").write_text(json.dumps(result))
            summary = summarize(source_path, p)
            self.assertFalse(summary["release_gate_passed"])
            self.assertTrue(summary["providers"]["mistral"]["passed"])
            self.assertTrue(summary["providers"]["google"]["passed"])
            self.assertFalse(summary["providers"]["groq"]["passed"])


if __name__ == "__main__":
    unittest.main()
