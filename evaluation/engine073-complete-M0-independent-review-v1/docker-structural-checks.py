"""No external network/credentials. Fixed OCI content, supplied controls, owned-container PID1 check."""
from pathlib import Path
import hashlib,json,os,subprocess,time,traceback
OUT=Path(__file__).resolve().parent
IMAGE='sha256:a71348d080761005e09b55c404489a9dfb1ca0c5d1f0e5ef5a4a9f446574eb82'
PREFIX='engine073-independent-m0-v1-'
RESULTS={}
def save(name,value):
 data=value if isinstance(value,bytes) else (json.dumps(value,indent=2)+'\n').encode()
 with (OUT/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
def run(label,args,timeout=90):
 command=['docker',*args]
 p=subprocess.run(command,capture_output=True,timeout=timeout)
 save(label+'.stdout',p.stdout);save(label+'.stderr',p.stderr)
 save(label+'.invocation.json',{'argv':command,'returncode':p.returncode,'network':'none for every run','credentials_forwarded':False})
 return p
BASE=['run','--rm','--pull=never','--platform','linux/amd64','--read-only','--network','none','--tmpfs','/tmp']
try:
 r=run('runtime-structural24',BASE+['--name',PREFIX+'tests',IMAGE,'python3','-B','-m','unittest','discover','-s','/repo/evaluation/engine073-realworld-evaluator-v1','-p','test_*.py','-v'])
 RESULTS['structural24']={'returncode':r.returncode,'pass':r.returncode==0};assert r.returncode==0
 r=run('runtime-loopback18',BASE+['--name',PREFIX+'loopback','-v',str(OUT)+':/review',IMAGE,'python3','-B','/repo/evaluation/engine073-realworld-evaluator-v1/preflight.py','--bin-dir','/eval/bin','--output','/review/independent-loopback18'],timeout=120)
 RESULTS['loopback18']={'returncode':r.returncode,'pass':r.returncode==0};assert r.returncode==0
 assert json.loads((OUT/'independent-loopback18/summary.json').read_text())['all_pass'] is True
 setup=r"""import hashlib,json,os,sys
from pathlib import Path
out=Path('/review');runner='/repo/evaluation/engine07-realworld-runner-v1/runner.py';fixture='/repo/evaluation/engine07-realworld-runner-v1/fixture_child.py'
plan={'schema':'engine07-operational-plan-v1','provider':'mistral','model':'fixture-only','token_budget':20,'artifacts':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in [sys.executable,fixture]},'jobs':[{'id':'pid1structural','arm':'helper','contract':'helper','argv':[sys.executable,fixture,'--mode','descendant-on-cancel','--contract','helper','--marker','{RUN_DIR}/marker','--provider','mistral','--model','fixture-only'],'budget':{'max_generations':1,'max_input_tokens_per_generation':4,'max_output_tokens_per_generation':6,'max_attempts_per_generation':1},'timeout_seconds':120,'cancel_grace_seconds':0.15}]}
with (out/'pid1-plan.json').open('x') as f:json.dump(plan,f);f.flush();os.fsync(f.fileno())
os.execv(sys.executable,[sys.executable,'-B',runner,'--plan','/review/pid1-plan.json','--output','/review/pid1-run'])
"""
 save('pid1-setup.py',setup.encode())
 name=PREFIX+'pid1kill'
 r=run('pid1-start',['run','-d','--pull=never','--platform','linux/amd64','--read-only','--network','none','--tmpfs','/tmp','--name',name,'-v',str(OUT)+':/review',IMAGE,'python3','-B','/review/pid1-setup.py'])
 assert r.returncode==0
 container_id=r.stdout.decode().strip();assert len(container_id)==64
 try:
  heartbeat=OUT/'pid1-run/pid1structural/heartbeat.txt'
  deadline=time.monotonic()+30
  while (not heartbeat.is_file() or heartbeat.stat().st_size<18) and time.monotonic()<deadline:time.sleep(.05)
  assert heartbeat.is_file() and heartbeat.stat().st_size>=18
  r=run('pid1-command',['exec',container_id,'python3','-c','from pathlib import Path;print(Path("/proc/1/cmdline").read_bytes().decode().replace("\\x00"," "))'])
  assert r.returncode==0 and b'runner.py' in r.stdout
  state=json.loads((OUT/'pid1-run/state.json').read_text());assert state['current_reserve']==10 and state['guard']['inflight'] is True
  save('pid1-state-before-kill.json',state)
  run('pid1-top-before-kill',['top',container_id,'-eo','pid,ppid,sid,args'])
  r=run('pid1-kill',['kill','--signal=KILL',container_id]);assert r.returncode==0
  r=run('pid1-wait',['wait',container_id]);assert r.returncode==0 and r.stdout.strip()==b'137'
  size1=heartbeat.stat().st_size;time.sleep(.4);size2=heartbeat.stat().st_size;time.sleep(.4);size3=heartbeat.stat().st_size
  assert size1==size2==size3
  r=run('pid1-inspect-stopped',['inspect',container_id]);inspect=json.loads(r.stdout)[0]
  assert inspect['State']['Running'] is False and inspect['State']['ExitCode']==137
  run('pid1-top-after-kill',['top',container_id,'-eo','pid,ppid,sid,args'])
  assert not (OUT/'pid1-run/summary.json').exists()
  save('pid1-postkill-check.json',{'pass':True,'container_exitcode':137,'container_running':False,'heartbeat_bytes_samples':[size1,size2,size3],'inflight_reservation_preserved':True,'summary_missing_not_success':True,'no_external_requests':True})
 finally:
  run('pid1-owned-container-cleanup',['rm','-f',container_id])
 r=run('pid1-restart-refusal',BASE+['--name',PREFIX+'restart','-v',str(OUT)+':/review',IMAGE,'python3','-B','/repo/evaluation/engine07-realworld-runner-v1/runner.py','--plan','/review/pid1-plan.json','--output','/review/pid1-run'])
 assert r.returncode!=0 and b'FileExistsError' in r.stderr
 RESULTS['PID1_SIGKILL']={'pass':True,'heartbeat_stopped':True,'child_and_descendant_contained_by_container_PID_namespace':True,'inflight_state_preserved':True,'restart_refused':True}
except BaseException as e:
 RESULTS['failure']={'type':type(e).__name__,'traceback':traceback.format_exc()}
 raise
finally:
 save('docker-structural-result.json',RESULTS)
print(json.dumps(RESULTS,indent=2))
