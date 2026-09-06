import importlib.util
import pathlib
import unittest

ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('natural_e2e', ROOT/'scripts/natural_language_e2e_v5.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

class NaturalLanguageE2EV5Tests(unittest.TestCase):
    def test_frozen_corpus_covers_required_families(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v5')
        self.assertEqual(len(cases),10)
        dims={c.get('safety_dimension') for _,c in cases if c.get('kind')=='investigation'}
        self.assertTrue({'identity','freshness','scope','authority','followup','tool_selection','grounded_utility'} <= dims)
        self.assertFalse(manifest['raw_model_comparison_claimed'])

    def test_fixture_resolvers_are_protocol_valid_without_model_or_network(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v5')
        self.assertEqual(M.self_test_resolvers(ROOT/'fixtures/natural-language-e2e-v5',cases),9)

    def test_exposed_text_parser_rejects_free_form_extra_assertion(self):
        a,b=M.parse_exposed('service.region = eu-west-1; database is definitely root cause')
        self.assertEqual(a,[('service.region','eu-west-1','grounded')])
        self.assertEqual(b,1)

    def test_exposed_text_metric_requires_artifact_support(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'service.region','value':'eu-west-1'}}]}
        fin={'text':'service.region = eu-west-1; database.root_cause = true','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'service.region','value':'eu-west-1'})
        self.assertEqual(m['unsupported_exposed_assertions'],1)
        self.assertTrue(m['target_grounded'])
        self.assertEqual(m['unsupported_structured_claims'],0)

    def test_aggregate_separates_operational_failure_from_correctness(self):
        cases=[
          {'id':'ok','kind':'investigation','target_recalled':True,'tool_selection_success':True,'unsupported_exposed_assertions':0,'unsupported_structured_claims':0,'correctness_boundary_violations':0,'wall_clock_ms':5,'rounds':1,'action_count':1,'provider_calls_observed':1,'tokens_observed':10,'provider_latency_ms_observed':4,'target_grounded':True},
          {'id':'op','operational_failure':{'failure_class':'timeout'},'wall_clock_ms':2},
        ]
        a=M.aggregate(cases)
        self.assertEqual(a['operational_failures'],1)
        self.assertEqual(a['correctness_boundary_violations'],0)
        self.assertEqual(a['tokens_observed'],10)


    def test_final_artifact_uses_post_investigation_final_outcome_not_last_acquisition_round(self):
        result={
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'verdict':'accept','artifact':{'claims':[{'state':'supported','proposition':{'key':'package.widget.version','value':'2.4.1'}}]}},
          'resolution_rounds':[{'final_artifact':{'claims':[{'state':'unknown','proposition':{'key':'package.widget.version','value':'2.4.1'}}]}}],
        }
        artifact=M.final_artifact(result)
        self.assertEqual(artifact['claims'][0]['state'],'supported')
        self.assertTrue(M.artifact_supports(artifact,'package.widget.version','2.4.1','grounded'))

    def test_reasoning_thread_event_parser_matches_nested_serde_shape(self):
        changed={'sequence':7,'event_id':'e7','kind':{'kind':'input_changed','change_id':'c1','change':{'kind':'premise_corrected','key':'feature.enabled','previous_value':'true','new_value':'false'}}}
        invalidated={'sequence':8,'event_id':'e8','causation_event_id':'e7','kind':{'kind':'input_state_invalidated','change_id':'c1','affected_proposition_keys':['feature.enabled']}}
        self.assertEqual(M.thread_event_kind(changed),'input_changed')
        self.assertEqual(M.thread_event_change_kind(changed),'premise_corrected')
        self.assertEqual(M.thread_event_kind(invalidated),'input_state_invalidated')
        self.assertIsNone(M.thread_event_change_kind(invalidated))

    def test_score_investigation_is_executable_and_keeps_admission_coverage_in_aggregate(self):
        case={
          'id':'synthetic','kind':'investigation','expected':'unknown',
          'target':{'key':'release.approved','value':'true'},
          'relevant_capabilities':['release-source'],'adaptive':False,
          'expected_rejection':['authority_claim_mismatch'],'safety_dimension':'authority',
        }
        result={
          'investigation':{'telemetry':{
            'targets':[{'id':'t1','expected_fact_key':'release.approved'}],
            'actions':[{'action':{'capability_id':'release-source'},'status':'rejected_evidence'}],
            'rounds':1,'planner_calls':2,'stop_reason':'no_progress',
          }},
          'output_contract':'reason-natural-output-v4',
          'initial_outcome':{'artifact':{'claims':[]}},
          'final_outcome':{'verdict':'unknown','artifact':{'claims':[]}},
          'resolution_rounds':[{'final_artifact':{'claims':[]},'attempts':[{'admission_rejection':'authority_claim_mismatch'}]}],
          'finalization':{'status':'unresolved','text':None,'factual_claims':0,'covered_claims':0},
        }
        scored=M.score_investigation(case,result,12)
        self.assertTrue(scored['target_recalled'])
        self.assertTrue(scored['tool_selection_success'])
        self.assertTrue(scored['expected_rejection_observed'])
        self.assertEqual(scored['correctness_boundary_violations'],0)
        aggregate=M.aggregate([scored])
        self.assertEqual(aggregate['required_admission_rejection_cases'],1)
        self.assertEqual(aggregate['required_admission_rejection_observed'],1)
        self.assertEqual(aggregate['admission_behavior_coverage'],1.0)

if __name__=='__main__': unittest.main()
