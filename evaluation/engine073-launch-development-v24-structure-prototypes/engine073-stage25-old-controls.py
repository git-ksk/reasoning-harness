from pathlib import Path
import hashlib,json,subprocess
r=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness');stage=Path('/private/tmp/engine073-stage25-structure02');out=Path('/private/tmp/engine073-stage25-old-controls');out.mkdir(exist_ok=False)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
exe=stage/'probe';identity=json.loads((stage/'identity-before.json').read_text());assert sha(exe)==identity['probe_executable_sha256'];(out/'identity-before.json').write_text(json.dumps(identity,indent=2)+'\n');summary={}
for v in [*range(2,11),12,13,14,15,16,18,19,20,21,22,23]:
 d=r/f'evaluation/engine073-launch-independent-v{v}';inputs=d/'inputs.json';cases=json.loads((d/'cases.json').read_text())['cases'];before={'exe':sha(exe),'inputs':sha(inputs),'cases':sha(d/'cases.json')};(out/f'v{v}-identity-before.json').write_text(json.dumps(before,indent=2)+'\n')
 res=subprocess.run([str(exe),str(inputs)],capture_output=True);(out/f'v{v}-stdout.jsonl').write_bytes(res.stdout);(out/f'v{v}-stderr.txt').write_bytes(res.stderr);assert res.returncode==0 and not res.stderr;rows=[json.loads(x) for x in res.stdout.splitlines()];by={x['id']:x for x in rows};assert len(rows)==len(by)==len(cases)==60
 misses=[]
 for c in cases:
  row=by[c['id']];o={'v17_requested_relation':row['effective_v17']['relation_scope']=='requested_relation','v30_relevant':row['v30']['disposition']=='relevant'}
  if any(c['expected'][a]!=o[a] for a in o):misses.append(c['id'])
 summary[f'v{v}']={'mismatches':misses,'passed':60-len(misses)};assert before=={'exe':sha(exe),'inputs':sha(inputs),'cases':sha(d/'cases.json')};print(v,summary[f'v{v}'],flush=True)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');assert not any(x['mismatches'] for x in summary.values());print('old1200 PASS',flush=True)
