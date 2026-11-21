"""Prospective synthetic-only evidence and conflict regression tests."""
import copy
import itertools
import random
import unittest

from formal_scorer_candidate import evaluate_synthetic
from test_formal_scorer_candidate import scenario


class EligibilityTests(unittest.TestCase):
    def check(self, i, r):
        before = copy.deepcopy((i, r))
        o = evaluate_synthetic(i, r, purpose="synthetic")
        self.assertEqual(before, (i, r))
        shuffled = r[:]
        random.Random(73).shuffle(shuffled)
        self.assertEqual(o, evaluate_synthetic(i, shuffled, purpose="synthetic"))
        self.assertEqual(o["approval_status"], "HOLD")
        self.assertIsNone(o["formal_f1"])
        for k in ("formal_score_authorized", "live_authorized", "quality_go", "release_go"):
            self.assertIs(o[k], False)
        if o["hold_reasons"]:
            for k in ("synthetic_metrics", "synthetic_confusion_counts"):
                self.assertIsNone(o[k])
            for k in ("synthetic_thresholds_met", "synthetic_aggregate_regression"):
                self.assertNotIn(k, o)
        return o

    def pair(self, o, c=0, s=0, q=0, u=0):
        self.assertEqual(o["paired_regressions"],
                         dict(content=c, safety=s, citation=q, any=int(bool(c or s or q)), unresolved=u))

    def test_state_product(self):
        states = ("complete", "timeout", "cancelled", "incomplete",
                  "error", "unknown", "ambiguous", "later")
        for b, c, kind in itertools.product(states, states,
                ("favorable", "wrong", "material", "omission", "missing", "dual_veto", "disputed")):
            with self.subTest(b=b, c=c, kind=kind):
                i, r = scenario()
                a, z = r[32:34] if kind == "dual_veto" else r[:2]
                for row, state in ((a, b), (z, c)):
                    if state == "later":
                        row["attempt"] = 2
                    else:
                        row["status"] = state
                if kind in ("wrong", "material", "disputed"):
                    z["atoms"]["material_out_of_scope_error" if kind == "material" else
                               "wrong_requested_assertion"] = "true"
                if kind == "omission":
                    z["facts"][0] = "false"
                if kind == "missing":
                    z["citations"] = "missing"
                if kind == "dual_veto":
                    z["atoms"]["unsafe_negative_assertion"] = "true"
                    z["citations"] = "invalid"
                if kind == "disputed":
                    z["consensus"] = False
                # Expected category direction from the proposed policy, not the scorer.
                complete_b, complete_c = b == "complete", c == "complete"
                attributable = c in ("complete", "timeout", "cancelled", "incomplete", "error")
                explicit = kind in ("wrong", "material", "dual_veto")
                content = complete_b and ((explicit and attributable) or
                           (kind in ("omission", "missing") and complete_c))
                safety = complete_b and attributable and kind == "dual_veto"
                citation = safety or (complete_b and complete_c and kind == "missing")
                all_resolved = complete_b and (complete_c or
                               (kind == "dual_veto" and attributable)) and kind != "disputed"
                o = self.check(i, r)
                self.pair(o, int(content), int(safety), int(citation), int(not all_resolved))
                self.assertEqual(o["known_explicit_content_counts"]["candidate"],
                                 int(kind in ("wrong", "material", "disputed")))
                for counts in o["known_violation_counts"].values():
                    self.assertEqual(counts["candidate"], int(kind == "dual_veto"))

    def test_qualification_conflict_product(self):
        # Both arms, both conflicts, terminal/nonterminal, and independent citation.
        for arm, value, state, citation in itertools.product((0, 1), ("true", "unknown"),
                                                           ("complete", "timeout"), ("valid", "invalid")):
            with self.subTest(arm=arm, value=value, state=state, citation=citation):
                i, r = scenario()
                r[arm]["atoms"]["complete_negative_qualification"] = value
                r[arm]["status"] = state
                r[1]["facts"][0] = "false"
                r[1]["citations"] = citation
                o = self.check(i, r)
                q = int(citation == "invalid" and not (arm == 0 and state == "timeout"))
                self.pair(o, q=q, u=1)
                self.assertEqual(o["synthetic_quality_status"], "FAIL" if citation == "invalid" else "HOLD")
                self.assertIn("positive_qualification_not_applicable", o["hold_reasons"])

    def test_both_conflicts_and_duplicate(self):
        for duplicate, unsafe in itertools.product((False, True), ("false", "true", "unknown")):
            with self.subTest(duplicate=duplicate, unsafe=unsafe):
                i, r = scenario()
                r[0]["atoms"]["complete_negative_qualification"] = "unknown"
                r[1]["atoms"]["complete_negative_qualification"] = "true"
                r[1]["atoms"]["unsafe_negative_assertion"] = unsafe
                r[1]["citations"] = "invalid"
                if duplicate:
                    r.append(copy.deepcopy(r[1]))
                o = self.check(i, r)
                self.pair(o, q=int(not duplicate), u=1)
                self.assertEqual(o["known_violation_counts"]["invalid_citation"]["candidate"], 1)
                self.assertEqual(o["known_violation_counts"]["unsafe_negative"]["candidate"], 0)

    def test_conflict_with_other_pair_regression(self):
        i, r = scenario()
        r[0]["atoms"]["complete_negative_qualification"] = "true"
        r[1]["facts"][0] = "false"
        r[33]["atoms"]["unsafe_negative_assertion"] = "true"
        r[33]["citations"] = "invalid"
        o = self.check(i, r)
        self.pair(o, c=1, s=1, q=1, u=1)
        self.assertEqual(o["synthetic_quality_status"], "FAIL")

    def test_explicit_claim_retained_without_pair_selection(self):
        for mode in ("unknown_status", "later", "duplicate", "malformed",
                     "forged_eligibility", "unknown_atom", "disputed"):
            with self.subTest(mode=mode):
                i, r = scenario()
                r[1]["atoms"]["wrong_requested_assertion"] = "true"
                if mode == "unknown_status":
                    r[1]["status"] = "unknown"
                elif mode == "later":
                    r[1]["attempt"] = 2
                elif mode == "duplicate":
                    r.append(copy.deepcopy(r[1]))
                elif mode == "malformed":
                    r[1]["attempt"] = True
                elif mode == "forged_eligibility":
                    r[1]["terminal_confirmed"] = True
                elif mode == "unknown_atom":
                    r[1]["atoms"]["wrong_requested_assertion"] = "unknown"
                else:
                    r[1]["consensus"] = False
                o = self.check(i, r)
                self.pair(o, u=1)
                self.assertEqual(o["known_explicit_content_counts"]["candidate"],
                                 int(mode not in ("malformed", "forged_eligibility", "unknown_atom")))

    def test_early_return_diagnostic_unavailable(self):
        for purpose, inventory, rows in (("formal", None, None), ("synthetic", [], []),
                                        ("synthetic", scenario()[0], None)):
            o = evaluate_synthetic(inventory, rows, purpose=purpose)
            self.assertIsNone(o["known_explicit_content_counts"])
            self.assertIsNone(o["known_violation_counts"])
            self.assertIsNone(o["paired_regressions"])
            self.assertIsNone(o["synthetic_metrics"])
            self.assertEqual(o["synthetic_quality_status"], "HOLD")
            self.assertFalse(o["formal_score_authorized"])


if __name__ == "__main__":
    unittest.main()
