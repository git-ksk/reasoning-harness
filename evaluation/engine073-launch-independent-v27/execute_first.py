#!/usr/bin/env python3
"""Future first-only driver. Not invoked during pre-author evaluator preparation."""
import sys,time
from evaluator_contract import *
def execution_passed(summary,recorder_returncode,scorer_returncode,scorer_stderr_bytes):
 return summary.get('decision')=='GO' and type(recorder_returncode) is int and type(scorer_returncode) is int and type(scorer_stderr_bytes) is int and recorder_returncode==0 and scorer_returncode==0 and scorer_stderr_bytes==0

def main():
 evaluator_integrity();manifest=load(ROOT/'manifest.json')
 if any((ROOT/n).exists() for n in ['execution.started.json','first-invocation.started.json','first-score.started.json','first-results.json','first-summary.json']):raise ValueError('canonical_restart_refused')
 if any(sha(ROOT/n)!=h for n,h in manifest['frozen_files'].items()) or not checked_artifacts(manifest['checked_artifacts']):raise ValueError('complete_freeze_changed')
 write_new(ROOT/'execution.started.json',{'identity':'engine073-launch-independent-v27','utc':utc(),'manifest_sha256':sha(ROOT/'manifest.json'),'Engine_processes_planned':1,'scorer_processes_planned':1,'retry':0})
 env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
 with (ROOT/'first-run.stdout.txt').open('xb') as out,(ROOT/'first-run.stderr.txt').open('xb') as err:
  first=subprocess.run([sys.executable,str(ROOT/'first-run.py')],cwd=ROOT,env=env,stdout=out,stderr=err,check=False)
  for f in [out,err]:f.flush();os.fsync(f.fileno())
 for n in ['first.stdout.jsonl','first.stderr.txt','first-invocation.started.json','first-invocation.json']:
  if (ROOT/n).exists():os.chmod(ROOT/n,0o444)
 # The new score.py owns its exclusive score-start marker and calls helper once.
 started=time.monotonic()
 with (ROOT/'first-score.stdout.txt').open('xb') as out,(ROOT/'first-score.stderr.txt').open('xb') as err:
  score=subprocess.run([sys.executable,str(ROOT/'score.py')],cwd=ROOT,env=env,stdout=out,stderr=err,check=False)
  for f in [out,err]:f.flush();os.fsync(f.fileno())
 write_new(ROOT/'first-score.invocation.json',{'identity':'engine073-launch-independent-v27','utc':utc(),'returncode':score.returncode,'elapsed_seconds':time.monotonic()-started,'command':[sys.executable,str(ROOT/'score.py')],'scorer_invocations':1,'Engine_additional_invocations':0,'stdout_sha256':sha(ROOT/'first-score.stdout.txt'),'stderr_sha256':sha(ROOT/'first-score.stderr.txt')})
 for n in ['first-results.json','first-summary.json','first-score.started.json','first-score.invocation.json','first-score.stdout.txt','first-score.stderr.txt']:
  if (ROOT/n).exists():os.chmod(ROOT/n,0o444)
 summary=load(ROOT/'first-summary.json') if (ROOT/'first-summary.json').exists() else {'outcome':'FAIL','decision':'NO_GO','blockers':['scorer_failed_no_retry']}
 passed=execution_passed(summary,first.returncode,score.returncode,(ROOT/'first-score.stderr.txt').stat().st_size)
 print(json.dumps({'execution_outcome':'PASS' if passed else 'FAIL','first_recorder_returncode':first.returncode,'scorer_returncode':score.returncode,'summary':summary},indent=2))
 return 0 if passed else 1
if __name__=='__main__':raise SystemExit(main())
