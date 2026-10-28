#!/usr/bin/env python3
"""Materialize the distinct ordinary-only runtime; never grant model access."""
import argparse, hashlib, json, shutil, subprocess, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]
RUNTIME=HERE/'runtime'
WORKFLOWS=['cli-normal-runtime.yml','cli-normal-real24-development.yml']

def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    repo=a.repo.resolve();output=a.output.resolve()
    subprocess.run([sys.executable,str(repo/'evaluation/cli-repair-paired-v1/prepare.py'),'--repo',str(repo),'--output',str(output)],check=True)
    prepared=output/'repo';manifest_path=output/'preparation.json';manifest=json.loads(manifest_path.read_text())
    base=prepared/'evaluation/cli-repair-paired-v1';normal=base/'normal_v1';normal.mkdir()
    # The inherited prepare overlay omits its host-side proof driver. Copy the
    # exact source explicitly: normal_v1/runtime_proof.py invokes this file.
    proof_src=repo/'evaluation/cli-repair-paired-v1/runtime_proof.py'
    if not proof_src.is_file():raise ValueError('inherited_runtime_proof_missing')
    shutil.copyfile(proof_src,base/'runtime_proof.py')
    files={}
    for src in sorted(RUNTIME.iterdir()):
        if src.is_file() and (src.suffix in ['.py','.json','.md'] or src.name=='Dockerfile.runtime'):
            shutil.copyfile(src,normal/src.name);files[src.name]=sha(src)
    profile=json.loads((normal/'protocol.json').read_text())
    if not (profile['identity']=='cli-normal-real24-development-v1' and profile['normal_only'] is True and profile['tracks']==['e2e_normal']):raise ValueError('ordinary_profile_required')
    if not (profile['execution_tag']=='cli-normal-real24-development-v1' and profile['required_providers']==['mistral']):raise ValueError('execution_identity_changed')
    if not (profile['providers']['mistral']['new_live_run_allowed'] is True and profile['providers']['google']['new_live_run_allowed'] is False and profile['providers']['groq']['new_live_run_allowed'] is False):raise ValueError('provider_policy_changed')
    base_protocol=base/'protocol.json';original=sha(base_protocol);data=json.loads(base_protocol.read_text())
    data['common_cli_host']=profile['common_cli_host'];data['offline_validation']['expected_unit_controls']=85
    base_protocol.write_text(json.dumps(data,indent=2)+'\n')
    for name in WORKFLOWS:
        src=repo/'.github/workflows'/name;dst=prepared/'.github/workflows'/name
        if not src.is_file():raise ValueError('normal_workflow_missing:'+name)
        shutil.copyfile(src,dst)
    inventory={str(p.relative_to(prepared)):sha(p) for p in sorted(prepared.rglob('*')) if p.is_file() and '__pycache__' not in p.parts}
    manifest['profile']='cli-normal-real24-development-v1';manifest['ordinary_runtime_overlay']=files
    manifest['ordinary_base_proof_source_sha256']=sha(proof_src)
    manifest['ordinary_prepare_sha256']=sha(Path(__file__))
    manifest['base_protocol_override']={'original_sha256':original,'new_sha256':sha(base_protocol),'changes':['common_cli_host','offline_validation.expected_unit_controls']}
    manifest['files']=inventory;manifest['frozen_execution_receipt']=False;manifest['independent_review_complete']=False;manifest['accuracy_cases']=0
    manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps({'profile':manifest['profile'],'runtime_files':len(inventory),'ordinary_files':len(files),'live_execution_admitted':False}))
if __name__=='__main__':main()
