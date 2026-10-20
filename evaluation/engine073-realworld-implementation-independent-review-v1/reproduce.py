#!/usr/bin/env python3
"""Structural adapter/scorer/freeze checks only. No Engine or accuracy corpus."""
from pathlib import Path
import json,sys,os,copy,hashlib,unittest,time,subprocess
IN=Path('/private/tmp/engine073-realworld-evaluator-review-snapshot-v1')
OUT=Path(__file__).resolve().parent
sys.path.insert(0,str(IN))
import scorer,capture,corpus,test_scorer
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def dump(n,d):
 with (OUT/n).open('x') as f:json.dump(d,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())
index=json.loads((IN/'input-manifest.json').read_text());assert all(sha(IN/n)==h for n,h in index.items())
dump('input-hashes.json',{'snapshot':str(IN),'manifest_sha256':sha(IN/'input-manifest.json'),'files':index})
suite=unittest.defaultTestLoader.loadTestsFromModule(test_scorer)
with (OUT/'existing12.stderr.txt').open('x') as f:
 result=unittest.TextTestRunner(stream=f,verbosity=2).run(suite);f.flush();os.fsync(f.fileno())
assert result.testsRun==12 and result.wasSuccessful()
deps=scorer.frozen_dependencies(test_scorer.REPO)
dump('existing12.json',{'tests':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),'successful':result.wasSuccessful(),'accuracy_Engine_provider_calls':0})
findings=[]
def rebind(obs,rev):rev['observation_sha256']=scorer.sha(json.dumps(obs,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())

# R1: extra captured citation has no matching artifact/span and disappears.
g,o,r,s=test_scorer.pair()
o['display_citations'].append({'id':'orphan','artifact_id':'absent-artifact','start_byte':0,'end_byte':14,'source_id':'not-a-source','source_version':'forged','url':'https://wrong.invalid/','literal_quote':'forged'})
rebind(o,r);row=scorer.validate_review(g,o,r,s,deps)
assert row['complete_useful_answer'] and row['citation_counts']['edges']==1 and row['hard_violations']==[]
findings.append({'id':'R1-orphan-citation-dropped','captured_citations':2,'counted_edges':row['citation_counts']['edges'],'complete_useful_answer':row['complete_useful_answer'],'hard_violations':row['hard_violations']})
dump('R1.input.json',{'gold':g,'observation':o,'review':r,'sources':s});dump('R1.output.json',row)

# R2: the citation's source was fetched but never admitted; other source grounds
# the proposition. Correct-edge annotation must not override the admission chain.
g,o,r,s=test_scorer.pair();s['rejected']={**s['s'],'url':'https://example.invalid/rejected'}
o['acquired_source_ids'].append('rejected');o['display_citations'][0].update(source_id='rejected',url=s['rejected']['url'])
rebind(o,r);row=scorer.validate_review(g,o,r,s,deps)
assert row['complete_useful_answer'] and row['hard_violations']==[]
findings.append({'id':'R2-unadmitted-citation-credit','admitted_source_ids':o['admitted_source_ids'],'correct_citation_source':'rejected','complete_useful_answer':True,'hard_violations':[]})
dump('R2.input.json',{'gold':g,'observation':o,'review':r,'sources':s});dump('R2.output.json',row)

# R3: bind_helper's actual artifact kind is rejected by the frozen display kernel.
g,o,r,s=test_scorer.pair();o['track']='helper_opt_in';o['terminal_status']='qualified';o['display_artifacts'][0]['kind']='source_local_answer';rebind(o,r)
try:scorer.validate_review(g,o,r,s,deps)
except ValueError as e:kind_error=str(e)
else:raise AssertionError('helper_kind_expected_failure_not_reproduced')
assert kind_error=='unknown_artifact_kind'
findings.append({'id':'R3-helper-artifact-kind-incompatible','capture_kind':'source_local_answer','error':kind_error})
dump('R3.input.json',{'gold':g,'observation':o,'review':r,'sources':s})

# R4: all helper baseline rows are incomplete, while candidate quality is perfect.
# This uses already-validated-shaped arithmetic rows, not real semantic outputs.
g,o,r,s=test_scorer.pair();template=scorer.validate_review(g,o,r,s,deps)
protocol=json.loads((IN/'protocol.json').read_text());rows=[];planned=[]
for provider in protocol['providers']:
 for track in protocol['tracks']:
  for arm in ['baseline','candidate']:
   for n in range(24):
    x=copy.deepcopy(template);ident='structure-'+str(n);x.update(id=ident,row_id=provider+'-'+track+'-'+arm+'-'+str(n),provider=provider,track=track,arm=arm,expected='positive' if n<12 else 'negative',decision='accept' if n<12 else 'withhold')
    x['complete_useful_answer']=n<12 and (arm=='candidate' or n<10)
    if track=='helper_opt_in' and arm=='baseline':
     x['complete']=False;x['complete_useful_answer']=False;x['decision']='abstain';x['claim_metrics']['counts']={k:None for k in x['claim_metrics']['counts']}
    rows.append(x);planned.append({'id':ident,'row_id':x['row_id'],'provider':provider,'track':track,'arm':arm,'expected':x['expected'],'required_groups_G':1})
summary=scorer.summarize(rows,planned,protocol,deps)
assert summary['semantic_gate_pass'] is True
findings.append({'id':'R4-incomplete-helper-baseline-passes','helper_baseline_incomplete_rows':72,'semantic_gate_pass':True,'blockers':summary['blockers'],'note':'score_first runtime gate checks only normal pairs; helper baseline completeness is not independently rejected there.'})
dump('R4.output.json',summary)

# Complete model audit transcript, fabricated structural metadata only.
def model_events():
 return [{'kind':'model_request','generation':1,'provider':'mistral','model':'structural-fixture','request':{'task':'structural test only','max_tokens':16}}, {'kind':'wire_request','generation':1,'provider':'mistral','body':{'model':'structural-fixture'}},{'kind':'wire_http_response','generation':1,'provider':'mistral','status':200},{'kind':'model_response','generation':1,'response':{'text':'inert fixture','model':'structural-fixture','usage':{'input_tokens':12,'output_tokens':2,'total_tokens':14},'provider_attempts':1}}]
# R5: normal trace has zero request/response events, despite usage/audit saying1.
final={'status':'unresolved','text':''};outcome={'artifact':{'evidence':[],'claims':[],'verification_receipts':[]}}
payload={'result':{'output_contract':'reason-natural-output-v4','configuration':{'provider':'mistral','model':'structural-fixture'},'usage':{'run':{'model_calls':1}},'finalization':final,'final_outcome':outcome}}
trace={'events':[{'sequence':0,'kind':'grounding_state','outcome':copy.deepcopy(outcome)},{'sequence':1,'kind':'finalization_state','result':copy.deepcopy(final)}]}
bound=capture.bind_normal(payload,trace,[],{'resolution':{'investigation':{'capabilities':[]}}},{'_capture_sha256':'c'*64,'sources':{}},model_events(),'mistral','structural-fixture')
assert bound['usage']['generations']==1
findings.append({'id':'R5-unbound-model-trace','actual_CLI_trace_model_events':0,'counted_audit_generations':1,'accepted':True})
dump('R5.input.json',{'payload':payload,'trace':trace,'model_events':model_events()});dump('R5.output.json',bound)

# R6: a supported verification receipt with no evidence passes vacuous all().
# The proposition deliberately has no matching structured fact. No truth judgment
# or source accuracy is being measured; only receipt structure is under test.
src={'quote':'Inert bytes.','context_summary':'structural only','document_id':'d','url':'https://fixture.invalid/','locator':'unit','source_version':'v','origin_group':'o','scope':{'product':'unit'},'synthetic_mutation':False,'reference_facts':{'expected.key':'yes'},'retrieved_at_unix_seconds':100}
qscope={'product':{'kind':'values','values':['unit']}}
record={'id':'e','source':'s','observation':json.dumps({k:src[k] for k in ['quote','context_summary','document_id','url','locator','source_version','origin_group','scope','synthetic_mutation']}),'facts':src['reference_facts']}
admitted={**copy.deepcopy(record),'metadata':{'scope':qscope,'provenance_class':'primary','temporal':{'effective_from_unix_seconds':100,'effective_until_unix_seconds':200}}}
prop={'key':'different.key','value':'not-present'}
artifact={'evidence':[admitted],'claims':[{'id':'claim','state':'supported','proposition':prop,'evidence_ids':['e']}],'verification_receipts':[{'id':'receipt','claim_id':'claim','proposition':prop,'conclusion':'supported','evidence_ids':[],'verifier':'non-structured-fixture'}]}
outcome={'artifact':artifact};final={'status':'grounded_answer','text':'different.key = not-present'}
payload['result'].update(finalization=final,final_outcome=outcome)
trace={'events':[{'sequence':0,'kind':'model_request','correlation_id':'r','attempt':1},{'sequence':1,'kind':'model_response','correlation_id':'r','attempt':1},{'sequence':2,'kind':'evidence_admission','target_id':'target','capability_id':'cap','admitted_evidence_ids':['e']},{'sequence':3,'kind':'grounding_state','outcome':copy.deepcopy(outcome)},{'sequence':4,'kind':'finalization_state','result':copy.deepcopy(final)}]}
policy={'sources':{'s':{'scope':qscope,'authority_class':'primary','max_age_seconds':100}},'required_scope':qscope,'authority_ranks':{'primary':20},'minimum_authority_class':'primary','evaluation_time_unix_seconds':150}
config={'resolution':{'investigation':{'capabilities':[{'id':'cap','admission':policy}]}}}
journal=[{'corpus_sha256':'c'*64,'case':'unrelated-case','request':{'request':{'id':'investigation:target:cap','target':{'target_id':'different-target','expected_fact_key':'unrelated'}}},'response':{'contribution':{'evidence':[record]}}}]
bound=capture.bind_normal(payload,trace,journal,config,{'_capture_sha256':'c'*64,'sources':{'s':src}},model_events(),'mistral','structural-fixture')
findings.append({'id':'R6-empty-receipt-and-wrong-target','accepted':True,'receipt_evidence_ids':[],'receipt_verifier':'non-structured-fixture','request_target':'different-target','admission_target':'target'})
dump('R6.input.json',{'payload':payload,'trace':trace,'journal':journal,'config':config,'corpus':{'_capture_sha256':'c'*64,'sources':{'s':src}}});dump('R6.output.json',bound)

# R7: a valid-shaped structural corpus with empty manifest inventory does not bind
# its gold. No independent evaluation question, document or accuracy gold is used.
root=OUT/'manifest-coverage-fixture';root.mkdir()
raw=b'structural transport bytes';(root/'raw.txt').write_bytes(raw);(root/'normalized.txt').write_bytes(raw)
h=scorer.sha(raw);doc={'product':'structural-fixture','url':'https://fixture.invalid/','publisher':'fixture','license_or_capture_basis':'inert structural test','raw_path':'raw.txt','normalized_path':'normalized.txt','raw_sha256':h,'normalized_sha256':h,'normalization':'utf8-plain-whitespace-v1','retrieved_at_unix_seconds':100}
sources={}
for sid in ['source-a','source-b']:
 sources[sid]={'source_id':sid,'document_id':'fixture','url':doc['url'],'normalized_path':'normalized.txt','raw_sha256':h,'normalized_sha256':h,'start_byte':0,'end_byte':len(raw),'quote':raw.decode(),'quote_sha256':h,'source_version':'sha256:'+h,'synthetic_mutation':False,'context_summary':'Only schema/immutability test; no real assertions.','locator':'fixture','origin_group':'fixture','scope':{},'reference_facts':{},'retrieved_at_unix_seconds':100}
cases=[{'id':'schema-'+str(i),'question':'structural schema field','helper_question':'structural schema field','source_ids':['source-a','source-b'],'required_scope':{},'lookup_key':'inert','question_as_of':'fixture','question_scope_note':'not an accuracy case'} for i in range(24)]
data={'identity':'engine073-realworld-independent-v1','cases':cases,'documents':{'fixture':doc},'sources':sources}
blob=json.dumps(data).encode();(root/'corpus.json').write_bytes(blob)
gold={'corpus_sha256':scorer.sha(blob),'author':{'role':'agent'},'source_review':{'role':'agent','complete':True},'records':[{'id':c['id']+'/'+track,'case_id':c['id'],'track':track,'label':'positive' if i<8 else 'negative','answerability':'none','groups':[],'unresolved_facets':[],'required_disclosures':[],'full_source_rationale':'schema-only','scope_time_origin_review':'schema-only'} for i,c in enumerate(cases) for track in ['e2e_normal','helper_opt_in']]}
(root/'gold.json').write_text(json.dumps(gold));(root/'host-policy.json').write_text(json.dumps({c['id']:{'controlled_reference_authorized':False,'authorization_rationale':'inert structural control'} for c in cases}))
manifest={'files':{}};(root/'manifest.json').write_text(json.dumps(manifest));anchor=sha(root/'manifest.json')
corpus.validate(root,anchor);before=sha(root/'gold.json')
gold['records'][0]['label']='negative';gold['records'][16]['label']='positive'
(root/'gold.before.json').write_bytes((root/'gold.json').read_bytes());(root/'gold.json').write_text(json.dumps(gold))
corpus.validate(root,anchor);after=sha(root/'gold.json');assert before!=after
findings.append({'id':'R7-M1-empty-inventory-does-not-bind-gold','same_M1_anchor':anchor,'old_gold_hash':before,'changed_gold_hash':after,'both_validations_accepted':True,'fixture_scope':'schema metadata only, never source truth/accuracy or real independent questions'})

# R8: incomplete helper output contains a baseline display; collector invents an
# empty answer for both arms instead of exposing the observed partial display.
lane=OUT/'partial-helper-lane';lane.mkdir();job=lane/'helper-job';job.mkdir();croot=OUT/'collect-corpus-stub';croot.mkdir();co=OUT/'collect-output'
(job/'spawn.json').write_text('{}');partial={'source_ids':['s','t'],'baseline':{'original':{'text':'visible partial source-local output','citations':[]}},'operational_failure':'fixture-stop'}
(job/'stdout.json').write_text(json.dumps(partial))
(lane/'plan.json').write_text(json.dumps({'provider':'mistral','model':'fixture','M1_sha256':'a'*64,'jobs':[{'id':'helper-job','contract':'helper','arm':'helper','argv':['wrapper','--case','structural-only']}]}));(lane/'summary.json').write_text(json.dumps({'rows':[{'id':'helper-job','complete':False,'operational_failure':'fixture-stop'}]}))
(croot/'corpus.json').write_text(json.dumps({'sources':{}}));(croot/'gold.json').write_text(json.dumps({'records':{'structural-only/helper_opt_in':{'id':'structural-only/helper_opt_in'}}}))
cmd=[sys.executable,'-B',str(IN/'collect.py'),'--corpus-dir',str(croot),'--provider','google','--lane',str(lane),'--output',str(co)]
with (OUT/'collect-first.stdout').open('xb') as out,(OUT/'collect-first.stderr').open('xb') as err:r0=subprocess.run(cmd,stdout=out,stderr=err,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),check=False)
assert r0.returncode==0
obs=[json.loads(f.read_text()) for f in co.glob('*.json') if not f.name.endswith('.review-packet.json') and f.name!='observation-index.json']
assert len(obs)==2 and all(x['display_artifacts'][0]['text']=='' for x in obs)
assert all(x['provider']=='google' for x in obs)
findings.append({'id':'R8-partial-helper-display-lost','partial_baseline_text':partial['baseline']['original']['text'],'collected_display_texts':[x['display_artifacts'][0]['text'] for x in obs],'operation':[x['operation'] for x in obs],'note':'Incomplete remains NO_GO, but full partial-output safety audit lacks the observed baseline display.'})
findings.append({'id':'R9-collector-provider-relabel','lane_provider':'mistral','collector_argument':'google','accepted_provider_labels':[x['provider'] for x in obs],'command_returncode':r0.returncode})
dump('collect-first.invocation.json',{'command':cmd,'returncode':r0.returncode,'Engine_or_provider_calls':0})

# Positive identity evidence from hash-only Git tree lookup supplied in tool log.
dump('paired-build-tree-check.json',{'common_host_core_tree':'fa32450addcbe9dfa3ad74f6ef8e94fbc7eda119','baseline_tag_core_tree':'fa32450addcbe9dfa3ad74f6ef8e94fbc7eda119','candidate_core_tree':'535d2f1ad21457905fa43722c74510de59015695','baseline_source_tree_matches_current_fixed_host':True,'lookup':'git rev-parse <pinned commit>:crates/reasoning-harness-core; no implementation contents read or Git mutation'})
assert all(sha(IN/n)==h for n,h in index.items())
dump('observations.json',{'snapshot_manifest_sha256':sha(IN/'input-manifest.json'),'existing_tests':12,'existing_tests_pass':True,'findings':findings,'Engine_accuracy_actual_cohort_scorer_provider_model_network_Oracle_calls':0,'pure_component_and_metadata_fixtures_only':True})
print(json.dumps({'existing_tests':'12 PASS','reproduced_findings':[x['id'] for x in findings],'Engine_accuracy_and_external_calls':0,'snapshot_unchanged':True},indent=2))
