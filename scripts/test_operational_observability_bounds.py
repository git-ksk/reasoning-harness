import json
from pathlib import Path
import unittest

from scripts.operational_observability_bounds import (
    CENSORED,
    FAIL,
    HIGHER_IS_BETTER,
    INCONCLUSIVE,
    LOWER_IS_BETTER,
    NON_WORSE,
    NOT_APPLICABLE,
    OBSERVED,
    PASS,
    STRICT_IMPROVEMENT,
    binary_bounds,
    candidate_hard_gate,
    row_safety_gate,
    classify_case,
    compare_bound,
    conditional_rate_bounds,
    paired_classification,
)


def investigation(**overrides):
    case = {
        "id": "case",
        "kind": "investigation",
        "expected": "grounded",
        "target_recalled": True,
        "tool_selection_success": True,
        "target_grounded": True,
        "false_abstention": 0,
        "action_count": 1,
        "coverage_contract_kind": None,
        "correctness_boundary_violations": 0,
        "generation_failure_observed": 0,
        "typed_operational_action_failures": 0,
        "stop_reason": "targets_exhausted",
    }
    case.update(overrides)
    return case


def followup(**overrides):
    case = investigation(
        coverage_contract_kind="no_result_followup_observational",
        avoidable_followup_stall=0,
        trigger_exposed=True,
        continuation_eligible=True,
        mechanism_conformant=True,
        mechanism_followup_status="no_result",
        downstream_followup_useful=False,
    )
    case.update(overrides)
    return case


def classified(*cases):
    return [
        {
            "id": case["id"],
            "observations": {k: v.as_dict() for k, v in classify_case(case).items()},
        }
        for case in cases
    ]


