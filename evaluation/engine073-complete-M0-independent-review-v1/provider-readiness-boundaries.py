"""Pure metadata/arithmetic boundaries; no accuracy authored or Engine/provider execution."""
from pathlib import Path
from decimal import Decimal
import importlib.util,json,sys,os
OUT=Path(__file__).resolve().parent
REPO=OUT/'runtime-code-v2/repo';HERE=REPO/'evaluation/engine073-realworld-evaluator-v1'
sys.path.insert(0,str(HERE))
import run_lane,scorer,verify_M0
protocol=json.loads((HERE/'protocol.json').read_text());results={}
def saved(name,value):
 with (OUT/name).open('x') as f:json.dump(value,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def refused(name,fn,message):
 try:fn()
 except ValueError as e:assert message in str(e);results[name]={'pass':True,'reason':str(e)}
 else:raise AssertionError(name)
stub=OUT/'pure-readiness-metadata.json';saved(stub.name,{'scope':'structural metadata only, not M0 approval','required_provider_readiness':{'groq':{'new_live_run_allowed':False},'mistral':{'new_live_run_allowed':False},'google':{'new_live_run_allowed':True}}})
refused('Groq_deferred_before_other_plan_fields',lambda:run_lane.check_plan({'provider':'groq','M0_receipt_file':str(stub)},protocol),'selected provider deferred before credentials')
refused('cannot_borrow_other_provider_readiness',lambda:run_lane.check_plan({'provider':'mistral','M0_receipt_file':str(stub)},protocol),'selected provider deferred before credentials')
refused('selected_allowed_provider_still_rejects_wrong_model',lambda:run_lane.check_plan({'provider':'google','M0_receipt_file':str(stub),'model':'wrong-structural-model'},protocol),'model swap')
old_argv=sys.argv
try:
 sys.argv=['verify_M0','--provider','mistral','--receipt',str(OUT/'M0-receipt.reviewed-draft.json'),'--image-archive','/not-accessed-before-state-check','--actual-image-id','not-accessed','--repo',str(REPO)]
 refused('draft_cannot_be_used_as_complete_M0',verify_M0.main,'M0_not_complete')
finally:sys.argv=old_argv
# Reuse the frozen arithmetic's row schema only: no question/source/gold content.
planned=[{'id':'structural-only-'+str(n),'row_id':'structural-only-'+track+'-'+arm+'-'+str(n),'provider':'groq','track':track,'arm':arm,'expected':'positive' if n<12 else 'negative','required_groups_G':1,'family':'pure-structural-only'} for track in protocol['tracks'] for arm in ['baseline','candidate'] for n in range(24)]
report=scorer.summarize([],planned,protocol,scorer.frozen_dependencies(REPO))
assert len(planned)==96 and report['semantic_gate_pass'] is False
for track in protocol['tracks']:
 p=report['provider_tracks']['groq/'+track];q=p['case_quality']
 assert p['actual_answer_f1'] is None and q['incomplete']==24 and all(q[x]==0 for x in ['tp','fp','tn','fn'])
results['deferred96_rows_never_TN_or_actual_F1']={'pass':True,'planned_observations':96,'actual_answer_f1':None,'complete_rows':0,'semantic_TP_FP_TN_FN':{'tp':0,'fp':0,'tn':0,'fn':0},'semantic_gate_pass':False}
i=48*24000+24*4096;o=48*4096+24*384
prices={}
for provider,setting in protocol['providers'].items():
 cost=(Decimal(i)*Decimal(str(setting['USD_per_million_input']))+Decimal(o)*Decimal(str(setting['USD_per_million_output'])))/Decimal(1000000)
 assert cost<=Decimal(str(setting['currency_self_cap_USD']))
 prices[provider]={'configured_worst_local_USD':str(cost),'configured_cap_USD':setting['currency_self_cap_USD'],'invoice_guarantee':False}
saved('provider-readiness-boundaries.json',{'scope':'pure frozen code metadata/constructed arithmetic fixtures only','checks':results,'all_pass':True,'configured_cost_checks':prices,'accuracy_questions_or_actual_gold_authored':0,'Engine_actual_cohort_scorer_provider_model_network_Oracle_calls':0})
print(json.dumps({'pure_boundary_checks':len(results),'pass':True,'configured_cost_checks':prices},indent=2))
