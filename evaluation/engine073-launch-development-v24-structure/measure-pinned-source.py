from pathlib import Path
import json,hashlib,subprocess,importlib.util
r=Path.cwd();shadow=r;out=r/'evaluation/engine073-launch-development-v24-structure';out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
source=shadow/'crates/reasoning-harness-core/src/evidence_relevance.rs';source_hash=sha(source)
events=[json.loads(x) for x in Path('/private/tmp/engine073-source25-final-build-artifacts.jsonl').read_text().splitlines()]
artifacts={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'}
core=next(Path(x) for x in artifacts['reasoning_harness_core'] if x.endswith('.rlib'))
deps_core=next(Path(x).with_suffix('.rlib') for x in artifacts['reasoning_harness_core'] if x.endswith('.rmeta'))
assert sha(core)==sha(deps_core)
serde=next(Path(x) for x in artifacts['serde_json'] if x.endswith('.rlib'))
deps=deps_core.parent
identity={'source_sha':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'kind':'consumed development on newly pinned actual worktree source, not fresh independent acceptance or CLI integration','base_source_sha':'3ccda6b4263eb5210477660b11f6926ec0186f4e','base_source_file_sha256':'85a64b506f8c47828a4a3296c5d07c13e921c59b528ea27e10ec4993c5f636e2','source_file_sha256':source_hash,'core':{'path':str(core),'sha256':sha(core)},'deps_core':{'path':str(deps_core),'sha256':sha(deps_core)},'serde':{'path':str(serde),'sha256':sha(serde)},'compiler_metadata_sha256':sha(Path('/private/tmp/engine073-source25-final-build-artifacts.jsonl'))}
assert sha(source)==json.loads(Path('/private/tmp/engine073-source25-freeze.json').read_text())['source_file_sha256']
assert hashlib.sha256(subprocess.check_output(['git','show','HEAD:crates/reasoning-harness-core/src/evidence_relevance.rs'])).hexdigest()==source_hash
(out/'identity-before-invocation.json').write_text(json.dumps(identity,indent=2)+'\n')
invocations=[]
def compile(name,relative):
 exe=Path('/private/tmp')/('engine073-source25-'+name)
 assert not exe.exists()
 subprocess.run(['rustc','--edition=2024',str(r/relative),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(deps),'-o',str(exe)],check=True)
 return exe
def run(exe,inputs,name):
 before={'executable_sha256':sha(exe),'inputs_path':str(inputs),'inputs_sha256':sha(inputs),'source_file_sha256':sha(source),'core_sha256':sha(core),'serde_sha256':sha(serde)}
 assert before['source_file_sha256']==source_hash and before['core_sha256']==identity['core']['sha256']
 result=subprocess.run([str(exe),str(inputs)],capture_output=True,check=True);assert not result.stderr
 (out/name).write_bytes(result.stdout)
 assert sha(inputs)==before['inputs_sha256'] and sha(exe)==before['executable_sha256']
 invocations.append({'before':before,'returncode':result.returncode,'stdout_file':name,'stdout_sha256':sha(out/name),'stderr_empty':True,'provider_calls':0})
 (out/'development-invocations.json').write_text(json.dumps(invocations,indent=2)+'\n')
 return [json.loads(x) for x in result.stdout.splitlines()]
hist=compile('historical-probe','evaluation/launch-520-521-final-validation/historical-compatibility-probe.rs')
history=run(hist,r/'evaluation/launch-520-521-final-validation/historical-compatibility-inputs.json','historical-after.jsonl');assert len(history)==597 and (out/'historical-after.jsonl').read_bytes()==(r/'evaluation/launch-520-521-final-validation/historical-base.jsonl').read_bytes()
audit=compile('original-audit','evaluation/engine-0.7.1-rereview-20261013/probe.rs');original=run(audit,r/'evaluation/engine-0.7.2-verification-audit-20261014/cases.json','original-audit-after.jsonl');assert len(original)==14 and all(not x['mismatch'] for x in original)
probe=compile('seen-probe','evaluation/engine073-launch-independent-v24/probe.rs')
spec=importlib.util.spec_from_file_location('metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');metrics=importlib.util.module_from_spec(spec);spec.loader.exec_module(metrics)
metrics.verified_versions(r,'v24','120f3c9e780512244c890635095069b100593ef8486300fba72390fcd2ae617a')
preserved=json.loads((r/'evaluation/engine073-launch-independent-v24/final-artifact-manifest.json').read_text())['preserved_preparation_files']
for rel,digest in preserved.items():assert sha(r/rel)==digest
summary={'identity':identity,'historical_byte_identical':597,'original_audit_passed':14,'seen_regressions':{},'v24':{},'fresh_independent_semantic_calls':0,'scorer_invocations':0,'provider_model_Oracle_calls':0,'development_probe_processes':23,'development_inputs':1871}
for version in [*range(2,11),12,13,14,15,16,18,19,20,21,22,23,24]:
 directory=r/f'evaluation/engine073-launch-independent-v{version}';cases=json.loads((directory/'cases.json').read_text())['cases']
 rows=run(probe,directory/'inputs.json',f'seen-v{version}-after.jsonl');by={row['id']:row for row in rows};assert len(by)==len(rows)==len(cases)==60
 pairs={axis:[] for axis in metrics.AXES};mismatches=[]
 for c in cases:
  row=by[c['id']];observed={'v17_requested_relation':row['effective_v17']['relation_scope']=='requested_relation','v30_relevant':row['v30']['disposition']=='relevant'}
  for axis in metrics.AXES:pairs[axis].append((c['expected'][axis],observed[axis]))
  if any(c['expected'][axis]!=observed[axis] for axis in metrics.AXES):mismatches.append(c['id'])
 axes={axis:metrics.binary_metrics(values) for axis,values in pairs.items()}
 if version==24:
  before=json.loads((r/'evaluation/engine073-launch-first-metrics-v24-report.json').read_text())['versions']['v24']['axes']
  summary['v24']={'inputs_sha256':sha(directory/'inputs.json'),'cases_sha256':sha(directory/'cases.json'),'baseline_first_source_sha':'3ccda6b4263eb5210477660b11f6926ec0186f4e','before_first_axes':before,'after_development_axes':axes,'passed':60-len(mismatches),'mismatches':mismatches,'scope':'same frozen consumed input/labels, new pinned source on same consumed input/labels; first FAIL unchanged; not fresh acceptance'}
  assert not mismatches and all(x['confusion']['fp']==0 for x in axes.values()),(mismatches,axes)
  baseline={x['id']:x for x in map(json.loads,(directory/'first.stdout.jsonl').read_text().splitlines())}
  assert all(row['v29']==baseline[row['id']]['v29'] for row in rows)
  summary['v24']['historical_v29_all60_equal']=True
 else:
  summary['seen_regressions'][f'v{version}']={'passed':60-len(mismatches),'mismatches':mismatches};assert not mismatches,(version,mismatches)
assert sha(source)==source_hash
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==identity['source_sha']
assert sha(core)==identity['core']['sha256'] and sha(serde)==identity['serde']['sha256']
metrics.verified_versions(r,'v24','120f3c9e780512244c890635095069b100593ef8486300fba72390fcd2ae617a')
for rel,digest in preserved.items():assert sha(r/rel)==digest
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'historical':597,'original':14,'old_seen':1200,'v24':summary['v24']},indent=2))
