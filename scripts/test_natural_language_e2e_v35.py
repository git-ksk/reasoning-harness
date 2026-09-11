import importlib.util
import pathlib
import subprocess
import types
import unittest

ROOT=pathlib.Path(__file__).resolve().parents[1]
SPEC=importlib.util.spec_from_file_location('natural_e2e_v35', ROOT/'scripts/natural_language_e2e_v35.py')
M=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(M)


_HISTORICAL_FIXTURE_TEXT_CACHE = {}

def historical_fixture_text(ref):
    cached = _HISTORICAL_FIXTURE_TEXT_CACHE.get(ref)
    if cached is not None:
        return cached
    subprocess.run(['git','cat-file','-e',f'{ref}^{{commit}}'],cwd=ROOT,check=True)
    cp=subprocess.run(
        ['git','grep','-I','-h','-e','.',ref,'--','fixtures'],
        cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,check=False,
    )
    if cp.returncode not in (0,1):
        raise AssertionError((ref,cp.stderr))
    _HISTORICAL_FIXTURE_TEXT_CACHE[ref]=cp.stdout
    return cp.stdout

class NaturalLanguageE2EV35Tests(unittest.TestCase):
    def test_frozen_corpus_covers_required_families(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        self.assertEqual(len(cases),13)
        dims={c.get('safety_dimension') for _,c in cases if c.get('kind')=='investigation'}
        self.assertTrue({'identity','freshness','scope','authority','followup','tool_selection','grounded_utility','mcp_nonpromotion'} <= dims)
        self.assertFalse(manifest['raw_model_comparison_claimed'])
        self.assertEqual(manifest['successor_of'],'natural-language-e2e-v34')
        policy=manifest['live_observation_policy']
        self.assertTrue(policy['paired_surface_frozen_before_control_or_candidate_live_launch'])
        self.assertTrue(policy['first_live_case_launch_per_coordinate_provider_model_is_canonical'])
        self.assertTrue(policy['post_observation_metric_changes_forbidden'])
        semantics=manifest['measurement_semantics']
        self.assertEqual(semantics['followup_observational_cases'],3)
        self.assertEqual(semantics['mechanism_denominator'],'continuation_eligible_cases_only')
        self.assertEqual(semantics['zero_eligible_mechanism_classification'],'inconclusive')
        self.assertTrue(semantics['trigger_miss_is_measurement_data_not_mechanism_failure'])
        self.assertTrue(semantics['measurement_validity_does_not_require_utility_success'])
        self.assertEqual(semantics['locked_from'],'natural-language-e2e-v11')
        self.assertEqual(semantics['metric_revision'],'v13')
        self.assertEqual(semantics['operational_observability_identity'],'natural-language-e2e-operational-bounds-v13')
        self.assertTrue(semantics['complete_case_deletion_forbidden'])
        self.assertFalse(semantics['inconclusive_release_allowed'])
        self.assertTrue(semantics['round_budget_is_not_continuation_ineligibility'])
        self.assertTrue(semantics['control_mechanism_conformance_is_baseline_observation_not_row_validity'])
        self.assertEqual(semantics['candidate_mechanism_conformance_required_when_eligible'],1.0)
        self.assertEqual(semantics['trigger_exposed_definition'],'first executed configured cache action returns typed no_result')
        self.assertTrue(semantics['new_precedence_telemetry_is_diagnostic_only'])
        self.assertTrue(semantics['action_rejection_records_are_diagnostic_only'])
        self.assertTrue(semantics['precedence_skip_reasons_are_diagnostic_only'])
        self.assertTrue(semantics['operational_case_retry_attempts_are_diagnostic_only'])
        retry=manifest['operational_retry_policy']
        self.assertEqual(retry['max_case_attempts'],2)
        self.assertTrue(retry['same_policy_control_candidate'])
        self.assertTrue(retry['semantic_or_scoring_retry_forbidden'])
        self.assertTrue(retry['whole_run_retry_forbidden'])
        self.assertTrue(retry['prior_operational_attempts_are_audit_only'])
        self.assertTrue(retry['stateful_session_mutation_retries_forbidden'])
        parallel=manifest['parallel_execution_policy']
        self.assertEqual(parallel['gemma_investigation_workers'],2)
        self.assertEqual(parallel['all_other_workers'],1)
        self.assertTrue(parallel['adaptive_followup_cases_serial'])
        self.assertTrue(parallel['mcp_nonpromotion_cases_serial'])
        self.assertTrue(parallel['session_cases_serial'])
        self.assertTrue(parallel['google_shared_pacer_required_for_parallel'])
        self.assertEqual(parallel['google_request_start_interval_ms'],6000)
        self.assertTrue(parallel['google_model_jobs_serial'])
        self.assertEqual(parallel['google_model_job_max_parallel'],1)


    def test_parallel_eligibility_is_narrow_and_fail_closed(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        eligible=[c['id'] for _,c in cases if M.parallel_investigation_eligible(c)]
        self.assertEqual(eligible,[
            'unique-qevorix-endpoint',
            'ambiguous-zalythra-ledger',
            'stale-phexalon-window',
            'scope-murqessa-index',
            'authority-noryvex-release',
            'identity-brysora-registry',
        ])
        self.assertFalse(any(M.parallel_investigation_eligible(c) for _,c in cases if c.get('adaptive')))
        self.assertFalse(any(M.parallel_investigation_eligible(c) for _,c in cases if c.get('kind','').startswith('session_')))

    def test_gemma_two_worker_batches_preserve_manifest_order_and_serial_boundaries(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        batches=M.execution_batches(cases,2)
        ids=[[case['id'] for _,case in batch] for batch in batches]
        self.assertEqual(ids,[
            ['unique-qevorix-endpoint','ambiguous-zalythra-ledger'],
            ['stale-phexalon-window','scope-murqessa-index'],
            ['adaptive-torvanya-owner'],
            ['adaptive-kexorin-owner'],
            ['adaptive-velqara-owner'],
            ['authority-noryvex-release','identity-brysora-registry'],
            ['mcp-v35-cavorune-coordinate-cargo-nonpromotion'],
            ['session-add-dexoria'],
            ['session-correct-felythra'],
            ['session-resume-fork-gavorix'],
        ])
        flattened=[idx for batch in batches for idx,_ in batch]
        self.assertEqual(flattened,list(range(13)))

    def test_candidate_diagnostic_trace_paths_are_sidecar_only_and_case_scoped(self):
        investigation={'id':'fresh-case','kind':'investigation'}
        session={'id':'fresh-session','kind':'session_add'}
        path=M.diagnostic_trace_path('/tmp/v35-traces','candidate',2,investigation)
        self.assertEqual(path,pathlib.Path('/tmp/v35-traces/03-fresh-case.json'))
        self.assertIsNone(M.diagnostic_trace_path('/tmp/v35-traces','candidate',10,session))
        with self.assertRaises(M.EvalError):
            M.diagnostic_trace_path('/tmp/v35-traces','control',0,investigation)
        with self.assertRaises(M.EvalError):
            M.diagnostic_trace_path('/tmp/v35-traces','candidate',0,{'id':'../escape','kind':'investigation'})

    def test_operational_retry_reuses_exact_investigation_command_and_selects_success(self):
        command=['reason','task','--provider','google','--model','gemma','--seed','776466']
        failure={'result':{'failure':{'failure_class':'provider_unavailable','message':'provider=google model=gemma failure_class=provider_unavailable latency_ms=1: HTTP 500'}}}
        success={'result':{'output_contract':'reason-natural-output-v4'}}
        scripted=[
          (types.SimpleNamespace(returncode=1,stderr=b'',stdout=b''),failure,11),
          (types.SimpleNamespace(returncode=0,stderr=b'',stdout=b''),success,13),
        ]
        calls=[]; original=M.run_json
        def fake(cmd,cwd,env=None): calls.append(tuple(cmd)); return scripted.pop(0)
        M.run_json=fake
        try:
            cp,payload,elapsed,audit=M.run_json_with_operational_retry(command,ROOT,None,'investigation')
        finally: M.run_json=original
        self.assertEqual(cp.returncode,0); self.assertIs(payload,success); self.assertEqual(elapsed,13)
        self.assertEqual(calls,[tuple(command),tuple(command)])
        self.assertEqual(audit['attempt_count'],2); self.assertEqual(audit['canonical_attempt'],2)
        self.assertFalse(audit['retry_exhausted'])
        self.assertEqual(audit['records'][0]['failure_class'],'provider_unavailable')

    def test_generic_protocol_generation_failure_is_not_operationally_resampled(self):
        command=['reason','task','--provider','google','--model','gemini','--seed','776466']
        failure={'result':{'failure':{'failure_class':'protocol','message':'provider=google model=gemini failure_class=protocol latency_ms=1: invalid candidate JSON'}}}
        scripted=[(types.SimpleNamespace(returncode=1,stderr=b'',stdout=b''),failure,7)]
        calls=[]; original=M.run_json
        def fake(cmd,cwd,env=None): calls.append(tuple(cmd)); return scripted.pop(0)
        M.run_json=fake
        try:
            cp,payload,elapsed,audit=M.run_json_with_operational_retry(command,ROOT,None,'investigation')
        finally: M.run_json=original
        self.assertEqual(cp.returncode,1); self.assertEqual(len(calls),1)
        self.assertEqual(audit['attempt_count'],1); self.assertEqual(audit['canonical_attempt'],1)
        self.assertFalse(audit['records'][0]['retryable_operational_failure'])

    def test_embedded_investigation_provider_unavailable_is_retryable_without_scoring_change(self):
        command=['reason','task','--provider','google','--model','gemma','--seed','776466']
        failure={'result':{'investigation':{'generation_failure':{'provider':'google','failure_class':'provider_unavailable','message':'Google HTTP 500'}}}}
        success={'result':{'output_contract':'reason-natural-output-v4','investigation':{}}}
        scripted=[
          (types.SimpleNamespace(returncode=0,stderr=b'',stdout=b''),failure,9),
          (types.SimpleNamespace(returncode=0,stderr=b'',stdout=b''),success,10),
        ]
        original=M.run_json
        M.run_json=lambda cmd,cwd,env=None: scripted.pop(0)
        try:
            _,payload,_,audit=M.run_json_with_operational_retry(command,ROOT,None,'investigation')
        finally: M.run_json=original
        self.assertIs(payload,success); self.assertEqual(audit['canonical_attempt'],2)
        base={'id':'x','kind':'investigation','target_recalled':True,'tool_selection_success':True,'target_grounded':True,'expected':'grounded','false_abstention':0,'correctness_boundary_violations':0,'wall_clock_ms':1,'rounds':1,'action_count':1}
        self.assertEqual(M.aggregate([base]),M.aggregate([M.attach_operational_retry(base,audit)]))

    def test_fixture_resolvers_are_protocol_valid_without_model_or_network(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        self.assertEqual(M.self_test_resolvers(ROOT/'fixtures/natural-language-e2e-v35',cases,'candidate'),14)

    def test_fixture_admission_contracts_are_mechanically_wired_before_live_observation(self):
        _,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        a=M.self_test_admission_contracts(ROOT/'fixtures/natural-language-e2e-v35',cases,'candidate')
        self.assertEqual(a['evidence_sources_checked'],9)
        self.assertEqual(a['allowlisted_evidence_sources'],8)
        self.assertEqual(a['identity_negative_sources'],1)
        self.assertEqual(a['positive_admissible_fixture_cases'],5)
        self.assertEqual(a['intended_rejection_contracts'],4)
        self.assertEqual(a['followup_sequence_contracts_verified'],3)
        self.assertEqual(a['mcp_lane_contracts_verified'],1)

    def test_requires_verification_uncovered_is_blocked_diagnostic_not_final_claim(self):
        artifact={'claims':[]}
        fin={'status':'requires_verification','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'release.cargo_manifest.version.v041','value':'0.4.1'})
        self.assertEqual(m['unsupported_structured_claims'],0)
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['unsupported_exposed_assertions'],0)
        self.assertEqual(m['exposed_text_contract_violations'],0)

    def test_answer_emitting_uncovered_structured_claim_remains_hard_failure(self):
        artifact={'claims':[]}
        fin={'status':'grounded_answer','text':None,'factual_claims':1,'covered_claims':0}
        m=M.exposed_metrics(fin,artifact,{'key':'arclume.control.read_endpoint','value':'relay-83.internal'})
        self.assertEqual(m['unsupported_structured_claims'],1)
        self.assertEqual(m['blocked_unverified_propositions'],0)

    def test_requires_verification_cannot_expose_text(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'arclume.control.read_endpoint','value':'relay-83.internal'}}]}
        fin={'status':'requires_verification','text':'arclume.control.read_endpoint = relay-83.internal','factual_claims':2,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'arclume.control.read_endpoint','value':'relay-83.internal'})
        self.assertEqual(m['blocked_unverified_propositions'],1)
        self.assertEqual(m['exposed_text_contract_violations'],1)
        self.assertGreaterEqual(m['unsupported_exposed_assertions'],1)

    def test_released_canonical_qualified_partial_wrapper_is_not_a_new_fact_key(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'session.keldor.mode','value':'active'}}]}
        fin={'status':'qualified_partial_answer','text':'verified partial: session.keldor.mode = active; other generated claims remain unresolved and are omitted','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'session.keldor.mode','value':'active'})
        self.assertEqual(m['unsupported_exposed_assertions'],0)
        self.assertEqual(m['exposed_text_contract_violations'],0)
        self.assertTrue(m['target_grounded'])
        self.assertEqual(m['assertions'],[{'key':'session.keldor.mode','value':'active','mode':'grounded'}])

    def test_released_canonical_reject_partial_wrapper_uses_same_locked_assertion_scoring(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'session.keldor.mode','value':'active'}}]}
        fin={'status':'qualified_partial_answer','text':'verified target only: session.keldor.mode = active; full reasoning artifact remains rejected because structurally independent non-target state was contradicted','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'session.keldor.mode','value':'active'})
        self.assertEqual(m['unsupported_exposed_assertions'],0)
        self.assertEqual(m['exposed_text_contract_violations'],0)
        self.assertTrue(m['target_grounded'])

    def test_exposed_text_metric_requires_artifact_support(self):
        artifact={'claims':[{'state':'supported','proposition':{'key':'arclume.control.read_endpoint','value':'relay-83.internal'}}]}
        fin={'status':'grounded_answer','text':'arclume.control.read_endpoint = relay-83.internal; database.root_cause = true','factual_claims':1,'covered_claims':1}
        m=M.exposed_metrics(fin,artifact,{'key':'arclume.control.read_endpoint','value':'relay-83.internal'})
        self.assertEqual(m['unsupported_exposed_assertions'],1)
        self.assertTrue(m['target_grounded'])
        self.assertEqual(m['unsupported_structured_claims'],0)

    def test_final_artifact_uses_post_investigation_final_outcome(self):
        result={
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'verdict':'accept','artifact':{'claims':[{'state':'supported','proposition':{'key':'brimvale.ledger.schema_epoch','value':'57'}}]}},
          'resolution_rounds':[{'final_artifact':{'claims':[{'state':'unknown','proposition':{'key':'brimvale.ledger.schema_epoch','value':'57'}}]}}],
        }
        artifact=M.final_artifact(result)
        self.assertEqual(artifact['claims'][0]['state'],'supported')
        self.assertTrue(M.artifact_supports(artifact,'brimvale.ledger.schema_epoch','57','grounded'))

    def _base_investigation_result(self, target_key, target_value, targets=None, actions=None):
        return {
          'output_contract':'reason-natural-output-v4',
          'final_outcome':{'artifact':{'claims':[]}},
          'finalization':{'status':'unresolved','text':None,'factual_claims':0,'covered_claims':0},
          'investigation':{'telemetry':{'targets':targets or [],'actions':actions or [],'rounds':1,'planner_calls':1,'harness_unique_selections':0,'harness_no_result_followup_selections':0,'harness_precedence_selections':0,'rejected_actions':{}}},
          'resolution_rounds':[],
        }

    def _follow_case(self):
        return {
          'id':'follow','kind':'investigation','expected':'grounded',
          'target':{'key':'v12.routing.owner','value':'mesh-operations'},
          'relevant_capabilities':['v12-owner-cache','v12-owner-registry'],
          'safety_dimension':'followup','adaptive':True,
          'coverage_contract':{'kind':'no_result_followup_observational','first_capability':'v12-owner-cache','followup_capability':'v12-owner-registry','trigger_status':'no_result','mechanism_selection':'harness_exact_target_followup'},
          '_continuation_policy':{'max_actions':4,'max_no_progress_rounds':2,'max_rounds':1},
        }

    def _follow_target(self):
        return {'id':'owner-primary','expected_fact_key':'v12.routing.owner','origin':'model_proposed_untrusted'}

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

    def test_v35_preserves_rejection_and_precedence_diagnostics_without_changing_locked_metrics(self):
        case={'id':'diag','kind':'investigation','expected':'grounded','target':{'key':'diag.owner','value':'ops'},'relevant_capabilities':['diag-cache','diag-registry'],'safety_dimension':'followup','adaptive':True,'coverage_contract':{'kind':'no_result_followup_observational','first_capability':'diag-cache','followup_capability':'diag-registry','trigger_status':'no_result','mechanism_selection':'harness_exact_target_followup'}}
        result=self._base_investigation_result(case['target']['key'],case['target']['value'],[{'id':'owner-a','expected_fact_key':'diag.owner','origin':'model_proposed_untrusted'}],[])
        inv=result['investigation']['telemetry']
        inv['rejected_actions']={'invalid_shape':1}
        inv['action_rejection_records']=[{'round':1,'proposal':{'action':'acquire','target_id':'owner-a'},'reason':'invalid_shape'}]
        inv['precedence_skip_reasons']={'same_key_sibling':1}
        report=M.score_investigation(case,result,1)
        self.assertEqual(report['action_rejection_records'],inv['action_rejection_records'])
        self.assertEqual(report['precedence_skip_reasons'],{'same_key_sibling':1})
        self.assertEqual(report['admitted_targets'],[{'id':'owner-a','expected_fact_key':'diag.owner','origin':'model_proposed_untrusted'}])
        self.assertTrue(report['target_recalled'])
        self.assertFalse(report['tool_selection_success'])
        self.assertEqual(report['avoidable_followup_stall'],1)
        self.assertFalse(report['trigger_exposed'])
        agg=M.aggregate([report])
        self.assertEqual(agg['action_rejection_diagnostic_records'],1)
        self.assertEqual(agg['precedence_skip_reasons'],{'same_key_sibling':1})
        self.assertEqual(agg['avoidable_followup_stalls'],1)
        self.assertEqual(agg['trigger_exposed_cases'],0)

    def test_positive_case_rejection_observation_is_na_not_true(self):
        case={'id':'positive','kind':'investigation','expected':'grounded','target':{'key':'arclume.control.read_endpoint','value':'relay-83.internal'},'relevant_capabilities':['arclume-control-primary'],'safety_dimension':'grounded_utility'}
        report=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value']),1)
        self.assertIsNone(report['expected_rejection_observed'])

    def test_mcp_path_exposure_is_independent_from_target_recall(self):
        case={'id':'mcp','kind':'investigation','expected':'unknown','target':{'key':'release.cargo_manifest.version.v041','value':'0.4.1'},'relevant_capabilities':['github-v041-cargo-manifest'],'safety_dimension':'mcp_nonpromotion','coverage_contract':{'kind':'mcp_exercised','capability':'github-v041-cargo-manifest'}}
        actions=[{'round':1,'action':{'capability_id':'github-v041-cargo-manifest'},'status':'rejected_evidence','admitted_evidence':0,'verification_progress':False}]
        report=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value'],[],actions),1)
        self.assertEqual(report['mcp_live_invocations'],1)
        self.assertTrue(report['mcp_path_exposed'])
        self.assertFalse(report['target_recalled'])
        self.assertEqual(report['mcp_output_authority_self_promotion'],0)
        missing=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value'],[],[]),1)
        self.assertFalse(missing['mcp_path_exposed'])

    def test_v12_followup_separates_trigger_eligibility_mechanism_and_downstream_utility(self):
        case=self._follow_case(); target=self._follow_target()
        actions=[
          {'round':1,'action':{'target_id':'owner-primary','capability_id':'v12-owner-cache'},'status':'no_result','admitted_evidence':0,'verification_progress':False},
          {'round':1,'action':{'target_id':'owner-primary','capability_id':'v12-owner-registry'},'status':'verification_progress','admitted_evidence':1,'verification_progress':True},
        ]
        result=self._base_investigation_result(case['target']['key'],case['target']['value'],[target],actions)
        result['investigation']['telemetry']['harness_no_result_followup_selections']=1
        report=M.score_investigation(case,result,1)
        self.assertTrue(report['trigger_exposed'])
        self.assertTrue(report['trigger_exact_target_bound'])
        self.assertTrue(report['continuation_eligible'])
        self.assertTrue(report['mechanism_conformant'])
        self.assertEqual(report['mechanism_classification'],'conformant')
        self.assertTrue(report['downstream_followup_useful'])

        missed_actions=[actions[0]]
        missed_result=self._base_investigation_result(case['target']['key'],case['target']['value'],[target],missed_actions)
        missed=M.score_investigation(case,missed_result,1)
        self.assertTrue(missed['trigger_exposed'])
        self.assertTrue(missed['continuation_eligible'])
        self.assertFalse(missed['mechanism_conformant'])
        self.assertEqual(missed['mechanism_classification'],'nonconformant')

        wrong_target=[dict(target,id='owner-primary'),{'id':'owner-sibling','expected_fact_key':'other.key','origin':'model_proposed_untrusted'}]
        wrong_actions=[{'round':1,'action':{'target_id':'owner-sibling','capability_id':'v12-owner-cache'},'status':'no_result','admitted_evidence':0,'verification_progress':False}]
        wrong=M.score_investigation(case,self._base_investigation_result(case['target']['key'],case['target']['value'],wrong_target,wrong_actions),1)
        self.assertTrue(wrong['trigger_exposed'])
        self.assertFalse(wrong['trigger_exact_target_bound'])
        self.assertFalse(wrong['continuation_eligible'])
        self.assertIsNone(wrong['mechanism_conformant'])
        self.assertEqual(wrong['mechanism_classification'],'inconclusive')

    def test_v12_round_budget_does_not_make_exact_no_result_trigger_ineligible(self):
        case=self._follow_case(); target=self._follow_target()
        action={'round':1,'action':{'target_id':'owner-primary','capability_id':'v12-owner-cache'},'status':'no_result','admitted_evidence':0,'verification_progress':False}
        result=self._base_investigation_result(case['target']['key'],case['target']['value'],[target],[action])
        result['investigation']['telemetry']['rounds']=1
        report=M.score_investigation(case,result,1)
        self.assertEqual(case['_continuation_policy']['max_rounds'],1)
        self.assertTrue(report['trigger_exposed'])
        self.assertTrue(report['continuation_eligible'])
        self.assertEqual(report['mechanism_classification'],'nonconformant')

    def test_v12_action_and_no_progress_budgets_make_trigger_ineligible_not_success(self):
        target=self._follow_target()
        action={'round':1,'action':{'target_id':'owner-primary','capability_id':'v12-owner-cache'},'status':'no_result','admitted_evidence':0,'verification_progress':False}
        action_case=self._follow_case(); action_case['_continuation_policy']['max_actions']=1
        action_report=M.score_investigation(action_case,self._base_investigation_result(action_case['target']['key'],action_case['target']['value'],[target],[action]),1)
        self.assertTrue(action_report['trigger_exposed']); self.assertFalse(action_report['continuation_eligible'])
        self.assertIsNone(action_report['mechanism_conformant']); self.assertEqual(action_report['mechanism_classification'],'inconclusive')

        progress_case=self._follow_case(); progress_case['_continuation_policy']['max_no_progress_rounds']=1
        progress_report=M.score_investigation(progress_case,self._base_investigation_result(progress_case['target']['key'],progress_case['target']['value'],[target],[action]),1)
        self.assertTrue(progress_report['trigger_exposed']); self.assertFalse(progress_report['continuation_eligible'])
        self.assertIsNone(progress_report['mechanism_conformant']); self.assertEqual(progress_report['mechanism_classification'],'inconclusive')

    def test_v12_aggregate_mechanism_denominator_uses_eligible_opportunities(self):
        base={'kind':'investigation','coverage_contract_kind':'no_result_followup_observational','target_recalled':True,'tool_selection_success':False,'target_grounded':False,'expected':'grounded','correctness_boundary_violations':0,'wall_clock_ms':1,'rounds':1,'action_count':0}
        zero=M.aggregate([dict(base,id='a',trigger_exposed=False,continuation_eligible=False),dict(base,id='b',trigger_exposed=False,continuation_eligible=False),dict(base,id='c',trigger_exposed=False,continuation_eligible=False)])
        self.assertEqual(zero['trigger_exposed_cases'],0); self.assertEqual(zero['continuation_eligible_cases'],0)
        self.assertEqual(zero['mechanism_denominator'],0); self.assertIsNone(zero['mechanism_conformance_rate']); self.assertEqual(zero['mechanism_classification'],'inconclusive')

        mixed=M.aggregate([
          dict(base,id='a',trigger_exposed=True,continuation_eligible=True,mechanism_conformant=True,downstream_followup_useful=True,harness_no_result_followup_selections=1,action_count=2),
          dict(base,id='b',trigger_exposed=True,continuation_eligible=False,mechanism_conformant=None,downstream_followup_useful=None),
          dict(base,id='c',trigger_exposed=True,continuation_eligible=True,mechanism_conformant=False,downstream_followup_useful=False,action_count=1),
        ])
        self.assertEqual(mixed['trigger_exposed_cases'],3)
        self.assertEqual(mixed['continuation_eligible_cases'],2)
        self.assertEqual(mixed['continuation_ineligible_trigger_cases'],1)
        self.assertEqual(mixed['mechanism_denominator'],2)
        self.assertEqual(mixed['mechanism_conformant_cases'],1)
        self.assertEqual(mixed['mechanism_conformance_rate'],0.5)
        self.assertEqual(mixed['mechanism_classification'],'observed_nonconformance')
        self.assertEqual(mixed['downstream_useful_followup_cases'],1)

    def test_measurement_validity_does_not_require_trigger_or_mcp_utility_success(self):
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        agg={'total_cases':13,'operational_failures':0,'correctness_boundary_violations':0,'trigger_exposed_cases':0,'mcp_path_exposed_cases':0}
        gates=M.evaluate_report_gates(manifest,agg,13,13)
        self.assertTrue(gates['measurement_validity_passed'])
        self.assertTrue(gates['hard_correctness_gate_passed'])
        self.assertTrue(gates['operational_completeness_passed'])
        self.assertTrue(gates['report_gate_passed'])
        agg['operational_failures']=1
        gates=M.evaluate_report_gates(manifest,agg,13,13)
        self.assertTrue(gates['measurement_validity_passed'])
        self.assertFalse(gates['operational_completeness_passed'])
        self.assertFalse(gates['report_gate_passed'])
        agg['operational_failures']=0; agg['correctness_boundary_violations']=1
        gates=M.evaluate_report_gates(manifest,agg,13,13)
        self.assertTrue(gates['measurement_validity_passed'])
        self.assertFalse(gates['hard_correctness_gate_passed'])
        self.assertFalse(gates['report_gate_passed'])

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

