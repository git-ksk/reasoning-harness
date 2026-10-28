"""Offline controls for the isolated, Mistral-only ordinary diagnostic profile."""
import contextlib
import copy
import hashlib
import io
import json
import sys
import tempfile
import unittest
import urllib.request
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import development
import make_development_plan
import plan_inputs
import prepare_consumed_sources
import run_lane
import operational_bridge
import run_task
import scorer
import tag_admission

HERE = Path(__file__).resolve().parent

class OrdinaryAdmission(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.corpus = {'cases': [{'id': f'inert-{i}'} for i in range(24)]}
        self.receipt = {'consumed_M1_sha256': development.INPUT_M1}
        self.artifacts = {'/inert-static-input': '0' * 64}
        self.protocol = json.loads((HERE / 'protocol.json').read_bytes())
        self.plan = self.make_plan()

    def make_plan(self):
        output = self.root / 'plan.json'
        args = ['make_development_plan.py', '--receipt', str(self.root / 'receipt.json'),
                '--corpus-dir', str(self.root), '--provider', 'mistral', '--output', str(output)]
        with patch.object(sys, 'argv', args), patch.object(make_development_plan, 'checked_receipt', return_value=self.receipt), \
             patch.object(make_development_plan, 'consumed_corpus', return_value=self.corpus), \
             patch.object(make_development_plan, 'inventory', return_value=self.artifacts), contextlib.redirect_stdout(io.StringIO()):
            make_development_plan.main()
        return json.loads(output.read_bytes())

    def check(self, plan):
        with patch.object(development, 'checked_receipt', return_value=self.receipt), \
             patch.object(development, 'consumed_corpus', return_value=self.corpus), \
             patch.object(plan_inputs, 'inventory', return_value=self.artifacts):
            return run_lane.check_plan(plan, self.protocol)

    def test_complete48_plan_is_ordinary_only_with_exact_reservation(self):
        self.assertEqual(len(self.plan['jobs']), 48)
        self.assertEqual(self.check(self.plan), '0.2022912')
        self.assertEqual(self.plan['token_budget'], 48 * (24000 + 4096))
        self.assertEqual([j['arm'] for j in self.plan['jobs'][:4]], ['baseline','candidate','candidate','baseline'])
        for j in self.plan['jobs']:
            self.assertEqual(j['contract'], 'normal')
            self.assertEqual(j['argv'][j['argv'].index('--track')+1], 'e2e_normal')
            self.assertEqual(j['budget']['max_attempts_per_generation'], 1)
            self.assertNotIn('helper_opt_in', j['argv'])

    def test_tampered_or_helper_plan_is_rejected_before_scheduler(self):
        changes = [
            lambda p:p['jobs'].pop(),
            lambda p:p['jobs'][0].update(contract='helper'),
            lambda p:p['jobs'][0]['argv'].__setitem__(p['jobs'][0]['argv'].index('--track')+1,'helper_opt_in'),
            lambda p:p['jobs'][0]['budget'].update(max_attempts_per_generation=2),
            lambda p:p['jobs'][0]['budget'].update(aggregate_output_tokens=4097),
            lambda p:p['jobs'].__setitem__(slice(0,2),list(reversed(p['jobs'][:2]))),
            lambda p:p.update(token_budget=1),
            lambda p:p.update(currency_self_cap_USD=100),
            lambda p:p.update(M1_sha256='f'*64),
            lambda p:p['jobs'][0]['argv'].extend(['--bin-dir','/other']),
        ]
        for mutate in changes:
            plan=copy.deepcopy(self.plan);mutate(plan)
            with self.subTest(plan=plan['jobs'][0]['id']), self.assertRaises((ValueError,KeyError)):
                self.check(plan)

    def test_callback_refuses_helper_and_unreserved_live_request(self):
        args=SimpleNamespace(track='helper_opt_in',arm='helper',model='fixture-model')
        with self.assertRaisesRegex(ValueError,'ordinary_only'):
            run_task.require_ordinary_callback(args,{})
        args=SimpleNamespace(track='e2e_normal',arm='baseline',model=self.protocol['providers']['mistral']['model'],provider='mistral')
        with self.assertRaisesRegex(ValueError,'callback_requires_frozen_receipt'):
            run_task.require_ordinary_callback(args,{})
        with self.assertRaisesRegex(ValueError,'live_callback_cannot_use_engineering_redirect'):
            run_task.require_ordinary_callback(args,{'ENGINE073_LOOPBACK_BASE_URL':'http://127.0.0.1:1/'})

    def test_callback_accepts_exact_reserved_scope_and_rejects_interrupted_state(self):
        # Inert receipt fixture: cryptographic receipt/proof admission is stubbed
        # here; the actual callback scope and durable reservation checks run.
        receipt=self.root/'receipt.json';receipt.write_text('{"fixture_only":true}')
        digest=hashlib.sha256(receipt.read_bytes()).hexdigest()
        plan=copy.deepcopy(self.plan);plan['artifacts'][str(receipt)]=digest
        job=plan['jobs'][0];lane=self.root/'lane';lane.mkdir();directory=lane/job['id'];directory.mkdir()
        (lane/'plan.json').write_text(json.dumps(plan))
        state={'provider':'mistral','model':plan['model'],'rows':[],
               'guard':{'inflight':True,'latch':None},'current_reserve':28096,'charged_reservations':28096}
        state_path=lane/'state.json';state_path.write_text(json.dumps(state))
        args=SimpleNamespace(track='e2e_normal',arm='baseline',provider='mistral',model=plan['model'],
                             case=plan['case_ids'][0],corpus=self.root/'corpus.json',host_policy=self.root/'host-policy.json',bin_dir=Path('/eval/bin'))
        env={'CLI_REPAIR_DEVELOPMENT_RECEIPT':str(receipt),'CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256':digest}
        with patch.object(Path,'cwd',return_value=directory),patch.object(sys,'argv',job['argv'][1:]), \
             patch.object(development,'checked_receipt',return_value=self.receipt), \
             patch.object(development,'consumed_corpus',return_value=self.corpus), \
             patch.object(run_lane,'check_plan',return_value='0'):
            run_task.require_ordinary_callback(args,env)
            state['guard']['inflight']=False;state_path.write_text(json.dumps(state))
            with self.assertRaisesRegex(ValueError,'callback_not_reserved_inflight'):
                run_task.require_ordinary_callback(args,env)

    def test_tag_history_rejects_cross_head_duplicate_and_rerun(self):
        event={'id':42,'event':'push','created_at':'2026-10-27T00:00:00Z','head_branch':tag_admission.TAG,
               'head_sha':'a'*40,'path':tag_admission.WORKFLOW_PATH,'run_attempt':1}
        def history(events):
            return {'repository_created_date_utc':'2026-10-01','checked_through_date_utc':'2026-10-31',
                    'windows':[{'start':'2026-10-01','end':'2026-10-31','pages':[{'total_count':len(events),'workflow_runs':events}]}]}
        self.assertEqual(tag_admission.single_first_push(history([event]),tag_admission.TAG,'a'*40,'42')['run_id'],42)
        for events in [[{**event,'head_sha':'b'*40}], [{**event,'run_attempt':2}], [event,{**event,'id':43}]]:
            with self.assertRaises(ValueError):tag_admission.single_first_push(history(events),tag_admission.TAG,'a'*40,'42')

    def test_profile_has_fixed_mistral_and_no_helper_or_other_live_provider(self):
        p=self.protocol
        self.assertTrue(p['normal_only'])
        self.assertEqual(p['tracks'],['e2e_normal'])
        self.assertEqual(p['required_providers'],['mistral'])
        self.assertTrue(p['providers']['mistral']['new_live_run_allowed'])
        self.assertFalse(p['providers']['google']['new_live_run_allowed'])
        self.assertFalse(p['providers']['groq']['new_live_run_allowed'])
        self.assertEqual(p['providers']['mistral']['currency_self_cap_USD'],2.5)
        self.assertEqual(p['case_quality_gate']['minimum_f1'],.9)
        self.assertEqual(p['case_quality_gate']['minimum_precision'],.95)
        self.assertEqual(p['case_quality_gate']['minimum_recall'],.85)
        self.assertNotIn('helper_bound',p)

    def test_prepared_runtime_contains_inherited_proof_driver_and_normal_dockerfile(self):
        proof=HERE.parent/'runtime_proof.py'
        dockerfile=HERE/'Dockerfile.runtime'
        self.assertTrue(proof.is_file())
        self.assertIn('BUILT_OFFLINE_NOT_EXECUTION_ADMISSION',proof.read_text())
        self.assertTrue(dockerfile.is_file())
        self.assertIn('ca-certificates gh',dockerfile.read_text())

    def test_missing_pair_is_incomplete_never_f1_or_false_regression(self):
        planned=[];rows=[]
        for i in range(24):
            for arm in ['baseline','candidate']:
                row_id=f'{i}-{arm}'; expected='positive' if i<8 else 'negative'
                planned.append({'id':str(i),'row_id':row_id,'provider':'mistral','track':'e2e_normal','arm':arm,
                                'expected':expected,'required_groups_G':1})
                if i==0 and arm=='candidate':continue
                rows.append({'id':str(i),'row_id':row_id,'provider':'mistral','track':'e2e_normal','arm':arm,
                             'expected':expected,'decision':'accept' if expected=='positive' else 'abstain','complete':True,
                             'hard_violations':[], 'complete_useful_answer':expected=='positive','correct_withhold':expected=='negative',
                             'claim_metrics':{'counts':{'displayed_unique_claims_A':1,'grounded_claims_S':1,'required_groups_G':1,'satisfied_groups_H':1},'f1':1},
                             'citation_counts':{'edges':1,'correct_edges':1,'claims_requiring_citation':1,'correctly_cited_occurrences':1,'literal_quotes':1,'faithful_literal_quotes':1}})
        deps=scorer.frozen_dependencies(HERE.parents[2]);report=scorer.summarize(rows,planned,self.protocol,deps)
        v=report['provider_tracks']['mistral/e2e_normal']
        self.assertIsNone(v['actual_answer_f1'])
        self.assertEqual(v['paired_utility']['paired_complete_cases'],23)
        self.assertEqual(v['paired_utility']['incomplete_pairs'],1)
        self.assertEqual(v['paired_utility']['regressions'],0)
        self.assertFalse(report['semantic_gate_pass'])
        self.assertEqual(set(report['provider_tracks']),{'mistral/e2e_normal'})

    def test_receipt_and_independent_review_fail_closed(self):
        receipt=self.root/'receipt.json';receipt.write_text(json.dumps({'identity':'cli-repair-paired-v1-development-freeze-v1','state':'DEVELOPMENT_FROZEN'}))
        with self.assertRaisesRegex(ValueError,'development_not_frozen'):
            development.checked_receipt(receipt,'mistral')
        with self.assertRaisesRegex(ValueError,'provider_deferred'):
            development.checked_receipt(receipt,'google')
        with self.assertRaisesRegex(ValueError,'provider_deferred'):
            development.checked_receipt(receipt,'groq')
        with self.assertRaisesRegex(ValueError,'development_full_operational_review_missing'):
            development.require_operational_review_identity(self.protocol,{'result':'PASS','reviewer_kind':'human'}, {})

    def test_source_redirect_rejects_foreign_target_before_request_construction(self):
        pinned='https://raw.githubusercontent.com/org/repo/'+'a'*40+'/file.txt'
        request=urllib.request.Request(pinned)
        handler=prepare_consumed_sources.FixedSourceRedirect()
        invalid=[
            'https://attacker.example/org/repo/'+'a'*40+'/file.txt',
            'http://raw.githubusercontent.com/org/repo/'+'a'*40+'/file.txt',
            'https://raw.githubusercontent.com.attacker.example/org/repo/'+'a'*40+'/file.txt',
            'https://user@raw.githubusercontent.com/org/repo/'+'a'*40+'/file.txt',
            'https://raw.githubusercontent.com:444/org/repo/'+'a'*40+'/file.txt',
            'https://raw.githubusercontent.com/org/repo/main/file.txt',
            '/org/repo/'+'a'*40+'/file.txt',
        ]
        with patch.object(urllib.request.HTTPRedirectHandler,'redirect_request',return_value='safe-request') as base:
            for url in invalid:
                with self.subTest(url=url), self.assertRaises(ValueError):
                    handler.redirect_request(request,None,302,'Found',{},url)
                base.assert_not_called()
            self.assertEqual(handler.redirect_request(request,None,302,'Found',{},pinned),'safe-request')
            base.assert_called_once()

    def test_nested_operational_failures_keep_usage_and_fail_closed(self):
        payload={'schema_version':'reason-cli-output-v1','command':'ask','result':{
            'output_contract':'reason-natural-output-v6','usage':{'run':{
                'tracked_from_session_start':True,'total_tokens':17,'provider_attempts':2}}}}
        variants=[
            {'rendering_failure':{'failure_class':'quota'}},
            {'investigation':{'generation_failure':{'failure_class':'rate_limit'}}},
            {'investigation':{'telemetry':{'stop_reason':'operational_terminal'}}},
            {'acquisition_coverage':{'stop_reason':'operational_terminal'}},
            {'investigation':{'telemetry':{'actions':[{'status':'operational_failure'}]}}},
        ]
        variants+=[{'resolution_rounds':[{'terminal_status':s,'attempts':[]}]} for s in operational_bridge.FAILED_TERMINALS]
        variants += [{'resolution_rounds':[{'terminal_status':'exhausted','attempts':[{'status':s}]}]} for s in operational_bridge.FAILED_ATTEMPTS]
        variants += [{'rendering_failure':{'failure_class':'future_class'}}, {'rendering_failure':{}},
                     {'investigation':{'telemetry':{'stop_reason':[]}}}, {'resolution_rounds':'invalid'},
                     {'resolution_rounds':[{'terminal_status':{},'attempts':[]}]}]
        for extra in variants:
            value=copy.deepcopy(payload);value['result'].update(extra)
            observed=operational_bridge.outcome_from_normal(value)
            with self.subTest(extra=extra):
                self.assertIsNotNone(observed.failure_class)
                self.assertEqual((observed.tokens,observed.provider_attempts),(17,2))
        mixed=copy.deepcopy(payload);mixed['result'].update(rendering_failure={'failure_class':'quota'},investigation={'telemetry':{'stop_reason':'operational_terminal'}})
        self.assertEqual(operational_bridge.outcome_from_normal(mixed).failure_class.value,'quota')
        mixed['result']['rendering_failure']={'failure_class':'cancelled'}
        self.assertEqual(operational_bridge.outcome_from_normal(mixed).failure_class.value,'cancelled')

    def test_normal_nonfailure_outcomes_are_not_reclassified(self):
        value={'schema_version':'reason-cli-output-v1','command':'ask','result':{
            'output_contract':'reason-natural-output-v6','usage':{'run':{'tracked_from_session_start':True,'total_tokens':5,'provider_attempts':1}},
            'investigation':{'plan_rejection':'invalid_target_count','telemetry':{'stop_reason':'no_progress','actions':[{'status':'ambiguous'},{'status':'no_result'}]}},
            'acquisition_coverage':{'stop_reason':'planner_stop'},
            'resolution_rounds':[{'terminal_status':'exhausted','attempts':[{'status':'no_result'},{'status':'rejected_untrusted_evidence'}]}]}}
        observed=operational_bridge.outcome_from_normal(value)
        self.assertIsNone(observed.failure_class)
        self.assertEqual((observed.tokens,observed.provider_attempts),(5,1))

    def test_nested_failure_latches_actual_scheduler_before_second_job(self):
        scheduler=run_lane.scheduler
        for raw in ['quota','rate_limit','future_failure']:
            with self.subTest(raw=raw), tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp);script=root/'inert.py';receipt=root/'inert-receipt.json'
                payload={'schema_version':'reason-cli-output-v1','command':'ask','result':{
                    'output_contract':'reason-natural-output-v6','usage':{'run':{'tracked_from_session_start':True,'total_tokens':17,'provider_attempts':2}},
                    'rendering_failure':{'failure_class':raw}}}
                script.write_text('import json\nprint(json.dumps('+repr(payload)+'))\n');receipt.write_text('{}')
                budget={'max_generations':2,'max_input_tokens_per_generation':64,'max_output_tokens_per_generation':64,'max_attempts_per_generation':1,'aggregate_input_tokens':64,'aggregate_output_tokens':64}
                jobs=[{'id':'fixture-'+str(i),'arm':'baseline','contract':'normal','argv':[sys.executable,str(script),'--provider','mistral','--model','fixture-model'],
                       'timeout_seconds':10,'cancel_grace_seconds':1,'budget':budget} for i in range(2)]
                plan={'schema':'engine07-operational-plan-v1','provider':'mistral','model':'fixture-model','purpose':development.PURPOSE,
                      'development_receipt_file':str(receipt),'token_budget':256,'jobs':jobs,
                      'artifacts':{sys.executable:scheduler.digest(sys.executable),str(script):scheduler.digest(script)}}
                with patch.object(run_lane,'check_plan',return_value='0'):
                    runner=run_lane.AuditedRunner(plan,root/'lane',{'currency_limit_scope':'inert local fixture'})
                    runner.load_credentials=lambda provider:{}
                    result=runner.run()
                self.assertFalse(result['complete'])
                self.assertEqual(result['known_total_tokens'],17)
                self.assertEqual(result['known_provider_attempts'],2)
                self.assertEqual(result['charged_reservations'],128)
                self.assertFalse(result['rows'][0]['complete'])
                self.assertFalse(result['rows'][1]['admission']['allowed'])
                self.assertTrue(result['latch'])

if __name__ == '__main__':
    unittest.main()
