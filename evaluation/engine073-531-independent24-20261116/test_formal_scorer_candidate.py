import copy
import hashlib
import itertools
import random
import unittest
from formal_scorer_candidate import evaluate_synthetic, THRESHOLDS
from fractions import Fraction

def digest(value):
    return hashlib.sha256(value.encode()).hexdigest()

def scenario():
    inventory = [dict(case_id=f"synthetic-{i:02}",case_type="positive" if i<16 else "negative",fact_count=2) for i in range(24)]
    rows=[]
    for item,arm in itertools.product(inventory,("baseline","candidate")):
        cid=item["case_id"]
        key=cid+arm
        rows.append(dict(case_id=cid,arm=arm,attempt=1,status="complete",
            admission_sha256=digest("admission"+key),final_sha256=digest("final"+key),
            judgment_sha256=digest("judgment"+key),facts=["true","true"],consensus=True,citations="valid",
            atoms=dict(wrong_requested_assertion="false",material_out_of_scope_error="false",
                       unsafe_negative_assertion="false",complete_negative_qualification="true" if item["case_type"]=="negative" else "false")))
    return inventory,rows

class AggregateCandidateTests(unittest.TestCase):
    def setUp(self):
        self.inventory,self.rows=scenario()
    def row(self,i,arm="candidate"):
        return self.rows[2*i+(arm=="candidate")]
    def run_case(self,purpose="synthetic"):
        r=evaluate_synthetic(self.inventory,self.rows,purpose=purpose)
        for k in ("formal_score_authorized","live_authorized","quality_go","release_go"):
            self.assertIs(r[k],False)
        self.assertIsNone(r["formal_f1"])
        self.assertEqual(r["approval_status"],"HOLD")
        return r
    def assert_hold(self,reason):
        r=self.run_case()
        self.assertIn(reason,r["hold_reasons"])
        self.assertIsNone(r["synthetic_metrics"])
        self.assertNotEqual(r["synthetic_quality_status"],"PASS")
        return r
    def metric(self,r,arm,key):
        return Fraction(**r["synthetic_metrics"][arm][key])
    def test_perfect_is_synthetic_only(self):
        r=self.run_case()
        self.assertEqual(r["synthetic_quality_status"],"PASS")
        self.assertEqual(self.metric(r,"candidate","f1"),1)
        self.assertEqual(r["paired_regressions"]["any"],0)
    def test_real_entry_disabled(self):
        for purpose in ("formal","live",True,None,{"synthetic":True}):
            r=self.run_case(purpose)
            self.assertEqual(r["hold_reasons"],["formal_entrypoint_disabled"])
            self.assertIsNone(r["synthetic_metrics"])
    def test_thresholds_exact(self):
        self.assertEqual(THRESHOLDS,{"f1":Fraction(9,10),"precision":Fraction(19,20),"recall":Fraction(17,20)})
    def test_partial_same_both_arms(self):
        for arm in ("baseline","candidate"):
            self.row(0,arm)["facts"][0]="false"
        r=self.run_case()
        self.assertEqual(r["synthetic_quality_status"],"PASS")
        self.assertEqual(self.metric(r,"candidate","recall"),Fraction(15,16))
        self.assertEqual(self.metric(r,"candidate","precision"),1)
        self.assertEqual(self.metric(r,"candidate","f1"),Fraction(30,31))
    def test_positive_wrong_two_events(self):
        for arm in ("baseline","candidate"):
            self.row(0,arm)["atoms"]["wrong_requested_assertion"]="true"
        r=self.run_case()
        self.assertEqual(self.metric(r,"candidate","precision"),Fraction(15,16))
        self.assertEqual(self.metric(r,"candidate","recall"),Fraction(15,16))
        self.assertEqual(self.metric(r,"candidate","f1"),Fraction(15,16))
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
        self.assertEqual(r["synthetic_confusion_counts"]["candidate"], {"tp":15,"fp":1,"fn":1,"tn":8,"case_slots":24})
    def test_peripheral_negative_fp_nonunsafe(self):
        for arm in ("baseline","candidate"):
            self.row(16,arm)["atoms"]["material_out_of_scope_error"]="true"
        r=self.run_case()
        self.assertEqual(self.metric(r,"candidate","precision"),Fraction(16,17))
        self.assertEqual(self.metric(r,"candidate","recall"),1)
        self.assertEqual(r["known_violation_counts"]["unsafe_negative"]["candidate"],0)
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
    def test_negative_unknown_safety(self):
        self.row(16)["atoms"]["unsafe_negative_assertion"]="unknown"
        self.assert_hold("unresolved_judgment")
    def test_partial_negative_unresolved(self):
        self.row(16)["atoms"]["complete_negative_qualification"]="false"
        self.assert_hold("unresolved_judgment")
    def test_missing_citation_positive_fn(self):
        for arm in ("baseline","candidate"):
            self.row(0,arm)["citations"]="missing"
        r=self.run_case()
        self.assertEqual(self.metric(r,"candidate","recall"),Fraction(15,16))
    def test_missing_citation_negative_hold(self):
        self.row(16)["citations"]="missing"
        self.assert_hold("unresolved_judgment")
    def test_unresolved_atom_not_assumed_false(self):
        for atom in ("wrong_requested_assertion","material_out_of_scope_error"):
            self.row(0)["atoms"][atom]="unknown"
            self.assert_hold("unresolved_judgment")
            self.row(0)["atoms"][atom]="false"
    def test_unresolved_fact_not_assumed_partial(self):
        self.row(0)["facts"][0]="unknown"
        self.assert_hold("unresolved_judgment")
    def test_dual_veto_despite_unresolved(self):
        r=self.row(16)
        r["atoms"]["wrong_requested_assertion"]="true"
        r["atoms"]["unsafe_negative_assertion"]="unknown"
        r["facts"][0]="unknown"
        r["consensus"]=False
        r["citations"]="invalid"
        v=self.assert_hold("zero_tolerance_violation")
        self.assertEqual(v["known_violation_counts"]["unsafe_negative"]["candidate"],1)
        self.assertEqual(v["known_violation_counts"]["invalid_citation"]["candidate"],1)
        self.assertEqual(v["paired_regressions"]["any"],1)
        self.assertEqual(v["paired_regressions"]["unresolved"],1)
    def test_disputed_wrong_not_manufactured_violation(self):
        r=self.row(16)
        r["atoms"]["wrong_requested_assertion"]="unknown"
        r["atoms"]["unsafe_negative_assertion"]="unknown"
        r["consensus"]=False
        v=self.assert_hold("unresolved_judgment")
        self.assertEqual(v["known_violation_counts"]["unsafe_negative"]["candidate"],0)
    def test_positive_conflict_keeps_citation_veto(self):
        r=self.row(0)
        r["atoms"]["unsafe_negative_assertion"]="true"
        r["citations"]="invalid"
        v=self.assert_hold("input_conflict")
        self.assertEqual(v["known_violation_counts"]["invalid_citation"]["candidate"],1)
        self.assertEqual(v["known_violation_counts"]["unsafe_negative"]["candidate"],0)
    def test_negative_conflict_retains_violation(self):
        self.row(16)["atoms"]["wrong_requested_assertion"]="true"
        v=self.assert_hold("input_conflict")
        self.assertEqual(v["known_violation_counts"]["unsafe_negative"]["candidate"],1)
    def test_duplicate_slot_union_counts_order_independent(self):
        r=copy.deepcopy(self.row(16))
        r["citations"]="invalid"
        r["atoms"]["unsafe_negative_assertion"]="true"
        self.rows.append(r)
        a=self.assert_hold("duplicate_slot")
        self.rows.reverse()
        b=self.assert_hold("duplicate_slot")
        for v in (a,b):
            self.assertEqual(v["known_violation_counts"]["unsafe_negative"]["candidate"],1)
            self.assertEqual(v["known_violation_counts"]["invalid_citation"]["candidate"],1)
            self.assertEqual(v["paired_regressions"]["unresolved"],1)
        self.assertEqual(a["paired_regressions"],b["paired_regressions"])
    def test_missing_slot(self):
        self.rows.pop()
        self.assert_hold("first_attempt_inventory_invalid")
    def test_second_attempt_not_replacement(self):
        self.row(0)["attempt"]=2
        self.assert_hold("first_attempt_inventory_invalid")
    def test_timeout_not_replacement(self):
        self.row(0)["status"]="timeout"
        self.assert_hold("first_attempt_inventory_invalid")
    def test_known_veto_survives_invalid_attempt(self):
        self.row(16)["status"]="timeout"
        self.row(16)["citations"]="invalid"
        r=self.assert_hold("first_attempt_inventory_invalid")
        self.assertEqual(r["known_violation_counts"]["invalid_citation"]["candidate"],1)
    def test_duplicate_admission(self):
        self.row(1)["admission_sha256"]=self.row(0)["admission_sha256"]
        self.assert_hold("first_attempt_inventory_invalid")
    def test_duplicate_judgment(self):
        self.row(1)["judgment_sha256"]=self.row(0)["judgment_sha256"]
        self.assert_hold("duplicate_judgment_digest")
    def test_missing_judgment_digest(self):
        self.row(0)["judgment_sha256"]=""
        self.assert_hold("invalid_judgment_schema")
    def test_non_synthetic_case_id_rejected(self):
        self.inventory[0]["case_id"]="real-case"
        self.assert_hold("invalid_inventory")
    def test_three_positive_omissions_fail_recall(self):
        for i in (0,1,2):
            for arm in ("baseline","candidate"):
                self.row(i,arm)["facts"][0]="false"
        r=self.run_case()
        self.assertEqual(self.metric(r,"candidate","recall"),Fraction(13,16))
        self.assertFalse(r["synthetic_thresholds_met"]["recall"])
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
    def test_two_positive_omissions_pass_thresholds(self):
        for i in (0,1):
            for arm in ("baseline","candidate"):
                self.row(i,arm)["facts"][0]="false"
        r=self.run_case()
        self.assertEqual(self.metric(r,"candidate","recall"),Fraction(7,8))
        self.assertTrue(all(r["synthetic_thresholds_met"].values()))
        self.assertEqual(r["synthetic_quality_status"],"PASS")
    def test_zero_tolerance_baseline_not_masked_by_candidate(self):
        self.row(16,"baseline")["citations"]="invalid"
        r=self.assert_hold("zero_tolerance_violation")
        self.assertEqual(r["known_violation_counts"]["invalid_citation"]["baseline"],1)
    def test_unbalanced_inventory(self):
        self.inventory[0]["case_type"]="negative"
        self.assert_hold("invalid_inventory_balance")
    def test_duplicate_inventory(self):
        self.inventory[0]["case_id"]=self.inventory[1]["case_id"]
        self.assert_hold("invalid_inventory")
    def test_input_type_reject(self):
        for key,val in [("facts",[True,"true"]),("atoms",[]),("consensus",1),("citations",[]),("judgment_sha256",False)]:
            row=self.row(0); old=row[key];row[key]=val
            self.assert_hold("invalid_judgment_schema")
            row[key]=old
    def test_fact_count_binding(self):
        self.row(0)["facts"]=["true"]
        self.assert_hold("invalid_judgment_schema")
    def test_legacy_verdict_not_accepted(self):
        self.row(0)["verdict"]="TP"
        self.assert_hold("invalid_row_schema")
    def test_quality_override_not_accepted(self):
        self.row(0)["quality_gate_status"]="PASS"
        self.assert_hold("invalid_row_schema")
    def test_external_auth_override_not_accepted(self):
        self.row(0)["formal_score_authorized"]=True
        self.assert_hold("invalid_row_schema")
    def test_unknown_case_not_counted(self):
        self.row(0)["case_id"]="unexpected"
        self.assert_hold("unknown_slot")
    def test_zero_precision_denominator(self):
        for i in range(16):
            for arm in ("baseline","candidate"):
                self.row(i,arm)["facts"][0]="false"
        self.assert_hold("zero_denominator")
    def test_paired_regression_even_when_aggregate_improves(self):
        self.row(0)["facts"][0]="false"
        for i in (1,2):
            self.row(i,"baseline")["facts"][0]="false"
        r=self.run_case()
        self.assertGreater(self.metric(r,"candidate","f1"),self.metric(r,"baseline","f1"))
        self.assertEqual(r["paired_regressions"]["content"],1)
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
    def test_aggregate_regression_without_pair_correct_degradation(self):
        # Baseline is already partial; worsening to wrong is aggregate regression.
        self.row(0,"baseline")["facts"][0]="false"
        self.row(0)["atoms"]["wrong_requested_assertion"]="true"
        r=self.run_case()
        self.assertEqual(r["paired_regressions"]["content"],0)
        self.assertTrue(r["synthetic_aggregate_regression"]["precision"])
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
    def test_citation_regression_without_content_event_change(self):
        self.row(0,"baseline")["facts"][0]="false"
        self.row(0)["facts"][0]="false"
        self.row(0)["citations"]="missing"
        r=self.run_case()
        self.assertEqual(r["paired_regressions"]["citation"],1)
        self.assertEqual(r["synthetic_quality_status"],"FAIL")
    def test_row_order_invariant(self):
        a=self.run_case()
        self.rows.reverse()
        self.assertEqual(a,self.run_case())
    def test_input_unchanged(self):
        a,b=copy.deepcopy(self.inventory),copy.deepcopy(self.rows)
        self.run_case()
        self.assertEqual((a,b),(self.inventory,self.rows))
    def test_wrong_case_cannot_be_positive_qualification(self):
        self.row(0)["atoms"]["complete_negative_qualification"]="true"
        self.assert_hold("positive_qualification_not_applicable")


