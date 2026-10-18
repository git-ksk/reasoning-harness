from pathlib import Path
import json,hashlib,subprocess,sys
r=Path.cwd();stage=Path('/private/tmp/engine073-source27-stage09');identity=json.loads((stage/'identity-before.json').read_text());source=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();assert sha(source)==identity['source_sha256'];assert not json.loads((stage/'summary.json').read_text())['unsafe_ids'];assert json.loads((stage/'summary.json').read_text())['joint_passed']==60
out=Path('/private/tmp/engine073-source27-known-stage09');out.mkdir();exe=stage/'probe';assert sha(exe)==identity['executable_sha256'];inv=[];total=0
for v in [*range(2,11),12,13,14,15,16,18,19,20,21,22,23,24]:
 d=r/f'evaluation/engine073-launch-independent-v{v}';p=subprocess.run([str(exe),str(d/'inputs.json')],capture_output=True,check=True);assert not p.stderr;(out/f'v{v}.jsonl').write_bytes(p.stdout);rows=[json.loads(x) for x in p.stdout.splitlines()];by={x['id']:x for x in rows};cases=json.loads((d/'cases.json').read_text())['cases'];assert len(rows)==len(by)==len(cases)==60
 bad=[]
 for c in cases:
  x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
  if o!=c['expected']:bad.append(c['id'])
 inv.append({'version':v,'inputs_sha256':sha(d/'inputs.json'),'cases_sha256':sha(d/'cases.json'),'stdout_sha256':sha(out/f'v{v}.jsonl'),'executable_sha256':sha(exe),'mismatches':bad});(out/'invocations.json').write_text(json.dumps(inv,indent=2)+'\n');total+=60-len(bad)
extra=r/'evaluation/engine073-launch-development-v24-structure-prototypes/engine073-owner25-adjunct-boundaries/inputs.json';p=subprocess.run([str(exe),str(extra)],capture_output=True,check=True);assert not p.stderr;(out/'owner-adjunct.jsonl').write_bytes(p.stdout);rows=[json.loads(x) for x in p.stdout.splitlines()];assert len(rows)==4 and all(x['effective_v17']['relation_scope']!='requested_relation' and x['v30']['disposition']!='relevant' for x in rows);total+=4
assert sha(source)==identity['source_sha256'];assert sha(Path(identity['core']['path']))==identity['core']['sha256'];summary={'identity':identity,'old_known_passed':total,'mismatches':{str(x['version']):x['mismatches'] for x in inv if x['mismatches']},'including_new_v25':total+60,'Engine_development_processes':22,'inputs':1264,'canonical_scorer_provider_model_Oracle_calls':0,'scope':'same consumed regression development; not fresh acceptance'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))

assert total==1264,total
