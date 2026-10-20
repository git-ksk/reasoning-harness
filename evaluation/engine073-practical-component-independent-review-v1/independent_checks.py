#!/usr/bin/env python3
"""Component fixtures only; no old corpus load, semantic gold, Engine or real scorer."""
from pathlib import Path
import sys,os,json,hashlib,copy,unittest,subprocess,time,traceback
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
OUT=Path(__file__).resolve().parent
MET=ROOT/'evaluation/engine073-answer-metrics-development-v1'
TRA=ROOT/'evaluation/engine073-cli-trace-binding-development-v1'
sys.path[:0]=[str(MET),str(TRA)]
import claim_metrics as cm
import display_binding as db
import trace_binding as tb

def dump(n,d):
 with (OUT/n).open('x') as f:json.dump(d,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())

def run_suite(name,suite):
 started=time.monotonic()
 with (OUT/(name+'.stderr.txt')).open('x') as f:
  r=unittest.TextTestRunner(stream=f,verbosity=2).run(suite);f.flush();os.fsync(f.fileno())
 result={'name':name,'tests_run':r.testsRun,'failures':len(r.failures),'errors':len(r.errors),'skipped':len(r.skipped),'successful':r.wasSuccessful(),'elapsed_seconds':time.monotonic()-started,'Engine_provider_model_network_Oracle_real_cohort_scorer_calls':0}
 dump(name+'.json',result)
 return result

# Execute only the new arithmetic/span constructed tests and trace tests whose
# bodies construct bytes/JSON in memory. Do not call the saved-corpus fixture.
metric_suite=unittest.defaultTestLoader.discover(str(MET),pattern='test_*.py')
metrics=run_suite('existing-metrics-pure32',metric_suite)
import test_trace_binding as tt
trace_suite=unittest.TestSuite([unittest.defaultTestLoader.loadTestsFromTestCase(tt.FullSnapshotBinding),tt.CaptureBinding('test_duplicate_json_keys_are_rejected')])
trace_pure=run_suite('existing-trace-inmemory4',trace_suite)
assert metrics['tests_run']==32 and metrics['successful']
assert trace_pure['tests_run']==4 and trace_pure['successful']

# No real question or truth labels: a mechanically constructed graph used only
# to test structural joins. Required case-like fields are adapter syntax.
def capture_fixture():
 scope={'product':'component-fixture'}
 qual_scope={k:{'kind':'values','values':[v]} for k,v in scope.items()}
 facts={'fixture.flag':'enabled'}
 source={'document_id':'fixture-doc','url':'https://fixture.invalid/doc','locator':'fixture-section','source_version':'fixture-version','origin_group':'fixture-origin','scope':scope,'quote':'Fixture flag: enabled.','context_summary':'Structural adapter fixture only.','synthetic_mutation':False,'reference_facts':facts,'retrieved_at_unix_seconds':100}
 source['quote_sha256']=tb.sha(source['quote'].encode())
 corpus={'cases':[{'id':'component-row','question':'structural adapter test','required_scope':scope,'lookup_key':'fixture.flag','source_ids':['fixture-source']}],'sources':{'fixture-source':source}}
 observed={k:source[k] for k in ['document_id','url','locator','source_version','origin_group','scope','quote','context_summary','synthetic_mutation']}
 raw={'id':'e1','source':'fixture-source','observation':json.dumps(observed),'facts':facts,'acquisition_metadata':{'observed_at_unix_seconds':100,'retrieved_at_unix_seconds':100,'scope':qual_scope}}
 evidence={**copy.deepcopy(raw),'metadata':{'scope':qual_scope,'provenance_class':'primary','temporal':{'effective_from_unix_seconds':100,'effective_until_unix_seconds':200}}}
 policy={'required_scope':qual_scope,'sources':{'fixture-source':{'authority_class':'primary','scope':qual_scope,'max_age_seconds':100}},'authority_ranks':{'primary':2},'minimum_authority_class':'primary','evaluation_time_unix_seconds':150}
 config={'resolution':{'investigation':{'capabilities':[{'id':'cap1','admission':policy}]}}}
 request={'id':'investigation:t1:cap1','target':{'target_id':'t1','expected_fact_key':'fixture.flag'}}
 journal=[{'case':'component-row','corpus_sha256':'c'*64,'request':{'schema_version':'reason-investigation-external-resolver-request-v1','request':request},'response':{'schema_version':'reason-external-resolver-response-v1','contribution':{'kind':'acquired_evidence','evidence':[raw]}}}]
 prop={'key':'fixture.flag','value':'enabled'}
 artifact={'evidence':[evidence],'claims':[{'id':'claim1','state':'known','proposition':prop,'evidence_ids':['e1']}],'verification_receipts':[{'id':'r1','claim_id':'claim1','proposition':prop,'conclusion':'supported','verifier':'structured_fact_equality','evidence_ids':['e1']}]}
 outcome={'verdict':'accept','artifact':artifact}
 final={'status':'grounded_answer','text':'fixture.flag = enabled','factual_claims':1,'covered_claims':1,'factual_claim_coverage':1.0,'uncovered_propositions':[]}
 output={'schema_version':'reason-cli-output-v1','command':'ask','result':{'output_contract':'reason-natural-output-v4','configuration':{'provider':'fixture-provider','model':'fixture-model'},'task':'structural adapter test','finalization':final,'final_outcome':outcome}}
 common={'provider':'fixture-provider','model':'fixture-model','attempt':1,'correlation_id':'call1','phase':'fixture','round':0}
 events=[{'kind':'model_request',**common,'request':{'task':'fixture task','max_tokens':16}}, {'kind':'model_response',**common,'response':{'text':'fixture response'}}, {'kind':'evidence_admission','target_id':'t1','capability_id':'cap1','admitted_evidence_ids':['e1'],'fact_keys':['fixture.flag']},{'kind':'grounding_state','outcome':copy.deepcopy(outcome)},{'kind':'finalization_state','result':copy.deepcopy(final)}]
 for i,e in enumerate(events):e['sequence']=i
 trace={'contract':'reason-natural-diagnostic-trace-v1','provider':'fixture-provider','model':'fixture-model','events':events}
 return [output,trace,journal,{'returncode':0},config,corpus,'c'*64]