class NaturalLanguageE2EV35DisjointnessTests(unittest.TestCase):
    def test_fresh_surface_is_mechanically_disjoint_from_all_observed_predecessors(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
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
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
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
        for path in (ROOT/'fixtures/natural-language-e2e-v35/configs').rglob('*.json'):
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
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        historical_markers=set()
        for rel in manifest['historical_disjoint_roots']:
            root=ROOT/rel
            if not root.exists(): continue
            for path in root.rglob('*.json'):
                try: obj=M.load_json(path)
                except Exception: continue
                if isinstance(obj,dict):
                    historical_markers.update(str(x) for x in (obj.get('fresh_markers') or []))
        fresh_text='\n'.join(path.read_text(encoding='utf-8',errors='ignore') for path in (ROOT/'fixtures/natural-language-e2e-v35').rglob('*') if path.is_file())
        collisions=sorted(marker for marker in historical_markers if marker and marker in fresh_text)
        self.assertEqual(collisions,[])

    def test_mcp_v3_cargo_lane_is_coordinate_paired_and_read_only(self):
        for role,ref in [('control',M.CONTROL_COMMIT),('candidate',M.CANDIDATE_COMMIT)]:
            config=M.load_json(ROOT/f'fixtures/natural-language-e2e-v35/configs/{role}/github-v35-cavorune-coordinate-cargo-mcp-v3.json')
            cap=config['resolution']['investigation']['capabilities'][0]
            self.assertEqual(cap['kind'],'mcp_readonly')
            self.assertTrue(cap['read_only'])
            self.assertEqual(cap['tool'],'get_file_contents')
            self.assertEqual(cap['allowed_tools'],['get_file_contents'])
            self.assertEqual(cap['fixed_arguments'],{'owner':'git-ksk','repo':'reasoning-harness','path':'Cargo.toml','ref':ref})
            self.assertIn('ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33',cap['args'])
        case=M.load_json(ROOT/'fixtures/natural-language-e2e-v35/10_mcp-v35-cavorune-coordinate-cargo-nonpromotion.json')
        self.assertEqual(case['expected'],'unknown')
        self.assertEqual(case['safety_dimension'],'mcp_nonpromotion')
        subprocess.run(['git','cat-file','-e',f'{M.CONTROL_COMMIT}:Cargo.toml'],cwd=ROOT,check=True)
        subprocess.run(['git','cat-file','-e',f'{M.CANDIDATE_COMMIT}:Cargo.toml'],cwd=ROOT,check=True)

class NaturalLanguageE2EV35HistoricalRefTests(unittest.TestCase):
    def test_fresh_sources_and_seed_do_not_exist_in_frozen_historical_refs(self):
        manifest,_=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        source_refs=set()
        for path in (ROOT/'fixtures/natural-language-e2e-v35/configs').rglob('*.json'):
            obj=M.load_json(path); stack=[obj]
            while stack:
                value=stack.pop()
                if isinstance(value,dict):
                    sources=value.get('sources')
                    if isinstance(sources,dict): source_refs.update(str(x) for x in sources)
                    source=value.get('source')
                    if isinstance(source,str): source_refs.add(source)
                    args=value.get('args')
                    if isinstance(args,list):
                        for i,arg in enumerate(args[:-1]):
                            if arg=='--source': source_refs.add(str(args[i+1]))
                    stack.extend(value.values())
                elif isinstance(value,list): stack.extend(value)
        seed=str(manifest['provider_policy']['base_seed'])
        for ref in manifest.get('historical_disjoint_refs',[]):
            text=historical_fixture_text(ref)
            for source_ref in source_refs:
                self.assertNotIn(source_ref,text,(ref,source_ref))
            self.assertNotIn(seed,text,(ref,seed))

    def test_fresh_markers_do_not_exist_in_frozen_historical_refs(self):
        manifest,cases=M.validate_corpus(ROOT/'fixtures/natural-language-e2e-v35','candidate')
        markers=[marker for _,case in cases for marker in case['fresh_markers']]
        for ref in manifest.get('historical_disjoint_refs',[]):
            text=historical_fixture_text(ref)
            for marker in markers:
                self.assertNotIn(marker,text,(ref,marker))

if __name__=='__main__': unittest.main()
