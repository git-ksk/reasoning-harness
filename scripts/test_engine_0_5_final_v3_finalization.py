import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("engine_0_5_final_v3_finalization.py")
SPEC = importlib.util.spec_from_file_location("engine050v3_finalization", SCRIPT)
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class Engine050V3FinalizationTests(unittest.TestCase):
    def test_fresh_corpus_and_matrix_validate(self):
        manifest, cases = MOD.validate_corpus()
        self.assertEqual(manifest["corpus_identity"], MOD.CORPUS_ID)
        self.assertEqual(len(manifest["provider_targets"]), 6)
        self.assertEqual(
            [case["kind"] for case in cases],
            ["investigation", "investigation", "session_correct"],
        )
        self.assertEqual(cases[0]["expected"], "grounded")
        self.assertEqual(cases[1]["expected"], "unknown")
        self.assertNotIn("hypothesis", cases[0])
        self.assertNotIn("hypothesis", cases[1])

    def test_grounded_exposure_requires_artifact_support(self):
        target = {"key": "fresh.key", "value": "fresh.value"}
        artifact = {"claims": [{"state": "supported", "proposition": target}]}
        grounded, unsupported, assertions = MOD.target_exposure(
            {"text": "fresh.key = fresh.value"}, artifact, target
        )
        self.assertTrue(grounded)
        self.assertEqual(unsupported, 0)
        self.assertEqual(len(assertions), 1)

    def test_unsupported_grounded_exposure_is_counted(self):
        target = {"key": "fresh.key", "value": "fresh.value"}
        grounded, unsupported, _ = MOD.target_exposure(
            {"text": "fresh.key = fresh.value"}, {"claims": []}, target
        )
        self.assertTrue(grounded)
        self.assertEqual(unsupported, 1)


    def grounded_result(self, *, target_recalled):
        target = {"key": "fresh.key", "value": "fresh.value"}
        targets = ([{"expected_fact_key": target["key"]}] if target_recalled else [])
        return (
            {"id": "grounded", "kind": "investigation", "expected": "grounded", "target": target},
            {
                "output_contract": "reason-natural-output-v4",
                "final_outcome": {
                    "artifact": {
                        "hypotheses": [],
                        "claims": [
                            {
                                "id": "harness_investigation_admitted_fact_1",
                                "state": "supported",
                                "proposition": target,
                            }
                        ],
                    }
                },
                "finalization": {
                    "status": "grounded_answer",
                    "text": "fresh.key = fresh.value",
                },
                "investigation": {
                    "telemetry": {
                        "targets": targets,
                        "actions": [{"admitted_evidence": 1}],
                        "stop_reason": "resolved",
                    }
                },
            },
        )

    def test_finalization_correctness_does_not_require_planner_target_recall(self):
        case, result = self.grounded_result(target_recalled=False)
        scored = MOD.investigation_result(case, result, 1)
        self.assertFalse(scored["target_recalled"])
        self.assertTrue(scored["artifact_exact_supported"])
        self.assertTrue(scored["harness_materialized_investigation_target"])
        self.assertTrue(scored["target_grounded"])
        self.assertEqual(scored["unsupported_exposed_assertions"], 0)
        self.assertTrue(scored["passed"])

    def test_grounded_investigation_requires_harness_owned_materialization(self):
        case, result = self.grounded_result(target_recalled=False)
        result["final_outcome"]["artifact"]["claims"][0]["id"] = "model-only"
        scored = MOD.investigation_result(case, result, 1)
        self.assertTrue(scored["artifact_exact_supported"])
        self.assertFalse(scored["harness_materialized_investigation_target"])
        self.assertFalse(scored["passed"])

    def test_target_recall_remains_observable_when_present(self):
        case, result = self.grounded_result(target_recalled=True)
        scored = MOD.investigation_result(case, result, 1)
        self.assertTrue(scored["target_recalled"])
        self.assertTrue(scored["passed"])

    def test_finalization_still_fails_without_exact_artifact_support(self):
        case, result = self.grounded_result(target_recalled=False)
        result["final_outcome"]["artifact"]["claims"] = []
        scored = MOD.investigation_result(case, result, 1)
        self.assertFalse(scored["artifact_exact_supported"])
        self.assertGreater(scored["unsupported_exposed_assertions"], 0)
        self.assertFalse(scored["passed"])

    def test_finalization_still_fails_without_grounded_exposure(self):
        case, result = self.grounded_result(target_recalled=False)
        result["finalization"]["text"] = "uncertain(fresh.key = fresh.value)"
        scored = MOD.investigation_result(case, result, 1)
        self.assertTrue(scored["artifact_exact_supported"])
        self.assertFalse(scored["target_grounded"])
        self.assertFalse(scored["passed"])



    def test_session_start_identity_uses_persisted_explicit_fact_not_model_claim(self):
        target = {"key": "session.fresh", "value": "old"}
        artifact = {
            "claims": [],
            "evidence": [
                {
                    "metadata": {"provenance_class": "explicit_user_fact"},
                    "facts": {"session.fresh": "old"},
                }
            ],
        }
        self.assertFalse(MOD.proposition_supported(artifact, target))
        self.assertTrue(MOD.exact_explicit_user_fact_persisted(artifact, target))

    def test_session_start_identity_fails_closed_on_conflicting_explicit_facts(self):
        target = {"key": "session.fresh", "value": "old"}
        artifact = {
            "evidence": [
                {
                    "metadata": {"provenance_class": "explicit_user_fact"},
                    "facts": {"session.fresh": "old"},
                },
                {
                    "metadata": {"provenance_class": "explicit_user_fact"},
                    "facts": {"session.fresh": "other"},
                },
            ]
        }
        self.assertFalse(MOD.exact_explicit_user_fact_persisted(artifact, target))

    def test_harness_owned_correction_target_is_observable_only_after_support(self):
        target = {"key": "session.fresh", "value": "new"}
        supported = {
            "claims": [
                {
                    "id": "harness_session_correction_target_0",
                    "state": "supported",
                    "proposition": target,
                }
            ]
        }
        assumed = {
            "claims": [
                {
                    "id": "harness_session_correction_target_0",
                    "state": "assumed",
                    "proposition": target,
                }
            ]
        }
        model_only = {
            "claims": [
                {
                    "id": "model-proposal",
                    "state": "supported",
                    "proposition": target,
                }
            ]
        }
        self.assertTrue(MOD.harness_owned_correction_target_supported(supported, target))
        self.assertFalse(MOD.harness_owned_correction_target_supported(assumed, target))
        self.assertFalse(MOD.harness_owned_correction_target_supported(model_only, target))


    def test_harness_owned_investigation_target_requires_supported_exact_claim(self):
        target = {"key": "fresh.key", "value": "fresh.value"}
        supported = {"claims": [{"id": "harness_investigation_admitted_fact_1", "state": "supported", "proposition": target}]}
        assumed = {"claims": [{"id": "harness_investigation_admitted_fact_1", "state": "assumed", "proposition": target}]}
        model_only = {"claims": [{"id": "model-only", "state": "supported", "proposition": target}]}
        self.assertTrue(MOD.harness_owned_investigation_target_supported(supported, target))
        self.assertFalse(MOD.harness_owned_investigation_target_supported(assumed, target))
        self.assertFalse(MOD.harness_owned_investigation_target_supported(model_only, target))



if __name__ == "__main__":
    unittest.main()
