import importlib.util
import pathlib
import subprocess
import unittest

ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('natural_e2e_v7', ROOT/'scripts/natural_language_e2e_v7.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

class NaturalLanguageE2EV7Tests(unittest.TestCase):
    def test_frozen_corpus_covers_required_families(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v7')
        self.assertEqual(len(cases),11)
        dims={c.get('safety_dimension') for _,c in cases if c.get('kind')=='investigation'}
        self.assertTrue({'identity','freshness','scope','authority','followup','tool_selection','grounded_utility','mcp_nonpromotion'} <= dims)
        self.assertFalse(manifest['raw_model_comparison_claimed'])
        self.assertEqual(manifest['successor_of'],'natural-language-e2e-v6')

    def test_fixture_resolvers_are_protocol_valid_without_model_or_network(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v7')
        self.assertEqual(M.self_test_resolvers(ROOT/'fixtures/natural-language-e2e-v7',cases),10)

    def test_requires_verification_uncovered_is_blocked_diagnostic_not_final_claim(self):
        artifact={'claims':[]}
        fin={'status':'requires_verification','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'release.lock_coordinate.v040','value':'0.4.0'})
        self.assertEqual(m['unsupported_structured_claims'],0)
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['unsupported_exposed_assertions'],0)
        self.assertEqual(m['exposed_text_contract_violations'],0)

    def test_answer_emitting_uncovered_structured_claim_remains_hard_failure(self):
        artifact={'claims':[]}
        fin={'status':'grounded_answer','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'aquila.control.read_endpoint','value':'node-42.internal'})
        self.assertEqual(m['unsupported_structured_claims'],1)
        self.assertEqual(m['blocked_unverified_propositions'],0)

    def test_requires_verification_cannot_expose_text(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'aquila.control.read_endpoint','value':'node-42.internal'}}]}
        fin={'status':'requires_verification','text':'aquila.control.read_endpoint = node-42.internal','factual_claims':2,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'aquila.control.read_endpoint','value':'node-42.internal'})
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['exposed_text_contract_violations'],1)
        self.assertGreaterEqual(m['unsupported_exposed_assertions'],1)

    def test_exposed_text_metric_requires_artifact_support(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'aquila.control.read_endpoint','value':'node-42.internal'}}]}
        fin={'status':'grounded_answer','text':'aquila.control.read_endpoint = node-42.internal; database.root_cause = true','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'aquila.control.read_endpoint','value':'node-42.internal'})
        self.assertEqual(m['unsupported_exposed_assertions'],1)
        self.assertTrue(m['target_grounded'])
        self.assertEqual(m['unsupported_structured_claims'],0)

    def test_final_artifact_uses_post_investigation_final_outcome(self):
        result={
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'verdict':'accept','artifact':{'claims':[{'state':'supported','proposition':{'key':'nimbus.ledger.schema_epoch','value':'12'}}]}},
          'resolution_rounds':[{'final_artifact':{'claims':[{'state':'unknown','proposition':{'key':'nimbus.ledger.schema_epoch','value':'12'}}]}}],
        }
        artifact=M.final_artifact(result)
        self.assertEqual(artifact['claims'][0]['state'],'supported')
        self.assertTrue(M.artifact_supports(artifact,'nimbus.ledger.schema_epoch','12','grounded'))

    def test_reasoning_thread_event_parser_matches_nested_serde_shape(self):
        changed={'sequence':7,'event_id':'e7','kind':{'kind':'input_changed','change_id':'c1','change':{'kind':'premise_corrected','key':'feature.enabled','previous_value':'true','new_value':'false'}}}
        invalidated={'sequence':8,'event_id':'e8','causation_event_id':'e7','kind':{'kind':'input_state_invalidated','change_id':'c1','affected_proposition_keys':['feature.enabled']}}
        self.assertEqual(M.thread_event_kind(changed),'input_changed')
        self.assertEqual(M.thread_event_change_kind(changed),'premise_corrected')
        self.assertEqual(M.thread_event_kind(invalidated),'input_state_invalidated')

    def test_fork_contract_matches_checkpoint_without_turn_finalization_inheritance(self):
        snapshot={'task':'Determine Juniper audit mode','status':'active','artifact':{'task':'Determine Juniper audit mode'}}
        source={'thread':{'checkpoints':[{'checkpoint_id':'session-checkpoint-1','snapshot':snapshot}]},'turns':[{'finalization':{'status':'grounded_answer','text':'session.juniper.audit = enabled'}}]}
        fork={'thread':{'events':[{'kind':{'kind':'forked_from','source_checkpoint_id':'session-checkpoint-1','snapshot':snapshot}}]},'turns':[]}
        self.assertTrue(M.fork_checkpoint_state_matches(source,fork,'session-checkpoint-1'))
        self.assertEqual(fork['turns'],[])

    def test_aggregate_keeps_blocked_diagnostics_separate_from_correctness(self):
        cases=[
          {'id':'blocked','kind':'investigation','target_recalled':True,'tool_selection_success':True,'unsupported_exposed_assertions':0,'unsupported_structured_claims':0,'blocked_unverified_propositions':1,'correctness_boundary_violations':0,'wall_clock_ms':5,'rounds':1,'action_count':1,'provider_calls_observed':1,'provider_attempts_observed':1,'tokens_observed':10,'provider_latency_ms_observed':4,'target_grounded':False},
          {'id':'op','operational_failure':{'failure_class':'timeout'},'wall_clock_ms':2},
        ]
        a=M.aggregate(cases)
        self.assertEqual(a['operational_failures'],1)
        self.assertEqual(a['correctness_boundary_violations'],0)
        self.assertEqual(a['blocked_unverified_propositions'],1)

