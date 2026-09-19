import importlib.util
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

SCRIPT = Path(__file__).with_name("engine_0_5_final_materialization.py")
SPEC = importlib.util.spec_from_file_location("engine050_materialization", SCRIPT)
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)
ROOT = Path(__file__).resolve().parents[1] / "fixtures" / "engine-0.5-final-v1-materialization"


class Engine050MaterializationTests(unittest.TestCase):
    def test_fresh_surface_validates_and_preflight_is_mechanical(self):
        manifest, cases = MOD.validate_corpus(ROOT)
        self.assertEqual(len(manifest["provider_targets"]), 8)
        self.assertEqual(len(cases), 2)
        preflight = MOD.self_test_fixtures(ROOT, cases)
        self.assertEqual(preflight["capabilities_checked"], 4)
        self.assertEqual(preflight["target_aware_identity_checks"], 2)
        self.assertEqual(preflight["stale_rejection_contracts"], 1)

    def base_summary(self):
        return {
            "operationally_complete": True,
            "action_path_success": True,
            "correctness_boundary_violations": 0,
            "same_key_sibling_exposed_cases": 2,
            "candidate_path_conformant_cases": 2,
            "legacy_action_planner_calls": 0,
            "intent_planner_calls": 3,
            "harness_intent_materializations": 2,
            "intent_rejection_count": 0,
            "action_rejection_count": 0,
            "downstream_finalization": {"false_abstention_cases": 2},
        }

    def policy(self):
        manifest, _ = MOD.validate_corpus(ROOT)
        return manifest["acceptance"]

    def test_candidate_acceptance_requires_materialization_path(self):
        result = MOD.candidate_acceptance(self.base_summary(), self.policy())
        self.assertTrue(result["acceptance_passed"])
        broken = self.base_summary()
        broken["legacy_action_planner_calls"] = 1
        self.assertFalse(MOD.candidate_acceptance(broken, self.policy())["acceptance_passed"])

    def test_correctness_or_rejection_regression_fails(self):
        broken = self.base_summary()
        broken["correctness_boundary_violations"] = 1
        self.assertFalse(MOD.candidate_acceptance(broken, self.policy())["acceptance_passed"])
        broken = self.base_summary()
        broken["action_rejection_count"] = 1
        self.assertFalse(MOD.candidate_acceptance(broken, self.policy())["acceptance_passed"])

    def test_downstream_finalization_is_recorded_not_gated(self):
        summary = self.base_summary()
        summary["downstream_finalization"] = {
            "false_abstention_cases": 99,
            "status_counts": {"requires_verification": 2},
        }
        self.assertTrue(MOD.candidate_acceptance(summary, self.policy())["acceptance_passed"])

    def test_incomplete_trial_fails(self):
        broken = self.base_summary()
        broken["operationally_complete"] = False
        broken["action_path_success"] = None
        self.assertFalse(MOD.candidate_acceptance(broken, self.policy())["acceptance_passed"])


if __name__ == "__main__":
    unittest.main()
