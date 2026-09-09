import importlib.util, json, pathlib, tempfile, unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('v24_acceptance',ROOT/'scripts/natural_language_e2e_v24_acceptance.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

def report(role,provider,model,stalls,triggers,recall=1.0,tool=1.0,false_abs=0,precedence=0):
    commit=M.CONTROL if role=='control' else M.CANDIDATE
    return {
      'schema_version':'reason-natural-language-e2e-v24','corpus_identity':'natural-language-e2e-v24','evaluator_identity':'reason-natural-language-e2e-v24','scoring_identity':'natural-language-e2e-scoring-v24-metric-locked-v11',
      'coordinate_role':role,'product_coordinate':{'commit':commit},'provider':provider,'model':model,'seed':81000,'max_tokens':1024,
      'operational_completeness_passed':True,'report_gate_passed':True,
      'aggregate':{
        'total_cases':13,'completed_cases':13,'operational_failures':0,'investigation_cases':10,'session_cases':3,
        'target_recall':recall,'tool_selection_success_rate':tool,'false_abstentions':false_abs,'avoidable_followup_stalls':stalls,'trigger_exposed_cases':triggers,
        'mechanism_conformance_rate':1.0 if triggers else None,'harness_precedence_selections':precedence,'model_selected_action_calls':4,'action_rejections':{},
        'provider_calls_observed':10,'tool_calls':4,
        **{k:0 for k in M.ZERO}
      }
    }

class AcceptanceTests(unittest.TestCase):
    def test_strict_locked_metric_improvement_passes(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',1,2,precedence=3)
        row=M.compare_pair(c,n)
        self.assertTrue(row['passed'])
        self.assertEqual(row['delta']['avoidable_followup_stalls'],-1)
        self.assertEqual(row['delta']['trigger_exposed_cases'],1)

    def test_precedence_is_diagnostic_only(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',1,1,precedence=0)
        row=M.compare_pair(c,n)
        self.assertTrue(row['passed'])
        self.assertEqual(row['candidate_metrics']['harness_precedence_selections'],0)

    def test_new_v24_diagnostics_are_never_release_gate_inputs(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',1,1)
        c['aggregate'].update({'action_rejection_diagnostic_records':0,'precedence_skip_reasons':{}})
        n['aggregate'].update({'action_rejection_diagnostic_records':99,'precedence_skip_reasons':{'same_key_sibling':99},'action_rejection_count':99})
        row=M.compare_pair(c,n)
        self.assertTrue(row['passed'])
        self.assertEqual(row['candidate_metrics']['precedence_skip_reasons'],{'same_key_sibling':99})

    def test_tradeoff_does_not_pass(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',1,0)
        row=M.compare_pair(c,n)
        self.assertFalse(row['passed'])
        self.assertIn('v11-defined trigger reachability regressed vs paired control',row['reasons'])

    def test_flat_non_ceiling_does_not_pass(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',2,1)
        row=M.compare_pair(c,n)
        self.assertFalse(row['passed'])
        self.assertIn('no strict improvement in locked follow-up utility metrics',row['reasons'])

    def test_control_ceiling_requires_exact_preservation(self):
        c=report('control','google','gemma-4-31b-it',0,3)
        self.assertTrue(M.compare_pair(c,report('candidate','google','gemma-4-31b-it',0,3))['passed'])
        self.assertFalse(M.compare_pair(c,report('candidate','google','gemma-4-31b-it',0,2))['passed'])

    def test_non_utility_regression_fails(self):
        c=report('control','google','gemini-3.5-flash-lite',3,0,recall=1.0,tool=.8,false_abs=2)
        n=report('candidate','google','gemini-3.5-flash-lite',2,1,recall=.9,tool=.8,false_abs=2)
        self.assertFalse(M.compare_pair(c,n)['passed'])

    def test_groq_candidate_generic_path(self):
        g=report('candidate','groq','openai/gpt-oss-120b',1,1)
        self.assertTrue(M.groq_row(g)['passed'])

if __name__=='__main__': unittest.main()
