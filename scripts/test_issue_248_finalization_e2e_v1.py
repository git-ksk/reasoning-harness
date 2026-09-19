import importlib.util
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("issue_248_finalization_e2e_v1.py")
SPEC = importlib.util.spec_from_file_location("issue248_eval", SCRIPT)
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)


class Issue248FinalizationE2EV1Tests(unittest.TestCase):
    def test_corpus_is_closed_and_has_required_paths(self):
        manifest, cases = MOD.validate_corpus()
        self.assertEqual(manifest["corpus_identity"], MOD.CORPUS_ID)
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
        artifact = {
            "claims": [{
                "state": "supported",
                "proposition": target,
            }]
        }
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


if __name__ == "__main__":
    unittest.main()
