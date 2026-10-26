"""Real scheduler accounting/continuation for the current CLI's v6 envelope."""
import copy,json,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import run_lane
from run_lane import scheduler,AuditedRunner
from development import PURPOSE
from operational_bridge import outcome_from_normal

def normal():
    return {'schema_version':'reason-cli-output-v1','command':'ask','result':{
        'output_contract':'reason-natural-output-v6','usage':{'run':{
            'tracked_from_session_start':True,'total_tokens':5,'provider_attempts':1}}}}

class CurrentOperationalBridge(unittest.TestCase):
    def run_payload(self,payload,returncode=0):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);fixture=root/'fixture.py'
            helper={'operational_failure':None,'equivalent':False,'usage':{'total_tokens':3},'provider_attempts':1}
            fixture.write_text('import json,sys\npayloads='+repr([payload,helper])+'\nprint(json.dumps(payloads[int(sys.argv[1])]))\nraise SystemExit('+repr(returncode)+' if sys.argv[1]=="0" else 0)\n')
            jobs=[]
            for n,contract in enumerate(['normal','helper']):
                jobs.append({'id':'inert-'+str(n),'arm':'baseline' if n==0 else 'helper','contract':contract,
                    'argv':[sys.executable,str(fixture),str(n),'--provider','google','--model','offline-fixture'],
                    'timeout_seconds':10,'cancel_grace_seconds':1,
                    'budget':{'max_generations':1,'max_input_tokens_per_generation':64,'max_output_tokens_per_generation':64,'max_attempts_per_generation':1}})
            receipt=root/'inert-receipt.json';receipt.write_text('{"scope":"inert post-admission replay only; not a live receipt"}')
            for job in jobs:job['budget'].update(aggregate_input_tokens=64,aggregate_output_tokens=64)
            plan={'schema':'engine07-operational-plan-v1','provider':'google','model':'offline-fixture','purpose':PURPOSE,
                  'development_receipt_file':str(receipt),
                  'token_budget':256,'jobs':jobs,'artifacts':{sys.executable:scheduler.digest(sys.executable),str(fixture):scheduler.digest(fixture)}}
            output=root/'lane';reads=[]
            def credentials(provider):
                state=json.loads((output/'state.json').read_bytes())
                self.assertTrue(state['guard']['inflight']);self.assertEqual(state['current_reserve'],128)
                reads.append(provider);return {}
            # Real admission is tested separately. Inject only the inert plan
            # admission here to exercise the *actual* AuditedRunner/process/run
            # accounting after admission, with fixed JSON children and no keys.
            with patch.object(run_lane,'check_plan',return_value='0'),patch.object(scheduler,'reserve_for',scheduler.reserve_for):
                runner=AuditedRunner(plan,output,{'currency_limit_scope':'inert engineering replay; no provider billing'})
                runner.load_credentials=credentials
                result=runner.run()
            result['_credential_reads']=len(reads)
            return result
    def test_v6_success_crosses_real_scheduler_accounting_and_continues_to_helper(self):
        result=self.run_payload(normal())
        self.assertTrue(result['complete']);self.assertEqual(result['_credential_reads'],2)
        self.assertEqual(result['known_total_tokens'],8);self.assertEqual(result['known_provider_attempts'],2)
        self.assertEqual(result['charged_reservations'],256);self.assertIsNone(result['latch'])
        self.assertEqual(result['semantic_acceptance'],'NOT_SCORED')
    def test_unknown_v6_usage_still_stops_and_keeps_reservation(self):
        payload=normal();payload['result']['usage']['run']['tracked_from_session_start']=False
        result=self.run_payload(payload)
        self.assertFalse(result['complete']);self.assertEqual(result['_credential_reads'],1)
        self.assertEqual(result['unknown_usage_rows'],1);self.assertEqual(result['charged_reservations'],128)
        self.assertFalse(result['rows'][1]['admission']['allowed'])
    def test_typed_failure_still_stops_normal_to_helper(self):
        payload=normal();payload['result'].update(status='failed',failure={'failure_class':'quota'},partial_diagnostics={
            'complete':False,'usage_is_lower_bound':False,'usage':{'run':{'tracked_from_session_start':True,'total_tokens':0,'provider_attempts':1}}})
        result=self.run_payload(payload)
        self.assertFalse(result['complete']);self.assertEqual(result['_credential_reads'],1)
        self.assertEqual(result['latch'],'typed_quota');self.assertFalse(result['rows'][1]['admission']['allowed'])
    def test_future_or_wrong_envelope_is_not_accounting_success(self):
        known=normal();known['result']['output_contract']='reason-natural-output-v5'
        self.assertIsNone(outcome_from_normal(known).failure_class)
        for key,value in [('schema_version','unknown'),('command','adviser'),('output_contract','reason-natural-output-v999')]:
            payload=copy.deepcopy(normal())
            if key=='output_contract':payload['result'][key]=value
            else:payload[key]=value
            outcome=outcome_from_normal(payload)
            self.assertIsNotNone(outcome.failure_class);self.assertIsNone(outcome.tokens)
        for key in ['status','failure','partial_diagnostics']:
            payload=normal();payload['result'][key]=None
            self.assertIsNotNone(outcome_from_normal(payload).failure_class)
        result=self.run_payload(normal(),returncode=1)
        self.assertFalse(result['complete']);self.assertEqual(result['_credential_reads'],1)
        self.assertFalse(result['rows'][1]['admission']['allowed'])

if __name__=='__main__':unittest.main()
