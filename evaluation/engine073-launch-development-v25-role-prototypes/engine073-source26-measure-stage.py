from pathlib import Path
import json,hashlib,subprocess,importlib.util,sys,gzip
r=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness');shadow=Path('/private/tmp/engine073-source26-trace-shadow');stage=sys.argv[1];out=Path('/private/tmp')/('engine073-source26-'+stage);out.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
events=[json.loads(x) for x in Path('/private/tmp/engine073-source26-role-build.jsonl').read_text().splitlines()];art={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'}
rlib=lambda name:next(Path(p) for p in art[name] if p.endswith('.rlib'))
core=rlib('reasoning_harness_core');serde=rlib('serde_json');source=shadow/'crates/reasoning-harness-core/src/evidence_relevance.rs'
identity={'private_development_source_sha256':sha(source),'base_source_commit':'95053007bba54c728eee428837b22eb9d52a4f61','core':{'path':str(core),'sha256':sha(core)},'serde':{'path':str(serde),'sha256':sha(serde)},'compiler_metadata_sha256':sha(Path('/private/tmp/engine073-source26-role-build.jsonl'))}
exe=out/'probe';cmd=['rustc','--edition=2024',str(r/'evaluation/engine073-launch-independent-v25/probe.rs'),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(serde.parent),'-o',str(exe)];subprocess.run(cmd,check=True)
inputs=r/'evaluation/engine073-launch-independent-v25/inputs.json';casespath=inputs.with_name('cases.json');cases=json.loads(casespath.read_text())['cases'];identity.update(executable_sha256=sha(exe),inputs_sha256=sha(inputs),cases_sha256=sha(casespath))
(out/'identity-before.json').write_text(json.dumps(identity,indent=2)+'\n');p=subprocess.run([str(exe),str(inputs)],capture_output=True,check=True);assert not p.stderr;(out/'output.jsonl').write_bytes(p.stdout)
rows=[json.loads(x) for x in p.stdout.splitlines()];assert len(rows)==60;by={x['id']:x for x in rows};assert len(by)==60
spec=importlib.util.spec_from_file_location('arithmetic',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);pairs={a:[] for a in m.AXES};failed=[];unsafe=[]
for c in cases:
 x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
 for a in m.AXES:pairs[a].append((c['expected'][a],o[a]))
 if any(c['expected'][a]!=o[a] for a in m.AXES):failed.append(c['id'])
 if c['polarity']=='negative' and any(o.values()):unsafe.append(c['id'])
assert sha(source)==identity['private_development_source_sha256'] and sha(core)==identity['core']['sha256'] and sha(serde)==identity['serde']['sha256']
assert sha(inputs)==identity['inputs_sha256'] and sha(casespath)==identity['cases_sha256']
summary={'stage':stage,'identity':identity,'axes':{a:m.binary_metrics(v) for a,v in pairs.items()},'joint_passed':60-len(failed),'failed_ids':failed,'unsafe_ids':unsafe,'scope':'same consumed v25 development only; original first FAIL unchanged; no fresh gate','Engine_development_processes':1,'inputs':60,'canonical_scorer_provider_model_Oracle_calls':0}
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
(out/'source.rs.gz').write_bytes(gzip.compress(source.read_bytes(),mtime=0));(out/'build.jsonl').write_bytes(Path('/private/tmp/engine073-source26-role-build.jsonl').read_bytes());(out/'build.stderr').write_bytes(Path('/private/tmp/engine073-source26-role-build.stderr').read_bytes())
print(json.dumps(summary,indent=2))
