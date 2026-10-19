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
    provider=plan['provider'];settings=protocol['providers'][provider]
    m0=json.loads(Path(plan['M0_receipt_file']).read_bytes())
    if m0['required_provider_readiness'][provider]['new_live_run_allowed'] is not True:raise ValueError('selected provider deferred before credentials')
    if plan['model']!=settings['model']:raise ValueError('model swap')
    if plan.get('callback_contract')!='audited-aggregate-callback-v1':raise ValueError('unrecognized callback bounds')
    if len(plan['jobs'])!=72:raise ValueError('complete48 normal +24 helper plan required')
    cost=Decimal(0);normal=helper=0;scheduled=set();cases=plan['case_ids']
    if len(cases)!=24 or len(set(cases))!=24:raise ValueError('unique24 case IDs required')
    for job in plan['jobs']:
        args=job['argv'];budget=job['budget'];track='e2e_normal' if job['contract']=='normal' else 'helper_opt_in'
        if args[0]!=sys.executable or args[1]!=str(HERE/'run_task.py'):raise ValueError('aggregate reserve requires exact audited task wrapper')
        for flag,value in [('--track',track),('--provider',provider),('--model',settings['model'])]:
            if args.count(flag)!=1 or args[args.index(flag)+1]!=value:raise ValueError('callback identity changed')
        for flag in ['--track','--case','--arm','--provider','--model']:
            if any(arg.startswith(flag+'=') for arg in args) or args.count(flag)!=1:raise ValueError('ambiguous callback identity')
        cid=args[args.index('--case')+1];arm=args[args.index('--arm')+1]
        if cid not in cases or arm!=job['arm']:raise ValueError('callback case/arm changed')
        identity=(cid,track,arm)
        if identity in scheduled:raise ValueError('duplicate case arm')
        scheduled.add(identity)
        expected=(24000,4096,12,1024) if track=='e2e_normal' else (4096,384,1,384)
        if (budget['aggregate_input_tokens'],budget['aggregate_output_tokens'],budget['max_generations'],budget['max_output_tokens_per_generation'])!=expected:raise ValueError('unproven callback aggregate cap')
        if budget['max_attempts_per_generation']!=1 or budget['max_input_tokens_per_generation']!=(12512 if track=='e2e_normal' else 4096):raise ValueError('unproven attempt/input bound')
        normal+=track=='e2e_normal';helper+=track=='helper_opt_in'
        cost+=(Decimal(budget['aggregate_input_tokens'])*Decimal(str(settings['USD_per_million_input']))+Decimal(budget['aggregate_output_tokens'])*Decimal(str(settings['USD_per_million_output'])))/Decimal(1000000)
    wanted={(c,'e2e_normal',a) for c in cases for a in ['baseline','candidate']}|{(c,'helper_opt_in','helper') for c in cases}
    if scheduled!=wanted:raise ValueError('incomplete paired case inventory')
    if (normal,helper)!=(48,24):raise ValueError('missing track rows')
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
