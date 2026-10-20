from pathlib import Path
import subprocess,json,hashlib,gzip,os,sys,importlib.util
sys.dont_write_bytecode=True
r=Path.cwd();stage=sys.argv[1];out=Path('/private/tmp')/('engine073-source27-'+stage);out.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();source=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';digest=sha(source);(out/'source.rs.gz').write_bytes(gzip.compress(source.read_bytes(),mtime=0))
env=dict(os.environ,CARGO_TARGET_DIR='/private/tmp/engine073-source27-prototype-target',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
with (out/'build.jsonl').open('x') as f,(out/'build.stderr').open('x') as e:subprocess.run(['cargo','build','--locked','--offline','-p','reasoning-harness-core','--message-format=json','--jobs','2'],env=env,stdout=f,stderr=e,check=True)
events=[json.loads(x) for x in (out/'build.jsonl').read_text().splitlines()];art={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'};lib=lambda name:next(Path(p) for p in art[name] if p.endswith('.rlib'));core,serde=lib('reasoning_harness_core'),lib('serde_json')
exe=out/'probe';subprocess.run(['rustc','--edition=2024',str(r/'evaluation/engine073-launch-independent-v25/probe.rs'),'-L','dependency='+str(serde.parent),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-o',str(exe)],check=True)
d=r/'evaluation/engine073-launch-independent-v25';ident={'source_sha256':digest,'base_commit':'8f483d118db329e213fb1c7ba03c9dec439dcba3','core':{'path':str(core),'sha256':sha(core)},'serde':{'path':str(serde),'sha256':sha(serde)},'executable_sha256':sha(exe),'inputs_sha256':sha(d/'inputs.json'),'cases_sha256':sha(d/'cases.json')};(out/'identity-before.json').write_text(json.dumps(ident,indent=2)+'\n');p=subprocess.run([str(exe),str(d/'inputs.json')],capture_output=True,check=True);assert not p.stderr;(out/'output.jsonl').write_bytes(p.stdout)
rows=[json.loads(x) for x in p.stdout.splitlines()];by={x['id']:x for x in rows};cases=json.loads((d/'cases.json').read_text())['cases'];assert len(rows)==len(by)==len(cases)==60
spec=importlib.util.spec_from_file_location('metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);pairs={k:[] for k in m.AXES};bad=[];unsafe=[]
for c in cases:
 x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
 for k in m.AXES:pairs[k].append((c['expected'][k],o[k]))
 if o!=c['expected']:bad.append(c['id'])
 if c['polarity']=='negative' and any(o.values()):unsafe.append(c['id'])
assert sha(source)==digest and sha(core)==ident['core']['sha256'] and sha(serde)==ident['serde']['sha256'];summary={'identity':ident,'axes':{k:m.binary_metrics(v) for k,v in pairs.items()},'joint_passed':60-len(bad),'failed_ids':bad,'unsafe_ids':unsafe,'Engine_development_processes':1,'inputs':60,'canonical_scorer_provider_model_Oracle_calls':0,'scope':'same consumed v25 only; original first immutable; NOT independent acceptance'};(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
