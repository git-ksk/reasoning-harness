import importlib.util, json, pathlib, tempfile, unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('v28_acceptance',ROOT/'scripts/natural_language_e2e_v28_acceptance.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

def report(role,provider,model,stalls,triggers,recall=1.0,tool=1.0,false_abs=0,precedence=0,eligible=None,mechanism=1.0):
    commit=M.CONTROL if role=='control' else M.CANDIDATE
    eligible=triggers if eligible is None else eligible
    return {
      'schema_version':'reason-natural-language-e2e-v28','corpus_identity':'natural-language-e2e-v28','evaluator_identity':'reason-natural-language-e2e-v28','scoring_identity':'natural-language-e2e-scoring-v28-metric-locked-v12',
      'coordinate_role':role,'product_coordinate':{'commit':commit},'provider':provider,'model':model,'seed':95100,'max_tokens':1024,
      'operational_completeness_passed':True,'report_gate_passed':True,
      'aggregate':{
        'total_cases':13,'completed_cases':13,'operational_failures':0,'investigation_cases':10,'session_cases':3,
        'target_recall':recall,'tool_selection_success_rate':tool,'false_abstentions':false_abs,'avoidable_followup_stalls':stalls,'trigger_exposed_cases':triggers,
        'continuation_eligible_cases':eligible,'continuation_ineligible_trigger_cases':max(0,triggers-eligible),
        'mechanism_conformance_rate':mechanism if eligible else None,'harness_precedence_selections':precedence,'model_selected_action_calls':4,'action_rejections':{},
        'provider_calls_observed':10,'tool_calls':4,
        **{k:0 for k in M.ZERO}
      }
    }

class AcceptanceTests(unittest.TestCase):
    def test_v12_released_control_mechanism_defect_is_baseline_observation_not_row_invalidity(self):
        c=report('control','mistral','ministral-8b-latest',1,2,false_abs=7,tool=.8,eligible=2,mechanism=.5)
        n=report('candidate','mistral','ministral-8b-latest',0,3,false_abs=2,tool=1.0,eligible=3,mechanism=1.0)
        row=M.compare_pair(c,n)
        self.assertTrue(row['passed'],row['reasons'])
        self.assertEqual(row['control_metrics']['mechanism_conformance_rate'],.5)
        self.assertEqual(row['candidate_metrics']['mechanism_conformance_rate'],1.0)
        self.assertEqual(row['delta']['mechanism_conformance_rate'],.5)

    def test_v12_candidate_mechanism_nonconformance_remains_hard_failure_when_eligible(self):
        c=report('control','mistral','ministral-8b-latest',2,1,eligible=1,mechanism=.5)
        n=report('candidate','mistral','ministral-8b-latest',1,2,eligible=2,mechanism=.5)
        row=M.compare_pair(c,n)
        self.assertFalse(row['passed'])
        self.assertIn('candidate: #249 candidate mechanism conformance must be 1.0 when legally executable',row['reasons'])

    def test_v12_candidate_with_zero_eligible_opportunities_is_inconclusive_not_mechanism_failure(self):
        c=report('control','mistral','ministral-8b-latest',2,0,eligible=0,mechanism=None)
        n=report('candidate','mistral','ministral-8b-latest',1,1,eligible=0,mechanism=None)
        row=M.compare_pair(c,n)
        self.assertTrue(row['passed'],row['reasons'])
        self.assertIsNone(row['candidate_metrics']['mechanism_conformance_rate'])

    def test_v12_control_correctness_boundary_is_still_hard_failure(self):
        c=report('control','mistral','ministral-8b-latest',2,1)
        n=report('candidate','mistral','ministral-8b-latest',1,2)
        c['aggregate']['correctness_boundary_violations']=1
        row=M.compare_pair(c,n)
        self.assertFalse(row['passed'])
        self.assertIn('control: correctness_boundary_violations must be zero',row['reasons'])

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

    def test_new_v28_diagnostics_are_never_release_gate_inputs(self):
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
