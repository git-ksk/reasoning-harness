#!/usr/bin/env python3
import importlib.util
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "planner_reliability_v1", ROOT / "scripts/planner_reliability_v1.py"
)
M = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(M)

FIXTURES = ROOT / "fixtures/planner-reliability-v1"

def complete_case(role, success=True):
    return {
        "id": role,
        "role": role,
        "operationally_complete": True,
        "planner_success": success,
        "target_recalled": True,
        "tool_selection_success": True,
        "valid_action_shape": True,
        "action_rejections": {},
        "action_rejection_count": 0,
        "invalid_shape_rejections": 0,
        "avoidable_followup_stall": 0,
        "false_abstention": 0,
        "correctness_boundary_violations": 0,
        "trigger_exposed": False if role == "no_result_followup" else None,
        "mechanism_conformant": False if role == "no_result_followup" else None,
        "stop_reason": "resolved_supported",
        "observed_model_identities": {"model-x": 1},
        "provider_calls_observed": 1,
        "provider_attempts_observed": 1,
        "tokens_observed": 10,
        "provider_latency_ms_observed": 20,
        "wall_clock_ms": 30,
        "typed_operational_failure_classes": {},
    }

def make_trial(seed, success=True, complete=True):
    cases = [
        complete_case("direct_grounded", success),
        complete_case("no_result_followup", success),
        complete_case("stale_unknown", success),
    ]
    if not complete:
        cases[0] = {
            "id": "direct_grounded",
            "role": "direct_grounded",
            "operationally_complete": False,
            "planner_success": None,
            "operational_failure": {
                "failure_class": "transport_failure",
                "exit_code": 1,
            },
            "wall_clock_ms": 1,
        }
    trial = {
        "trial_id": f"surface-a-seed-{seed}",
        "surface": "surface-a",
        "seed": seed,
        "cases": cases,
    }
    trial["summary"] = M.summarize_trial(trial)
    return trial

