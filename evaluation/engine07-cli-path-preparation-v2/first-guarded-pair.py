from pathlib import Path
import json,hashlib,importlib.util,sys
previous=Path('/private/tmp/engine073-cli-path-preparation-v1');w=Path('/private/tmp/engine073-cli-path-preparation-v2');w.mkdir(exist_ok=False)
import shutil
shutil.copyfile(previous/'controlled-transcript.json',w/'controlled-transcript.json')
r=Path('/Users/sawadakousuke/.codex/worktrees/engine073-safe-runner/reasoning-harness');v3=r/'evaluation/engine07-realworld-v3'
def module(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);sys.modules[name]=m;spec.loader.exec_module(m);return m
resolver=module('old_read_only_resolver',v3/'resolver.py');runner=module('new_operational_runner',r/'evaluation/engine07-realworld-runner-v1/runner.py')
corpus=json.loads((v3/'corpus.json').read_text());case=next(c for c in corpus['cases'] if c['id']=='rw03-node-positive')
out=w/'guarded-pair-first';jobs=[];artifacts={}
def pin(p):artifacts[str(p)]=hashlib.sha256(p.read_bytes()).hexdigest()
for arm in ['baseline','candidate']:
 binary=previous/'bin'/('reason-'+arm);cfgpath=w/(arm+'-guarded-config.json');cfg=resolver.config(corpus,case,v3/'corpus.json',out/arm/'resolver.jsonl');cfgpath.write_text(json.dumps(cfg,indent=2)+'\n')
 argv=[str(binary),case['question'],'--provider','mistral','--model','ministral-8b-2512','--seed','531003','--max-tokens','1024','--max-model-calls','12','--max-output-tokens','4096','--max-total-tokens','28096','--config',str(cfgpath),'--format','json','--diagnostic-trace','{RUN_DIR}/trace.json']
 jobs.append({'id':arm,'arm':arm,'contract':'normal','argv':argv,'budget':{'max_generations':5,'max_input_tokens_per_generation':20,'max_output_tokens_per_generation':10,'max_attempts_per_generation':1},'timeout_seconds':30,'cancel_grace_seconds':1})
 pin(binary);pin(cfgpath)
for p in [w/'controlled-transcript.json',v3/'corpus.json',v3/'resolver.py',r/'evaluation/engine07-realworld-runner-v1/runner.py']:pin(p)
plan={'schema':'engine07-operational-plan-v1','provider':'mistral','model':'ministral-8b-2512','token_budget':300,'artifacts':artifacts,'jobs':jobs,'scope':'offline controlled callback accounting; these synthetic token values are NOT live provider bounds'}
(w/'guarded-plan.json').write_text(json.dumps(plan,indent=2)+'\n')
credentials=lambda _: {'MISTRAL_API_KEY':'offline-fixture-not-a-credential','ENGINE07_OFFLINE_TRANSCRIPT':str(w/'controlled-transcript.json'),'ENGINE07_OFFLINE_MODEL_LOG':'model.jsonl','ENGINE07_OFFLINE_API_LOG':'relevance-api.log'}
result=runner.OperationalRunner(plan,out,credentials).run()
assert result['complete'] and len(result['rows'])==2 and result['known_total_tokens']==300 and result['charged_reservations']==300 and result['known_provider_attempts']==10 and result['unknown_usage_rows']==0,result
for row in result['rows']:
 directory=out/row['id'];assert row['complete'];payload=json.loads((directory/'stdout.json').read_text());assert payload['result']['finalization']['status']=='grounded_answer';assert not (directory/'relevance-api.log').exists()
print(json.dumps({k:v for k,v in result.items() if k!='rows'},indent=2))
