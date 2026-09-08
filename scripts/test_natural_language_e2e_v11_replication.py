import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import natural_language_e2e_v11_replication as repl
import natural_language_e2e_v11 as v11


class ReplicationManifestTests(unittest.TestCase):
    def setUp(self):
        self.path = Path("fixtures/natural-language-e2e-v11-cross-model-v1/manifest.json")
        self.manifest = repl.load_replication_manifest(self.path)

    def test_reference_is_exact_frozen_v11(self):
        self.assertEqual(self.manifest["reference"]["freeze_commit"], "a758af17a998493c1005702365b100e05b05f95d")
        self.assertEqual(self.manifest["reference"]["product_commit"], "29a9e4be6273dbffeda324e15517dc64930ad315")

    def test_canonical_mistral_is_not_a_replication_target(self):
        self.assertFalse(repl.allowed_target(self.manifest, "mistral", "ministral-8b-latest"))

    def test_all_frozen_replication_targets_are_allowed(self):
        for target in self.manifest["targets"]:
            self.assertTrue(repl.allowed_target(self.manifest, target["provider"], target["model"]))

    def test_reference_corpus_still_validates_with_v11_logic(self):
        manifest, cases = v11.validate_corpus(Path("fixtures/natural-language-e2e-v11"))
        self.assertEqual(manifest["corpus_identity"], "natural-language-e2e-v11")
        self.assertEqual(len(cases), 13)


if __name__ == "__main__":
    unittest.main()
