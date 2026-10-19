import copy
import unittest

from claim_metrics import report, score_case


def fixture(id_="c1", required=("g1",), claims=None):
    plan = dict(id=id_, provider="fixture-provider", track="e2e_normal",
                arm="candidate", family="correction", answerability="full",
                required_groups=list(required), observation_sha256="a" * 64)
    review = {k: v for k, v in plan.items() if k != "required_groups"}
    review.update(operation="complete", review_complete=True,
                  reviewer={"id": "fixture-reviewer", "role": "host"},
                  hard_violations=[], claims=claims if claims is not None else [
                      dict(id="o1", proposition_id="p1", grounding="grounded",
                           satisfied_groups=["g1"])])
    return plan, review


def claim(id_, prop, groups=(), grounding="grounded"):
    return dict(id=id_, proposition_id=prop, grounding=grounding,
                satisfied_groups=list(groups))


class ClaimMetrics(unittest.TestCase):
    def test_grounded_extra_claim_cannot_create_recall(self):
        p, r = fixture(required=("g1", "g2"), claims=[claim("a", "p", ("g1",)), claim("b", "extra")])
        s = score_case(p, r)
        self.assertEqual((s["precision"], s["recall"], s["f1"]), (1, .5, 2 / 3))

    def test_unsupported_word_match_is_not_recall(self):
        p, r = fixture(claims=[claim("a", "p", grounding="unsupported")])
        s = score_case(p, r)
        self.assertEqual((s["precision"], s["recall"], s["f1"]), (0, 0, 0))
        r["claims"][0]["satisfied_groups"] = ["g1"]
        with self.assertRaisesRegex(ValueError, "cannot_earn_recall"):
            score_case(p, r)

    def test_equivalent_alternatives_and_repeats_are_one_point(self):
        p, r = fixture(claims=[claim("a", "same", ("g1",)), claim("b", "same", ("g1",))])
        s = score_case(p, r)
        self.assertEqual(s["counts"], dict(displayed_unique_claims_A=1, grounded_claims_S=1,
                                        required_groups_G=1, satisfied_groups_H=1, displayed_claim_occurrences=2))

    def test_conflicting_occurrence_annotations_conservatively_remove_credit(self):
        p, r = fixture(claims=[claim("a", "same", ("g1",)), claim("b", "same", grounding="overstated")])
        s = score_case(p, r)
        self.assertEqual((s["precision"], s["recall"]), (0, 0))

    def test_no_claims_with_required_groups_has_zero_f1(self):
        p, r = fixture(claims=[])
        s = score_case(p, r)
        self.assertIsNone(s["precision"])
        self.assertEqual((s["recall"], s["f1"]), (0, 0))

    def test_no_required_groups_keeps_f1_null_with_or_without_claims(self):
        for claims in ([], [claim("a", "extra")]):
            p, r = fixture(required=(), claims=claims)
            p["answerability"] = r["answerability"] = "none"
            s = score_case(p, r)
            self.assertIsNone(s["recall"])
            self.assertIsNone(s["f1"])

    def test_partial_output_is_unavailable_but_known_violation_is_retained(self):
        p, r = fixture()
        r.update(operation="incomplete", hard_violations=["misbound_citation"])
        s = score_case(p, r)
        self.assertIsNone(s["f1"])
        self.assertIsNone(s["counts"]["satisfied_groups_H"])
        self.assertEqual(s["known_hard_violations"], {"misbound_citation": 1})
        self.assertFalse(s["full_display_safety_audit_complete"])

    def test_unreviewed_success_is_not_semantic_success(self):
        p, r = fixture()
        r["review_complete"] = False
        s = score_case(p, r)
        self.assertIsNone(s["f1"])
        self.assertFalse(s["semantic_metrics_available"])

    def test_micro_uses_same_complete_cohort_and_delivered_includes_missing(self):
        p, r = fixture()
        p2, _ = fixture("c2")
        s = report([p, p2], [r])["strata"][0]
        self.assertEqual(s["micro"]["f1"], 1)
        self.assertEqual(s["delivered_required_coverage"], .5)
        self.assertEqual((s["planned_rows"], s["complete_reviewed_rows"], s["safety_unreviewed_rows"]), (2, 1, 1))

    def test_no_completed_rows_is_unavailable_not_perfect(self):
        p, _ = fixture()
        p["observation_sha256"] = None
        s = report([p], [])["strata"][0]
        self.assertIsNone(s["micro"]["f1"])
        self.assertEqual(s["delivered_required_coverage"], 0)
        self.assertIsNone(s["macro_f1"])
        self.assertEqual(s["operational_states"], {"unobserved": 1})

    def test_observed_answer_requires_hash_but_missing_bundle_does_not(self):
        p, r = fixture()
        p["observation_sha256"] = r["observation_sha256"] = None
        with self.assertRaises(ValueError): score_case(p, r)
        r.update(operation="not_run", claims=[], review_complete=False)
        self.assertIsNone(score_case(p, r)["observation_sha256"])

    def test_strata_separate_provider_track_arm_family_and_answerability(self):
        plans, reviews = [], []
        for field, value in [("provider", "other"), ("track", "helper_opt_in"),
                             ("arm", "baseline"), ("family", "lineage"), ("answerability", "partial")]:
            p, r = fixture(field)
            p[field] = r[field] = value
            plans.append(p); reviews.append(r)
        self.assertEqual(len(report(plans, reviews)["strata"]), 5)

    def test_negation_is_a_distinct_reviewed_proposition(self):
        p, r = fixture(claims=[claim("a", "p", ("g1",)), claim("b", "not-p", grounding="contradicted")])
        self.assertEqual(score_case(p, r)["precision"], .5)

    def test_duplicate_and_foreign_ids_fail(self):
        p, r = fixture()
        for plans, reviews in [([p, p], [r]), ([p], [r, r]), ([p], [{**r, "id": "foreign"}])]:
            with self.assertRaises(ValueError): report(plans, reviews)
        r["claims"].append(copy.deepcopy(r["claims"][0]))
        with self.assertRaisesRegex(ValueError, "duplicate_claim"): score_case(p, r)

    def test_binding_changes_fail(self):
        p, r = fixture()
        for field, value in [("id", "other"), ("provider", "other"), ("observation_sha256", "b" * 64)]:
            with self.assertRaises(ValueError): score_case(p, {**r, field: value})

    def test_bool_unknown_grounding_and_foreign_gold_fail(self):
        p, r = fixture()
        with self.assertRaises(ValueError): score_case(p, {**r, "review_complete": 1})
        r["claims"][0]["grounding"] = True
        with self.assertRaises(ValueError): score_case(p, r)
        r["claims"][0].update(grounding="grounded", satisfied_groups=["foreign"])
        with self.assertRaises(ValueError): score_case(p, r)

    def test_macro_excludes_null_but_retains_null_count(self):
        p, r = fixture(required=(), claims=[])
        s = report([p], [r])["strata"][0]
        self.assertIsNone(s["macro_f1"])
        self.assertEqual(s["macro_null_complete_rows"], 1)

    def test_no_acceptance_or_mutation_from_pure_calculation(self):
        p, r = fixture()
        before = copy.deepcopy((p, r))
        s = report([p], [r])
        self.assertEqual(s["acceptance"], "NOT_EVALUATED")
        self.assertEqual((p, r), before)


if __name__ == "__main__":
    unittest.main()
