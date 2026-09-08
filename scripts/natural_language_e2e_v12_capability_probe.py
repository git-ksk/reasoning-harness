#!/usr/bin/env python3
"""No-network CLI capability probe for every frozen v12 provider/model coordinate."""
import argparse,json,os,subprocess
from pathlib import Path
CREDENTIAL_ENV={'mistral':'MISTRAL_API_KEY','google':'GEMINI_API_KEY','groq':'GROQ_API_KEY'}

def load_targets(path):
    m=json.loads(Path(path).read_text()); targets=m.get('provider_targets') or []
    expected=[('mistral','ministral-8b-latest'),('google','gemma-4-31b-it'),('google','gemini-3.5-flash-lite'),('groq','openai/gpt-oss-120b')]
    got=[(x.get('provider'),x.get('model')) for x in targets]
    if got!=expected: raise ValueError(f'frozen provider targets drift: {got}')
    return targets

def run_probe(reason_bin,manifest):
    rows=[]
    for t in load_targets(manifest):
        provider=t['provider']; model=t['model']; env=os.environ.copy()
        for k in ('MISTRAL_API_KEY','GEMINI_API_KEY','GOOGLE_API_KEY','GROQ_API_KEY'): env.pop(k,None)
        cp=subprocess.run([reason_bin,'probe frozen provider/model capability without network','--provider',provider,'--model',model,'--max-tokens','8','--seed','61000','--no-config','--format','json'],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=env)
        try: payload=json.loads(cp.stdout)
        except Exception: payload={}
        failure=((payload.get('result') or {}).get('failure') or {})
        expected_env=CREDENTIAL_ENV[provider]
        ok=cp.returncode==1 and payload.get('schema_version')=='reason-cli-output-v1' and payload.get('command')=='ask' and failure.get('failure_class')=='credentials' and expected_env in str(failure.get('message',''))
        rows.append({'provider':provider,'model':model,'exit_code':cp.returncode,'failure_class':failure.get('failure_class'),'expected_credential':expected_env,'passed':ok})
    return {'schema_version':'natural-language-e2e-v12-capability-probe-v1','model_or_network_used':False,'rows':rows,'passed':all(x['passed'] for x in rows)}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--reason-bin',required=True); ap.add_argument('--manifest',default='fixtures/natural-language-e2e-v12/manifest.json'); ap.add_argument('--output'); a=ap.parse_args(); out=run_probe(a.reason_bin,a.manifest); text=json.dumps(out,indent=2,sort_keys=True); print(text); a.output and Path(a.output).write_text(text+'\n'); return 0 if out['passed'] else 2
if __name__=='__main__': raise SystemExit(main())