base=capture_fixture();base_result=tb.validate_capture(*base)
assert base_result['binding_status']=='STRUCTURAL_MATCH'
dump('constructed-capture-fixture.json',base)
observations=[]

# F1: one occurrence ID is reused at two physical display locations.
artifacts=[{'id':'answer','kind':'final_answer','text':'Fixture fact.'},{'id':'card','kind':'claim_card','text':'Fixture fact.'}]
segments=[{'id':'span-'+a['id'],'artifact_id':a['id'],'start_byte':0,'end_byte':len(a['text'].encode()),'text':a['text'],'kind':'fact','claim_id':'one-occurrence','proposition_id':'one-proposition'} for a in artifacts]
plan={'id':'fixture-run','provider':'fixture','track':'e2e_normal','arm':'candidate','family':'structural','answerability':'full','required_groups':['required'],'observation_sha256':db.bundle_sha256(artifacts)}
review={k:v for k,v in plan.items() if k!='required_groups'}
review.update(operation='complete',review_complete=True,reviewer={'id':'fixture','role':'host'},hard_violations=[],claims=[{'id':'one-occurrence','proposition_id':'one-proposition','grounding':'grounded','satisfied_groups':['required']}])
joined=db.score_bound_case(plan,review,artifacts,['answer','card'],segments)
observations.append({'id':'display-occurrence-reused','expected':'reject one occurrence ID mapped to two distinct displayed occurrences, or explicitly model/count all covered occurrences','actual':'accepted','artifact_fact_spans':2,'reported_claim_occurrences':joined['display_binding']['claim_occurrences'],'reported_arithmetic_occurrences':joined['claim_metrics']['counts']['displayed_claim_occurrences'],'semantic_f1_from_constructed_annotation':joined['claim_metrics']['f1'],'result':joined})
dump('display-occurrence-reuse.input.json',{'plan':plan,'review':review,'artifacts':artifacts,'expected_artifact_ids':['answer','card'],'segments':segments})

# Positive control: two distinct occurrence IDs for the same proposition retain
# two occurrences and deduplicate the proposition only for A/S.
fixed_r=copy.deepcopy(review);fixed_segments=copy.deepcopy(segments)
fixed_r['claims'].append({**copy.deepcopy(fixed_r['claims'][0]),'id':'second-occurrence'})
fixed_segments[1]['claim_id']='second-occurrence'
control=db.score_bound_case(plan,fixed_r,artifacts,['answer','card'],fixed_segments)
assert control['claim_metrics']['counts']['displayed_claim_occurrences']==2 and control['claim_metrics']['counts']['displayed_unique_claims_A']==1
observations.append({'id':'display-distinct-occurrence-control','actual':'accepted_with_two_occurrences_one_proposition','result':control})

