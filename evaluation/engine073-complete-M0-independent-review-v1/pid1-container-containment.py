"""Fixed-image PID1 SIGKILL boundary. Only self-created container, no sockets/providers."""
from pathlib import Path
import json,os,subprocess,time,tarfile,traceback
OUT=Path(__file__).resolve().parent
IMAGE='sha256:a71348d080761005e09b55c404489a9dfb1ca0c5d1f0e5ef5a4a9f446574eb82'
NAME='engine073-independent-m0-v1-pid1kill'
MOUNT=str(OUT)+':/review'
result={}
def save(name,data):
 data=data if isinstance(data,bytes) else (json.dumps(data,indent=2)+'\n').encode()
 with (OUT/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
def run(label,args):
 cmd=['docker',*args];r=subprocess.run(cmd,capture_output=True,timeout=45)
 save(label+'.stdout',r.stdout);save(label+'.stderr',r.stderr);save(label+'.invocation.json',{'argv':cmd,'returncode':r.returncode,'no_external_network':True,'credentials_forwarded':False})
 return r
setup=r"""import hashlib,json,os,sys
from pathlib import Path
out=Path('/review');runner='/repo/evaluation/engine07-realworld-runner-v1/runner.py';fixture='/repo/evaluation/engine07-realworld-runner-v1/fixture_child.py'
plan={'schema':'engine07-operational-plan-v1','provider':'mistral','model':'fixture-only','token_budget':20,'artifacts':{p:hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in [sys.executable,fixture]},'jobs':[{'id':'pid1structural','arm':'helper','contract':'helper','argv':[sys.executable,fixture,'--mode','descendant-on-cancel','--contract','helper','--marker','{RUN_DIR}/marker','--provider','mistral','--model','fixture-only'],'budget':{'max_generations':1,'max_input_tokens_per_generation':4,'max_output_tokens_per_generation':6,'max_attempts_per_generation':1},'timeout_seconds':120,'cancel_grace_seconds':0.15}]}
with (out/'pid1-plan.json').open('x') as f:json.dump(plan,f);f.flush();os.fsync(f.fileno())
os.execv(sys.executable,[sys.executable,'-B',runner,'--plan','/review/pid1-plan.json','--output','/review/pid1-run'])
"""
save('pid1-inline-setup.py',setup.encode())
container_id=None
try:
 r=run('pid1-inline-start',['run','-d','--pull=never','--platform','linux/amd64','--read-only','--network','none','--tmpfs','/tmp','--name',NAME,'-v',MOUNT,IMAGE,'python3','-B','-c',setup]);assert r.returncode==0
 container_id=r.stdout.decode().strip();assert len(container_id)==64
 probe='import json;from pathlib import Path;p=Path("/review/pid1-run/pid1structural/heartbeat.txt");print(json.dumps({"heartbeat_bytes":p.stat().st_size if p.exists() else 0,"pid1":Path("/proc/1/cmdline").read_bytes().decode().replace(chr(0)," ")}))'
 seen=None
 for n in range(15):
  r=run('pid1-inline-heartbeat-'+str(n),['exec',container_id,'python3','-c',probe]);assert r.returncode==0
  seen=json.loads(r.stdout)
  if seen['heartbeat_bytes']>=18:break
  time.sleep(.1)
 assert seen and seen['heartbeat_bytes']>=18 and 'runner.py' in seen['pid1']
 r=run('pid1-inline-state-before-kill',['exec',container_id,'python3','-c','from pathlib import Path;print(Path("/review/pid1-run/state.json").read_text())']);state=json.loads(r.stdout)
 assert state['current_reserve']==10 and state['guard']['inflight'] is True
 run('pid1-inline-top-before-kill',['top',container_id,'-eo','pid,ppid,sid,args'])
 r=run('pid1-inline-kill',['kill','--signal=KILL',container_id]);assert r.returncode==0
 r=run('pid1-inline-wait',['wait',container_id]);assert r.returncode==0 and r.stdout.strip()==b'137'
 r=run('pid1-inline-inspect-stopped',['inspect',container_id]);inspect=json.loads(r.stdout)[0]
 assert inspect['State']['Running'] is False and inspect['State']['ExitCode']==137
 run('pid1-inline-top-after-kill',['top',container_id,'-eo','pid,ppid,sid,args'])
 # Inspect persisted heartbeat from a distinct read-only helper, after PID1 death.
 check=r'''import json,time;from pathlib import Path;p=Path('/review/pid1-run/pid1structural/heartbeat.txt');sizes=[p.stat().st_size];time.sleep(.4);sizes.append(p.stat().st_size);time.sleep(.4);sizes.append(p.stat().st_size);print(json.dumps({'heartbeat_bytes_samples':sizes,'summary_exists':Path('/review/pid1-run/summary.json').exists()}))'''
 r=run('pid1-inline-postkill-helper',['run','--rm','--pull=never','--platform','linux/amd64','--read-only','--network','none','--name',NAME+'-verify','-v',MOUNT+':ro',IMAGE,'python3','-c',check]);assert r.returncode==0
 post=json.loads(r.stdout);assert len(set(post['heartbeat_bytes_samples']))==1 and post['summary_exists'] is False
 r=run('pid1-inline-restart-refusal',['run','--rm','--pull=never','--platform','linux/amd64','--read-only','--network','none','--tmpfs','/tmp','--name',NAME+'-restart','-v',MOUNT,IMAGE,'python3','-B','/repo/evaluation/engine07-realworld-runner-v1/runner.py','--plan','/review/pid1-plan.json','--output','/review/pid1-run'])
 assert r.returncode!=0 and b'FileExistsError' in r.stderr
 recover="import tarfile,sys;t=tarfile.open(fileobj=sys.stdout.buffer,mode='w|');t.add('/review/pid1-run',arcname='pid1-run');t.add('/review/pid1-plan.json',arcname='pid1-plan.json');t.close()"
 r=run('pid1-inline-records-tar',['run','--rm','--pull=never','--platform','linux/amd64','--read-only','--network','none','--name',NAME+'-recover','-v',MOUNT+':ro',IMAGE,'python3','-c',recover]);assert r.returncode==0
 with tarfile.open(fileobj=__import__('io').BytesIO(r.stdout)) as t:t.extractall(OUT,filter='data')
 result={'pass':True,'actual_PID1':seen['pid1'],'container_exitcode':137,'container_running':False,'heartbeat_bytes_samples':post['heartbeat_bytes_samples'],'leader_and_SIGINT_ignoring_descendant_stopped':True,'current_reserve_preserved':10,'guard_inflight_preserved':True,'summary_missing_not_success':True,'restart_refused':True,'network_provider_model_accuracy_calls':0}
except BaseException as e:
 result={'pass':False,'type':type(e).__name__,'traceback':traceback.format_exc(),'network_provider_model_accuracy_calls':0};raise
finally:
 if container_id:run('pid1-inline-owned-cleanup',['rm','-f',container_id])
 save('pid1-containment-result.json',result)
print(json.dumps(result,indent=2))
