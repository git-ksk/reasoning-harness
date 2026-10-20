from pathlib import Path
import argparse,hashlib,json,subprocess,importlib.util,shutil,datetime
p=argparse.ArgumentParser();p.add_argument('stage');p.add_argument('source_root',type=Path);p.add_argument('metadata',type=Path);a=p.parse_args()
r=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness');d=Path('/private/tmp')/('engine073-stage25-'+a.stage);d.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
source=a.source_root/'crates/reasoning-harness-core/src/evidence_relevance.rs';assert 'record_launch_development_trace_v17' not in source.read_text();assert (a.source_root/'Cargo.lock').read_bytes()==(r/'Cargo.lock').read_bytes()
events=[json.loads(x) for x in a.metadata.read_text().splitlines()];artifacts={x['target']['name']:x['filenames'] for x in events if x.get('reason')=='compiler-artifact'}
core=next(Path(x) for x in artifacts['reasoning_harness_core'] if x.endswith('.rlib'));deps_core=next(Path(x).with_suffix('.rlib') for x in artifacts['reasoning_harness_core'] if x.endswith('.rmeta'));assert sha(core)==sha(deps_core);serde=next(Path(x) for x in artifacts['serde_json'] if x.endswith('.rlib'))
inputs=r/'evaluation/engine073-launch-independent-v24/inputs.json';labels=r/'evaluation/engine073-launch-independent-v24/cases.json';cases=json.loads(labels.read_text())['cases']
exe=d/'probe';probe=r/'evaluation/engine073-launch-independent-v24/probe.rs';subprocess.run(['rustc','--edition=2024',str(probe),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(deps_core.parent),'-o',str(exe)],check=True)
identity={'stage':a.stage,'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'kind':'consumed development prototype only; no independent acceptance','source_file_sha256':sha(source),'core_sha256':sha(core),'serde_sha256':sha(serde),'metadata_sha256':sha(a.metadata),'inputs_sha256':sha(inputs),'cases_sha256':sha(labels),'probe_source_sha256':sha(probe),'probe_executable_sha256':sha(exe),'base_source_sha':'3ccda6b4263eb5210477660b11f6926ec0186f4e'}
(d/'identity-before.json').write_text(json.dumps(identity,indent=2)+'\n');shutil.copyfile(source,d/'source.rs');shutil.copyfile(a.metadata,d/'cargo-artifacts.jsonl')
result=subprocess.run([str(exe),str(inputs)],capture_output=True);(d/'stdout.jsonl').write_bytes(result.stdout);(d/'stderr.txt').write_bytes(result.stderr)
inv={'returncode':result.returncode,'stdout_sha256':sha(d/'stdout.jsonl'),'stderr_sha256':sha(d/'stderr.txt'),'provider_model_Oracle_calls':0,'scorer_invocations':0,'development_probe_processes':1,'development_inputs':60}
(d/'invocation.json').write_text(json.dumps(inv,indent=2)+'\n');assert result.returncode==0 and not result.stderr
rows=[json.loads(x) for x in result.stdout.splitlines()];by={x['id']:x for x in rows};assert len(by)==len(rows)==len(cases)==60 and set(by)=={c['id'] for c in cases}
spec=importlib.util.spec_from_file_location('metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
pairs={axis:[] for axis in m.AXES};misses=[];unsafe=[]
for c in cases:
 x=by[c['id']];o={'v17_requested_relation':x['effective_v17']['relation_scope']=='requested_relation','v30_relevant':x['v30']['disposition']=='relevant'}
 for axis in m.AXES:pairs[axis].append((c['expected'][axis],o[axis]))
 if any(c['expected'][axis]!=o[axis] for axis in m.AXES):misses.append(c['id'])
 if c['polarity']=='negative' and any(o.values()):unsafe.append(c['id'])
stats={axis:m.binary_metrics(values) for axis,values in pairs.items()};baseline=json.loads((r/'evaluation/engine073-launch-first-metrics-v24-report.json').read_text())['versions']['v24']['axes']
assert sha(source)==identity['source_file_sha256'] and sha(core)==identity['core_sha256'] and sha(serde)==identity['serde_sha256'] and sha(inputs)==identity['inputs_sha256'] and sha(labels)==identity['cases_sha256']
summary={'identity':identity,'after_development_axes':stats,'before_first_v24_axes':baseline,'joint_passed':60-len(misses),'mismatches':misses,'unsafe_ids':unsafe,'safety_gate':'PASS' if not unsafe else 'FAIL','scope':'same frozen consumed inputs; old independent first outputs/labels/scorer unchanged; partial prototype not merge/release acceptance'}
(d/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');(d/'artifact-sha256.json').write_text(json.dumps({p.name:sha(p) for p in sorted(d.iterdir()) if p.is_file()},indent=2)+'\n')
print(json.dumps({'stage':a.stage,'v30':stats['v30_relevant'],'v17':stats['v17_requested_relation'],'joint_passed':summary['joint_passed'],'mismatches':misses,'unsafe':unsafe},indent=2))
raise SystemExit(2 if unsafe else 0)
