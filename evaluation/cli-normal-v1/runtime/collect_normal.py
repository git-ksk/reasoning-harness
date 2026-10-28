#!/usr/bin/env python3
"""Reconstruct observations only from an authenticated exact-run artifact."""
import argparse,json,subprocess,sys,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(1,str(HERE.parent))
from development import consumed_corpus,INPUT_M1
from provenance import REPOSITORY,fetch_verified,sha

def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def dump(path,value):
    with Path(path).open('x') as f:json.dump(value,f,indent=2,ensure_ascii=False);f.write('\n')
def binding_hash(provenance):return sha(canonical(provenance))
def new_id(binding,old_id):return sha(('cli-normal-observation-v1|'+binding+'|'+old_id).encode())[:24]

def reconstruct(root,provenance,corpus,output):
    consumed=consumed_corpus(corpus)
    plan=json.loads((root/'lane/plan.json').read_bytes())
    if [c['id'] for c in consumed['cases']]!=plan['case_ids']:raise ValueError('frozen_corpus_case_order_changed')
    output=Path(output);output.mkdir()
    binding=binding_hash(provenance)
    with tempfile.TemporaryDirectory(prefix='cli-normal-collect-') as temporary:
        legacy=Path(temporary)/'raw-derived'
        subprocess.run([sys.executable,str(HERE.parent/'collect_checked.py'),'--corpus-dir',str(corpus),
            '--provider','mistral','--lane',str(root/'lane'),'--output',str(legacy)],check=True,capture_output=True)
        index=json.loads((legacy/'observation-index.json').read_bytes())
        if index.get('provider')!='mistral' or index.get('observations')!=48:raise ValueError('collector_did_not_emit_full_plan')
        items=sorted(p for p in legacy.glob('*.json') if p.name!='observation-index.json' and not p.name.endswith('.review-packet.json'))
        if len(items)!=48:raise ValueError('derived_observation_inventory_mismatch')
        for path in items:
            obs=json.loads(path.read_bytes());old=obs['row_id'];new=new_id(binding,old)
            if path.name!=old+'.json' or obs.get('provider')!='mistral' or obs.get('track')!='e2e_normal':raise ValueError('derived_row_identity_mismatch')
            obs['row_id']=new
            obs['execution_provenance']={'binding_sha256':binding,'run_id':provenance['run_id'],
                'artifact_id':provenance['artifact_id'],'receipt_sha256':provenance['receipt_sha256']}
            dump(output/(new+'.json'),obs)
            packet=json.loads((legacy/(old+'.review-packet.json')).read_bytes())
            if packet.get('row_id')!=old:raise ValueError('derived_packet_identity_mismatch')
            packet['row_id']=new;packet['observation_sha256']=sha(canonical(obs));packet['execution_provenance']=obs['execution_provenance']
            dump(output/(new+'.review-packet.json'),packet)
        index.update(execution_provenance_sha256=binding,artifact_id=provenance['artifact_id'],run_id=provenance['run_id'],
                     source='RECONSTRUCTED_FROM_VERIFIED_ARTIFACT',previous_observations_reused=False)
        dump(output/'observation-index.json',index)
    return binding

def main():
    p=argparse.ArgumentParser();p.add_argument('--run-id',type=int,required=True);p.add_argument('--artifact-id',type=int,required=True)
    p.add_argument('--receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    with tempfile.TemporaryDirectory(prefix='cli-normal-github-readback-') as temporary:
        root,provenance=fetch_verified(REPOSITORY,a.run_id,a.artifact_id,a.receipt,Path(temporary)/'verified')
        binding=reconstruct(root,provenance,a.corpus_dir,a.output)
        dump(a.output/'verified-run.json',provenance)
    print(json.dumps({'run_id':a.run_id,'artifact_id':a.artifact_id,'binding_sha256':binding,'accuracy_acceptance':False,'M1':INPUT_M1}))
if __name__=='__main__':main()
