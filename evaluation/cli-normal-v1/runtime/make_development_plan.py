#!/usr/bin/env python3
"""Freeze an ordinary-only seen24 diagnostic plan; this is not fresh acceptance."""
import argparse, json, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(1,str(HERE.parent))
from plan_inputs import inventory
from development import checked_receipt, consumed_corpus, INPUT_M1, PURPOSE

def main():
    p=argparse.ArgumentParser();p.add_argument('--receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--provider',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    receipt=checked_receipt(a.receipt,a.provider);data=consumed_corpus(a.corpus_dir)
    protocol=json.loads((HERE/'protocol.json').read_bytes());settings=protocol['providers'][a.provider]
    if (protocol.get('identity')!='cli-normal-real24-development-v1' or protocol.get('normal_only') is not True
        or protocol.get('tracks')!=['e2e_normal'] or protocol.get('required_providers')!=['mistral']
        or a.provider!='mistral' or protocol['providers']['google']['new_live_run_allowed'] is not False
        or protocol['providers']['groq']['new_live_run_allowed'] is not False):
        raise ValueError('ordinary_profile_required')
    root=a.corpus_dir.resolve();bins=Path('/eval/bin');jobs=[]
    for index,c in enumerate(data['cases']):
        arms=['baseline','candidate'] if index%2==0 else ['candidate','baseline']
        for arm in arms:
            jobs.append({'id':c['id']+'-'+arm,'arm':arm,'contract':'normal','timeout_seconds':360,'cancel_grace_seconds':1,
               'budget':{'max_generations':24,'max_input_tokens_per_generation':12512,'max_output_tokens_per_generation':1024,'max_attempts_per_generation':1,'aggregate_input_tokens':24000,'aggregate_output_tokens':4096},
               'argv':[sys.executable,str(HERE/'run_task.py'),'--provider',a.provider,'--model',settings['model'],'--corpus',str(root/'corpus.json'),'--host-policy',str(root/'host-policy.json'),'--case',c['id'],'--track','e2e_normal','--arm',arm,'--bin-dir',str(bins)]})
    if len(jobs)!=48:raise ValueError('complete48 normal plan required')
    artifacts=inventory(root,a.receipt)
    plan={'schema':'engine07-operational-plan-v1','callback_contract':'audited-aggregate-callback-v1','purpose':PURPOSE,'provider':a.provider,'model':settings['model'],
          'token_budget':sum(j['budget']['aggregate_input_tokens']+j['budget']['aggregate_output_tokens'] for j in jobs),
          'development_receipt_file':str(a.receipt.resolve()),'corpus_root':str(root),'artifacts':artifacts,'jobs':jobs,'case_ids':[c['id'] for c in data['cases']],'M1_sha256':INPUT_M1,'currency_self_cap_USD':settings['currency_self_cap_USD']}
    with a.output.open('x') as f:json.dump(plan,f,indent=2);f.write('\n')
    print(json.dumps({'planned_jobs':len(jobs),'helper_model_jobs':0,'purpose':PURPOSE,'token_reservation':plan['token_budget'],'provider_model_calls':0}))
if __name__=='__main__':main()
