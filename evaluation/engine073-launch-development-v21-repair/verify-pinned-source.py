from pathlib import Path
import subprocess,json,hashlib,shutil,re
r=Path.cwd(); out=r/'evaluation/engine073-launch-development-v21-repair'; out.mkdir(exist_ok=False)
source_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
source_hash=hashlib.sha256((r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()).hexdigest()
assert source_sha==json.loads(Path('/private/tmp/engine073-source21-freeze.json').read_text())['source_sha']
assert source_hash==json.loads(Path('/private/tmp/engine073-source21-freeze.json').read_text())['source_file_sha256']
deps=Path('/private/tmp/engine073-target/debug/deps');bins=Path('/private/tmp/engine073-probes'); bins.mkdir(exist_ok=True)
def lib(name):
 files=sorted(deps.glob('lib'+name+'-*.rlib'),key=lambda p:p.stat().st_mtime_ns)
 assert files
 return files[-1]
core=deps/'libreasoning_harness_core-af992f1b2d90d349.rlib';serde=deps/'libserde_json-4247b0767bfe4a83.rlib'
freeze=json.loads(Path('/private/tmp/engine073-source21-freeze.json').read_text())
for key,p in [('reasoning_harness_core',core),('serde_json',serde)]: assert hashlib.sha256(p.read_bytes()).hexdigest()==freeze[key]['sha256']
def compile(name,path):
 exe=bins/(name+'-successor21');assert not exe.exists()
 subprocess.run(['rustc','--edition=2024',str(r/path),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(deps),'-o',str(exe)],check=True)
 return exe
def run(exe,inputs,filename):
 p=subprocess.run([str(exe),str(inputs)],check=True,capture_output=True);assert not p.stderr
 (out/filename).write_bytes(p.stdout)
 return [json.loads(x) for x in p.stdout.splitlines()]
hist=compile('historical','evaluation/launch-520-521-final-validation/historical-compatibility-probe.rs')
h=run(hist,r/'evaluation/launch-520-521-final-validation/historical-compatibility-inputs.json','historical-current.jsonl')
assert len(h)==597
assert (out/'historical-current.jsonl').read_bytes()==(r/'evaluation/launch-520-521-final-validation/historical-base.jsonl').read_bytes()
audit=compile('audit','evaluation/engine-0.7.1-rereview-20261013/probe.rs')
a=run(audit,r/'evaluation/engine-0.7.2-verification-audit-20261014/cases.json','original-audit-after.jsonl')
assert len(a)==14 and all(not x['mismatch'] for x in a)
seen=compile('seen-replay','evaluation/engine-07-realworld-pilot-v1/probe.rs'); stats={}
for v in [*range(2,11),12,13,14,15,16,18,19,20,21]:
 cases=json.loads((r/f'evaluation/engine073-launch-independent-v{v}/cases.json').read_text())['cases']
 ip=bins/f'seen-inputs-v{v}-successor21.json';ip.write_text(json.dumps([dict(c['input'],id=c['id']) for c in cases]))
 obs=run(seen,ip,f'seen-independent-v{v}-after.jsonl');actual={x['id']:x for x in obs};assert len(actual)==len(cases)==60
 mismatches=[]
 for c in cases:
  x=actual[c['id']]; q=x['effective_v17']['relation_scope']=='requested_relation'; d=x['v30']['disposition']=='relevant'
  if q!=c['expected']['v17_requested_relation'] or d!=c['expected']['v30_relevant']:mismatches.append((c['id'],q,d))
 stats[f'v{v}']={'cases':60,'passed':60-len(mismatches),'mismatches':mismatches}
 assert not mismatches,(v,mismatches)
print(json.dumps({'source_sha':source_sha,'source_file_sha256':source_hash,'historical_complete_equality':len(h),'original_audit':len(a),'known_seen_regression':stats,'provider_calls':0},indent=2))
(out/'probe-summary.json').write_text(json.dumps({'source_sha':source_sha,'source_file_sha256':source_hash,'historical_complete_equality':len(h),'original_audit':len(a),'known_seen_regression':stats,'provider_calls':0,'fresh_independent_acceptance':False,'core_rlib_sha256':hashlib.sha256(core.read_bytes()).hexdigest()},indent=2)+'\n')

root=run(seen,Path('/private/tmp/engine073-typed-compound-probe.json'),'root-compound-after.jsonl')
assert len(root)==4 and all(x['effective_v17']['relation_scope']!='requested_relation' and x['v30']['disposition']!='relevant' for x in root)
print('root compound boundary4/4 withheld')

unknown=run(seen,Path('/private/tmp/engine073-unknown-owner-development-v2/inputs.json'),'parent-unknown-owner-after.jsonl')
assert len(unknown)==4 and all(x['effective_v17']['relation_scope']!='requested_relation' and x['v30']['disposition']!='relevant' for x in unknown)
print('parent unknown-owner variants4/4 withheld')

variant=run(seen,Path('/private/tmp/engine073-probes/literal-owner-development20-input.json'),'literal-owner-after.jsonl')
assert len(variant)==1 and variant[0]['effective_v17']['relation_scope']=='requested_relation' and variant[0]['v30']['disposition']=='relevant'
before=json.loads(Path('/private/tmp/engine073-literal-owner-development20-before.jsonl').read_text());assert variant[0]['v29']==before['v29']
print('quoted owner development variant recovered; historicalv29 observation unchanged')
