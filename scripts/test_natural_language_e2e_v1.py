import importlib.util
import pathlib
import unittest

ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('natural_e2e', ROOT/'scripts/natural_language_e2e_v1.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

class NaturalLanguageE2EV1Tests(unittest.TestCase):
    def test_frozen_corpus_covers_required_families(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v1')
        self.assertEqual(len(cases),10)
        dims={c.get('safety_dimension') for _,c in cases if c.get('kind')=='investigation'}
        self.assertTrue({'identity','freshness','scope','authority','followup','tool_selection','grounded_utility'} <= dims)
        self.assertFalse(manifest['raw_model_comparison_claimed'])

    def test_fixture_resolvers_are_protocol_valid_without_model_or_network(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v1')
        self.assertEqual(M.self_test_resolvers(ROOT/'fixtures/natural-language-e2e-v1',cases),9)

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

if __name__=='__main__': unittest.main()
