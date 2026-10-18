from pathlib import Path
import json,hashlib,subprocess,os,sys,gzip,shutil
stage=sys.argv[1];assert stage.startswith('stage') and stage[5:].isdigit();r=Path.cwd();d=Path('/private/tmp')/('engine073-source29-'+stage);src=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();expected=hashlib.sha256(gzip.decompress((d/'source.rs.gz').read_bytes())).hexdigest();assert sha(src)==expected
origin=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness/evaluation/engine073-launch-independent-v27');cases=json.loads((origin/'cases.json').read_text())['cases'];before=sha(src)
env=dict(os.environ,CARGO_TARGET_DIR='/private/tmp/engine073-source29-trace-target',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',CARGO_BUILD_JOBS='2')
with (d/'build.jsonl').open('xb') as out,(d/'build.stderr').open('xb') as err:subprocess.run(['cargo','build','--locked','--offline','-p','reasoning-harness-core','--lib','--message-format=json-render-diagnostics'],env=env,stdout=out,stderr=err,check=True)
artifacts={}
for line in (d/'build.jsonl').read_text().splitlines():
 x=json.loads(line)
 if x.get('reason')=='compiler-artifact' and x['target']['name'] in ['reasoning_harness_core','serde_json']:
  files=[Path(f) for f in x['filenames'] if f.endswith('.rlib')]
  if files:artifacts[x['target']['name']]=files[0]
assert set(artifacts)=={'reasoning_harness_core','serde_json'};assert sha(src)==before
probe=d/'probe.rs';shutil.copyfile(origin/'probe.rs',probe);exe=d/'probe'
cmd=['rustc','--edition=2024',str(probe),'-L','dependency=/private/tmp/engine073-source29-trace-target/debug/deps','--extern','reasoning_harness_core='+str(artifacts['reasoning_harness_core']),'--extern','serde_json='+str(artifacts['serde_json']),'-o',str(exe)]
subprocess.run(cmd,check=True,capture_output=True);identity={'source_sha256':before,'libraries':{n:{'path':str(p),'sha256':sha(p)} for n,p in artifacts.items()},'probe_source_sha256':sha(probe),'executable_sha256':sha(exe),'inputs_sha256':sha(origin/'inputs.json'),'labels_sha256':sha(origin/'cases.json'),'scope':'Normal standalone consumed development, not independent retry/scorer'};(d/'identity-before.json').write_text(json.dumps(identity,indent=2)+'\n')
with (d/'output.jsonl').open('xb') as out,(d/'probe.stderr').open('xb') as err:p=subprocess.run([str(exe),str(origin/'inputs.json')],stdout=out,stderr=err,check=False)
assert p.returncode==0 and (d/'probe.stderr').stat().st_size==0 and sha(src)==before and sha(exe)==identity['executable_sha256']
rows=[json.loads(s) for s in (d/'output.jsonl').read_text().splitlines()];assert len(rows)==60;byid={x['id']:x for x in rows};assert len(byid)==60;original={x['id']:x['raw_observation'] for x in json.loads((origin/'first-results.json').read_text())['cases']};metrics={};misses={};joint=[]
for axis in ['v17_requested_relation','v30_relevant']:
 c=dict(tp=0,fp=0,tn=0,fn=0);miss=[]
 for case in cases:
  row=byid[case['id']];gold=case['expected'][axis];obs=row['effective_v17']['relation_scope']=='requested_relation' if axis=='v17_requested_relation' else row['v30']['disposition']=='relevant';c['tp' if gold and obs else 'fn' if gold else 'fp' if obs else 'tn']+=1
  if gold!=obs:miss.append(case['id'])
 tp,fp,tn,fn=(c[n] for n in ['tp','fp','tn','fn']);metrics[axis]={**c,'precision':tp/(tp+fp) if tp+fp else None,'recall':tp/(tp+fn) if tp+fn else None,'f1':2*tp/(2*tp+fp+fn) if 2*tp+fp+fn else 0};misses[axis]=miss
for case in cases:
 row=byid[case['id']];assert row['v29']==original[case['id']]['v29'];g=case['polarity']=='positive';joint.append((row['effective_v17']['relation_scope']=='requested_relation')==g and (row['v30']['disposition']=='relevant')==g)
summary={'source_sha256':before,'axes':metrics,'misses':misses,'joint_passed':sum(joint),'planned':60,'original_first_stays':'FAIL/NO_GO TP24/FP0/TN30/FN6,F1.888889','historical_v29_all60_equal':True,'normal_Engine_processes':1,'input_count':60,'canonical_scorer_calls':0,'provider_model_Oracle_calls':0,'decision':'CONSUMED_DEVELOPMENT_ONLY_NOT_FRESH_ACCEPTANCE'};(d/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
