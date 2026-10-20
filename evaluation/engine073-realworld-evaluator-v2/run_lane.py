#!/usr/bin/env python3
"""Audited callback integration for the preserved POSIX durable scheduler.

The inherited implementation stays byte-exact. Only this new entry point uses
the aggregate bounds enforced by audit.rs. It never executes old cohorts.
"""
import argparse,importlib.util,json,os,sys
from decimal import Decimal
from pathlib import Path

HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]
LEGACY=REPO/'evaluation/engine07-realworld-runner-v1/runner.py'
sys.path.insert(0,str(LEGACY.parent))
spec=importlib.util.spec_from_file_location('preserved_scheduler',LEGACY);scheduler=importlib.util.module_from_spec(spec);spec.loader.exec_module(scheduler)
original_reserve=scheduler.reserve_for

def aggregate_reserve(budget):
    original_reserve(budget) # retain inherited strict bound/type checks
    i,o=budget.get('aggregate_input_tokens'),budget.get('aggregate_output_tokens')
    if type(i) is not int or type(o) is not int or i<1 or o<1:raise ValueError('audited aggregate bounds required')
    return i+o

def check_plan(plan,protocol):
    from plan_inputs import inventory
    provider=plan['provider'];settings=protocol['providers'][provider]
    root=Path(plan['corpus_root'])
    if not root.is_absolute() or str(root.resolve())!=str(root):raise ValueError('canonical absolute corpus root required')
    if plan.get('purpose')=='CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE':
        from development import checked_receipt,consumed_corpus,INPUT_M1
        receipt_path=plan['development_receipt_file'];receipt=checked_receipt(receipt_path,provider)
        if plan['M1_sha256']!=INPUT_M1 or plan['M1_sha256']!=receipt['consumed_M1_sha256']:raise ValueError('plan consumed M1 mismatch')
        data=consumed_corpus(root)
    else:
        from corpus import validate,verify_provenance
        if plan.get('purpose') is not None:raise ValueError('unknown evaluation purpose')
        receipt_path=plan['M0_receipt_file'];m0=verify_provenance(root,receipt_path)
        if m0['required_provider_readiness'][provider]['new_live_run_allowed'] is not True:raise ValueError('selected provider deferred before credentials')
        data,_,_=validate(root,plan['M1_sha256'])
    recovery=plan.get('groq_recovery_file') if provider=='groq' else None
    if plan['artifacts']!=inventory(root,receipt_path,recovery):raise ValueError('complete callback input inventory changed')
    if plan['model']!=settings['model']:raise ValueError('model swap')
    if plan.get('callback_contract')!='audited-aggregate-callback-v1':raise ValueError('unrecognized callback bounds')
    if len(plan['jobs'])!=72:raise ValueError('complete48 normal +24 helper plan required')
    cases=[c['id'] for c in data['cases']]
    if len(cases)!=24 or len(set(cases))!=24 or plan['case_ids']!=cases:raise ValueError('frozen24 case order required')
    order=[]
    for index,cid in enumerate(cases):
        order.extend((cid,'e2e_normal',arm) for arm in (['baseline','candidate'] if index%2==0 else ['candidate','baseline']))
        order.append((cid,'helper_opt_in','helper'))
    cost=Decimal(0);reserved=0
    for job,(cid,track,arm) in zip(plan['jobs'],order):
        normal=track=='e2e_normal'
        expected_args=[sys.executable,str(HERE/'run_task.py'),'--provider',provider,'--model',settings['model'],
            '--corpus',str(root/'corpus.json'),'--host-policy',str(root/'host-policy.json'),'--case',cid,
            '--track',track,'--arm',arm,'--bin-dir','/eval/bin']
        if job['argv']!=expected_args:raise ValueError('frozen callback paths identity or order changed')
        if (job['id'],job['contract'],job['arm'])!=(cid+'-'+arm,'normal' if normal else 'helper',arm):raise ValueError('frozen job identity changed')
        if (job.get('timeout_seconds'),job.get('cancel_grace_seconds'))!=(360 if normal else 100,1):raise ValueError('frozen timeout or grace changed')
        budget=job['budget']
        expected={'max_generations':24 if normal else 1,'max_input_tokens_per_generation':12512 if normal else 4096,
            'max_output_tokens_per_generation':1024 if normal else 384,'max_attempts_per_generation':1,
            'aggregate_input_tokens':24000 if normal else 4096,'aggregate_output_tokens':4096 if normal else 384}
        if budget!=expected:raise ValueError('unproven callback aggregate cap')
        reserved+=aggregate_reserve(budget)
        cost+=(Decimal(budget['aggregate_input_tokens'])*Decimal(str(settings['USD_per_million_input']))+Decimal(budget['aggregate_output_tokens'])*Decimal(str(settings['USD_per_million_output'])))/Decimal(1000000)
    if plan['token_budget']!=reserved:raise ValueError('plan token reservation changed')
    if plan['currency_self_cap_USD']!=settings['currency_self_cap_USD']:raise ValueError('plan currency self-cap changed')
    if cost>Decimal(str(settings['currency_self_cap_USD'])):raise ValueError('currency reservations exceed frozen self-cap')
    return str(cost)

class AuditedRunner(scheduler.OperationalRunner):
    def __init__(self,plan,output,protocol):
        self.currency_reservation=check_plan(plan,protocol)
        # Explicit successor integration: the pure reserve formula now uses the
        # aggregate pre-HTTP bounds in the pinned host, not per-generation
        # product bounds. Inherited immutable plan, inflight, accounting, grace,
        # restart refusal and attempt-count checks are unchanged.
        scheduler.reserve_for=aggregate_reserve
        super().__init__(plan,output,lambda provider:{})
        if plan['provider']=='groq':
            recovery_path=Path(plan['groq_recovery_file'])
            recovery=json.loads(recovery_path.read_text())
            allowed=(recovery.get('new_live_run_allowed') is True and recovery.get('other_organization_usage_stopped') is True
                     and recovery.get('observed_TPM',0)>=100000 and recovery.get('daily_reserve_capacity_verified') is True
                     and recovery.get('requested_model')==plan['model'] and recovery.get('provider')=='groq'
                     and recovery.get('previous_state_sha256')==self.groq_state_sha256)
            if allowed and self.guard.latch=='known_daily_quota_exhausted':
                self.guard.latch=None;self.persist('separate_frozen_groq_recovery_verified')
        scheduler.write_new(self.output/'currency-reservation.json',{'USD':self.currency_reservation,'scope':protocol['currency_limit_scope'],'refunded':False})

def main():
    p=argparse.ArgumentParser();p.add_argument('--plan',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--protocol',type=Path,required=True);a=p.parse_args()
    runner=AuditedRunner(json.loads(a.plan.read_text()),a.output,json.loads(a.protocol.read_text()));result=runner.run()
    print(json.dumps({k:v for k,v in result.items() if k!='rows'}));return 0 if result['complete'] else 1
if __name__=='__main__':raise SystemExit(main())
