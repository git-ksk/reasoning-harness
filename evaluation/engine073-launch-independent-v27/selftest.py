#!/usr/bin/env python3
"""Pre-author pure evaluator fixtures only. Never run candidate or real cohort."""
import ast,sys,re
from evaluator_contract import *
def main():
 for n in ['author_cases.py','authoring-start.json','cases.json','inputs.json','manifest.json','execution.started.json','first-invocation.started.json','first-score.started.json','selftest.json']:
  if (ROOT/n).exists():raise ValueError('pre_author_exclusive_fixture_barrier:'+n)
 preserved()
 for p in ROOT.glob('*.py'):ast.parse(p.read_text())
 command=[sys.executable,'-B','-m','unittest','test_evaluator']
 started=utc();r=subprocess.run(command,cwd=ROOT,env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1'),capture_output=True)
 for n,b in [('selftest.stdout.txt',r.stdout),('selftest.stderr.txt',r.stderr)]:
  with (ROOT/n).open('xb') as f:f.write(b);f.flush();os.fsync(f.fileno())
 match=re.search(r'Ran (\d+) tests',r.stderr.decode());count=int(match[1]) if match else None
 result={'identity':'engine073-launch-independent-v27-pre-author','kind':'Pure constructed output/provenance/refusal/hash/process-envelope tests; not case authorship or candidate observations.','started_utc':started,'finished_utc':utc(),'command':command,'returncode':r.returncode,'tests':count,'stdout_sha256':sha(ROOT/'selftest.stdout.txt'),'stderr_sha256':sha(ROOT/'selftest.stderr.txt'),'tested_files':{n:sha(ROOT/n) for n in ['score.py','execute_first.py','test_evaluator.py','evaluator_contract.py','source_gate_helper.py','source_gate_policy.json']},'actual_Engine_invocations':0,'real_cohort_scorer_processes':0,'authored_cases':0,'provider_model_network_Oracle_calls':0}
 write_new(ROOT/'selftest.json',result);print(json.dumps(result,indent=2))
 if r.returncode!=0 or count!=59:raise ValueError('pure_fixture_failure')
if __name__=='__main__':main()
