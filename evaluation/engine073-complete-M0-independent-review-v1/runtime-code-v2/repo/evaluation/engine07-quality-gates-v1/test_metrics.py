"""Offline regression checks for the independent quality threshold calculator."""
import unittest

from metrics import configured_policy, score_quality


def rows(tp=8, fn=0, fp=0, tn=16, ambiguous=0):
    return ([{"id": f"p{i}", "provider": "mistral", "track": "e2e_normal", "arm": "candidate",
              "expected": "positive", "decision": decision,
              "complete": True, "hard_violations": []}
             for i, decision in enumerate(["accept"] * tp + ["abstain"] * fn)]
            + [{"id": f"n{i}", "provider": "mistral", "track": "e2e_normal", "arm": "candidate",
                "expected": "negative", "decision": decision,
                "complete": True, "hard_violations": []}
               for i, decision in enumerate(["accept"] * fp + ["withhold"] * tn)]
            + [{"id": f"a{i}", "provider": "mistral", "track": "e2e_normal", "arm": "candidate",
                "expected": "ambiguous", "decision": "abstain",
                "complete": True, "hard_violations": []}
               for i in range(ambiguous)])


class CandidateQuality(unittest.TestCase):
    def setUp(self):
        self.policy = configured_policy()

    def test_reference_thresholds_and_separate_conjunction(self):
        self.assertEqual((self.policy["minimum_f1"], self.policy["minimum_precision"],
                          self.policy["minimum_recall"]), (0.90, 0.95, 0.85))
        self.assertLess(2 * .95 * .85 / (.95 + .85), .90)
        self.assertTrue(score_quality(rows(tp=8, tn=16), self.policy)["metric_gate_pass"])
        self.assertTrue(score_quality(rows(tp=7, fn=1, tn=16), self.policy)["metric_gate_pass"])
        self.assertEqual(score_quality(rows(tp=7, fn=1, tn=16), self.policy)["fn"], 1)

    def test_zero_positive_predictions_cannot_pass(self):
        s = score_quality(rows(tp=0, fn=8, tn=16), self.policy)
        self.assertEqual(s["f1"], 0)
        self.assertIsNone(s["precision"])
        self.assertIn("undefined_precision_no_positive_predictions", s["blockers"])
        self.assertFalse(s["metric_gate_pass"])

    def test_false_accept_fails_precision(self):
        s = score_quality(rows(tp=8, tn=15, fp=1), self.policy)
        self.assertEqual(s["fp"], 1)
        self.assertIn("precision_below_minimum", s["blockers"])

    def test_missing_or_failed_cannot_be_silent_negative(self):
        s = score_quality(rows(tp=8, tn=15), self.policy)
        self.assertIn("missing_or_extra_observations", s["blockers"])
        incomplete = rows(tp=8, tn=16)
        incomplete[0]["complete"] = False
        s = score_quality(incomplete, self.policy)
        self.assertIn("operationally_incomplete", s["blockers"])
        self.assertEqual(s["tp"], 7)
        self.assertEqual(s["incomplete"], 1)

    def test_hard_safety_gates_separate_from_f1(self):
        values = rows(tp=8, tn=16)
        values[0]["hard_violations"] = ["misbound_citation"]
        s = score_quality(values, self.policy)
        self.assertEqual(s["f1"], 1.0)
        self.assertIn("hard_safety_violation", s["blockers"])

    def test_ambiguous_accept_is_not_ignored(self):
        values = rows(tp=8, tn=15, ambiguous=1)
        self.assertTrue(score_quality(values, self.policy)["metric_gate_pass"])
        values[-1]["decision"] = "accept"
        s = score_quality(values, self.policy)
        self.assertEqual(s["ambiguous"], 1)
        self.assertEqual(s["hard_violations"], 1)
        self.assertFalse(s["metric_gate_pass"])

    def test_no_label_or_denominator_manipulation(self):
        v = rows(tp=8, tn=16)
        v[1]["id"] = v[0]["id"]
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)
        v = rows(tp=8, tn=16)
        v[0]["expected"] = "other"
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)
        v = rows(tp=8, tn=16)
        v[0].pop("complete")
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)

    def test_mixed_provider_or_arm_never_pools_into_pass(self):
        v = rows(tp=8, tn=16)
        v[-1]["provider"] = "google"
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)
        v = rows(tp=8, tn=16)
        v[-1]["track"] = "helper_opt_in"
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)
        v = rows(tp=8, tn=16)
        v[-1]["arm"] = "baseline"
        with self.assertRaises(ValueError):
            score_quality(v, self.policy)
        with self.assertRaises(ValueError):
            score_quality(v[:8], self.policy, expected_count=8)

    def test_incomplete_known_hard_violation_remains_visible(self):
        v = rows(tp=8, tn=16)
        v[0]["complete"] = False
        v[0]["hard_violations"] = ["known_unsafe"]
        s = score_quality(v, self.policy)
        self.assertEqual(s["hard_violations"], 1)
        self.assertFalse(s["metric_gate_pass"])

    def test_f05_is_stretch_only(self):
        s = score_quality(rows(tp=20, fn=2, fp=1, tn=1), self.policy)
        self.assertTrue(s["metric_gate_pass"])
        self.assertFalse(s["f0_5_stretch_met"])


if __name__ == "__main__":
    unittest.main()
