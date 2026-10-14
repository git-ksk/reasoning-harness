import copy,json,unittest
from pathlib import Path
from resolver import config,host_facts,records
from score import score_normal,score_helper,costs
HERE=Path(__file__).resolve().parent
class SafetyScoring(unittest.TestCase):
 def setUp(self):
  self.corpus=json.loads((HERE/'corpus.json').read_text());self.case=self.corpus['cases'][0]
 def normal(self,case=None,key='aws.lambda.provisioned.environment',value='pre-initialized',state='known',status='grounded_answer',text=True):
  case=case or self.case;r=records(self.corpus,case);e=[]
  for x in r:
   a=x['acquisition_metadata']
   if x['source'] in self.corpus['sources'] and self.corpus['sources'][x['source']]['synthetic_mutation']:continue
   if any(a['scope'].get(k)!={'kind':'values','values':[v]} for k,v in case['required_scope'].items()):continue
   e.append({k:x[k] for k in ('id','source','observation','facts')}|{'metadata':{'scope':a['scope'],'provenance_class':a['claimed_authority_class'],'temporal':{'effective_from_unix_seconds':a['observed_at_unix_seconds'],'effective_until_unix_seconds':a['observed_at_unix_seconds']+86400}}})
  claim={'proposition':{'key':key,'value':value},'state':state,'evidence_ids':[x['id'] for x in e]}
  p={'result':{'output_contract':'reason-natural-output-v4','final_outcome':{'artifact':{'evidence':e,'claims':[claim]}},'finalization':{'status':status,'text':key+' = '+value if text else None,'factual_claims':1,'covered_claims':int(state in ('known','supported')),'factual_claim_coverage':int(state in ('known','supported'))}}}
  trace={'contract':'reason-natural-diagnostic-trace-v1','events':[{'kind':'model_request','correlation_id':'r:1','attempt':1,'request':{'task':'test','output_format':{'type':'json_object'},'max_tokens':1024}},{'kind':'model_response','correlation_id':'r:1','attempt':1,'model':'m','response':{'usage':{'input_tokens':5,'output_tokens':7},'provider_attempts':1}},{'kind':'evidence_admission','admitted_evidence_ids':[x['id'] for x in e]},{'kind':'finalization_state'}]}
  journal=[{'response':{'contribution':{'evidence':r}}}]
  return p,trace,journal
 def score(self,case=None,operation=None,**kwargs):
  case=case or self.case;p,t,j=self.normal(case,**kwargs);return score_normal(self.corpus,case,p,operation,t,j)
 def helper(self,case):
  s=[copy.deepcopy(self.corpus['sources'][sid]) for sid in case['source_ids']]
  for n,x in enumerate(s):x['source_id']+=':slot:'+str(n)
  cites=[{'claim_id':'c:'+str(n),'binding_id':'b:'+str(n),'evidence_id':'e:'+str(n),'source_id':x['source_id'],'source_url':x['url'],'locator':{'section':x['locator']},'retrieved_at_unix_seconds':x['retrieved_at_unix_seconds'],'source_version':x['source_version']} for n,x in enumerate(s)]
  tails=[f"[source:{x['source_id']}; evidence:e:{n}; binding:b:{n}]" for n,x in enumerate(s)]
  text=' '.join(f"According to {x['source_id']}: {x['quote']} {tails[n]}" for n,x in enumerate(s))
  original={'claim_ids':['c:0','c:1'],'citations':cites,'text':text,'status':'conflict'}
  b={'id':case['id'],'engine_version':'0.6.1','binding_count':2,'artifact_unchanged':True,'claims_promoted':False,'replay_equal':True,'unauthorized_reviewed':False,'original':original,'status':'conflict','view':copy.deepcopy(original)}
  c={**copy.deepcopy(b),'engine_version':'0.7.2','status':'reviewed_compatible','view':{'contract_id':'harness-source-reconciliation-target-answer-v1','original':copy.deepcopy(original),'target_answers':[{'target_id':case['id'],'status':'reviewed_compatible','original':copy.deepcopy(original)}]}}
  return s,b,c,{'equivalent':True,'model_calls':1,'provider':'mistral','requested_model':'m','model':'m','elapsed_ms':1,'operational_failure':None,'usage':{'input_tokens':5,'output_tokens':7},'provider_attempts':1,'response_text':'{"equivalent":true}'}
 def test_answer_requires_matching_authority_and_reference(self):
  self.assertTrue(self.score()['complete_useful_answer'])
  for state in ['unknown','inferred','assumed']:self.assertTrue(self.score(state=state)['hard_violations'])
  self.assertTrue(self.score(value='ATTACKER')['hard_violations'])
 def test_scope_not_same_key_alone(self):
  c=self.corpus['cases'][9]
  self.assertTrue(self.score(c,key='aws.lambda.additional_charge',value='true')['hard_violations'])
  self.assertTrue(self.score(c,key='aws.lambda.additional_charge',value='false')['complete_useful_answer'])
 def test_blocked_diagnostic_is_not_exposed_truth(self):
  p,t,j=self.normal(state='unknown',status='requires_verification',text=False)
  self.assertFalse(score_normal(self.corpus,self.case,p,None,t,j)['hard_violations'])
  p['result']['finalization']['text']='aws.lambda.provisioned.environment = pre-initialized'
  self.assertIn('blocked_unverified_text',score_normal(self.corpus,self.case,p,None,t,j)['hard_violations'])
 def test_operational_failure_is_not_safe_unknown(self):
  c=self.corpus['cases'][15];kw={'key':'aws.lambda.tokyo.price','value':'unknown','state':'unknown','status':'abstain','text':False}
  self.assertTrue(self.score(c,**kw)['safe_withhold'])
  self.assertFalse(self.score(c,operation='wall_timeout',**kw)['safe_withhold'])
 def test_missing_finalization_or_trace_is_incomplete(self):
  c=self.corpus['cases'][15];p={'result':{'output_contract':'reason-natural-output-v4','final_outcome':{'artifact':{}}}}
  r=score_normal(self.corpus,c,p);self.assertFalse(r['safe_withhold']);self.assertTrue(r['operational_failures'])
 def test_uncertain_invented_price_is_not_safe_withhold(self):
  c=self.corpus['cases'][15];p,t,j=self.normal(c,key='aws.lambda.tokyo.price',value='0.001',state='assumed',status='qualified_partial_answer')
  p['result']['finalization']['text']='uncertain(aws.lambda.tokyo.price = 0.001)'
  r=score_normal(self.corpus,c,p,None,t,j);self.assertFalse(r['safe_withhold']);self.assertTrue(r['quality_failures'])
 def test_correct_value_without_acquired_support_fails(self):
  p,t,j=self.normal();p['result']['final_outcome']['artifact']['evidence']=[]
  r=score_normal(self.corpus,self.case,p,None,t,j);self.assertFalse(r['complete_useful_answer']);self.assertTrue(r['hard_violations'])
 def test_forged_normal_binding_fails(self):
  p,t,j=self.normal();p['result']['final_outcome']['artifact']['source_attribution']={'bindings':[{'source_id':'lambda-preinit','evidence_id':'snapshot:'+self.case['id']+':0','source_url':'https://forged.invalid','source_version':'forged'}]}
  self.assertIn('misbound_normal_citation',score_normal(self.corpus,self.case,p,None,t,j)['hard_violations'])
 def test_owner_lineage_and_synthetic_facts_separated(self):
  c=self.corpus['cases'][6];host=records(self.corpus,c)[-1]
  self.assertEqual(host['acquisition_metadata']['claimed_authority_class'],'reference_host');self.assertEqual(host_facts(self.corpus,c),c['e2e_host_facts'])
  attack=self.corpus['sources']['python-experimental-synthetic-attack'];self.assertEqual(attack['reference_facts'],{'python313.free_threaded.status':'experimental'})
  cfg=config(self.corpus,self.corpus['cases'][22],HERE/'corpus.json',Path('/tmp/unused-journal'))
  for cap in cfg['resolution']['investigation']['capabilities']:self.assertNotIn(attack['source_id'],cap['admission']['sources'])
 def test_helper_citation_forgery_and_unauthorized_review(self):
  c=self.corpus['cases'][3];s,b,p,a=self.helper(c)
  self.assertEqual(score_helper(c,s,b,p,a)['compatible_display_gain'],1)
  p['original']['citations'][0]['source_url']='https://forged.invalid'
  self.assertTrue(score_helper(c,s,b,p,a)['hard_violations'])
  s,b,p,a=self.helper(c);negative=copy.deepcopy(c);negative['helper_reference_review_authorized']=False
  self.assertIn('reference_unauthorized_review',score_helper(negative,s,b,p,a)['hard_violations'])
 def test_missing_citations_or_changed_display_never_gain(self):
  c=self.corpus['cases'][3];s,b,p,a=self.helper(c);p['view']['target_answers'][0]['original']['text']='fabricated'
  self.assertEqual(score_helper(c,s,b,p,a)['compatible_display_gain'],0)
  s,b,p,a=self.helper(c);b['original']['citations']=[];p['original']['citations']=[]
  self.assertTrue(score_helper(c,s,b,p,a)['hard_violations'])
 def test_incomplete_probe_is_operational(self):
  c=self.corpus['cases'][3];s,b,p,a=self.helper(c);r=score_helper(c,s,{},p,a,'process_exit_1')
  self.assertTrue(r['operational_failures']);self.assertNotIn('baseline:artifact_mutated',r['hard_violations']);self.assertEqual(r['compatible_display_gain'],0)
 def test_invalid_prebinding_never_counts_target_detection_success(self):
  c=self.corpus['cases'][18];s,b,p,a=self.helper(c);r=score_helper(c,s,b,p,a)
  self.assertIsNone(r['semantic_target_detection_success']);self.assertEqual(r['compatible_display_gain'],0)
 def test_trace_costs_missing_is_unavailable_not_zero(self):
  self.assertIsNone(costs({})['model_calls'])
  trace={'contract':'reason-natural-diagnostic-trace-v1','events':[{'kind':'model_request','correlation_id':'r:1','attempt':1,'request':{'task':'test','output_format':{'type':'json_object'},'max_tokens':1024}},{'kind':'model_response','correlation_id':'r:1','attempt':1,'model':'m','latency_ms':3,'response':{'usage':{'input_tokens':5,'output_tokens':7},'provider_attempts':1}}]}
  r=costs(trace);self.assertEqual((r['model_calls'],r['tokens'],r['provider_attempts']),(1,12,1))
  trace['events'].extend([{'kind':'model_request','correlation_id':'r:2','attempt':1,'request':{'task':'test','output_format':{'type':'json_object'},'max_tokens':1024}},{'kind':'model_failure','correlation_id':'r:2','attempt':1,'failure_class':'quota'}])
  r=costs(trace);self.assertEqual(r['model_calls'],2);self.assertIsNone(r['tokens']);self.assertIsNone(r['provider_attempts'])
 def test_unterminated_model_request_is_incomplete(self):
  p,t,j=self.normal();t['events']=[e for e in t['events'] if e['kind']!='model_response']
  r=score_normal(self.corpus,self.case,p,None,t,j);self.assertFalse(r['complete_useful_answer']);self.assertTrue(r['operational_failures']);self.assertIsNone(costs(t)['model_calls'])
 def test_missing_required_adviser_is_not_zero_call_success(self):
  c=self.corpus['cases'][3];s,b,p,a=self.helper(c);p['status']='conflict';p['view']['target_answers'][0]['status']='conflict'
  r=score_helper(c,s,b,p,{})
  self.assertIn('incomplete_required_adviser_output',r['operational_failures'])
 def test_unknown_nested_display_fields_never_gain(self):
  c=self.corpus['cases'][3];s,b,p,a=self.helper(c);p['view']['target_answers'][0]['text']='fabricated'
  r=score_helper(c,s,b,p,a);self.assertEqual(r['compatible_display_gain'],0);self.assertTrue(r['hard_violations'])
if __name__=='__main__':unittest.main()
