#!/usr/bin/env python3
"""Fail closed on an incomplete external M0; never creates that receipt itself."""
import argparse,hashlib,json
from pathlib import Path
from scorer import require,sha as blob_sha

def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--receipt',type=Path,required=True);p.add_argument('--image-archive',type=Path,required=True);p.add_argument('--actual-image-id',required=True);p.add_argument('--repo',type=Path,required=True);p.add_argument('--bin-dir',type=Path,default=Path('/eval/bin'));a=p.parse_args()
    m=json.loads(a.receipt.read_text());repo=a.repo.resolve()
    require(m['identity']=='engine073-realworld-M0-v1' and m['state']=='COMPLETE_FROZEN','M0_not_complete')
    review=m['independent_implementation_review']
    require(review['result']=='PASS' and review['reviewer']=='/root/engine07_realworld_independent' and len(review['manifest_sha256'])==64,'independent_M0_review_missing')
    require(sha(a.image_archive)==m['runtime_image_archive_sha256'] and a.actual_image_id==m['runtime_image_id'],'runtime_image_changed')
    files={str(p.relative_to(repo)):sha(p) for p in repo.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    require(bool(files) and files==m['files'],'M0_runtime_inventory_mismatch')
    require(blob_sha(json.dumps(files,sort_keys=True,separators=(',',':')).encode())==review['reviewed_runtime_files_sha256'],'runtime_not_reviewed')
    identity=json.loads((a.bin_dir/'identity.json').read_text())
    require(sha(a.bin_dir/'identity.json')==m['paired_host_identity_sha256'],'paired_identity_changed')
    require(m['candidate_source_commit']==identity['arms']['candidate']['engine_commit']=='ab3c6322921c8c738ec35bd018345917b5afa538','candidate_changed')
    require(identity['arms']['baseline']['engine_commit']=='4abee90f0501f0d8fc09b453a0b45c942fbed69e' and identity['host_commit']=='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7','paired_host_changed')
    for arm,record in identity['arms'].items():
        for name,digest in record['binaries'].items():require(sha(a.bin_dir/(name+'-'+arm))==digest,'binary_changed')
    require(type(m['independent_case_authorship_before_M0']) is int and m['independent_case_authorship_before_M0']==0 and type(m['completed_unix_seconds']) is int,'authorship_order_missing')
    protocol=json.loads((repo/'evaluation/engine073-realworld-evaluator-v1/protocol.json').read_text())
    require(m['required_provider_readiness']['groq']['new_live_run_allowed'] is True and protocol['providers']['groq']['currency_self_cap_USD']>0,'required_Groq_lane_not_ready')
    recovery=a.receipt.parent/'groq-recovery.json'
    require(sha(recovery)==m['required_provider_readiness']['groq']['recovery_sha256'],'Groq_recovery_receipt_changed')
    print(json.dumps({'M0':'VERIFIED','model_provider_calls':0}))
if __name__=='__main__':main()
