"""Synthetic tests; never read private Gold, observed responses or scores."""
import unittest
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
        self.assertEqual(paired_no_regression_gate("FN", "TP", True, True)["status"],
                         "PENDING_RATIFICATION")

    def test_pairwise_unresolved_holds(self):
        self.assertIsNone(paired_no_regression_gate("HOLD", "TP", True, True)["regression"])

    def test_reject_non_boolean_fact(self):
        with self.assertRaises(ValueError):
            scenario(mandatory_fact_supported=[True, 1])

    def test_never_live_authorized(self):
        self.assertFalse(scenario()["live_authorized"])


if __name__ == "__main__":
    unittest.main()
