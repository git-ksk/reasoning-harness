from pathlib import Path
import json,hashlib,subprocess,importlib.util
r=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness');shadow=Path('/private/tmp/engine073-source21-trace-shadow');out=Path('/private/tmp/engine073-foundation22-prototype');out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
source=shadow/'crates/reasoning-harness-core/src/evidence_relevance.rs';source_hash=sha(source)
events=[json.loads(x) for x in Path('/private/tmp/engine073-foundation22-build-final.jsonl').read_text().splitlines()]
artifacts={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'}
core=next(Path(x) for x in artifacts['reasoning_harness_core'] if x.endswith('.rlib'))
deps_core=next(Path(x).with_suffix('.rlib') for x in artifacts['reasoning_harness_core'] if x.endswith('.rmeta'))
assert sha(core)==sha(deps_core)
serde=next(Path(x) for x in artifacts['serde_json'] if x.endswith('.rlib'))
deps=deps_core.parent
identity={'kind':'consumed development prototype, not a new independent candidate or CLI integration','base_source_sha':'7431333d212938147a5d9ab32a0b8bdb44ee16b8','base_source_file_sha256':'b7fec3e0f7c6b76dfbff1b801dd085a22661df9b22457431653eb2073a4e08f0','prototype_source_file_sha256':source_hash,'core':{'path':str(core),'sha256':sha(core)},'deps_core':{'path':str(deps_core),'sha256':sha(deps_core)},'serde':{'path':str(serde),'sha256':sha(serde)},'compiler_metadata_sha256':sha(Path('/private/tmp/engine073-foundation22-build-final.jsonl'))}
assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')==identity['base_source_file_sha256']
def compile(name,relative):
 exe=out/name
 subprocess.run(['rustc','--edition=2024',str(r/relative),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(deps),'-o',str(exe)],check=True)
 return exe
def run(exe,inputs,name):
 result=subprocess.run([str(exe),str(inputs)],capture_output=True,check=True);assert not result.stderr
 (out/name).write_bytes(result.stdout);return [json.loads(x) for x in result.stdout.splitlines()]
hist=compile('historical-probe','evaluation/launch-520-521-final-validation/historical-compatibility-probe.rs')
history=run(hist,r/'evaluation/launch-520-521-final-validation/historical-compatibility-inputs.json','historical-after.jsonl');assert len(history)==597 and (out/'historical-after.jsonl').read_bytes()==(r/'evaluation/launch-520-521-final-validation/historical-base.jsonl').read_bytes()
audit=compile('original-audit','evaluation/engine-0.7.1-rereview-20261013/probe.rs');original=run(audit,r/'evaluation/engine-0.7.2-verification-audit-20261014/cases.json','original-audit-after.jsonl');assert len(original)==14 and all(not x['mismatch'] for x in original)
probe=compile('seen-probe','evaluation/engine073-launch-independent-v22/probe.rs')
spec=importlib.util.spec_from_file_location('metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');metrics=importlib.util.module_from_spec(spec);spec.loader.exec_module(metrics)
summary={'identity':identity,'historical_byte_identical':597,'original_audit_passed':14,'seen_regressions':{},'v22':{},'Engine_independent_scorer_provider_model_Oracle_calls':0}
for version in [*range(2,11),12,13,14,15,16,18,19,20,21,22]:
 directory=r/f'evaluation/engine073-launch-independent-v{version}';cases=json.loads((directory/'cases.json').read_text())['cases']
 rows=run(probe,directory/'inputs.json',f'seen-v{version}-after.jsonl');by={row['id']:row for row in rows};assert len(by)==len(rows)==len(cases)==60
 pairs={axis:[] for axis in metrics.AXES};mismatches=[]
 for c in cases:
  row=by[c['id']];observed={'v17_requested_relation':row['effective_v17']['relation_scope']=='requested_relation','v30_relevant':row['v30']['disposition']=='relevant'}
  for axis in metrics.AXES:pairs[axis].append((c['expected'][axis],observed[axis]))
  if any(c['expected'][axis]!=observed[axis] for axis in metrics.AXES):mismatches.append(c['id'])
 axes={axis:metrics.binary_metrics(values) for axis,values in pairs.items()}
 if version==22:
  before=json.loads((r/'evaluation/engine073-launch-first-metrics-v1/report.json').read_text())['versions']['v22']['axes']
  summary['v22']={'inputs_sha256':sha(directory/'inputs.json'),'cases_sha256':sha(directory/'cases.json'),'baseline_first_source_sha':identity['base_source_sha'],'before_first_axes':before,'after_development_axes':axes,'passed':60-len(mismatches),'mismatches':mismatches,'scope':'same frozen consumed input/labels, source prototype only; first FAIL unchanged; not fresh acceptance'}
  assert all(x['confusion']['fp']==0 for x in axes.values()),axes
 else:
  summary['seen_regressions'][f'v{version}']={'passed':60-len(mismatches),'mismatches':mismatches};assert not mismatches,(version,mismatches)
assert sha(source)==source_hash
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({'historical':597,'original':14,'old_seen':1080,'v22':summary['v22']},indent=2))
