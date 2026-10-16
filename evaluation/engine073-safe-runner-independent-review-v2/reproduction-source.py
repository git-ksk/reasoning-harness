"""Independent repair check; fresh outputs only; this file frozen before running."""
import hashlib,json,os,signal,subprocess,sys,time
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parent
REPO=Path('/Users/sawadakousuke/.codex/worktrees/engine073-safe-runner/reasoning-harness')
sys.path.insert(0,str(REPO/'evaluation/engine07-realworld-runner-v1'))
from runner import OperationalRunner,digest

def write(path,obj):
 with path.open('x') as f:json.dump(obj,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def fixture(name):
 p=ROOT/(name+'-fixture.py')
 with p.open('xb') as f:f.write((REPO/'evaluation/engine07-realworld-runner-v1/fixture_child.py').read_bytes())
 return p
def job(name,script,mode='success',contract='normal'):
 return {'id':name,'arm':'baseline' if contract=='normal' else 'helper','contract':contract,'argv':[sys.executable,'-B',str(script),'--mode',mode,'--marker',str(ROOT/(name+'-spawns.txt')),'--provider','google','--model','offline-fixture'],'budget':{'max_generations':2,'max_input_tokens_per_generation':20,'max_output_tokens_per_generation':20,'max_attempts_per_generation':2},'timeout_seconds':2,'cancel_grace_seconds':.15}
def plan(jobs,paths,provider='google'):
 for j in jobs:j['argv'][j['argv'].index('--provider')+1]=provider
 return {'schema':'engine07-operational-plan-v1','provider':provider,'model':'offline-fixture','token_budget':1000,'jobs':jobs,'artifacts':{str(p):digest(p) for p in [Path(sys.executable),*paths]}}
def credential_probe(output,reads,extra=None):
 def cb(provider):
  s=json.loads((output/'state.json').read_text())
  reads.append({'provider':provider,'inflight':s['guard']['inflight'],'reserve':s['current_reserve'],'charged':s['charged_reservations']})
  if extra:extra()
  return {}
 return cb
def result_case(name,result,reads,checks,extra=None):
 return {'name':name,'checks':checks,'outcome':'PASS' if all(checks.values()) else 'FAIL','credential_observations':reads,'result':result,**(extra or {})}
def group_case(name,mode):
 script=ROOT/'descendant_fixture.py';ordinary=fixture(name+'-follow')
 j={'id':name,'arm':'baseline','contract':'normal','argv':[sys.executable,'-B',str(script),'--mode',mode,'--provider','google','--model','offline-fixture'],'budget':{'max_generations':2,'max_input_tokens_per_generation':20,'max_output_tokens_per_generation':20,'max_attempts_per_generation':2},'timeout_seconds':.4,'cancel_grace_seconds':.15}
 follow=job(name+'-follow',ordinary,contract='helper');out=ROOT/(name+'-first');reads=[];leader=None;cleanup=[]
 try:
  result=OperationalRunner(plan([j,follow],[script,ordinary]),out,credential_probe(out,reads)).run()
  d=out/name;leader=json.loads((d/'spawn.json').read_text())['pid'];desc=int((d/'descendant.pid').read_text())
  before=(d/'heartbeat.txt').stat().st_size;time.sleep(.2);after=(d/'heartbeat.txt').stat().st_size
  ps=subprocess.run(['ps','-o','stat=','-p',str(desc)],capture_output=True,text=True)
  status=ps.stdout.strip();not_live=not status or status.startswith('Z')
  row=result['rows'][0]
  checks={'incomplete':result['complete'] is False,'hardkill_after_group_grace':row['termination']=='sigkill_after_grace','heartbeat_stopped':before==after,'descendant_not_live':not_live,'following_row_suppressed':result['rows'][1]['admission']['allowed'] is False,'credential_calls1':len(reads)==1,'unknown_usage_retained':result['unknown_usage_rows']==1 and row['outcome']['tokens'] is None,'reserve_not_refunded':result['charged_reservations']==160,'original_leader_exit_retained':row['returncode']==(1 if mode=='cancel' else 0)}
  return result_case(name,result,reads,checks,{'heartbeat_bytes_at_return':before,'heartbeat_bytes_200ms_later':after,'descendant_state':status,'leader_pid':leader,'descendant_pid':desc,'cleanup':cleanup})
 finally:
  # Same new-session group only. Signals do not target any external process.
  if leader:
   try:os.killpg(leader,signal.SIGKILL);cleanup.append('own_group_SIGKILL')
   except ProcessLookupError:cleanup.append('own_group_absent')
   except PermissionError:cleanup.append('group_only_zombie_or_permission_state; heartbeat/status observation retained')

def main():
 freeze=json.loads((ROOT/'reproduction-freeze.json').read_text())
 for n,h in freeze['files'].items():assert digest(ROOT/n)==h
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()==freeze['HEAD']
 code=json.loads((ROOT/'reviewed-source-hashes.json').read_text())
 for n,h in code['files'].items():assert digest(REPO/n)==h
 write(ROOT/'reproduction.started.json',{'utc':datetime.now(timezone.utc).isoformat(),'HEAD':freeze['HEAD'],'ordinal':1,'source_sha256':digest(ROOT/'reproduction-source.py'),'network_calls':0,'provider_calls':0})
 cases=[]
 f=fixture('alias');p=plan([job('alias',f)],[f]);reads=[];out=ROOT/'alias-first';r=OperationalRunner(p,out,credential_probe(out,reads))
 p['provider']='groq';p['model']='mutated-model';p['token_budget']=999999;p['jobs'][0]['argv'][p['jobs'][0]['argv'].index('--provider')+1]='groq';p['jobs'][0]['argv'][-1]='mutated-model';p['jobs'][0]['budget']['max_generations']=999;p['artifacts'].clear()
 exposed=r.plan;exposed['provider']='groq';exposed['model']='getter-mutation';exposed['jobs'][0]['argv'][-1]='getter-mutation';exposed['jobs'][0]['budget']['max_generations']=777;exposed['jobs'].clear()
 setter_rejected=False
 try:r.plan=p
 except AttributeError:setter_rejected=True
 result=r.run();saved=json.loads((out/'plan.json').read_text());state=json.loads((out/'state.json').read_text())
 checks={'complete_original':result['complete'] is True,'provider_and_model_fixed':saved['provider']==state['provider']=='google' and saved['model']==state['model']=='offline-fixture','auth_original_provider':len(reads)==1 and reads[0]['provider']=='google','reserve_fixed160':result['charged_reservations']==160,'getter_nested_copy':len(r.plan['jobs'])==1 and r.plan['jobs'][0]['budget']['max_generations']==2,'setter_rejected':setter_rejected,'no_Groq_credentials':all(x['provider']!='groq' for x in reads)}
 cases.append(result_case('caller_and_getter_nested_plan_mutation',result,reads,checks))
 f=fixture('groq');p=plan([job('groq',f)],[f],provider='groq');out=ROOT/'groq-first';reads=[];r=OperationalRunner(p,out,credential_probe(out,reads));p['provider']='google';p['jobs'][0]['argv'][p['jobs'][0]['argv'].index('--provider')+1]='google';r.plan['jobs'].clear();result=r.run()
 cases.append(result_case('known_Groq_cannot_be_cleared_by_mutation',result,reads,{'auth0':not reads,'spawn0':not (ROOT/'groq-spawns.txt').exists(),'incomplete':not result['complete'],'known_prohibition':result['latch']=='known_daily_quota_exhausted','saved_provider_Groq':json.loads((out/'plan.json').read_text())['provider']=='groq'}))
 for name,static_input in [('credential-script-change',False),('credential-static-input-change',True)]:
  f=fixture(name);follow=job(name+'-follow',f,contract='helper');j=job(name,f);paths=[f];target=f
  if static_input:
   target=ROOT/(name+'-input.json');target.write_text('{"fixture":1}\n');paths.append(target)
  frozen_hash=digest(target);out=ROOT/(name+'-first');reads=[]
  def mutate():
   if static_input:target.write_text('{"fixture":2}\n')
   else:target.write_text(target.read_text().replace('def usage(tokens=10):','def usage(tokens=11):'))
  result=OperationalRunner(plan([j,follow],paths),out,credential_probe(out,reads,mutate)).run()
  checks={'auth1':len(reads)==1,'reserve_and_inflight_before_auth':reads[0]['inflight'] is True and reads[0]['charged']==reads[0]['reserve']==160,'hash_changed':digest(target)!=frozen_hash,'spawn0':not (ROOT/(name+'-spawns.txt')).exists() and not (out/name/'spawn.json').exists(),'incomplete':result['complete'] is False,'artifact_latch':result['latch']=='frozen_artifact_changed','following_suppressed':result['rows'][1]['admission']['allowed'] is False,'unknown_not_zero':result['unknown_usage_rows']==1 and result['known_total_tokens']==0,'reserve_not_refunded':result['charged_reservations']==160}
  cases.append(result_case(name,result,reads,checks,{'frozen_artifact_sha256':frozen_hash,'changed_artifact_sha256':digest(target)}))
 cases.append(group_case('leader-exits-on-cancel','cancel'))
 cases.append(group_case('leader-normal-exit','normal'))
 report={'identity':'engine073-safe-runner-independent-review-v2','HEAD':freeze['HEAD'],'cases':cases,'outcome':'PASS' if all(c['outcome']=='PASS' for c in cases) else 'FAIL','case_count':len(cases),'checks':sum(len(c['checks']) for c in cases),'provider_calls':0,'network_calls':0,'Engine_core_read':False,'existing_first_artifacts_modified':0,'scope':'Bounded cooperative artifact/plan/group-containment repairs, not live acceptance.'}
 write(ROOT/'report.json',report);print(json.dumps(report,indent=2))
if __name__=='__main__':main()
