from pathlib import Path
import json,subprocess,os,hashlib
w=Path('/private/tmp/engine073-cli-path-preparation-v1');repo=Path('/Users/sawadakousuke/x-code/reasoning-harness');sr=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
c=json.loads((sr/'evaluation/engine073-launch-independent-v13/cases.json').read_text())['cases'][0];ip=w/'positive-control-input.json';ip.write_text(json.dumps([dict(c['input'],id='offline-instrumentation-positive-control')],indent=2)+'\n')
proof=json.loads((w/'build-proof.json').read_text())
for arm in ['baseline','candidate']:
 artifacts={}
 for line in (w/(arm+'-build.jsonl')).read_text().splitlines():
  m=json.loads(line)
  if m.get('reason')=='compiler-artifact':
   for name in m['filenames']:
    if name.endswith('.rlib'):artifacts[m['target']['name']]=Path(name)
 core=artifacts['reasoning_harness_core'];serde=artifacts['serde_json'];exe=w/'bin'/('positive-control-'+arm)
 subprocess.run(['rustc','--edition=2024',str(repo/'evaluation/engine-07-realworld-pilot-v1/probe.rs'),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(core.parent),'-o',str(exe)],check=True)
 log=w/(arm+'-positive-control-apis.log');env=os.environ.copy();env['ENGINE07_OFFLINE_API_LOG']=str(log)
 p=subprocess.run([str(exe),str(ip)],env=env,check=True,capture_output=True);assert not p.stderr
 (w/(arm+'-positive-control.jsonl')).write_bytes(p.stdout)
 calls=log.read_text().splitlines();assert 'derive_effective_evidence_local_qualification_v17' in calls and 'materialize_evidence_relevance_v30' in calls
 proof['arms'][arm]['positive_control_public_api_calls']=len(calls)
 proof['arms'][arm]['positive_control_v17_v30_seen']=True
 proof['arms'][arm]['normal_path_public_relevance_api_calls']=0
 print(arm,'positive control logger works, calls',len(calls))
requests=[];summaries=[]
for arm in ['baseline','candidate']:
 d=w/('run-'+arm);models=[json.loads(x) for x in (d/'model.jsonl').read_text().splitlines()];assert len(models)==5;requests.append(models)
 r=json.loads((d/'stdout.json').read_text())['result'];assert r['finalization']['status']=='grounded_answer';assert r['finalization']['text']=='node.buffer.uint8array = true'
 assert not (d/'relevance-api.log').exists()
 events=json.loads((d/'trace.json').read_text())['events'];assert any(x['kind']=='evidence_admission' for x in events)
 summaries.append({'arm':arm,'offline_callback_count':5,'resolver_calls':len((d/'resolver.jsonl').read_text().splitlines()),'relevance_api_calls':0,'finalization_status':r['finalization']['status'],'finalization_text':r['finalization']['text']})
assert requests[0]==requests[1]
proof.update(model_requests_and_controlled_responses_equal=True,summary=summaries,scope='one new controlled Node.js case; instrumented common host; not original provider-response replay or canonical real-source acceptance')
(w/'verified-result.json').write_text(json.dumps(proof,indent=2)+'\n')
print('paired path verified')