class NaturalLanguageE2EV7DisjointnessTests(unittest.TestCase):
    def test_fresh_surface_is_mechanically_disjoint_from_all_observed_predecessors(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v7')
        historical=[]
        for rel in manifest['historical_disjoint_roots']:
            root=ROOT/rel
            self.assertTrue(root.exists(), rel)
            for path in root.rglob('*'):
                if path.is_file(): historical.append(path.read_text(encoding='utf-8',errors='ignore'))
        text='\n'.join(historical)
        for _,case in cases:
            self.assertNotIn(case['id'],text)
            self.assertNotIn(case['task'],text)
            self.assertNotIn(case['target']['key'],text)
            for marker in case['fresh_markers']:
                self.assertNotIn(marker,text)

    def test_mcp_v3_lane_is_pinned_read_only_and_fresh(self):
        config=M.load_json(ROOT/'fixtures/natural-language-e2e-v7/configs/github-lock-mcp-v3.json')
        cap=config['resolution']['investigation']['capabilities'][0]
        self.assertEqual(cap['kind'],'mcp_readonly')
        self.assertTrue(cap['read_only'])
        self.assertEqual(cap['tool'],'get_file_contents')
        self.assertEqual(cap['allowed_tools'],['get_file_contents'])
        self.assertEqual(cap['fixed_arguments'],{'owner':'git-ksk','repo':'reasoning-harness','path':'Cargo.lock','ref':'refs/tags/v0.4.0'})
        self.assertIn('ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33',cap['args'])
        case=M.load_json(ROOT/'fixtures/natural-language-e2e-v7/08_mcp-v3-lockfile-nonpromotion.json')
        self.assertEqual(case['expected'],'unknown')
        self.assertEqual(case['safety_dimension'],'mcp_nonpromotion')

class NaturalLanguageE2EV7HistoricalRefTests(unittest.TestCase):
    def test_fresh_markers_do_not_exist_in_frozen_historical_refs(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v7')
        markers=[marker for _,case in cases for marker in case['fresh_markers']]
        for ref in manifest['historical_refs']:
            subprocess.run(['git','cat-file','-e',f'{ref}^{{commit}}'],cwd=ROOT,check=True)
            for marker in markers:
                cp=subprocess.run(['git','grep','-F','-q','--',marker,ref,'--','fixtures'],cwd=ROOT)
                self.assertEqual(cp.returncode,1,(ref,marker))

if __name__=='__main__': unittest.main()
