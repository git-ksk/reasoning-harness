import unittest
from copy import deepcopy
from score import score, policy


def cases(tp=26, fn=4, fp=0, tn=30):
    def make(prefix, gold, count, v17, v30):
        return [{"id":f"{prefix}{i}", "gold":gold, "complete":True,
                 "v17_requested_relation":v17, "v30_relevant":v30} for i in range(count)]
    return (make("p", "positive", tp, True, True)
            + make("miss", "positive", fn, False, False)
            + make("bad", "negative", fp, True, True)
            + make("n", "negative", tn, False, False))


def controls():
    return {"known_regressions_total":1200, "known_regressions_passed":1200,
            "original_audit_total":14, "original_audit_passed":14,
            "historical_byte_equivalent":True, "known_unfixed_critical_defects":[],
            "operational_failures":0, "labels_frozen_before_observation":True,
            "candidate_frozen_before_observation":True, "first_observation_immutable":True}


class SourceQualificationPolicy(unittest.TestCase):
    def test_safe_four_positive_misses_can_pass_without_sixty_perfect(self):
        result=score(cases(),controls())
        self.assertTrue(result["gate_pass"])
        self.assertEqual(result["axes"]["v30_relevant"]["f1"],52/56)
        self.assertEqual(result["axes"]["v17_requested_relation"]["recall"],26/30)
        self.assertEqual(result["observations"],60)

    def test_five_misses_fail_recall_but_not_safety(self):
        result=score(cases(tp=25,fn=5),controls())
        self.assertFalse(result["gate_pass"])
        self.assertEqual(result["axes"]["v30_relevant"]["fp"],0)
        self.assertTrue(any("recall_below_floor" in x for x in result["blockers"]))

    def test_unsafe_single_false_relevant_must_fail_high_f1(self):
        result=score(cases(tp=29,fn=1,fp=1,tn=29),controls())
        self.assertGreater(result["axes"]["v30_relevant"]["f1"],.90)
        self.assertFalse(result["gate_pass"])
        self.assertIn("v30_relevant:unsafe_negative_admitted",result["blockers"])

    def test_v17_and_v30_must_both_pass(self):
        rows=cases()
        for r in rows:
            if r["id"]=="n0":
                r["v17_requested_relation"]=True
        result=score(rows,controls())
        self.assertFalse(result["gate_pass"])
        self.assertIn("v17_requested_relation:unsafe_negative_admitted",result["blockers"])

    def test_regression_and_historical_gate(self):
        for k,v in [("known_regressions_passed",1199),("original_audit_passed",13),
                    ("historical_byte_equivalent",False),("known_unfixed_critical_defects",["non-event admitted"])]:
            c=controls();c[k]=v
            self.assertFalse(score(cases(),c)["gate_pass"],k)

    def test_incomplete_or_missing_never_count_negative(self):
        rows=cases();rows[0]["complete"]=False
        self.assertFalse(score(rows,controls())["gate_pass"])
        c=controls();c["operational_failures"]=1
        self.assertFalse(score(cases(),c)["gate_pass"])
        self.assertFalse(score(cases()[:-1],controls())["gate_pass"])

    def test_no_positive_acceptance_fails(self):
        result=score(cases(tp=0,fn=30),controls())
        self.assertIsNone(result["axes"]["v17_requested_relation"]["precision"])
        self.assertFalse(result["gate_pass"])

    def test_source_label_and_identity_validation(self):
        for key,val in [("gold","ambiguous"),("complete",0),("v30_relevant","yes")]:
            rows=cases();rows[0][key]=val
            with self.assertRaises(ValueError):
                score(rows,controls())
        rows=cases();rows[1]["id"]=rows[0]["id"]
        with self.assertRaises(ValueError):
            score(rows,controls())

    def test_provenance_cannot_be_skipped(self):
        for k in ("labels_frozen_before_observation", "candidate_frozen_before_observation", "first_observation_immutable"):
            c=controls();c[k]=False
            self.assertFalse(score(cases(),c)["gate_pass"])

    def test_rule_does_not_change_531_policy(self):
        p=policy()
        self.assertEqual((p["thresholds"]["minimum_f1"],
                          p["thresholds"]["minimum_precision"],
                          p["thresholds"]["minimum_recall"]),(.90,.95,.85))
        self.assertEqual(p["cohort"]["minimum_total"],60)
        self.assertEqual(p["regressions"]["original_14_case_audit_all_pass"],True)


if __name__=="__main__":
    unittest.main()
