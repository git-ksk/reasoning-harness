import copy,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import natural_language_e2e_v12_replication as repl
import natural_language_e2e_v12 as v12

class V12ReplicationTests(unittest.TestCase):
    def setUp(self): self.path=Path('fixtures/natural-language-e2e-v12-cross-model-v1/manifest.json'); self.m=repl.load_replication_manifest(self.path)
    def test_reference_is_exact_candidate_and_freeze_tag(self):
        self.assertEqual(self.m['reference']['product_commit'],repl.PRODUCT_COMMIT); self.assertEqual(self.m['reference']['freeze_tag'],repl.FREEZE_TAG)
    def test_canonical_mistral_is_never_replication_target(self): self.assertFalse(repl.allowed_target(self.m,'mistral','ministral-8b-latest'))
    def test_required_google_and_groq_targets_are_frozen(self):
        self.assertTrue(repl.allowed_target(self.m,'google','gemma-4-31b-it')); self.assertTrue(repl.allowed_target(self.m,'google','gemini-3.5-flash-lite')); self.assertTrue(repl.allowed_target(self.m,'groq','openai/gpt-oss-120b'))
    def test_provider_lanes_encode_google_serial_without_cross_provider_global_serial(self):
        self.assertTrue(self.m['semantic_policy']['cross_provider_global_serialisation_forbidden']); self.assertEqual([x['max_parallel_in_lane'] for x in self.m['targets'] if x['lane']=='google'],[1,1]); self.assertEqual({x['lane'] for x in self.m['targets']},{'google','groq'})
    def test_reference_corpus_validates(self):
        m,cases=v12.validate_corpus(Path('fixtures/natural-language-e2e-v12')); self.assertEqual(m['corpus_identity'],'natural-language-e2e-v12'); self.assertEqual(len(cases),13)
if __name__=='__main__': unittest.main()
