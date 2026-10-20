from pathlib import Path
import hashlib,json,subprocess,importlib.util
r=Path.cwd();out=r/'evaluation/engine073-launch-development-v25-role-frames';out.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();pin='28cd088192bcaa15e9d064e025b6a1a4c67d0ade';source=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';sourcehash=sha(source);assert sourcehash=='5264d210fb25f1c6f6c3c2eec3e50ff42379947f99bd64242ae761e019bd4711';assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==pin
meta=Path('/private/tmp/engine073-source26-actual-build.jsonl');events=[json.loads(x) for x in meta.read_text().splitlines()];art={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'};lib=lambda n:next(Path(x) for x in art[n] if x.endswith('.rlib'));core,serde=lib('reasoning_harness_core'),lib('serde_json')
identity={'source_sha':pin,'source_file_sha256':sourcehash,'core':{'path':str(core),'sha256':sha(core)},'serde':{'path':str(serde),'sha256':sha(serde)},'cargo_metadata_sha256':sha(meta)};(out/'identity-before.json').write_text(json.dumps(identity,indent=2)+'\n')
idx=r/'evaluation/engine073-launch-independent-v25/final-artifact-manifest.json';assert sha(idx)=='f3ef8db2393aa6c458981e92099d23792856dd1e1431c5876bb51760aefab1aa';archive=json.loads(idx.read_text())
def verify_archive():
 for rel,digest in archive['files'].items():assert sha(r/rel)==digest
 assert sha(idx)=='f3ef8db2393aa6c458981e92099d23792856dd1e1431c5876bb51760aefab1aa'
verify_archive();spec=importlib.util.spec_from_file_location('arithmetic',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);inv=[]
def compile(name,rel):
 exe=Path('/private/tmp')/('engine073-source26-actual-'+pin[:7]+'-'+name);assert not exe.exists();subprocess.run(['rustc','--edition=2024',str(r/rel),'-L','dependency='+str(serde.parent),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-o',str(exe)],check=True);return exe
def run(exe,path,name):
 before={'source_sha':pin,'source_file_sha256':sha(source),'core_sha256':sha(core),'executable_sha256':sha(exe),'inputs_path':str(path),'inputs_sha256':sha(path)};assert before['source_file_sha256']==sourcehash and before['core_sha256']==identity['core']['sha256'];p=subprocess.run([str(exe),str(path)],capture_output=True,check=True);assert not p.stderr;(out/name).write_bytes(p.stdout);inv.append(dict(before=before,stdout_file=name,stdout_sha256=sha(out/name),returncode=0,stderr_bytes=0));(out/'invocations.json').write_text(json.dumps(inv,indent=2)+'\n');return [json.loads(x) for x in p.stdout.splitlines()]
h=compile('history','evaluation/launch-520-521-final-validation/historical-compatibility-probe.rs');run(h,r/'evaluation/launch-520-521-final-validation/historical-compatibility-inputs.json','historical-after.jsonl');assert (out/'historical-after.jsonl').read_bytes()==(r/'evaluation/launch-520-521-final-validation/historical-base.jsonl').read_bytes()
a=compile('audit','evaluation/engine-0.7.1-rereview-20261013/probe.rs');audit=run(a,r/'evaluation/engine-0.7.2-verification-audit-20261014/cases.json','original-audit-after.jsonl');assert len(audit)==14 and all(not x['mismatch'] for x in audit)
probe=compile('seen','evaluation/engine073-launch-independent-v25/probe.rs');summary={'identity':identity,'historical_byte_identical':597,'original_audit_passed':14,'old_regressions':{},'source_scope':'same consumed development; original v25 FAIL immutable; NOT new independent/real24','canonical_scorer_provider_model_Oracle_calls':0,'new_independent_authorship_or_runs':0}
for v in [*range(2,11),12,13,14,15,16,18,19,20,21,22,23,24,25]:
 d=r/f'evaluation/engine073-launch-independent-v{v}';cs=json.loads((d/'cases.json').read_text())['cases'];rows=run(probe,d/'inputs.json',f'seen-v{v}-after.jsonl');by={x['id']:x for x in rows};assert len(by)==len(rows)==len(cs)==60;pairs={axis:[] for axis in m.AXES};bad=[];unsafe=[]
 for c in cs:
  x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
  for axis in m.AXES:pairs[axis].append((c['expected'][axis],o[axis]))
  if o!=c['expected']:bad.append(c['id'])
  if c['polarity']=='negative' and any(o.values()):unsafe.append(c['id'])
 axes={k:m.binary_metrics(values) for k,values in pairs.items()}
 if v!=25:summary['old_regressions'][f'v{v}']={'passed':60-len(bad),'mismatches':bad};assert not bad,(v,bad)
 else:
  assert not unsafe;assert (out/f'seen-v{v}-after.jsonl').read_bytes()==Path('/private/tmp/engine073-source26-roles04/output.jsonl').read_bytes()
  baseline={x['id']:x for x in map(json.loads,(d/'first.stdout.jsonl').read_text().splitlines())};assert all(x['v29']==baseline[x['id']]['v29'] for x in rows)
  summary['v25']={'before_first_axes':json.loads((r/'evaluation/engine073-launch-first-metrics-v25-report.json').read_text())['versions']['v25']['axes'],'after_development_axes':axes,'joint_passed':60-len(bad),'remaining_ids':bad,'unsafe_ids':unsafe,'historical_v29_all60_equal':True,'whole_output_equal_private_roles04':True,'inputs_sha256':sha(d/'inputs.json'),'cases_sha256':sha(d/'cases.json')}
extra=r/'evaluation/engine073-launch-development-v24-structure-prototypes/engine073-owner25-adjunct-boundaries/inputs.json';rows=run(probe,extra,'owner-adjunct-after.jsonl');assert len(rows)==4 and all(x['effective_v17']['relation_scope']!='requested_relation' and x['v30']['disposition']!='relevant' for x in rows)
summary.update(old_known_controls_passed=1264,new_v25_controls_passed=summary['v25']['joint_passed'],new_v25_total=60,development_probe_processes=len(inv),development_inputs=1935,extra_owner_adjunct_passed=4)
assert len(inv)==25 and sha(source)==sourcehash and sha(core)==identity['core']['sha256'] and sha(serde)==identity['serde']['sha256'];assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==pin;verify_archive();(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({'source':pin,'old_known':1264,'history':597,'original':14,'v25':summary['v25']},indent=2))
