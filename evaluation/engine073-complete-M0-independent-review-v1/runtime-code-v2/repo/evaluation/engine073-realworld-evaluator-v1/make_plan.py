#!/usr/bin/env python3
"""Create the complete first-only provider lane before credential access."""
import argparse,hashlib,json,sys
from pathlib import Path
from corpus import validate,verify_provenance

HERE=Path(__file__).resolve().parent;REPO=HERE.parents[1]
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--M0-receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--M1-sha256',required=True);p.add_argument('--provider',required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--groq-recovery-file',type=Path);a=p.parse_args()
    m0=verify_provenance(a.corpus_dir,a.M0_receipt)
    if m0['required_provider_readiness'][a.provider]['new_live_run_allowed'] is not True:raise ValueError('selected provider deferred before credentials')
    data,_,_=validate(a.corpus_dir,a.M1_sha256);protocol=json.loads((HERE/'protocol.json').read_text());settings=protocol['providers'][a.provider]
    root=a.corpus_dir.resolve();bins=Path('/eval/bin');jobs=[]
    for index,c in enumerate(data['cases']):
        arms=['baseline','candidate'] if index%2==0 else ['candidate','baseline']
        for arm in arms:
            jobs.append({'id':c['id']+'-'+arm,'arm':arm,'contract':'normal','timeout_seconds':180,'cancel_grace_seconds':1,
               'budget':{'max_generations':12,'max_input_tokens_per_generation':12512,'max_output_tokens_per_generation':1024,'max_attempts_per_generation':1,'aggregate_input_tokens':24000,'aggregate_output_tokens':4096},
               'argv':[sys.executable,str(HERE/'run_task.py'),'--provider',a.provider,'--model',settings['model'],'--corpus',str(root/'corpus.json'),'--host-policy',str(root/'host-policy.json'),'--case',c['id'],'--track','e2e_normal','--arm',arm,'--bin-dir',str(bins)]})
        jobs.append({'id':c['id']+'-helper','arm':'helper','contract':'helper','timeout_seconds':100,'cancel_grace_seconds':1,
          'budget':{'max_generations':1,'max_input_tokens_per_generation':4096,'max_output_tokens_per_generation':384,'max_attempts_per_generation':1,'aggregate_input_tokens':4096,'aggregate_output_tokens':384},
          'argv':[sys.executable,str(HERE/'run_task.py'),'--provider',a.provider,'--model',settings['model'],'--corpus',str(root/'corpus.json'),'--host-policy',str(root/'host-policy.json'),'--case',c['id'],'--track','helper_opt_in','--arm','helper','--bin-dir',str(bins)]})
    # Image digest fixes the Python runtime, standard library, dynamic libraries,
    # binaries and scripts. This concrete list independently rechecks every
    # mutable mounted corpus/static executable input before each spawn.
    paths=[p for folder in [root,HERE,REPO/'evaluation/engine07-realworld-runner-v1',REPO/'evaluation/engine07-realworld-operations-v1',bins] for p in folder.rglob('*') if p.is_file()]
    paths.extend([Path(sys.executable),a.M0_receipt.resolve()]);artifacts={str(p):sha(p) for p in sorted(set(paths))}
    plan={'schema':'engine07-operational-plan-v1','callback_contract':'audited-aggregate-callback-v1','provider':a.provider,'model':settings['model'],
          'token_budget':sum(j['budget']['aggregate_input_tokens']+j['budget']['aggregate_output_tokens'] for j in jobs),
          'M0_receipt_file':str(a.M0_receipt.resolve()),'artifacts':artifacts,'jobs':jobs,'case_ids':[c['id'] for c in data['cases']],'M1_sha256':a.M1_sha256,'currency_self_cap_USD':settings['currency_self_cap_USD']}
    if a.provider=='groq':
        if a.groq_recovery_file is None:raise ValueError('separate Groq recovery record required')
        plan['groq_recovery_file']=str(a.groq_recovery_file.resolve());plan['artifacts'][plan['groq_recovery_file']]=sha(a.groq_recovery_file)
    with a.output.open('x') as f:json.dump(plan,f,indent=2);f.write('\n')
    print(json.dumps({'planned_jobs':len(jobs),'token_reservation':plan['token_budget'],'auth_or_model_calls':0}))
if __name__=='__main__':main()