class OperationalObservabilityTests(unittest.TestCase):
    def test_metric_policy_identity_matches_module(self):
        from scripts.operational_observability_bounds import BOUND_IDENTITY, HARD_ZERO_FIELDS

        policy = json.loads(Path("config/natural-language-e2e-metric-v13.json").read_text())
        self.assertEqual(policy["metric_revision"], "v13")
        self.assertEqual(policy["bound_identity"], BOUND_IDENTITY)
        self.assertTrue(policy["prospective_only"])
        self.assertTrue(policy["historical_rescore_forbidden"])
        self.assertFalse(policy["principles"]["asymmetric_diagnostics_scoring_input"])
        self.assertFalse(policy["comparison"]["inconclusive_releases"])
        self.assertEqual(tuple(policy["hard_zero_fields"]), HARD_ZERO_FIELDS)
        self.assertTrue(policy["principles"]["operational_terminal_requires_semantic_case_envelope"])

    def test_clean_success_and_semantic_miss_are_observed(self):
        success = classify_case(investigation())
        miss = classify_case(
            investigation(
                target_recalled=False,
                tool_selection_success=False,
                target_grounded=False,
                false_abstention=1,
            )
        )
        self.assertEqual(success["target_recalled"].state, OBSERVED)
        self.assertEqual(miss["target_recalled"].state, OBSERVED)
        self.assertFalse(miss["tool_selection_success"].value)
        self.assertTrue(miss["false_abstention"].value)

    def test_target_recall_survives_protocol_terminal_but_later_metrics_censor(self):
        obs = classify_case(
            investigation(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                target_recalled=True,
                tool_selection_success=False,
                target_grounded=False,
                false_abstention=1,
            )
        )
        self.assertEqual(obs["target_recalled"].state, OBSERVED)
        self.assertTrue(obs["target_recalled"].value)
        self.assertEqual(obs["tool_selection_success"].state, CENSORED)
        self.assertEqual(obs["false_abstention"].state, CENSORED)
        self.assertEqual(obs["target_grounded"].state, CENSORED)

    def test_positive_tool_witness_survives_later_terminal(self):
        obs = classify_case(
            investigation(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                tool_selection_success=True,
            )
        )
        self.assertEqual(obs["tool_selection_success"].state, OBSERVED)
        self.assertTrue(obs["tool_selection_success"].value)

    def test_no_tool_before_terminal_is_censored(self):
        obs = classify_case(
            investigation(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                tool_selection_success=False,
            )
        )
        self.assertEqual(obs["tool_selection_success"].state, CENSORED)

    def test_trigger_witness_survives_later_terminal(self):
        obs = classify_case(
            followup(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                trigger_exposed=True,
            )
        )
        self.assertEqual(obs["trigger_exposed"].state, OBSERVED)
        self.assertTrue(obs["trigger_exposed"].value)

    def test_observed_non_no_result_first_relevant_action_proves_trigger_false(self):
        obs = classify_case(
            followup(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                trigger_exposed=False,
                trigger_first_relevant_capability="cache",
                trigger_first_relevant_status="applied_evidence",
                continuation_eligible=False,
            )
        )
        self.assertEqual(obs["trigger_exposed"].state, OBSERVED)
        self.assertFalse(obs["trigger_exposed"].value)
        self.assertEqual(obs["continuation_eligible"].state, NOT_APPLICABLE)

    def test_no_trigger_before_terminal_censors_dependents(self):
        obs = classify_case(
            followup(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                trigger_exposed=False,
                continuation_eligible=False,
                mechanism_conformant=None,
                mechanism_followup_status=None,
            )
        )
        self.assertEqual(obs["trigger_exposed"].state, CENSORED)
        self.assertEqual(obs["continuation_eligible"].state, CENSORED)
        self.assertEqual(obs["mechanism_conformant"].state, CENSORED)

    def test_action_disproves_zero_action_stall_despite_terminal(self):
        obs = classify_case(
            followup(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                action_count=2,
                avoidable_followup_stall=0,
            )
        )
        self.assertEqual(obs["avoidable_followup_stall"].state, OBSERVED)
        self.assertFalse(obs["avoidable_followup_stall"].value)

    def test_zero_action_terminal_does_not_become_semantic_stall(self):
        obs = classify_case(
            followup(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                action_count=0,
                avoidable_followup_stall=1,
            )
        )
        self.assertEqual(obs["avoidable_followup_stall"].state, CENSORED)

    def test_correctness_positive_witness_survives_terminal(self):
        obs = classify_case(
            investigation(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                correctness_boundary_violations=1,
            )
        )
        self.assertEqual(obs["correctness_boundary_violation"].state, OBSERVED)
        self.assertTrue(obs["correctness_boundary_violation"].value)

    def test_correctness_zero_before_terminal_is_censored(self):
        obs = classify_case(
            investigation(
                stop_reason="operational_terminal",
                generation_failure_observed=1,
                correctness_boundary_violations=0,
            )
        )
        self.assertEqual(obs["correctness_boundary_violation"].state, CENSORED)

    def test_false_abstention_keeps_locked_session_and_unknown_case_semantics(self):
        unknown = classify_case(investigation(expected="unknown", false_abstention=0))
        session = classify_case(
            {
                "id": "session",
                "kind": "session_add",
                "target_grounded": False,
                "false_abstention": 1,
                "correctness_boundary_violations": 0,
            }
        )
        self.assertEqual(unknown["false_abstention"].state, OBSERVED)
        self.assertFalse(unknown["false_abstention"].value)
        self.assertEqual(session["false_abstention"].state, OBSERVED)
        self.assertTrue(session["false_abstention"].value)
        self.assertEqual(session["target_recalled"].state, NOT_APPLICABLE)

    def test_fixed_denominator_binary_bounds(self):
        a = investigation(id="a", tool_selection_success=True)
        b = investigation(
            id="b",
            tool_selection_success=False,
            stop_reason="operational_terminal",
            generation_failure_observed=1,
        )
        c = investigation(id="c", tool_selection_success=False)
        bounds = binary_bounds(classified(a, b, c), "tool_selection_success", as_rate=True)
        self.assertAlmostEqual(bounds["lower_bound"], 1 / 3)
        self.assertAlmostEqual(bounds["upper_bound"], 2 / 3)
        self.assertEqual(bounds["censored_case_ids"], ["b"])

    def test_conditional_rate_bounds_with_censored_denominator(self):
        a = followup(id="a", continuation_eligible=True, mechanism_conformant=True)
        b = followup(
            id="b",
            stop_reason="operational_terminal",
            generation_failure_observed=1,
            trigger_exposed=False,
            continuation_eligible=False,
            mechanism_conformant=None,
            mechanism_followup_status=None,
        )
        bounds = conditional_rate_bounds(
            classified(a, b), "continuation_eligible", "mechanism_conformant"
        )
        self.assertEqual(bounds["lower_bound"], 0.5)
        self.assertEqual(bounds["upper_bound"], 1.0)
        self.assertFalse(bounds["undefined_possible"])

    def test_higher_is_better_tri_state(self):
        bound = {"lower_bound": 0.6, "upper_bound": 0.8}
        self.assertEqual(
            compare_bound(0.8, bound, direction=HIGHER_IS_BETTER, requirement=NON_WORSE)["classification"],
            PASS,
        )
        self.assertEqual(
            compare_bound(0.7, bound, direction=HIGHER_IS_BETTER, requirement=NON_WORSE)["classification"],
            INCONCLUSIVE,
        )
        self.assertEqual(
            compare_bound(0.5, bound, direction=HIGHER_IS_BETTER, requirement=NON_WORSE)["classification"],
            FAIL,
        )
        self.assertEqual(
            compare_bound(0.8, bound, direction=HIGHER_IS_BETTER, requirement=STRICT_IMPROVEMENT)["classification"],
            INCONCLUSIVE,
        )
        self.assertEqual(
            compare_bound(0.9, bound, direction=HIGHER_IS_BETTER, requirement=STRICT_IMPROVEMENT)["classification"],
            PASS,
        )

    def test_lower_is_better_tri_state(self):
        bound = {"lower_bound": 2, "upper_bound": 5}
        self.assertEqual(compare_bound(2, bound, direction=LOWER_IS_BETTER, requirement=NON_WORSE)["classification"], PASS)
        self.assertEqual(compare_bound(4, bound, direction=LOWER_IS_BETTER, requirement=NON_WORSE)["classification"], INCONCLUSIVE)
        self.assertEqual(compare_bound(6, bound, direction=LOWER_IS_BETTER, requirement=NON_WORSE)["classification"], FAIL)
        self.assertEqual(compare_bound(1, bound, direction=LOWER_IS_BETTER, requirement=STRICT_IMPROVEMENT)["classification"], PASS)

    def test_candidate_operational_and_all_safety_zero_failures_are_hard(self):
        self.assertFalse(candidate_hard_gate({"aggregate": {"operational_failures": 1}})["passed"])
        for field in (
            "correctness_boundary_violations",
            "unsupported_structured_claims",
            "unsupported_exposed_assertions",
            "exposed_text_contract_violations",
            "missed_target_insufficiency",
            "identity_unsafe_admission",
            "mcp_output_authority_self_promotion",
            "session_external_calls_replayed",
            "duplicate_action_rejections",
        ):
            report = {"aggregate": {"operational_failures": 0, field: 1}}
            self.assertFalse(candidate_hard_gate(report)["passed"], field)
            self.assertFalse(row_safety_gate(report)["passed"], field)

    def test_diagnostic_sidecar_cannot_be_used_as_scoring_input(self):
        from scripts.operational_observability_bounds import report_bounds

        with self.assertRaises(ValueError):
            report_bounds(
                {
                    "schema_version": "natural-language-e2e-v29-diagnostic-validation-v1",
                    "scoring_input": False,
                    "cases": [],
                    "aggregate": {},
                }
            )

    def test_operational_terminal_without_semantic_envelope_fails_closed(self):
        with self.assertRaises(ValueError):
            classify_case({"id": "lost", "operational_failure": {"failure_class": "process"}})
        with self.assertRaises(ValueError):
            classify_case(
                {
                    "id": "lost-coverage",
                    "kind": "investigation",
                    "operational_failure": {"failure_class": "process"},
                }
            )

    def test_observed_control_safety_violation_is_hard_fail(self):
        control = {
            "cases": [investigation(id="c")],
            "aggregate": {"operational_failures": 0, "unsupported_structured_claims": 1},
        }
        candidate = {
            "cases": [investigation(id="c")],
            "aggregate": {"operational_failures": 0},
        }
        result = paired_classification(control, candidate, [])
        self.assertEqual(result["classification"], FAIL)
        self.assertIn("control", result["reason"])

    def test_required_inconclusive_comparison_blocks_pass(self):
        control_case = investigation(
            id="c",
            target_recalled=False,
            stop_reason="operational_terminal",
            generation_failure_observed=1,
        )
        candidate_case = investigation(id="c", target_recalled=True)
        control = {
            "cases": [control_case],
            "aggregate": {"operational_failures": 1, "correctness_boundary_violations": 0},
        }
        candidate = {
            "cases": [candidate_case],
            "aggregate": {"operational_failures": 0, "correctness_boundary_violations": 0},
        }
        result = paired_classification(
            control,
            candidate,
            [{"metric": "target_recall", "direction": HIGHER_IS_BETTER, "requirement": STRICT_IMPROVEMENT}],
        )
        self.assertEqual(result["classification"], INCONCLUSIVE)
        self.assertEqual(result["comparisons"][0]["classification"], INCONCLUSIVE)


if __name__ == "__main__":
    unittest.main()