class PlannerReliabilityV1Tests(unittest.TestCase):
    def test_corpus_contract_and_fixture_preflight(self):
        manifest, surfaces = M.validate_corpus(FIXTURES)
        self.assertEqual(manifest["trial_plan"]["declared_k"], [1, 5])
        self.assertEqual(
            manifest["trial_plan"]["primary_trial_seeds"],
            [86101, 86102, 86103, 86104, 86105],
        )
        self.assertEqual(set(surfaces), {"surface-a", "surface-b"})
        preflight = M.self_test_fixtures(FIXTURES, surfaces)
        self.assertEqual(preflight["capabilities_checked"], 18)
        self.assertEqual(preflight["followup_sequence_contracts"], 2)
        self.assertEqual(preflight["stale_rejection_contracts"], 2)

    def test_complete_trial_distribution_and_all_k_are_observed_not_iid_derived(self):
        manifest, _ = M.validate_corpus(FIXTURES)
        trials = [make_trial(seed) for seed in manifest["trial_plan"]["primary_trial_seeds"]]
        aggregate = M.aggregate_primary(trials, manifest)
        self.assertEqual(aggregate["complete_trials"], 5)
        self.assertEqual(aggregate["planner_success_denominator"], 5)
        self.assertEqual(aggregate["pass_at_1"], 1.0)
        self.assertTrue(aggregate["all_k"]["defined"])
        self.assertTrue(aggregate["all_k"]["observed_all_k_success"])
        self.assertIsNone(aggregate["all_k"]["derived_independence_p_to_k"])
        self.assertEqual(
            aggregate["all_k"]["derived_independence_p_to_k_reason"],
            "not_reported_no_iid_assumption",
        )

    def test_incomplete_trial_is_excluded_from_semantic_denominator(self):
        manifest, _ = M.validate_corpus(FIXTURES)
        seeds = manifest["trial_plan"]["primary_trial_seeds"]
        trials = [make_trial(seed) for seed in seeds]
        trials[2] = make_trial(seeds[2], complete=False)
        aggregate = M.aggregate_primary(trials, manifest)
        self.assertEqual(aggregate["attempted_trials"], 5)
        self.assertEqual(aggregate["complete_trials"], 4)
        self.assertEqual(aggregate["planner_success_denominator"], 4)
        self.assertEqual(aggregate["pass_at_1"], 1.0)
        self.assertFalse(aggregate["all_k"]["defined"])
        self.assertIsNone(aggregate["all_k"]["observed_all_k_success"])
        self.assertEqual(
            aggregate["operational_failure_classes"]["transport_failure"], 1
        )

    def test_utility_failure_changes_pass_at_1_without_invalidating_completion(self):
        manifest, _ = M.validate_corpus(FIXTURES)
        seeds = manifest["trial_plan"]["primary_trial_seeds"]
        trials = [make_trial(seed) for seed in seeds]
        trials[1] = make_trial(seeds[1], success=False)
        aggregate = M.aggregate_primary(trials, manifest)
        self.assertEqual(aggregate["complete_trials"], 5)
        self.assertEqual(aggregate["pass_at_1"], 0.8)
        self.assertTrue(aggregate["all_k"]["defined"])
        self.assertFalse(aggregate["all_k"]["observed_all_k_success"])

    def test_score_case_rejection_is_planner_failure_but_not_correctness_failure(self):
        _, surfaces = M.validate_corpus(FIXTURES)
        case = surfaces["surface-a"][0]
        target = case["target"]
        relevant = case["relevant_capabilities"][0]
        result = {
            "output_contract": "reason-natural-output-v4",
            "final_outcome": {
                "artifact": {
                    "claims": [
                        {
                            "proposition": {
                                "key": target["key"],
                                "value": target["value"],
                            },
                            "state": "supported",
                        }
                    ]
                }
            },
            "finalization": {
                "status": "grounded_answer",
                "text": f"{target['key']} = {target['value']}",
                "factual_claims": 1,
                "covered_claims": 1,
            },
            "investigation": {
                "telemetry": {
                    "targets": [{"expected_fact_key": target["key"]}],
                    "actions": [
                        {
                            "round": 1,
                            "action": {"capability_id": relevant},
                            "status": "applied_evidence",
                        }
                    ],
                    "rejected_actions": {},
                    "action_rejection_records": [],
                    "planner_calls": 2,
                    "stop_reason": "resolved_supported",
                }
            },
            "resolution_rounds": [],
        }
        scored = M.score_case(case, result, 1)
        self.assertTrue(scored["planner_success"])
        self.assertEqual(scored["correctness_boundary_violations"], 0)

        result["investigation"]["telemetry"]["rejected_actions"] = {"invalid_shape": 1}
        scored = M.score_case(case, result, 1)
        self.assertFalse(scored["valid_action_shape"])
        self.assertFalse(scored["planner_success"])
        self.assertEqual(scored["correctness_boundary_violations"], 0)

    def test_surface_information_equivalence_drift_fails_closed(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td) / "fixtures" / "planner-reliability-v1"
            shutil.copytree(FIXTURES, root)
            config_path = root / "surface-b/configs/vestril-direct.json"
            config = json.loads(config_path.read_text())
            duplicate = dict(config["resolution"]["investigation"]["capabilities"][-1])
            duplicate["id"] = "vestril-extra-distractor"
            config["resolution"]["investigation"]["capabilities"].append(duplicate)
            config_path.write_text(json.dumps(config, indent=2) + "\n")
            with self.assertRaisesRegex(M.EvalError, "information-equivalence"):
                M.validate_corpus(root)

    def test_historical_fresh_marker_reuse_fails_closed(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td) / "fixtures" / "planner-reliability-v1"
            shutil.copytree(FIXTURES, root)
            historical = Path(td) / "fixtures" / "historical"
            historical.mkdir(parents=True)
            (historical / "old.json").write_text(
                json.dumps({"marker": "thalvex.mesh.primary_endpoint"}) + "\n"
            )
            with self.assertRaisesRegex(M.EvalError, "fresh marker reused"):
                M.validate_corpus(root)

    def test_matched_surface_pair_is_descriptive_only(self):
        manifest, _ = M.validate_corpus(FIXTURES)
        seeds = manifest["trial_plan"]["primary_trial_seeds"]
        primary = [make_trial(seed) for seed in seeds]
        matched = make_trial(86105)
        matched["trial_id"] = "surface-b-seed-86105"
        matched["surface"] = "surface-b"
        matched["summary"]["action_rejection_count"] = 2
        pair = M.surface_pair(primary, matched, manifest)
        self.assertTrue(pair["pair_complete"])
        self.assertEqual(
            pair["metric_deltas_surface_b_minus_surface_a"]["action_rejection_count"],
            2,
        )
        self.assertIn("descriptive_only", pair["claim_scope"])

if __name__ == "__main__":
    unittest.main()
