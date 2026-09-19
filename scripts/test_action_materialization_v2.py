import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import action_materialization_v2 as evaluator


def trial(coordinate, seed, *, success=True, legacy=0, intent=0, materialized=0,
          intent_rejections=0, action_rejections=0, correctness=0):
    return {
        "trial_id": f"{coordinate}-seed-{seed}",
        "coordinate": coordinate,
        "seed": seed,
        "cases": [],
        "summary": {
            "operationally_complete": True,
            "planner_success": success,
            "case_count": 2,
            "same_key_sibling_exposed_cases": 2,
            "control_path_exposed_cases": 2 if coordinate == "control" and legacy > 0 else 0,
            "candidate_path_conformant_cases": 2 if coordinate == "candidate" and intent > 0 and materialized > 0 and legacy == 0 and intent_rejections == 0 else 0,
            "legacy_action_planner_calls": legacy,
            "intent_planner_calls": intent,
            "harness_intent_materializations": materialized,
            "intent_rejection_count": intent_rejections,
            "action_rejection_count": action_rejections,
            "correctness_boundary_violations": correctness,
            "provider_calls_observed": 1,
            "provider_attempts_observed": 1,
            "tokens_observed": 10,
            "provider_latency_ms_observed": 5,
            "wall_clock_ms": 6,
            "observed_model_identities": {"fixture-model": 1},
            "operational_failure_classes": {},
        },
    }


class ActionMaterializationV2Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.root = (
            Path(__file__).resolve().parents[1]
            / "fixtures"
            / "action-materialization-v2"
        )
        cls.manifest, cls.cases = evaluator.validate_corpus(cls.root)

    def test_fresh_surface_validates_and_fixture_preflight_is_mechanical(self):
        summary = evaluator.self_test_fixtures(self.root, self.cases)
        self.assertEqual(summary["capabilities_checked"], 4)
        self.assertEqual(summary["positive_evidence_capabilities"], 1)
        self.assertEqual(summary["no_result_capabilities"], 2)
        self.assertEqual(summary["stale_rejection_contracts"], 1)
        self.assertEqual(summary["target_aware_identity_checks"], 2)

    def test_paired_acceptance_requires_candidate_path_and_strict_call_reduction(self):
        seeds = self.manifest["trial_plan"]["primary_trial_seeds"]
        control = [
            trial("control", seed, legacy=2)
            for seed in seeds
        ]
        candidate = [
            trial("candidate", seed, intent=2, materialized=2)
            for seed in seeds
        ]
        result = evaluator.paired_acceptance(
            control, candidate, self.manifest
        )
        self.assertTrue(result["acceptance_passed"])
        self.assertTrue(all(result["checks"].values()))
        self.assertEqual(
            result["candidate"]["legacy_action_planner_calls"], 0
        )
        self.assertGreater(
            result["control"]["legacy_action_planner_calls"], 0
        )

    def test_candidate_fallback_or_correctness_regression_fails_acceptance(self):
        seeds = self.manifest["trial_plan"]["primary_trial_seeds"]
        control = [
            trial("control", seed, legacy=2)
            for seed in seeds
        ]
        candidate = [
            trial(
                "candidate",
                seed,
                legacy=1 if index == 0 else 0,
                intent=2,
                materialized=2,
                correctness=1 if index == 1 else 0,
            )
            for index, seed in enumerate(seeds)
        ]
        result = evaluator.paired_acceptance(
            control, candidate, self.manifest
        )
        self.assertFalse(result["acceptance_passed"])
        self.assertFalse(
            result["checks"]["candidate_legacy_action_path_eliminated"]
        )
        self.assertFalse(result["checks"]["candidate_correctness_zero"])

    def test_incomplete_trial_does_not_enter_semantic_denominator(self):
        seeds = self.manifest["trial_plan"]["primary_trial_seeds"]
        trials = [
            trial("candidate", seed, intent=1, materialized=1)
            for seed in seeds
        ]
        trials[-1]["summary"]["operationally_complete"] = False
        trials[-1]["summary"]["planner_success"] = None
        aggregate = evaluator.aggregate(trials, self.manifest)
        self.assertEqual(aggregate["attempted_trials"], 5)
        self.assertEqual(aggregate["complete_trials"], 4)
        self.assertEqual(aggregate["planner_success_denominator"], 4)
        self.assertFalse(aggregate["all_k"]["defined"])
        self.assertIsNone(aggregate["all_k"]["derived_independence_p_to_k"])


if __name__ == "__main__":
    unittest.main()