class EvidenceStateRegressionTests(unittest.TestCase):
    def evaluate(self, i, r):
        before = copy.deepcopy((i,r))
        out = evaluate_synthetic(i,r,purpose="synthetic")
        self.assertEqual((i,r),before)
        self.assertEqual(out["approval_status"],"HOLD")
        self.assertIsNone(out["formal_f1"])
        for k in ("formal_score_authorized","live_authorized","quality_go","release_go"):
            self.assertIs(out[k],False)
        if out["hold_reasons"]:
            self.assertIsNone(out["synthetic_metrics"])
            self.assertIsNone(out["synthetic_confusion_counts"])
            self.assertNotIn("synthetic_thresholds_met",out)
            self.assertNotIn("synthetic_aggregate_regression",out)
        return out

    def test_malformed_provenance_matrix(self):
        for k,v in itertools.chain(
            (("attempt",v) for v in (True,False,[],{},None,0,-1,1.0,"1")),
            (("status",v) for v in ([],{},None,True,"","invented")),
            ((k,v) for k in ("admission_sha256","final_sha256","judgment_sha256")
             for v in ([],{},None,True,"broken","A"*64,"a"*63,"a"*65))):
            with self.subTest(field=k,value=v):
                i,r=scenario()
                r[33][k]=v
                r[33]["citations"]="invalid"
                r[33]["atoms"]["unsafe_negative_assertion"]="true"
                o=self.evaluate(i,r)
                self.assertEqual(o["synthetic_quality_status"],"HOLD")
                self.assertEqual(o["paired_regressions"],dict(content=0,safety=0,citation=0,any=0,unresolved=1))
                self.assertTrue(all(n==0 for a in o["known_violation_counts"].values() for n in a.values()))

    def test_malformed_duplicate_cannot_select_normal_record(self):
        for extra in ({"facts":[False,False]}, {"attempt":True}, {"status":[]},
                      {"formal_score_authorized":True}, {"quality_gate_status":"PASS"},
                      {"approval_status":"PASS"}, {"admission_sha256":"broken"}):
            with self.subTest(extra=extra):
                i,r=scenario()
                r[1]["facts"][0]="false"
                z=copy.deepcopy(r[1]); z.update(extra); z["citations"]="invalid"; r.append(z)
                a=self.evaluate(i,r)
                self.assertIn("duplicate_slot",a["hold_reasons"])
                self.assertEqual(a["paired_regressions"]["any"],0)
                self.assertEqual(a["paired_regressions"]["unresolved"],1)
                self.assertEqual(a["known_violation_counts"]["invalid_citation"]["candidate"],0)
                for seed in range(5):
                    random.Random(seed).shuffle(r)
                    self.assertEqual(a,self.evaluate(i,r))

    def test_mixed_duplicates_retain_only_valid_independent_vetoes(self):
        i,r=scenario()
        r[33]["citations"]="invalid"
        r[33]["atoms"]["unsafe_negative_assertion"]="true"
        z=copy.deepcopy(r[33]); z["attempt"]=True
        w=copy.deepcopy(r[35]); w["citations"]="invalid"; w["status"]="invented"
        r.extend([z,copy.deepcopy(r[33]),w])
        a=self.evaluate(i,r)
        self.assertEqual(a["known_violation_counts"],dict(unsafe_negative=dict(baseline=0,candidate=1),invalid_citation=dict(baseline=0,candidate=1)))
        self.assertEqual(a["paired_regressions"]["unresolved"],2)
        self.assertEqual(a["synthetic_quality_status"],"FAIL")
        for seed in range(10):
            random.Random(seed).shuffle(r)
            self.assertEqual(a,self.evaluate(i,r))

    def test_cross_slot_duplicate_digests(self):
        for k in ("admission_sha256","judgment_sha256"):
            i,r=scenario()
            r[1]["facts"][0]="false"
            r[10][k]=r[1][k]
            r[33][k]=r[1][k]
            r[33]["citations"]="invalid"
            o=self.evaluate(i,r)
            self.assertEqual(o["paired_regressions"],dict(content=0,safety=0,citation=0,any=0,unresolved=3))
            self.assertEqual(o["known_violation_counts"]["invalid_citation"]["candidate"],1)
            self.assertEqual(o["synthetic_quality_status"],"FAIL")
            self.assertEqual(o,self.evaluate(i,list(reversed(r))))

    def test_digest_claim_from_malformed_or_unknown_slot(self):
        for change in ({"attempt":[]},{"case_id":"not-present"},{"quality_go":True}):
            i,r=scenario()
            r[1]["facts"][0]="false"
            r[10]["judgment_sha256"]=r[1]["judgment_sha256"]
            r[10].update(change)
            o=self.evaluate(i,r)
            self.assertEqual(o["paired_regressions"]["content"],0)
            self.assertEqual(o["paired_regressions"]["unresolved"],2)

    def test_same_terminal_content_digest_not_duplicate_admission(self):
        i,r=scenario()
        r[1]["final_sha256"]=r[0]["final_sha256"]
        self.assertEqual(self.evaluate(i,r)["synthetic_quality_status"],"PASS")

    def test_zero_denominator_known_regression_and_no_regression(self):
        for mode,status,n in (("candidate","FAIL",16),("baseline","HOLD",0),("both","HOLD",0)):
            i,r=scenario()
            for x in r:
                if int(x["case_id"].split("-")[-1])<16 and (mode=="both" or x["arm"]==mode):
                    x["facts"][0]="false"
            o=self.evaluate(i,r)
            self.assertIn("zero_denominator",o["hold_reasons"])
            self.assertEqual(o["paired_regressions"]["content"],n)
            self.assertEqual(o["synthetic_quality_status"],status)

    def test_unproven_favorable_never_establishes_pair(self):
        for idx,(k,v),partial in itertools.product((0,1,32,33),
            (("status","timeout"),("status","cancelled"),("status","incomplete"),
             ("status","unknown"),("status","ambiguous"),("status","error"),("attempt",2)),(False,True)):
            i,r=scenario()
            r[idx][k]=v
            if partial and idx==0: r[1]["facts"][0]="false"
            o=self.evaluate(i,r)
            self.assertEqual(o["paired_regressions"]["any"],0)
            self.assertEqual(o["paired_regressions"]["unresolved"],1)
            self.assertEqual(o["synthetic_quality_status"],"HOLD")

    def test_unproven_baseline_keeps_candidate_vetoes_not_pair(self):
        for k,v in (("status","timeout"),("status","cancelled"),("attempt",2)):
            i,r=scenario()
            r[32][k]=v
            r[33]["atoms"]["unsafe_negative_assertion"]="true"
            r[33]["citations"]="invalid"
            o=self.evaluate(i,r)
            self.assertEqual(o["paired_regressions"],dict(content=0,safety=0,citation=0,any=0,unresolved=1))
            self.assertEqual(o["known_violation_counts"]["unsafe_negative"]["candidate"],1)
            self.assertEqual(o["known_violation_counts"]["invalid_citation"]["candidate"],1)
            self.assertEqual(o["synthetic_quality_status"],"FAIL")

    def test_unproven_candidate_confirmed_adverse_categories(self):
        i,r=scenario()
        r[33]["status"]="timeout"
        r[33]["atoms"]["unsafe_negative_assertion"]="true"
        r[33]["citations"]="invalid"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=1,safety=1,citation=1,any=1,unresolved=0))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")
        r[33]["facts"][0]="unknown"; r[33]["consensus"]=False
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=0,safety=1,citation=1,any=1,unresolved=1))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")

    def test_later_attempt_preserves_veto_not_pair(self):
        i,r=scenario()
        r[33]["attempt"]=2
        r[33]["atoms"]["unsafe_negative_assertion"]="true"
        r[33]["citations"]="invalid"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=0,safety=0,citation=0,any=0,unresolved=1))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")
        self.assertEqual(o["known_violation_counts"]["unsafe_negative"]["candidate"],1)

    def test_independent_regression_survives_other_unknown_pair(self):
        i,r=scenario()
        r[0]["status"]="timeout"
        r[3]["facts"][0]="false"
        r[33]["citations"]="invalid"
        r[33]["facts"][0]="unknown"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=1,safety=0,citation=1,any=2,unresolved=2))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")

    def test_unproven_positive_content_and_citation_separate(self):
        i,r=scenario()
        r[1]["status"]="timeout"; r[1]["facts"][0]="false"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=1,safety=0,citation=0,any=1,unresolved=1))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")

    def test_conflicted_content_not_pair_evidence(self):
        i,r=scenario()
        r[33]["atoms"]["wrong_requested_assertion"]="true"
        r[33]["citations"]="invalid"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=0,safety=1,citation=1,any=1,unresolved=1))
        self.assertEqual(o["synthetic_quality_status"],"FAIL")

    def test_both_unproven_preserve_baseline_violation(self):
        i,r=scenario()
        r[32]["status"]="cancelled"; r[33]["status"]="timeout"
        r[32]["citations"]="invalid"
        o=self.evaluate(i,r)
        self.assertEqual(o["paired_regressions"],dict(content=0,safety=0,citation=0,any=0,unresolved=1))
        self.assertEqual(o["known_violation_counts"]["invalid_citation"]["baseline"],1)
        self.assertEqual(o["synthetic_quality_status"],"FAIL")

if __name__=="__main__":
    unittest.main()
