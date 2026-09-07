import importlib.util
import pathlib
import subprocess
import unittest

ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('natural_e2e_v10', ROOT/'scripts/natural_language_e2e_v10.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)

class NaturalLanguageE2EV10Tests(unittest.TestCase):
    def test_frozen_corpus_covers_required_families(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        self.assertEqual(len(cases),11)
        dims={c.get('safety_dimension') for _,c in cases if c.get('kind')=='investigation'}
        self.assertTrue({'identity','freshness','scope','authority','followup','tool_selection','grounded_utility','mcp_nonpromotion'} <= dims)
        self.assertFalse(manifest['raw_model_comparison_claimed'])
        self.assertEqual(manifest['successor_of'],'natural-language-e2e-v9')
        policy=manifest['live_observation_policy']
        self.assertTrue(policy['first_live_case_launch_is_canonical'])
        self.assertTrue(policy['pre_live_infrastructure_retry_allowed'])
        self.assertTrue(policy['post_observation_surface_changes_forbidden'])

    def test_fixture_resolvers_are_protocol_valid_without_model_or_network(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        self.assertEqual(M.self_test_resolvers(ROOT/'fixtures/natural-language-e2e-v10',cases),10)

    def test_fixture_admission_contracts_are_mechanically_wired_before_live_observation(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        a=M.self_test_admission_contracts(ROOT/'fixtures/natural-language-e2e-v10',cases)
        self.assertEqual(a['evidence_sources_checked'],7)
        self.assertEqual(a['allowlisted_evidence_sources'],6)
        self.assertEqual(a['identity_negative_sources'],1)
        self.assertEqual(a['positive_admissible_fixture_cases'],3)
        self.assertEqual(a['intended_rejection_contracts'],4)
        self.assertEqual(a['followup_sequence_contracts_verified'],1)
        self.assertEqual(a['mcp_lane_contracts_verified'],1)

    def test_requires_verification_uncovered_is_blocked_diagnostic_not_final_claim(self):
        artifact={'claims':[]}
        fin={'status':'requires_verification','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'release.changelog_marker.v041','value':'0.4.1'})
        self.assertEqual(m['unsupported_structured_claims'],0)
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['unsupported_exposed_assertions'],0)
        self.assertEqual(m['exposed_text_contract_violations'],0)

    def test_answer_emitting_uncovered_structured_claim_remains_hard_failure(self):
        artifact={'claims':[]}
        fin={'status':'grounded_answer','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'lyra.control.read_endpoint','value':'relay-47.internal'})
        self.assertEqual(m['unsupported_structured_claims'],1)
        self.assertEqual(m['blocked_unverified_propositions'],0)

    def test_requires_verification_cannot_expose_text(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'lyra.control.read_endpoint','value':'relay-47.internal'}}]}
        fin={'status':'requires_verification','text':'lyra.control.read_endpoint = relay-47.internal','factual_claims':2,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'lyra.control.read_endpoint','value':'relay-47.internal'})
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['exposed_text_contract_violations'],1)
        self.assertGreaterEqual(m['unsupported_exposed_assertions'],1)

    def test_exposed_text_metric_requires_artifact_support(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'lyra.control.read_endpoint','value':'relay-47.internal'}}]}
        fin={'status':'grounded_answer','text':'lyra.control.read_endpoint = relay-47.internal; database.root_cause = true','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'lyra.control.read_endpoint','value':'relay-47.internal'})
        self.assertEqual(m['unsupported_exposed_assertions'],1)
        self.assertTrue(m['target_grounded'])
        self.assertEqual(m['unsupported_structured_claims'],0)

    def test_final_artifact_uses_post_investigation_final_outcome(self):
        result={
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'verdict':'accept','artifact':{'claims':[{'state':'supported','proposition':{'key':'corvus.ledger.schema_epoch','value':'34'}}]}},
          'resolution_rounds':[{'final_artifact':{'claims':[{'state':'unknown','proposition':{'key':'corvus.ledger.schema_epoch','value':'34'}}]}}],
        }
        artifact=M.final_artifact(result)
        self.assertEqual(artifact['claims'][0]['state'],'supported')
        self.assertTrue(M.artifact_supports(artifact,'corvus.ledger.schema_epoch','34','grounded'))

    def _base_investigation_result(self, target_key, target_value, targets=None, actions=None):
        return {
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'artifact':{'claims':[]}},
          'finalization':{'status':'unresolved','text':None,'factual_claims':0,'covered_claims':0},
          'investigation':{'telemetry':{'targets':targets or [],'actions':actions or [],'rounds':1,'planner_calls':1,'harness_unique_selections':0,'harness_no_result_followup_selections':0}},
          'resolution_rounds':[],
        }

    def test_typed_investigation_operational_failure_is_not_hidden_by_zero_exit(self):
        case={'id':'typed-op','kind':'investigation','expected':'unknown','target':{'key':'x.key','value':'x'},'relevant_capabilities':['cap'],'safety_dimension':'mcp_nonpromotion'}
        result=self._base_investigation_result('x.key','x',[{'expected_fact_key':'x.key'}],[{'round':1,'action':{'capability_id':'cap'},'status':'operational_failure','admitted_evidence':0,'verification_progress':False}])
        result['resolution_rounds']=[{'attempts':[{'status':'negotiation_failure'}]}]
        report=M.score_investigation(case,result,1)
        self.assertEqual(report['typed_operational_action_failures'],1)
        self.assertEqual(report['typed_operational_failure_classes'],{'negotiation_failure':1})
        aggregate=M.aggregate([report])
        self.assertEqual(aggregate['process_operational_failures'],0)
        self.assertEqual(aggregate['typed_operational_action_failures'],1)
        self.assertEqual(aggregate['operational_failures'],1)

    def test_investigation_generation_failure_is_operational_even_with_result_envelope(self):
        case={'id':'gen-op','kind':'investigation','expected':'unknown','target':{'key':'x.key','value':'x'},'relevant_capabilities':[],'safety_dimension':'grounded_utility'}
        result=self._base_investigation_result('x.key','x')
        result['investigation']['generation_failure']={'failure_class':'rate_limit','message':'limited'}
        report=M.score_investigation(case,result,1)
        self.assertEqual(report['generation_failure_observed'],1)
        self.assertEqual(report['generation_failure_class'],'rate_limit')
        aggregate=M.aggregate([report])
        self.assertEqual(aggregate['generation_failures'],1)
        self.assertEqual(aggregate['generation_failure_classes'],{'rate_limit':1})
        self.assertEqual(aggregate['operational_failures'],1)

    def test_positive_case_rejection_observation_is_na_not_true(self):
        case={'id':'positive','kind':'investigation','expected':'grounded','target':{'key':'lyra.control.read_endpoint','value':'relay-47.internal'},'relevant_capabilities':['lyra-control-primary'],'safety_dimension':'grounded_utility'}
        report=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value']),1)
        self.assertIsNone(report['expected_rejection_observed'])

    def test_mcp_live_lane_requires_target_recall_and_actual_nonfailed_invocation(self):
        case={'id':'mcp','kind':'investigation','expected':'unknown','target':{'key':'release.changelog_marker.v041','value':'0.4.1'},'relevant_capabilities':['github-v041-changelog'],'safety_dimension':'mcp_nonpromotion','coverage_contract':{'kind':'mcp_exercised','capability':'github-v041-changelog'}}
        targets=[{'expected_fact_key':'release.changelog_marker.v041'}]
        actions=[{'round':1,'action':{'capability_id':'github-v041-changelog'},'status':'rejected_evidence','admitted_evidence':0,'verification_progress':False}]
        report=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value'],targets,actions),1)
        self.assertEqual(report['mcp_live_invocations'],1)
        self.assertTrue(report['mcp_lane_exercised'])
        self.assertEqual(report['mcp_output_authority_self_promotion'],0)
        missing=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value'],[],[]),1)
        self.assertFalse(missing['mcp_lane_exercised'])

    def test_dedicated_followup_lane_requires_no_result_then_useful_registry(self):
        case={'id':'follow','kind':'investigation','expected':'grounded','target':{'key':'cobalt.routing.owner','value':'edge-routing'},'relevant_capabilities':['cobalt-owner-cache','cobalt-owner-registry'],'safety_dimension':'followup','adaptive':True,'coverage_contract':{'kind':'no_result_followup','first_capability':'cobalt-owner-cache','followup_capability':'cobalt-owner-registry','requires_harness_no_result_followup_selection':True,'expected_model_action_planner_calls':1}}
        actions=[
          {'round':1,'action':{'capability_id':'cobalt-owner-cache'},'status':'no_result','admitted_evidence':0,'verification_progress':False},
          {'round':2,'action':{'capability_id':'cobalt-owner-registry'},'status':'verification_progress','admitted_evidence':1,'verification_progress':True},
        ]
        result=self._base_investigation_result(case['target']['key'],case['target']['value'],[{'expected_fact_key':'cobalt.routing.owner'}],actions)
        result['investigation']['telemetry']['harness_no_result_followup_selections']=1
        report=M.score_investigation(case,result,1)
        self.assertTrue(report['adaptive_followup_sequence_observed'])
        self.assertTrue(report['adaptive_followup_lane_exercised'])
        self.assertTrue(report['followup_opportunity'])
        self.assertTrue(report['useful_followup'])
        self.assertEqual(report['harness_no_result_followup_selections'],1)
        self.assertTrue(report['no_result_followup_harness_selected'])
        self.assertTrue(report['no_result_followup_planner_call_contract_met'])
        self.assertTrue(report['no_result_followup_no_duplicate_execution'])
        weak_actions=[actions[0],dict(actions[1],status='ambiguous',verification_progress=False)]
        weak_result=self._base_investigation_result(case['target']['key'],case['target']['value'],[{'expected_fact_key':'cobalt.routing.owner'}],weak_actions)
        weak_result['investigation']['telemetry']['harness_no_result_followup_selections']=1
        weak=M.score_investigation(case,weak_result,1)
        self.assertTrue(weak['adaptive_followup_sequence_observed'])
        self.assertFalse(weak['adaptive_followup_lane_exercised'])

    def test_aggregate_reports_exercised_lane_coverage_separately(self):
        cases=[
          {'id':'mcp','kind':'investigation','coverage_contract_kind':'mcp_exercised','mcp_lane_exercised':True,'target_recalled':True,'tool_selection_success':True,'target_grounded':False,'expected':'unknown','correctness_boundary_violations':0,'wall_clock_ms':1,'rounds':1,'action_count':1},
          {'id':'follow','kind':'investigation','coverage_contract_kind':'no_result_followup','adaptive_followup_lane_exercised':True,'no_result_followup_harness_selected':True,'no_result_followup_planner_call_contract_met':True,'no_result_followup_no_duplicate_execution':True,'harness_no_result_followup_selections':1,'followup_opportunity':True,'useful_followup':True,'target_recalled':True,'tool_selection_success':True,'target_grounded':False,'expected':'grounded','correctness_boundary_violations':0,'wall_clock_ms':1,'rounds':2,'action_count':2},
        ]
        a=M.aggregate(cases)
        self.assertEqual(a['mcp_live_required_cases'],1)
        self.assertEqual(a['mcp_live_exercised_cases'],1)
        self.assertEqual(a['mcp_live_coverage'],1.0)
        self.assertEqual(a['adaptive_followup_required_cases'],1)
        self.assertEqual(a['adaptive_followup_exercised_cases'],1)
        self.assertEqual(a['adaptive_followup_coverage'],1.0)
        self.assertEqual(a['adaptive_followup_harness_selected_cases'],1)
        self.assertEqual(a['adaptive_followup_planner_contract_cases'],1)
        self.assertEqual(a['adaptive_followup_no_duplicate_cases'],1)
        self.assertEqual(a['harness_no_result_followup_selections'],1)

    def test_reasoning_thread_event_parser_matches_nested_serde_shape(self):
        changed={'sequence':7,'event_id':'e7','kind':{'kind':'input_changed','change_id':'c1','change':{'kind':'premise_corrected','key':'feature.enabled','previous_value':'true','new_value':'false'}}}
        invalidated={'sequence':8,'event_id':'e8','causation_event_id':'e7','kind':{'kind':'input_state_invalidated','change_id':'c1','affected_proposition_keys':['feature.enabled']}}
        self.assertEqual(M.thread_event_kind(changed),'input_changed')
        self.assertEqual(M.thread_event_change_kind(changed),'premise_corrected')
        self.assertEqual(M.thread_event_kind(invalidated),'input_state_invalidated')

    def test_fork_contract_matches_checkpoint_without_turn_finalization_inheritance(self):
        snapshot={'task':'Determine Sierra audit mode','status':'active','artifact':{'task':'Determine Sierra audit mode'}}
        source={'thread':{'checkpoints':[{'checkpoint_id':'session-checkpoint-1','snapshot':snapshot}]},'turns':[{'finalization':{'status':'grounded_answer','text':'session.sierra.audit = enabled'}}]}
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

