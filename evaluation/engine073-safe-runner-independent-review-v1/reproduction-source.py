from pathlib import Path
import tempfile,sys,json,hashlib,os,signal,time,subprocess
repo=Path('/Users/sawadakousuke/.codex/worktrees/engine073-safe-runner/reasoning-harness');rundir=repo/'evaluation/engine07-realworld-runner-v1';sys.path.insert(0,str(rundir));from runner import OperationalRunner,digest
root=Path(tempfile.mkdtemp(prefix='engine073-safe-runner-independent-',dir='/private/tmp'));root.chmod(0o700)
(root/'review-root.txt').write_text(str(repo)+'\nHEAD=ed2c8dea78d27c8d9f3f9e8eff72fcf6e1147362\n')
fixture=root/'pinned_fixture.py';fixture.write_bytes((rundir/'fixture_child.py').read_bytes())
def job(cid,exe=None):
 return {'id':cid,'contract':'normal','arm':'baseline','argv':[sys.executable,'-B',str(exe or fixture),'--mode','success','--marker',str(root/(cid+'-spawns.txt')),'--provider','google','--model','offline-fixture'],'budget':{'max_generations':2,'max_input_tokens_per_generation':20,'max_output_tokens_per_generation':20,'max_attempts_per_generation':2},'timeout_seconds':2,'cancel_grace_seconds':0.1}
def plan(jobs,extra=None):
 arts={sys.executable:digest(sys.executable),str(fixture):digest(fixture)};arts.update(extra or {})
 return {'schema':'engine07-operational-plan-v1','provider':'google','model':'offline-fixture','token_budget':1000,'jobs':jobs,'artifacts':arts}
report={'repository':str(repo),'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),'network_calls':0,'provider_calls':0,'cases':{}}
p=plan([job('pin-race')]);frozen=p['artifacts'][str(fixture)];reads=[]
def late_change(provider):
 reads.append(provider)
 fixture.write_text(fixture.read_text().replace('def usage(tokens=10):','def usage(tokens=11):'))
 return {}
r=OperationalRunner(p,root/'pin-race-first',late_change).run()
report['cases']['post_validation_artifact_change']={'credential_reads':reads,'frozen_hash':frozen,'executed_file_hash':digest(fixture),'mismatch':frozen!=digest(fixture),'complete':r['complete'],'known_total_tokens':r['known_total_tokens'],'spawned':(root/'pin-race-spawns.txt').exists(),'latch':r['latch']}
p=plan([job('plan-alias')]);reads=[];runner=OperationalRunner(p,root/'plan-alias-first',lambda prov:(reads.append(prov) or {}))
p['provider']='groq';p['jobs'][0]['argv'][p['jobs'][0]['argv'].index('--provider')+1]='groq'
r=runner.run();saved=json.loads((root/'plan-alias-first/plan.json').read_text())
report['cases']['mutable_plan_alias']={'frozen_provider':saved['provider'],'observed_provider':json.loads((root/'plan-alias-first/state.json').read_text())['provider'],'credential_reads':reads,'complete':r['complete'],'spawned':(root/'plan-alias-spawns.txt').exists(),'latch':r['latch'],'offline_only':True}
child=root/'descendant_fixture.py';child.write_text("import os,signal,time,json,sys\nfrom pathlib import Path\nchild=os.fork()\nif child==0:\n signal.signal(signal.SIGINT,signal.SIG_IGN)\n while True:\n  Path('heartbeat.txt').write_text(str(time.monotonic()))\n  time.sleep(0.02)\nPath('descendant.pid').write_text(str(child))\ndef done(sig,frame):\n print(json.dumps({'result':{'status':'failed','failure':{'failure_class':'cancelled'},'partial_diagnostics':{'complete':False,'usage_is_lower_bound':True}}}),flush=True)\n raise SystemExit(1)\nsignal.signal(signal.SIGINT,done)\nwhile True: time.sleep(0.02)\n")
j=job('group-cancel',child);j['timeout_seconds']=0.18;j['cancel_grace_seconds']=0.1;p=plan([j],{str(child):digest(child)})
leader=None;desc=None
try:
 r=OperationalRunner(p,root/'group-cancel-first',lambda _:{}).run();d=root/'group-cancel-first/group-cancel';leader=json.loads((d/'spawn.json').read_text())['pid'];desc=int((d/'descendant.pid').read_text());before=(d/'heartbeat.txt').read_text();time.sleep(0.25);after=(d/'heartbeat.txt').read_text()
 report['cases']['leader_exit_leaves_running_descendant']={'leader_returncode':r['rows'][0]['returncode'],'reported_termination':r['rows'][0]['termination'],'grace_seconds':0.1,'descendant_heartbeat_advanced_after_runner_return':before!=after,'descendant_pid':desc,'complete':r['complete'],'latch':r['latch'],'offline_only':True}
finally:
 if leader:
  try:os.killpg(leader,signal.SIGKILL)
  except ProcessLookupError:pass
 if desc:
  try:os.kill(desc,signal.SIGKILL)
  except ProcessLookupError:pass
report['cleanup']='All created descendant test groups were SIGKILLed; no provider sockets/imports used.'
with (root/'report.json').open('x') as f:json.dump(report,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
print(json.dumps({'output_root':str(root),'report':report},indent=2))