# F2: a response appears only after terminal finalization while all joins pass.
late=capture_fixture();response=late[1]['events'].pop(1);late[1]['events'].append(response)
for i,e in enumerate(late[1]['events']):e['sequence']=i
late_result=tb.validate_capture(*late)
observations.append({'id':'model-response-after-terminal','expected':'reject response completing an outstanding model request after final terminal display','actual':late_result['binding_status'],'ordered_events':[{'sequence':e['sequence'],'kind':e['kind']} for e in late[1]['events']],'result':late_result})
dump('late-response.input.json',late)

# Further contract boundary: schema-empty response dictionaries count as a
# complete request/response pair. Record it as missing full-M0 schema validation,
# not an unsupported semantic score or proof of fraudulent recorded data.
empty=capture_fixture();empty[1]['events'][1]['response']={}
empty_result=tb.validate_capture(*empty)
observations.append({'id':'empty-model-response','expected_full_M0':'require actual response schema/finish/usage validation before complete operational accounting','actual':empty_result['binding_status'],'model_calls_counted':empty_result['model_calls_in_saved_trace'],'classification':'integration prerequisite: current component verifies identities, not full response/usage schema'})

# Negative controls establish that source-target and response identity mutations
# fail in the same constructed adapter fixture.
for ident,mut in [('wrong-acquisition-target',lambda a:a[2][0]['request']['request']['target'].update(target_id='foreign')),('wrong-model-correlation',lambda a:a[1]['events'][1].update(correlation_id='foreign'))]:
 args=capture_fixture();mut(args)
 try:tb.validate_capture(*args)
 except ValueError as e:observations.append({'id':ident,'expected':'reject','actual':'rejected','reason':str(e)})
 else:raise AssertionError('negative_control_unexpectedly_accepted:'+ident)

# F3 reproduces only renderer_host.rs's Option equality guard with standard Rust,
# no Engine dependency, binary or source loaded. It does not prove a final forged
# citation: downstream source APIs may separately reject the span.
rs=OUT/'renderer_option_guard.rs'
with rs.open('x') as f:
 f.write('''fn main() {\n let body = "abc";\n let start = 0usize; let end = 99usize;\n let absent_or_non_string_quote: Option<&str> = None;\n assert_eq!(body.get(start..end), absent_or_non_string_quote, "quote differs from retained full source");\n println!("guard_accepted_invalid_span_and_absent_quote");\n}\n''');f.flush();os.fsync(f.fileno())
compile_cmd=['rustc','--edition=2024',str(rs),'-o',str(OUT/'renderer_option_guard')]
with (OUT/'renderer-guard-compile.stdout').open('xb') as out,(OUT/'renderer-guard-compile.stderr').open('xb') as err:
 r=subprocess.run(compile_cmd,stdout=out,stderr=err,check=False)
assert r.returncode==0
with (OUT/'renderer-guard-first.stdout').open('xb') as out,(OUT/'renderer-guard-first.stderr').open('xb') as err:
 r2=subprocess.run([str(OUT/'renderer_option_guard')],stdout=out,stderr=err,check=False)
assert r2.returncode==0
observations.append({'id':'renderer-null-option-guard','expected':'reject invalid byte span and missing/non-string quote before calling source APIs','actual':'guard accepted None == None','compile_returncode':r.returncode,'returncode':r2.returncode,'stdout':(OUT/'renderer-guard-first.stdout').read_text(),'scope':'Exact expression isolated from renderer_host.rs19, no Engine linked or executed. No successful finalization/citation exploit claimed.'})
dump('renderer-guard-invocations.json',{'compile_command':compile_cmd,'compile_returncode':r.returncode,'run_command':[str(OUT/'renderer_option_guard')],'run_returncode':r2.returncode,'Engine_linked_or_executed':False,'Cargo_shared_target_changes':0})
dump('adversarial-observations.json',{'scope':'new constructed component fixtures only; not semantic gold, fresh evaluation cases or rescoring of old first output','existing_metrics_tests':metrics,'existing_trace_pure_tests':trace_pure,'observations':observations,'Engine_actual_cohort_scorer_provider_model_network_Oracle_calls':0})
print(json.dumps({'existing_metrics_pure_tests':metrics,'existing_trace_inmemory_tests':trace_pure,'reproduced_boundaries':['display-occurrence-reused','model-response-after-terminal','renderer-null-option-guard'],'full_M0_schema_gap':'empty-model-response','new_real_or_source_cases_authored':0,'Engine_actual_cohort_scorer_provider_model_network_Oracle_calls':0},indent=2))
