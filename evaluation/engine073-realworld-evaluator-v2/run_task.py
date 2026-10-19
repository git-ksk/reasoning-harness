#!/usr/bin/env python3
"""One actual task inside the durable provider lane. No semantic scoring."""
import argparse,json,os,subprocess,sys
from pathlib import Path
from resolver import config

def dump(path,value):
    with Path(path).open('x') as f:json.dump(value,f,ensure_ascii=False,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def invoke(argv,name,env):
    with open(name+'.stdout','xb') as out,open(name+'.stderr','xb') as err:r=subprocess.run(argv,env=env,stdout=out,stderr=err)
    dump(name+'.invocation.json',{'argv':argv,'returncode':r.returncode})
    try:value=json.loads(Path(name+'.stdout').read_text())
    except (ValueError,OSError):value={}
    return r.returncode,value
def main():
    p=argparse.ArgumentParser();p.add_argument('--provider',required=True);p.add_argument('--model',required=True);p.add_argument('--corpus',type=Path,required=True);p.add_argument('--host-policy',type=Path,required=True);p.add_argument('--case',required=True);p.add_argument('--track',choices=['e2e_normal','helper_opt_in'],required=True);p.add_argument('--arm',choices=['baseline','candidate','helper'],required=True);p.add_argument('--bin-dir',type=Path,required=True);a=p.parse_args()
    corpus=json.loads(a.corpus.read_text());case=next(c for c in corpus['cases'] if c['id']==a.case);seed=49200+corpus['cases'].index(case)
    work=Path.cwd();env=os.environ.copy();env.update(ENGINE073_MODEL_JOURNAL=str(work/'model-journal.jsonl'),ENGINE073_GENERATION_CAP='24' if a.track=='e2e_normal' else '1',REASON_GROQ_STRUCTURED_OUTPUT_RETRIES='0',REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS='0')
    if 'ENGINE073_LOOPBACK_BASE_URL' in env and a.model!='fixture-model':raise ValueError('live_callback_cannot_use_engineering_redirect')
    import time
    time.sleep(10.5) # also pace first starts across separate callback processes
    (work/'model-journal.jsonl').touch(mode=0o600,exist_ok=False)
    if a.track=='e2e_normal':
        dump(work/'config.json',config(corpus,case,a.corpus,work/'resolver.jsonl'))
        cmd=[str(a.bin_dir/('reason-'+a.arm)),case['question'],'--provider',a.provider,'--model',a.model,'--seed',str(seed),'--max-tokens','1024','--safety-profile','current','--max-model-calls','12','--max-output-tokens','4096','--max-total-tokens','28096','--config',str(work/'config.json'),'--format','json','--diagnostic-trace',str(work/'trace.json')]
        rc,result=invoke(cmd,'actual-cli',env)
        # Preserve the actual CLI envelope. Do not turn an error into abstention.
        print(json.dumps(result));return rc
    policy=json.loads(a.host_policy.read_text())[a.case]
    sources=[]
    for sid in case['source_ids']:
        source=dict(corpus['sources'][sid]);source['normalized_path']=str((a.corpus.parent/source['normalized_path']).resolve());sources.append(source)
    visible=[{k:s[k] for k in ['quote','context_summary','scope','source_version','url','locator','origin_group','synthetic_mutation']} for s in sources]
    dump(work/'advice-input.json',{'question':case['helper_question'],'sources':visible})
    cmd=[str(a.bin_dir/'adviser-baseline'),'--provider',a.provider,'--model',a.model,'--seed',str(seed),'--input',str(work/'advice-input.json')]
    rc,advice=invoke(cmd,'untrusted-adviser',env)
    result={**advice,'source_ids':case['source_ids']}
    if rc or advice.get('operational_failure') is not None:print(json.dumps(result));return 1
    source_input={'id':case['id'],'question':case['helper_question'],'sources':sources,
                  'reference_authorized':policy['controlled_reference_authorized'],'advice_equivalent':advice.get('equivalent') is True}
    dump(work/'source-host-input.json',source_input)
    for arm in ['baseline','candidate']:
        cmd=[str(a.bin_dir/('source-host-'+arm)),str(work/'source-host-input.json')]
        rc,output=invoke(cmd,'source-host-'+arm,env);result[arm]=output
        if rc:result['operational_failure']='provider_Protocol';break
    result['model_events']=[json.loads(line) for line in (work/'model-journal.jsonl').read_text().splitlines()]
    print(json.dumps(result));return 0 if result.get('operational_failure') is None else 1
if __name__=='__main__':raise SystemExit(main())
