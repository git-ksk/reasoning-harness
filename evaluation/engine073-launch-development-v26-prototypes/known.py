from pathlib import Path
import json,hashlib,subprocess
r=Path.cwd();origin=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness');stage=Path('/private/tmp/engine073-source28-stage02');identity=json.loads((stage/'identity-before.json').read_text());source=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();assert sha(source)==identity['source_sha256'];assert json.loads((stage/'summary.json').read_text())['joint_passed']==60
exe=stage/'probe';assert sha(exe)==identity['executable_sha256'];out=Path('/private/tmp/engine073-source28-known-stage02');out.mkdir();inv=[];total=passed=0
for v in [*range(2,11),12,13,14,15,16,18,19,20,21,22,23,24,25]:
 d=origin/f'evaluation/engine073-launch-independent-v{v}';p=subprocess.run([str(exe),str(d/'inputs.json')],capture_output=True,check=True);assert not p.stderr;(out/f'v{v}.jsonl').write_bytes(p.stdout);rows=[json.loads(x) for x in p.stdout.splitlines()];by={x['id']:x for x in rows};cs=json.loads((d/'cases.json').read_text())['cases'];assert len(rows)==len(by)==len(cs)==60;bad=[];unsafe=[]
 for c in cs:
  x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
  if o!=c['expected']:bad.append(c['id'])
  if c['polarity']=='negative' and any(o.values()):unsafe.append(c['id'])
 inv.append({'version':v,'inputs_sha256':sha(d/'inputs.json'),'cases_sha256':sha(d/'cases.json'),'stdout_sha256':sha(out/f'v{v}.jsonl'),'executable_sha256':sha(exe),'mismatches':bad,'unsafe':unsafe});(out/'invocations.json').write_text(json.dumps(inv,indent=2)+'\n');print('v',v,'passed',60-len(bad),'bad',bad,'unsafe',unsafe,flush=True);total+=60;passed+=60-len(bad)
extra=origin/'evaluation/engine073-launch-development-v24-structure-prototypes/engine073-owner25-adjunct-boundaries/inputs.json';p=subprocess.run([str(exe),str(extra)],capture_output=True,check=True);assert not p.stderr;(out/'owner-adjunct.jsonl').write_bytes(p.stdout);rows=[json.loads(x) for x in p.stdout.splitlines()];assert len(rows)==4;extra_passed=sum(x['effective_v17']['relation_scope']!='requested_relation' and x['v30']['disposition']!='relevant' for x in rows);total+=4;passed+=extra_passed
supp=origin/'evaluation/engine073-launch-development-v25-reference-prototypes/supplemental-literals';p=subprocess.run([str(exe),str(supp/'inputs.json')],capture_output=True,check=True);assert not p.stderr;(out/'supplemental.jsonl').write_bytes(p.stdout);by={x['id']:x for x in map(json.loads,p.stdout.splitlines())};cs=json.loads((supp/'cases.json').read_text())['cases'];assert len(by)==len(cs)==16;bad=[]
for c in cs:
 x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
 if o!=c['expected']:bad.append(c['id'])
total+=16;passed+=16-len(bad);assert total==1340 and sha(source)==identity['source_sha256'] and sha(exe)==identity['executable_sha256']
summary={'source_sha256':sha(source),'former_known_total':total,'former_known_passed':passed,'new_v26_passed':60,'including_consumed_v26':passed+60,'known_total':1400,'owner_adjunct_passed':extra_passed,'supplemental_mismatches':bad,'development_Engine_processes':24,'inputs':1340,'original_firsts_unchanged':True,'canonical_scorer_provider_model_Oracle_calls':0,'scope':'consumed development scan; no fresh acceptance'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
