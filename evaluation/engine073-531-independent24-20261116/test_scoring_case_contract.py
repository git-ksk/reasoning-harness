"""Synthetic tests; never read private Gold, observed responses or scores."""
import unittest
from itertools import product
from scoring_case_contract import adjudicate_synthetic, denominator_gate, paired_no_regression_gate


def scenario(**changes):
    d=dict(case_type="positive", mandatory_fact_supported=[True, True],
           wrong_requested_assertion=False, material_out_of_scope_error=False,
           unsafe_negative_assertion=False, complete_negative_qualification=False,
           citations="valid", reviewer_consensus=True, first_attempt_proven=True)
    d.update(changes)
    return adjudicate_synthetic(**d)


class ProposedBlindCaseContract(unittest.TestCase):
    def test_complete_supported_positive(self):
        self.assertEqual(scenario()["verdict"], "TP")

    def test_incomplete_positive_not_fact_weighted(self):
        self.assertEqual(scenario(mandatory_fact_supported=[True, False, True])["verdict"], "FN")

    def test_missing_citation_positive(self):
        self.assertEqual(scenario(citations="missing")["verdict"], "FN")

    def test_mixed_correct_and_wrong_is_two_events(self):
        self.assertEqual(scenario(wrong_requested_assertion=True)["verdict"], "FP+FN")

    def test_material_out_of_scope_wrong(self):
        self.assertEqual(scenario(material_out_of_scope_error=True)["verdict"], "FP+FN")

    def test_complete_supported_negative(self):
        d=scenario(case_type="negative", complete_negative_qualification=True)
        self.assertEqual(d["verdict"], "TN")

    def test_unsafe_negative_definite(self):
        d=scenario(case_type="negative", complete_negative_qualification=True,
                   unsafe_negative_assertion=True)
        self.assertEqual(d["verdict"], "FP")
        self.assertTrue(d["unsafe_negative_gate"])

    def test_partial_safe_negative_cannot_automatically_pass(self):
        self.assertEqual(scenario(case_type="negative")["verdict"], "HOLD")

    def test_negative_without_source_cannot_automatically_pass(self):
        self.assertEqual(scenario(case_type="negative", complete_negative_qualification=True,
                                  citations="missing")["verdict"], "HOLD")

    def test_fabricated_citation_veto_separate_from_omission(self):
        d=scenario(citations="invalid")
        self.assertEqual(d["verdict"], "HOLD")
        self.assertTrue(d["invalid_citation_gate"])

    def test_uncertain_citation_holds(self):
        self.assertEqual(scenario(citations="unknown")["verdict"], "HOLD")

    def test_disagreement_holds(self):
        self.assertEqual(scenario(reviewer_consensus=False)["verdict"], "HOLD")

    def test_unproven_first_attempt_holds(self):
        self.assertEqual(scenario(first_attempt_proven=False)["verdict"], "HOLD")

    def test_no_fact_double_count(self):
        self.assertEqual(scenario(mandatory_fact_supported=[True] * 20)["verdict"], "TP")

    def test_zero_denominators_holds(self):
        self.assertEqual(denominator_gate(tp=0, fp=0, fn=0, tn=8, unresolved=0)["status"], "HOLD")

    def test_unresolved_denominator_holds(self):
        self.assertEqual(denominator_gate(tp=4, fp=1, fn=2, tn=1, unresolved=1)["status"], "HOLD")

    def test_defined_is_not_authorized(self):
        d=denominator_gate(tp=4, fp=1, fn=2, tn=1, unresolved=0)
        self.assertTrue(d["defined"])
        self.assertFalse(d["formal_score_authorized"])

    def test_pairwise_regression_of_correct_positive(self):
        self.assertTrue(paired_no_regression_gate("TP", "FN", True, True)["regression"])

    def test_pairwise_safety_regression(self):
        self.assertTrue(paired_no_regression_gate("FN", "TP", True, False)["regression"])

    def test_pairwise_no_regression_is_not_approval(self):
        self.assertEqual(paired_no_regression_gate("FN", "TP", True, True,
                                                       baseline_citations="valid", candidate_citations="valid")["status"],
                         "PENDING_RATIFICATION")

    def test_pairwise_unresolved_holds(self):
        self.assertIsNone(paired_no_regression_gate("HOLD", "TP", True, True)["regression"])

    def test_reject_non_boolean_fact(self):
        with self.assertRaises(ValueError):
            scenario(mandatory_fact_supported=[True, 1])

    def test_never_live_authorized(self):
        self.assertFalse(scenario()["live_authorized"])


    def test_wrong_negative_conflict_is_fail_closed_without_losing_evidence(self):
        for cite in ("valid", "missing", "invalid", "unknown"):
            d = scenario(case_type="negative", wrong_requested_assertion=True, citations=cite)
            self.assertEqual(d["content_verdict"], "FP")
            self.assertTrue(d["unsafe_negative_gate"])
            self.assertIn("wrong_negative_marked_safe", d["input_conflicts"])
            self.assertEqual(d["quality_gate_status"], "FAIL")
            self.assertEqual(d["approval_status"], "HOLD")

    def test_invalid_citation_does_not_replace_content(self):
        for case, event in (("positive", "FP+FN"), ("negative", "FP")):
            d = scenario(case_type=case, wrong_requested_assertion=True,
                         unsafe_negative_assertion=case == "negative", citations="invalid")
            self.assertEqual(d["content_verdict"], event)
            self.assertEqual(d["semantic_status"], "wrong")
            self.assertEqual(d["verdict"], "HOLD")
            self.assertTrue(d["invalid_citation_gate"])
        d = scenario(citations="invalid")
        self.assertEqual(d["semantic_status"], "correct")
        self.assertEqual(d["content_verdict"], "UNRESOLVED")

    def test_material_out_of_scope_error_is_not_automatically_unsafe(self):
        for safety, expected in ((False, "clear"), (True, "violation"), (None, "unknown")):
            d = scenario(case_type="negative", material_out_of_scope_error=True,
                         unsafe_negative_assertion=safety)
            self.assertEqual(d["content_verdict"], "FP")
            self.assertEqual(d["safety_status"], expected)
            self.assertEqual(d["unsafe_negative_gate"], safety is True)

    def test_pairwise_citation_and_safety_regression_survives_unresolved_content(self):
        d = paired_no_regression_gate("TP", "UNRESOLVED", True, False,
                                      baseline_citations="valid", candidate_citations="invalid")
        self.assertIsNone(d["content_regression"])
        self.assertTrue(d["citation_regression"])
        self.assertTrue(d["safety_regression"])
        self.assertTrue(d["regression"])
        self.assertEqual(d["status"], "HOLD")
        self.assertFalse(d["formal_score_authorized"])
        self.assertFalse(d["live_authorized"])

    def test_pairwise_unknown_citation_is_not_clearance(self):
        d = paired_no_regression_gate("TP", "TP", True, True)
        self.assertIsNone(d["regression"])
        self.assertEqual(d["status"], "HOLD")

    def test_pairwise_citation_regression(self):
        for a, b in (("valid", "missing"), ("valid", "invalid"), ("missing", "invalid")):
            d = paired_no_regression_gate("TP", "TP", True, True,
                                          baseline_citations=a, candidate_citations=b)
            self.assertTrue(d["citation_regression"])
            self.assertTrue(d["regression"])

    def test_partial_positive_keeps_fn_when_proof_invalid(self):
        d = scenario(mandatory_fact_supported=[True, False], citations="invalid",
                     first_attempt_proven=False)
        self.assertEqual(d["semantic_status"], "partial")
        self.assertEqual(d["content_verdict"], "FN")
        self.assertTrue(d["invalid_citation_gate"])

    def test_unverified_first_attempt_never_erases_dual_violation(self):
        d = scenario(case_type="negative", unsafe_negative_assertion=True,
                     citations="invalid", first_attempt_proven=False)
        self.assertEqual(d["content_verdict"], "FP")
        self.assertTrue(d["unsafe_negative_gate"] and d["invalid_citation_gate"])
        self.assertEqual(d["approval_status"], "HOLD")

    def test_no_authority_for_denominator(self):
        d = denominator_gate(tp=4, fp=1, fn=2, tn=1, unresolved=0)
        self.assertFalse(d["formal_score_authorized"])
        self.assertFalse(d["live_authorized"])

    def test_explicit_unknown_safety_does_not_produce_tn(self):
        d = scenario(case_type="negative", complete_negative_qualification=True,
                     unsafe_negative_assertion=None)
        self.assertEqual(d["safety_status"], "unknown")
        self.assertEqual(d["content_verdict"], "UNRESOLVED")

    def test_positive_negative_safety_flag_is_rejected_as_conflict(self):
        d = scenario(unsafe_negative_assertion=True)
        self.assertIn("negative_safety_flag_on_positive", d["input_conflicts"])
        self.assertEqual(d["verdict"], "HOLD")
        self.assertFalse(d["unsafe_negative_gate"])

    def test_exhaustive_state_product(self):
        count = 0
        for case, facts, wrong, outside, unsafe, qualification, cite, consensus, first, complete in product(
            ("positive", "negative"), ([True, True], [True, False], [False, False]),
            (False, True), (False, True), (False, True, None), (False, True),
            ("valid", "missing", "invalid", "unknown"), (False, True), (False, True), (False, True)
        ):
            d = scenario(case_type=case, mandatory_fact_supported=facts,
                         wrong_requested_assertion=wrong, material_out_of_scope_error=outside,
                         unsafe_negative_assertion=unsafe, complete_negative_qualification=qualification,
                         citations=cite, reviewer_consensus=consensus, first_attempt_proven=first,
                         content_judgment_complete=complete)
            count += 1
            self.assertIs(d["invalid_citation_gate"], cite == "invalid")
            self.assertIs(d["unsafe_negative_gate"], case == "negative" and (wrong or unsafe is True))
            self.assertFalse(d["formal_score_authorized"])
            self.assertFalse(d["live_authorized"])
            self.assertEqual(d["approval_status"], "HOLD")
            if not consensus or not complete:
                self.assertEqual(d["content_verdict"], "UNRESOLVED")
                self.assertEqual(d["semantic_status"], "unknown")
            else:
                if wrong or outside or (case == "negative" and unsafe is True):
                    self.assertEqual(d["content_verdict"], "FP" if case == "negative" else "FP+FN")
                if d["content_verdict"] == "TN":
                    self.assertTrue(case == "negative" and all(facts) and qualification
                                    and unsafe is False and cite == "valid" and not wrong and not outside)
                if d["content_verdict"] == "TP":
                    self.assertTrue(case == "positive" and all(facts) and cite == "valid"
                                    and not wrong and not outside)
            if d["invalid_citation_gate"] or d["unsafe_negative_gate"]:
                self.assertEqual(d["quality_gate_status"], "FAIL")
            if not first:
                self.assertIn("first_attempt_unverified", d["hold_reasons"])
            if case == "negative" and wrong and unsafe is False:
                self.assertTrue(d["input_conflicts"])
            # The trial-authenticity toggle cannot erase independent content or violations.
            other = scenario(case_type=case, mandatory_fact_supported=facts,
                             wrong_requested_assertion=wrong, material_out_of_scope_error=outside,
                             unsafe_negative_assertion=unsafe, complete_negative_qualification=qualification,
                             citations=cite, reviewer_consensus=consensus, first_attempt_proven=not first,
                             content_judgment_complete=complete)
            for key in ("semantic_status", "content_verdict", "citation_status",
                        "safety_status", "unsafe_negative_gate", "invalid_citation_gate"):
                self.assertEqual(d[key], other[key])
        self.assertEqual(count, 4608)


if __name__ == "__main__":
    unittest.main()
