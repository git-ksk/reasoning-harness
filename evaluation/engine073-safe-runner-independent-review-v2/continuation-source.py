"""Complete only unexecuted normal-exit boundary; inspect preserved first data."""
import importlib.util,json,os,subprocess,time,hashlib
from pathlib import Path
from datetime import datetime,timezone
ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('frozen_independent_repro',ROOT/'reproduction-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
freeze=json.loads((ROOT/'continuation-freeze.json').read_text())
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(ROOT/'continuation-source.py')==freeze['source_sha256']
assert not (ROOT/'leader-normal-exit-first').exists()
m.write(ROOT/'continuation.started.json',{'ordinal':1,'utc':datetime.now(timezone.utc).isoformat(),'already_executed_cases_rerun':0,'only_new_case':'leader-normal-exit','original_ps_diagnostic_error_preserved':True})
first=[]
for name in ['alias','groq','credential-script-change','credential-static-input-change','leader-exits-on-cancel']:
 out=ROOT/(name+'-first');summary=json.loads((out/'summary.json').read_text());state=json.loads((out/'state.json').read_text());plan=json.loads((out/'plan.json').read_text());checks={}
 if name=='alias':
  checks={'complete_original':summary['complete'],'frozen_provider_model':plan['provider']==state['provider']=='google' and plan['model']==state['model']=='offline-fixture','nested_budget_fixed':plan['jobs'][0]['budget']['max_generations']==2,'reserve160':summary['charged_reservations']==160,'known_usage10':summary['known_total_tokens']==10,'one_started_invocation':state['guard']['started_invocations']==1}
 elif name=='groq':
  checks={'incomplete':not summary['complete'],'fixed_Groq':state['provider']==plan['provider']=='groq','known_prohibition':summary['latch']=='known_daily_quota_exhausted','spawn0':not (out/name/'spawn.json').exists(),'started0':state['guard']['started_invocations']==0,'reserve0':summary['charged_reservations']==0}
 elif name.startswith('credential-'):
  checks={'incomplete':not summary['complete'],'changed_artifact_latch':summary['latch']=='frozen_artifact_changed','spawn0':not (out/name/'spawn.json').exists(),'following_suppressed':summary['rows'][1]['admission']['allowed'] is False,'unknown1_known_subtotal0':summary['unknown_usage_rows']==1 and summary['known_total_tokens']==0,'reserve160_retained':summary['charged_reservations']==160,'reserved_inflight_precedes_observation':json.loads((out/'event-000002.json').read_text())['event']=='reserved_inflight'}
 else:
  d=out/name;pid=int((d/'descendant.pid').read_text());before=(d/'heartbeat.txt').stat().st_size;time.sleep(.2);after=(d/'heartbeat.txt').stat().st_size
  ps=subprocess.run(['ps','-o','stat=','-p',str(pid)],capture_output=True,text=True);status=ps.stdout.strip()
  checks={'incomplete':not summary['complete'],'hardkill_after_grace':summary['rows'][0]['termination']=='sigkill_after_grace','leader_exit1':summary['rows'][0]['returncode']==1,'heartbeat_stopped':before==after,'no_live_descendant':not status or status.startswith('Z'),'following_suppressed':summary['rows'][1]['admission']['allowed'] is False,'unknown1':summary['unknown_usage_rows']==1,'reserve160':summary['charged_reservations']==160}
  first.append({'name':name,'checks':checks,'outcome':'PASS' if all(checks.values()) else 'FAIL','summary':summary,'heartbeat_bytes_before':before,'heartbeat_bytes_after':after,'descendant_state':status});continue
 first.append({'name':name,'checks':checks,'outcome':'PASS' if all(checks.values()) else 'FAIL','summary':summary})
new=m.group_case('leader-normal-exit','normal')
report={'identity':'engine073-safe-runner-independent-review-v2','HEAD':'93a9d5a3dba976f134395409037df7c1a73d7845','outcome':'PASS' if all(c['outcome']=='PASS' for c in first+[new]) else 'FAIL','cases':first+[new],'new_case_count':6,'existing_tests':47,'initial_harness':'First aggregation aborted at sandbox-forbidden ps after5 runner cases completed. Original rc1/stderr retained; none of those cases rerun. This continuation reads their durable first evidence and executes only the previously unexecuted normal-exit case. Callback in-memory observation list from interrupted aggregation was not saved; no independent callback-count claim beyond durable start/spawn markers. Existing47 tests separately verify callback suppression and pre-auth reserves.','original_first_cases_rerun':0,'new_normal_exit_case_runs':1,'provider_calls':0,'network_calls':0,'core_implementation_read':False,'scope':'Bounded cooperative plan/artifact/process-group repairs only, not live acceptance.'}
m.write(ROOT/'report.json',report);print(json.dumps({'outcome':report['outcome'],'cases':[{'name':c['name'],'outcome':c['outcome'],'failed_checks':[k for k,v in c['checks'].items() if not v]} for c in report['cases']]},indent=2))
