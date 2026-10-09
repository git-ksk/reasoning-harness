"""Frozen independent-source holdout novelty tests (no holdout material)."""
import runpy
import unittest
from pathlib import Path

F = runpy.run_path(str(Path(__file__).with_name("check_engine_0_7_independent_v1_freshness.py")))
check = F["check_freshness"]
surfaces = F["previous_surfaces"]


def dummy():
    cases = []
    for index in range(12):
        cases.append({
            "id": f"tester-preflight-case-{index}",
            "family": f"novel-family-{index % 4}",
            "targets": [{
                "target_id": f"tester-preflight-target-{index}",
                "relation": "compatible" if index < 5 else "opposed",
                "sources": [
                    {"source_id": f"tester-preflight-source-{index}-a",
                     "quote": f"Unique tangerine marble atlas epsilon {index} grows ninety mellow velvet islands after noon."},
                    {"source_id": f"tester-preflight-source-{index}-b",
                     "quote": f"Unique sapphire amber apricot zeta {index} settles across seventy quiet harbors at dusk."},
                ]
            }]
        })
    return {"cases": cases}


class Freshness(unittest.TestCase):
    def test_fresh_dev_only_dummy_identities(self):
        # Tests only the validator, not independent holdout passage content.
        result = check(dummy())
        self.assertEqual(result["status"], "FRESH_DISJOINT")
        self.assertEqual(result["case_count"], 12)

    def test_reused_source_identifier_fails(self):
        corpus = dummy()
        corpus["cases"][1]["targets"][0]["sources"][0]["source_id"] = corpus["cases"][0]["targets"][0]["sources"][0]["source_id"]
        with self.assertRaises(AssertionError):
            check(corpus)

    def test_reused_existing_eight_word_window_fails(self):
        _, old_windows = surfaces()
        self.assertTrue(old_windows)
        corpus = dummy()
        first = sorted(old_windows)[0]
        corpus["cases"][0]["targets"][0]["sources"][0]["quote"] = " ".join(first) + " plus another distant object today"
        with self.assertRaises(AssertionError):
            check(corpus)

    def test_insufficient_family_variety_fails(self):
        corpus = dummy()
        for case in corpus["cases"]:
            case["family"] = "single-development-family"
        with self.assertRaises(AssertionError):
            check(corpus)


if __name__ == "__main__":
    unittest.main()
