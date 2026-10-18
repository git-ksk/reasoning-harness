"""Pure constructed output fixtures, NOT independent cases or Engine observations."""
import copy,json,unittest
from unittest import mock
import score,execute_first
from evaluator_contract import ROOT,SOURCE,VERSION,strict_json,checked_artifacts,sha
from tempfile import TemporaryDirectory
from pathlib import Path

def base():return {'known_regressions_total':1400,'known_regressions_passed':1400,'original_audit_total':14,'original_audit_passed':14,'historical_byte_equivalent':True,'known_unfixed_critical_defects':[]}
def proof():return {'checks':{'unit_fixture_only':True},'labels_frozen_before_observation':True,'candidate_frozen_before_observation':True,'first_observation_immutable':True}
def fixtures(tp17=26,tp30=None,fp17=0,fp30=None):
 if tp30 is None:tp30=tp17
 if fp30 is None:fp30=fp17
 cases=[];rows=[]
 for i in range(60):
  gold=i<30;cid=f'unit-output-{i:03d}'
  inp={'policy':{'policy_id':cid+'-policy','target_id':cid+'-target'},'candidate':{'evidence_id':cid+'-evidence','source_id':cid+'-source'},'raw':{'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}}
  cases.append({'id':cid,'pair_id':'unit-pair','polarity':'positive' if gold else 'negative','axis':'constructed_output_only','expected':{'v17_requested_relation':gold,'v30_relevant':gold},'input':inp})
  a=i<tp17 if gold else i-30<fp17;b=i<tp30 if gold else i-30<fp30
  binding={'policy_id':inp['policy']['policy_id'],'target_id':inp['policy']['target_id'],'evidence_id':inp['candidate']['evidence_id'],'source_id':inp['candidate']['source_id'],'contract_id':'unit-contract','materialization_policy_id':'unit-policy','path':'unit-path','reasons':['unit fixture, no Engine']}
  rows.append({'id':cid,'engine_version':VERSION,'effective_v17':{'identity_scope':'exact_target','relation_scope':'requested_relation' if a else 'unresolved','scope_risk':'none'},'v29':dict(binding,disposition='relevant'),'v30':dict(binding,disposition='relevant' if b else 'ambiguous')})
 return cases,rows

def bundle():
 m0hash='0'*64;mh='1'*64;ah='2'*64;raw='3'*64;err='4'*64;ss='5'*64
 m0files={'score.py':ss,'protocol.json':'6'*64};frozen=dict(m0files,**{'evaluator-freeze.json':m0hash,'cases.json':'a'*64,'inputs.json':'b'*64,'author_cases.py':'c'*64,'authoring-start.json':ah,'scope-review.json':'d'*64,'static-preflight.json':'e'*64})
 artifacts={'probe_executable':{'path':'/private/tmp/unit-only-not-a-real-probe','sha256':'f'*64,'bytes':1}}
 m0={'identity':'engine073-launch-independent-v27-pre-author','candidate':{'source_sha':SOURCE,'package_version':VERSION},'frozen_utc':'2000-01-01T00:00:00+00:00','frozen_files':m0files,'checked_artifacts':copy.deepcopy(artifacts)}
 m={'candidate':{'source_sha':SOURCE,'package_version':VERSION},'frozen_utc':'2000-01-01T00:00:03+00:00','frozen_files':frozen,'checked_artifacts':artifacts,'authoring':{'evaluator_freeze_sha256':m0hash,'authoring_start_sha256':ah}}
 guard={'manifest_sha256':mh,'frozen_files':copy.deepcopy(frozen),'head':SOURCE,'artifacts':{k:{'path':v['path'],'sha256':v['sha256']} for k,v in artifacts.items()}}
 command=[artifacts['probe_executable']['path'],str(ROOT/'inputs.json')]
 before={'state':'started','utc':'2000-01-01T00:00:04+00:00','command':command,'pre_invocation_identity':copy.deepcopy(guard)}
 after={'state':'finished','utc':'2000-01-01T00:00:05+00:00','command':command,'pre_invocation_identity':copy.deepcopy(guard),'stdout_sha256':raw,'stderr_sha256':err}
 return {'m0':m0,'manifest':m,'before':before,'after':after,'author_start':{'utc':'2000-01-01T00:00:01+00:00','evaluator_freeze_sha256':m0hash,'authorization_reference':'unit-fixture-only'},'score_start':{'identity':'engine073-launch-independent-v27','state':'started','utc':'2000-01-01T00:00:06+00:00','scorer_sha256':ss,'manifest_sha256':mh},'cases_header':{'evaluator_freeze_sha256':m0hash,'authoring_start_sha256':ah,'authored_utc':'2000-01-01T00:00:02+00:00'},'current_head':SOURCE,'artifacts_match':True,'M0_files_match':True,'M1_files_match':True,'M0_sha256':m0hash,'manifest_sha256':mh,'author_start_sha256':ah,'raw_sha256':raw,'stderr_sha256':err,'scorer_sha256':ss,'raw_readonly':True}

class EvaluatorTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.config,cls.helper=score.config_and_helper()
 def run_fixture(self,tp=26,**kwargs):
  cases,rows=fixtures(tp,kwargs.pop('tp30',None),kwargs.pop('fp',0),kwargs.pop('fp30',None))
  if 'mutate' in kwargs:kwargs.pop('mutate')(rows)
  raw=kwargs.pop('raw',None)
  return score.assess(cases,raw if raw is not None else '\n'.join(json.dumps(x) for x in rows),kwargs.pop('invocation',{'state':'finished','returncode':0,'execution_error':None}),kwargs.pop('stderr',0),kwargs.pop('base',base()),kwargs.pop('proof',proof()),self.config,self.helper)
 def test_tp26_pass_with_four_joint_false_abstentions(self):
  x=self.run_fixture()['summary'];self.assertEqual(x['outcome'],'PASS');self.assertEqual(x['strict_joint_outcome'],'FAIL');self.assertEqual(x['failed'],4);self.assertEqual(x['axes']['v17_requested_relation']['tp'],26)
 def test_tp25_recall_floor_fail(self):self.assertEqual(self.run_fixture(25)['summary']['decision'],'NO_GO')
 def test_fp1_always_fail_despite_high_f1(self):
  x=self.run_fixture(30,fp=1)['summary'];self.assertFalse(x['gate']['gate_pass']);self.assertEqual(x['axes']['v30_relevant']['fp'],1)
 def test_one_axis_fails(self):self.assertFalse(self.run_fixture(30,tp30=25)['summary']['gate']['gate_pass'])
 def test_one_axis_unsafe(self):self.assertFalse(self.run_fixture(30,fp30=1)['summary']['gate']['gate_pass'])
 def test_zero_predicted_positive_undefined(self):self.assertIsNone(self.run_fixture(0)['summary']['axes']['v17_requested_relation']['precision'])
 def test_missing_negative_not_true_negative(self):
  cases,rows=fixtures();x=self.run_fixture(raw='\n'.join(json.dumps(r) for r in rows[:-1]))['summary'];self.assertEqual(x['incomplete'],1);self.assertEqual(x['axes']['v30_relevant']['tn'],29);self.assertEqual(x['decision'],'NO_GO')
 def test_duplicate_observed_id_no_go(self):
  c,r=fixtures();x=self.run_fixture(raw='\n'.join(json.dumps(x) for x in r+[r[0]]))['summary'];self.assertTrue(x['operational_errors']);self.assertFalse(x['gate']['gate_pass'])
 def test_foreign_id_no_go(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0].update(id='foreign'))['summary']['gate']['gate_pass'])
 def test_malformed_json_no_go(self):self.assertFalse(self.run_fixture(raw='{malformed') ['summary']['gate']['gate_pass'])
 def test_duplicate_json_key_rejected(self):
  with self.assertRaises(ValueError):strict_json('{"id":"a","id":"b"}')
 def test_nonfinite_json_rejected(self):
  with self.assertRaises(ValueError):strict_json('{"x":NaN}')
 def test_engine_version_mismatch(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0].update(engine_version='foreign'))['summary']['gate']['gate_pass'])
 def test_invalid_effective_enum(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0]['effective_v17'].update(identity_scope='invented'))['summary']['gate']['gate_pass'])
 def test_misbound_source(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0]['v30'].update(source_id='foreign'))['summary']['gate']['gate_pass'])
 def test_missing_v29(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0].pop('v29'))['summary']['gate']['gate_pass'])
 def test_invalid_reasons(self):self.assertFalse(self.run_fixture(mutate=lambda r:r[0]['v30'].update(reasons='string'))['summary']['gate']['gate_pass'])
 def test_invocation_failure(self):self.assertEqual(self.run_fixture(invocation={'state':'finished','returncode':1})['summary']['incomplete'],60)
 def test_stderr_no_go(self):self.assertFalse(self.run_fixture(stderr=1)['summary']['gate']['gate_pass'])
 def test_invocation_failure_never_joint_success(self):
  x=self.run_fixture(30,invocation={'state':'finished','returncode':1})['summary']
  self.assertEqual((x['positive_passed'],x['negative_passed'],x['passed'],x['failed'],x['incomplete']),(0,0,0,0,60))
  self.assertEqual(x['decision'],'NO_GO')
 def test_stderr_never_joint_success(self):
  x=self.run_fixture(30,stderr=1)['summary']
  self.assertEqual((x['positive_passed'],x['negative_passed'],x['passed'],x['failed'],x['incomplete']),(0,0,0,0,60))
  self.assertEqual(x['decision'],'NO_GO')
 def test_known_regression_failure(self):
  b=base();b['known_regressions_passed']=1399;self.assertFalse(self.run_fixture(base=b)['summary']['gate']['gate_pass'])
 def test_original_audit_failure(self):
  b=base();b['original_audit_passed']=13;self.assertFalse(self.run_fixture(base=b)['summary']['gate']['gate_pass'])
 def test_history_failure(self):
  b=base();b['historical_byte_equivalent']=False;self.assertFalse(self.run_fixture(base=b)['summary']['gate']['gate_pass'])
 def test_known_critical_failure(self):
  b=base();b['known_unfixed_critical_defects']=['unit-critical'];self.assertFalse(self.run_fixture(base=b)['summary']['gate']['gate_pass'])
 def test_parent_cannot_supply_future_provenance(self):
  b=base();b['labels_frozen_before_observation']=True
  with self.assertRaises(ValueError):self.run_fixture(base=b)
 def test_provenance_failure(self):
  q=proof();q['checks']['unit_fixture_only']=False;q['labels_frozen_before_observation']=False;self.assertFalse(self.run_fixture(proof=q)['summary']['gate']['gate_pass'])
 def test_zero_gold_positive_rejected(self):
  c,r=fixtures()
  for x in c:x.update(polarity='negative',expected={'v17_requested_relation':False,'v30_relevant':False})
  with self.assertRaises(ValueError):score.validate_gold(c)
 def test_integer_gold_rejected(self):
  c,r=fixtures();c[0]['expected']={'v17_requested_relation':1,'v30_relevant':1}
  with self.assertRaises(ValueError):score.validate_gold(c)
 def test_duplicate_gold_rejected(self):
  c,r=fixtures();c[1]['id']=c[0]['id']
  with self.assertRaises(ValueError):score.validate_gold(c)
 def test_helper_called_once_with_explicit_config(self):
  c,r=fixtures();proxy=mock.Mock();proxy.score.side_effect=self.helper.score
  score.assess(c,'\n'.join(json.dumps(x) for x in r),{'state':'finished','returncode':0,'execution_error':None},0,base(),proof(),self.config,proxy)
  self.assertEqual(proxy.score.call_count,1);self.assertIs(proxy.score.call_args.kwargs['config'],self.config)
 def test_default_helper_policy_not_used(self):
  with mock.patch.object(self.helper,'policy',side_effect=AssertionError('must pass snapshot config')):self.assertTrue(self.run_fixture()['summary']['gate']['gate_pass'])
 def test_policy_pin_checked(self):
  original=score.sha
  with mock.patch.object(score,'sha',side_effect=lambda p:'changed' if PathName(p)=='source_gate_policy.json' else original(p)):
   with self.assertRaises(ValueError):score.config_and_helper()
 def test_risk_diagnostic_literal_input_and_output(self):
  c,r=fixtures();c[0]['input']['raw']['scope_risk']='ownership_scope';r[0]['effective_v17']['scope_risk']='multiple'
  d=score.exact_risk_diagnostic(c[0],r[0]);self.assertEqual(d['raw_provided_tuple']['scope_risk'],'ownership_scope');self.assertEqual(d['effective_v17_tuple']['scope_risk'],'multiple');self.assertNotIn('components',d)
 def test_risk_diagnostic_missing_output_stays_unknown(self):
  c,r=fixtures();d=score.exact_risk_diagnostic(c[0],None);self.assertIsNone(d['effective_v17_tuple']);self.assertIsNone(d['v30_disposition'])
 def test_risk_diagnostic_does_not_mutate_inputs(self):
  c,r=fixtures();before=copy.deepcopy((c[0],r[0]));score.exact_risk_diagnostic(c[0],r[0]);self.assertEqual((c[0],r[0]),before)
 def test_execution_success_requires_complete_process_chain(self):
  self.assertTrue(execute_first.execution_passed({'decision':'GO'},0,0,0))
  self.assertFalse(execute_first.execution_passed({'decision':'NO_GO'},0,0,0))
 def test_execution_nonzero_or_stderr_cannot_report_success(self):
  for recorder,scorer,stderr in [(1,0,0),(0,1,0),(0,0,1)]:self.assertFalse(execute_first.execution_passed({'decision':'GO'},recorder,scorer,stderr))
 def test_bool_returncode_is_incomplete_not_success(self):
  x=self.run_fixture(30,invocation={'state':'finished','returncode':False,'execution_error':None})['summary']
  self.assertEqual((x['decision'],x['passed'],x['incomplete']),('NO_GO',0,60))
 def test_missing_execution_error_field_is_incomplete(self):
  x=self.run_fixture(30,invocation={'state':'finished','returncode':0})['summary']
  self.assertEqual((x['decision'],x['passed'],x['incomplete']),('NO_GO',0,60))
 def test_refused_observation_not_safe_negative(self):
  c,r=fixtures(30);r[-1]={'id':c[-1]['id'],'error':'pure fixture refused'}
  x=self.run_fixture(raw='\n'.join(json.dumps(v) for v in r))['summary']
  self.assertEqual((x['decision'],x['incomplete'],x['axes']['v30_relevant']['tn']),('NO_GO',1,29))
 def test_runtime_after_guard_artifact_disagreement(self):
  b=bundle();b['after']['pre_invocation_identity']['artifacts']['probe_executable']['sha256']='changed';self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_wrong_score_hash(self):
  b=bundle();b['score_start']['scorer_sha256']='changed';self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_wrong_input_command(self):
  b=bundle();b['before']['command']=['/private/tmp/foreign-probe','/private/tmp/foreign-inputs'];self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_changed_complete_freeze(self):
  b=bundle();b['M1_files_match']=False;self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_score_precedes_first_finish(self):
  b=bundle();b['score_start']['utc']='2000-01-01T00:00:04+00:00';self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_checked_artifact_hash_and_size(self):
  with TemporaryDirectory(prefix='engine073-v27-pure-artifact-') as d:
   f=Path(d)/'constructed-bytes';f.write_bytes(b'pure fixture, no runtime')
   a={'unit':{'path':str(f),'sha256':sha(f),'bytes':f.stat().st_size}}
   self.assertTrue(checked_artifacts(a));a['unit']['bytes']+=1;self.assertFalse(checked_artifacts(a));a['unit']['bytes']-=1;a['unit']['sha256']='changed';self.assertFalse(checked_artifacts(a))
 def test_missing_parent_control_rejected(self):
  b=base();b.pop('known_unfixed_critical_defects')
  with self.assertRaises(ValueError):self.run_fixture(base=b)
 def test_execution_bool_returncode_rejected(self):
  self.assertFalse(execute_first.execution_passed({'decision':'GO'},False,0,0));self.assertFalse(execute_first.execution_passed({'decision':'GO'},0,False,0))
 def test_runtime_positive_fixture(self):self.assertTrue(score.provenance(bundle())['first_observation_immutable'])
 def test_runtime_changed_raw(self):
  b=bundle();b['raw_sha256']='changed';self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_raw_not_readonly(self):
  b=bundle();b['raw_readonly']=False;self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_wrong_candidate(self):
  b=bundle();b['current_head']='foreign';self.assertFalse(score.provenance(b)['candidate_frozen_before_observation'])
 def test_runtime_missing_gold_pin(self):
  b=bundle();b['manifest']['frozen_files'].pop('cases.json');self.assertFalse(score.provenance(b)['labels_frozen_before_observation'])
 def test_runtime_author_before_evaluator_freeze(self):
  b=bundle();b['author_start']['utc']='1999-01-01T00:00:00+00:00';self.assertFalse(score.provenance(b)['labels_frozen_before_observation'])
 def test_runtime_guard_manifest_mismatch(self):
  b=bundle();b['before']['pre_invocation_identity']['manifest_sha256']='foreign';self.assertFalse(score.provenance(b)['first_observation_immutable'])
 def test_runtime_changed_evaluator(self):
  b=bundle();b['M0_files_match']=False;self.assertFalse(score.provenance(b)['labels_frozen_before_observation'])
 def test_runtime_changed_artifacts(self):
  b=bundle();b['artifacts_match']=False;self.assertFalse(score.provenance(b)['candidate_frozen_before_observation'])
 def test_runtime_malformed_frame(self):self.assertFalse(score.provenance({})['first_observation_immutable'])

def PathName(p):return str(p).rsplit('/',1)[-1]
if __name__=='__main__':unittest.main()
