"""One-factor structural confirmations; no source accuracy/Engine execution."""
from pathlib import Path
import copy,json,sys,os,subprocess
P=Path(__file__).resolve().parent;S=Path('/private/tmp/engine073-realworld-evaluator-review-snapshot-v1');sys.path.insert(0,str(S))
import capture,scorer
v=json.loads((P/'R6.input.json').read_text());events=json.loads((P/'R5.input.json').read_text())['model_events']
prop={'key':'expected.key','value':'yes'}
a=v['payload']['result']['final_outcome']['artifact'];a['claims'][0]['proposition']=prop;a['verification_receipts'][0].update(proposition=prop,evidence_ids=['e'],verifier='structured_fact_equality')
v['payload']['result']['finalization']['text']='expected.key = yes';v['payload']['result']['task']='structural adapter test'
v['journal'][0]['request']['request']['target'].update(target_id='target',expected_fact_key='expected.key');v['journal'][0]['case']='structural-normal'
for e in v['trace']['events']:
 if e['kind']=='grounding_state':e['outcome']=copy.deepcopy(v['payload']['result']['final_outcome'])
 if e['kind']=='finalization_state':e['result']=copy.deepcopy(v['payload']['result']['finalization'])
def call(x):return capture.bind_normal(x['payload'],x['trace'],x['journal'],x['config'],x['corpus'],events,'mistral','structural-fixture')
def dump(n,x):
 with (P/n).open('x') as f:json.dump(x,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
control=call(v);dump('R6-valid-control.input.json',v);dump('R6-valid-control.output.json',control)
empty=copy.deepcopy(v);empty['payload']['result']['final_outcome']['artifact']['verification_receipts'][0]['evidence_ids']=[]
for e in empty['trace']['events']:
 if e['kind']=='grounding_state':e['outcome']=copy.deepcopy(empty['payload']['result']['final_outcome'])
empty_out=call(empty);dump('R6-empty-receipt-only.input.json',empty);dump('R6-empty-receipt-only.output.json',empty_out)
wrong=copy.deepcopy(v);wrong['journal'][0]['request']['request']['target']['target_id']='wrong-target'
wrong_out=call(wrong);dump('R6-wrong-target-only.input.json',wrong);dump('R6-wrong-target-only.output.json',wrong_out)

# Complete-row provider relabel without changing the underlying model trace.
lane=P/'provider-relabel-complete-lane';lane.mkdir();job=lane/'job';job.mkdir();cr=P/'provider-relabel-corpus';cr.mkdir();out=P/'provider-relabel-complete-output'
corpus={k:x for k,x in v['corpus'].items() if k!='_capture_sha256'};blob=json.dumps(corpus).encode();(cr/'corpus.json').write_bytes(blob)
j=copy.deepcopy(v['journal']);j[0]['corpus_sha256']=scorer.sha(blob)
(cr/'gold.json').write_text(json.dumps({'records':{'structural-normal/e2e_normal':{'id':'structural-normal/e2e_normal'}}}))
for n,x in [('stdout.json',v['payload']),('trace.json',v['trace']),('config.json',v['config']),('spawn.json',{})]:(job/n).write_text(json.dumps(x))
(job/'resolver.jsonl').write_text('\n'.join(json.dumps(x) for x in j)+'\n');(job/'model-journal.jsonl').write_text('\n'.join(json.dumps(x) for x in events)+'\n')
(lane/'plan.json').write_text(json.dumps({'provider':'mistral','model':'structural-fixture','M1_sha256':'a'*64,'jobs':[{'id':'job','contract':'normal','arm':'candidate','argv':['wrapper','--case','structural-normal']}]}));(lane/'summary.json').write_text(json.dumps({'rows':[{'id':'job','complete':True,'wall_ms':1}]}))
cmd=[sys.executable,'-B',str(S/'collect.py'),'--corpus-dir',str(cr),'--provider','google','--lane',str(lane),'--output',str(out)]
with (P/'provider-relabel-first.stdout').open('xb') as so,(P/'provider-relabel-first.stderr').open('xb') as se:r=subprocess.run(cmd,stdout=so,stderr=se,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'))
assert r.returncode==0
obs=[json.loads(f.read_text()) for f in out.glob('*.json') if not f.name.endswith('.review-packet.json') and f.name!='observation-index.json'];assert len(obs)==1 and obs[0]['operation']=='complete' and obs[0]['provider']=='google'
dump('isolated-confirmations.json',{'valid_capture_control':'ACCEPTED','empty_receipt_only':'ACCEPTED','wrong_target_only':'ACCEPTED','complete_provider_relabel':{'underlying_lane_and_model_journal_provider':'mistral','collected_provider':obs[0]['provider'],'operation':obs[0]['operation'],'returncode':r.returncode},'collect_command':cmd,'Engine_provider_accuracy_actual_scorer_calls':0})
print('ONE_FACTOR_CONFIRMED: valid control, empty receipt only, wrong target only; complete Mistral row relabeled Google')
