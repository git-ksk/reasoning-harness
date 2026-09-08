import copy,importlib.util,json,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('acc',ROOT/'scripts/natural_language_e2e_v12_acceptance.py'); A=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(A)
class AcceptanceTests(unittest.TestCase):
    def setUp(self): self.m=A.validate_manifest(ROOT/'fixtures/natural-language-e2e-v12/manifest.json')
    def report(self,provider,model):
        return {'schema_version':'reason-natural-language-e2e-v12','corpus_identity':'natural-language-e2e-v12','evaluator_identity':'reason-natural-language-e2e-v12','scoring_identity':'natural-language-e2e-scoring-v12','product_coordinate':{'commit':A.PRODUCT_COMMIT},'provider':provider,'model':model,'seed':61000,'max_tokens':1024,'operational_completeness_passed':True,'report_gate_passed':True,'aggregate':{'completed_cases':13,'operational_failures':0,'correctness_boundary_violations':0,'unsupported_structured_claims':0,'unsupported_exposed_assertions':0,'exposed_text_contract_violations':0,'missed_target_insufficiency':0,'identity_unsafe_admission':0,'mcp_output_authority_self_promotion':0,'session_external_calls_replayed':0,'deterministic_acquisition_ambiguities':0,'duplicate_action_rejections':0,'target_recall':1.0,'tool_selection_success_rate':1.0,'false_abstentions':0,'avoidable_followup_stalls':0,'trigger_exposed_cases':3,'mechanism_conformance_rate':1.0,'investigation_cases':10,'session_cases':3,'provider_calls_observed':10,'tool_calls':6}}
    def test_manifest_forbids_averaging_and_has_exact_rows(self): self.assertEqual(list(self.m['utility_release_gate']['required_models']),A.REQUIRED); self.assertTrue(self.m['utility_release_gate']['no_cross_model_averaging'])
    def test_mistral_and_gemini_require_strict_improvement(self):
        m=A.evaluate_one(self.m,self.report('mistral','ministral-8b-latest')); self.assertTrue(m['passed'])
        r=self.report('mistral','ministral-8b-latest'); r['aggregate']['avoidable_followup_stalls']=2; self.assertFalse(A.evaluate_one(self.m,r)['passed'])
        g=A.evaluate_one(self.m,self.report('google','gemini-3.5-flash-lite')); self.assertTrue(g['passed'])
        r=self.report('google','gemini-3.5-flash-lite'); r['aggregate']['trigger_exposed_cases']=0; r['aggregate']['mechanism_conformance_rate']=None; self.assertFalse(A.evaluate_one(self.m,r)['passed'])
    def test_gemma_ceiling_cannot_be_hidden(self):
        r=self.report('google','gemma-4-31b-it'); self.assertTrue(A.evaluate_one(self.m,r)['passed']); r['aggregate']['trigger_exposed_cases']=2; self.assertFalse(A.evaluate_one(self.m,r)['passed'])
    def test_groq_requires_real_generic_generation_and_tool_path(self):
        r=self.report('groq','openai/gpt-oss-120b'); r['cross_model_replication']=True; self.assertTrue(A.evaluate_one(self.m,r)['passed']); r['aggregate']['tool_calls']=0; self.assertFalse(A.evaluate_one(self.m,r)['passed'])
    def test_any_safety_or_repeat_regression_fails_row(self):
        r=self.report('mistral','ministral-8b-latest'); r['aggregate']['duplicate_action_rejections']=1; self.assertFalse(A.evaluate_one(self.m,r)['passed']); r=self.report('mistral','ministral-8b-latest'); r['aggregate']['correctness_boundary_violations']=1; self.assertFalse(A.evaluate_one(self.m,r)['passed'])
if __name__=='__main__': unittest.main()