class NaturalLanguageE2EV10DisjointnessTests(unittest.TestCase):
    def test_fresh_surface_is_mechanically_disjoint_from_all_observed_predecessors(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
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

    def test_fresh_config_source_refs_and_seed_are_disjoint_from_predecessors(self):
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        historical_text=[]
        historical_seeds=set()
        for rel in manifest['historical_disjoint_roots']:
            root=ROOT/rel
            if not root.exists():
                continue
            for path in root.rglob('*'):
                if path.is_file():
                    historical_text.append(path.read_text(encoding='utf-8',errors='ignore'))
            manifest_path=root/'manifest.json'
            if manifest_path.exists():
                try:
                    old=M.load_json(manifest_path)
                except Exception:
                    continue
                seed=(old.get('provider_policy') or {}).get('base_seed')
                if seed is not None:
                    historical_seeds.add(int(seed))
        old_text='\n'.join(historical_text)
        source_refs=set()
        for path in (ROOT/'fixtures/natural-language-e2e-v10/configs').glob('*.json'):
            obj=M.load_json(path)
            stack=[obj]
            while stack:
                value=stack.pop()
                if isinstance(value,dict):
                    sources=value.get('sources')
                    if isinstance(sources,dict):
                        source_refs.update(str(x) for x in sources)
                    source=value.get('source')
                    if isinstance(source,str):
                        source_refs.add(source)
                    args=value.get('args')
                    if isinstance(args,list):
                        for i,arg in enumerate(args[:-1]):
                            if arg=='--source':
                                source_refs.add(str(args[i+1]))
                    stack.extend(value.values())
                elif isinstance(value,list):
                    stack.extend(value)
        self.assertTrue(source_refs)
        for source_ref in sorted(source_refs):
            self.assertNotIn(source_ref,old_text)
        self.assertNotIn(int(manifest['provider_policy']['base_seed']),historical_seeds)

    def test_fresh_surface_contains_no_historical_fresh_markers(self):
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        historical_markers=set()
        for rel in manifest['historical_disjoint_roots']:
            root=ROOT/rel
            if not root.exists(): continue
            for path in root.rglob('*.json'):
                try: obj=M.load_json(path)
                except Exception: continue
                if isinstance(obj,dict):
                    historical_markers.update(str(x) for x in (obj.get('fresh_markers') or []))
        fresh_text='\n'.join(path.read_text(encoding='utf-8',errors='ignore') for path in (ROOT/'fixtures/natural-language-e2e-v10').rglob('*') if path.is_file())
        collisions=sorted(marker for marker in historical_markers if marker and marker in fresh_text)
        self.assertEqual(collisions,[])

    def test_mcp_v3_lane_is_pinned_read_only_and_fresh(self):
        config=M.load_json(ROOT/'fixtures/natural-language-e2e-v10/configs/github-v041-changelog-mcp-v3.json')
        cap=config['resolution']['investigation']['capabilities'][0]
        self.assertEqual(cap['kind'],'mcp_readonly')
        self.assertTrue(cap['read_only'])
        self.assertEqual(cap['tool'],'get_file_contents')
        self.assertEqual(cap['allowed_tools'],['get_file_contents'])
        self.assertEqual(cap['fixed_arguments'],{'owner':'git-ksk','repo':'reasoning-harness','path':'CHANGELOG.md','ref':'refs/tags/v0.4.1'})
        self.assertIn('ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33',cap['args'])
        case=M.load_json(ROOT/'fixtures/natural-language-e2e-v10/08_mcp-v10-v041-changelog-nonpromotion.json')
        self.assertEqual(case['expected'],'unknown')
        self.assertEqual(case['safety_dimension'],'mcp_nonpromotion')
        subprocess.run(['git','cat-file','-e','v0.4.1:CHANGELOG.md'],cwd=ROOT,check=True)

class NaturalLanguageE2EV10HistoricalRefTests(unittest.TestCase):
    def test_fresh_markers_do_not_exist_in_frozen_historical_refs(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v10')
        markers=[marker for _,case in cases for marker in case['fresh_markers']]
        for ref in manifest['historical_refs']:
            subprocess.run(['git','cat-file','-e',f'{ref}^{{commit}}'],cwd=ROOT,check=True)
            for marker in markers:
                cp=subprocess.run(['git','grep','-F','-q','--',marker,ref,'--','fixtures'],cwd=ROOT)
                self.assertEqual(cp.returncode,1,(ref,marker))

if __name__=='__main__': unittest.main()
