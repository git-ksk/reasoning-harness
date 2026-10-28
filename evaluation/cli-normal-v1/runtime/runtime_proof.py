#!/usr/bin/env python3
"""Extend the offline paired proof with ordinary-profile controls, never live approval."""
import argparse, hashlib, json, subprocess, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
PARENT=HERE.parent

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--prepared',type=Path,required=True);p.add_argument('--bin-dir',type=Path,required=True);p.add_argument('--controls',type=Path,required=True);p.add_argument('--image-id',required=True);p.add_argument('--image-archive',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    base=a.controls/'ordinary-base-runtime-proof.json'
    subprocess.run([sys.executable,str(PARENT/'runtime_proof.py'),'--repo',str(a.repo),'--prepared',str(a.prepared),'--bin-dir',str(a.bin_dir),'--controls',str(a.controls),'--image-id',a.image_id,'--image-archive',str(a.image_archive),'--output',str(base)],check=True)
    proof=json.loads(base.read_bytes());prepared=a.prepared/'repo';normal=prepared/'evaluation/cli-repair-paired-v1/normal_v1';protocol=json.loads((normal/'protocol.json').read_bytes())
    assert protocol['identity']=='cli-normal-real24-development-v1' and protocol['normal_only'] is True and protocol['tracks']==['e2e_normal']
    assert protocol['required_providers']==['mistral'] and protocol['providers']['mistral']['new_live_run_allowed'] is True
    assert protocol['common_cli_host']=='c38d87d3af1a2b576cf5e4985c765935876837b8' and protocol['providers']['google']['new_live_run_allowed'] is False and protocol['providers']['groq']['new_live_run_allowed'] is False
    text=(a.controls/'ordinary-unit.log').read_text();assert 'Ran 12 tests' in text and text.rstrip().endswith('OK')
    integration_path=a.controls/'ordinary-integration/summary.json';integration=json.loads(integration_path.read_bytes())
    assert integration=={'scope':'SYNTHETIC_LOOPBACK_ONLY_NOT_REAL_ADMISSION','fixture_plan_jobs':3,'complete':1,'failed':1,'suppressed':1,
        'loopback_requests':6,'external_provider_requests':0,'models_used':False,'precredential_guard_stubbed_for_fixture':True,
        'real_receipt_admission_tested':False,'f1_measured':False,'durable_restart_refused':True,'collected_observations':3,'binding_failures':[]}
    proof.update(profile='cli-normal-real24-development-v1',normal_only=True,normal_protocol_sha256=sha(normal/'protocol.json'),ordinary_unit_controls=12,ordinary_unit_log_sha256=sha(a.controls/'ordinary-unit.log'),ordinary_integration_sha256=sha(integration_path),ordinary_integration_scope=integration['scope'],ordinary_base_proof_sha256=sha(base),ordinary_builder_sha256=sha(normal/'build.py'),ordinary_proof_script_sha256=sha(Path(__file__)),helper_model_jobs=0)
    with a.output.open('x') as f:json.dump(proof,f,indent=2);f.write('\n')
    print(json.dumps({'profile':proof['profile'],'ordinary_controls':12,'helper_model_jobs':0,'API_execution_admitted':False}))
if __name__=='__main__':main()
