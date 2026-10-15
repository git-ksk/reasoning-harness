#!/usr/bin/env python3
"""First-attempt paired released engines; raw outputs survive every failure."""
import argparse,copy,hashlib,json,os,shutil,signal,subprocess,time
from pathlib import Path
from resolver import config,records,host_facts
from score import score_normal,score_helper,costs,summarize
HERE=Path(__file__).resolve().parent
ARMS=('0.6.1','0.7.2')
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def dump(path,obj):
    with Path(path).open('x') as f:json.dump(obj,f,ensure_ascii=False,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def read(path):
    try:return json.loads(Path(path).read_text())
    except (OSError,ValueError):return {}
def validate():
    manifest=HERE/'manifest.sha256'
    for line in manifest.read_text().splitlines():
        digest,name=line.split('  ',1);p=HERE/name
        assert p.resolve().is_relative_to(HERE.parents[1]) and sha(p)==digest,'frozen surface changed: '+name
    corpus=read(HERE/'corpus.json');protocol=read(HERE/'protocol.json')
    assert corpus['suite_id']==protocol['identity'] and len(corpus['cases'])==24
    assert len({x['id'] for x in corpus['cases']})==24
    for sid,s in corpus['sources'].items():
        assert hashlib.sha256(s['quote'].encode()).hexdigest()==s['quote_sha256']
        if s['synthetic_mutation']:
            parent=corpus['sources'][s['parent_source_id']]
            assert s['quote']==parent['quote']+' '+s['attack_payload']
            assert s['reference_facts']==parent['reference_facts']
        else:assert s['source_version']==corpus['documents'][s['document_id']]['source_version']
    for x in corpus['cases']:
        assert len(x['source_ids'])==2 and all(s in corpus['sources'] for s in x['source_ids'])
        host_facts(corpus,x)
    return corpus,protocol

def verify_binaries(bins):
    identity=read(bins/'identity.json')
    assert identity['host_commit']=='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7'
    assert identity['probe_sha256']==sha(HERE/'probe.rs') and identity['adviser_sha256']==sha(HERE/'adviser.rs')
    assert identity['adviser_binary_sha256']==sha(bins/'adviser')
    assert identity['build_script_sha256']==sha(HERE/'build.py')
    for v in ARMS:
        a=identity['arms'][v]
        assert a['engine_commit']=={'0.6.1':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','0.7.2':'296690a86e049b75ee693ae5032a71504366137c'}[v]
        assert a['versions']=={'cli':'0.5.5','engine':v}
        assert a['binary_sha256']==sha(bins/('reason-'+v)) and a['probe_binary_sha256']==sha(bins/('probe-'+v))
    return identity

def helper_budget_failures(advice):
    calls=advice.get('model_calls');output=(advice.get('usage') or {}).get('output_tokens');elapsed=advice.get('elapsed_ms')
    return ['helper_budget_exceeded'] if ((calls is not None and calls>1) or (output is not None and output>384) or (elapsed is not None and elapsed>110000)) else []

def normal_budget_failures(payload,cost,resolver_calls,wall_ms,model):
    failures=[];usage=(payload.get('result') or {}).get('usage',{}).get('run',{});logical=usage.get('model_calls')
    if not cost['available']:failures.append('cost_trace_unavailable')
    if not isinstance(logical,int):failures.append('logical_usage_unavailable')
    elif logical>12:failures.append('logical_model_operation_budget_exceeded')
    if cost['provider_attempts'] is not None and cost['provider_attempts']>72:failures.append('provider_attempt_budget_exceeded')
    if cost['model_calls'] is not None and cost['model_calls']>24:failures.append('adapter_generation_budget_exceeded')
    if cost['tokens'] is not None and cost['tokens']>28096:failures.append('total_token_budget_exceeded')
    if cost['output_tokens'] is not None and cost['output_tokens']>4096:failures.append('output_token_budget_exceeded')
    if cost['available'] and cost['tokens'] is not None and usage.get('total_tokens') is not None and cost['tokens']!=usage['total_tokens']:failures.append('trace_usage_token_mismatch')
    if cost['provider_attempts'] is not None and usage.get('provider_attempts') is not None and cost['provider_attempts']!=usage['provider_attempts']:failures.append('trace_usage_attempt_mismatch')
    if resolver_calls>4:failures.append('resolver_budget_exceeded')
    if any(m!=model for m in cost['models']):failures.append('unexpected_actual_model')
    if wall_ms>181000:failures.append('wall_budget_exceeded')
    return failures

class Runner:
    def __init__(self,out,env,delay,bins):self.out=out;self.env=env;self.delay=delay;self.last=0;self.bins=bins
    def journal(self,obj):
        with (self.out/'invocations.jsonl').open('a') as f:f.write(json.dumps(obj,sort_keys=True)+'\n');f.flush();os.fsync(f.fileno())
    def invoke(self,args,path,timeout=180,external=False):
        validate();verify_binaries(self.bins)
        if external:
            time.sleep(max(0,self.delay-(time.monotonic()-self.last)));self.last=time.monotonic()
        start=time.monotonic();self.journal({'event':'START','directory':str(path.relative_to(self.out)),'command':args,'external':external,'at_unix':time.time()})
        failure=None
        with (path/'stdout.json').open('xb') as stdout,(path/'stderr.txt').open('xb') as stderr:
            env=self.env.copy();env['REASON_HOME']=str(path/'private-reason-home')
            proc=subprocess.Popen(args,cwd=self.out,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
            try:code=proc.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid,signal.SIGKILL);proc.wait();code=proc.returncode;failure='wall_timeout'
            stdout.flush();stderr.flush();os.fsync(stdout.fileno());os.fsync(stderr.fileno())
        if code and not failure:failure='process_exit_'+str(code)
        wall=round((time.monotonic()-start)*1000)
        self.journal({'event':'FINISH','directory':str(path.relative_to(self.out)),'code':code,'operational_failure':failure,'wall_ms':wall,'stdout_sha256':sha(path/'stdout.json'),'stderr_sha256':sha(path/'stderr.txt'),'at_unix':time.time()})
        return read(path/'stdout.json'),failure,wall

def main():
    p=argparse.ArgumentParser();p.add_argument('--validate-only',action='store_true');p.add_argument('--provider',choices=['mistral','google','groq']);p.add_argument('--bin-dir',type=Path);p.add_argument('--output',type=Path);a=p.parse_args()
    corpus,protocol=validate()
    if a.validate_only:
        print(json.dumps({'valid':True,'live_observation_performed':False,'identity':corpus['suite_id'],'denominators':corpus['denominators']}));return
    assert a.provider and a.bin_dir and a.output
    # Never resume/overwrite the canonical first run; a rerun needs a successor identity.
    assert os.environ.get('GITHUB_RUN_ATTEMPT','1')=='1','first attempt only; preserve failed run'
    bins=a.bin_dir.resolve();identity=verify_binaries(bins);out=a.output.resolve();out.mkdir()
    for name in ('corpus.json','protocol.json','manifest.sha256'):shutil.copyfile(HERE/name,out/name)
    dump(out/'build-identity.json',identity)
    dump(out/'execution-identity.json',{'git_commit':os.getenv('ENGINE07_EXECUTION_COMMIT') or subprocess.check_output(['git','rev-parse','HEAD'],cwd=HERE,text=True).strip(),'run_id':os.getenv('GITHUB_RUN_ID'),'attempt':os.getenv('GITHUB_RUN_ATTEMPT'),'provider':a.provider,'models':protocol['models'],'manifest_sha256':sha(HERE/'manifest.sha256')})
    env=os.environ.copy();env.update(REASON_HOME=str(out/'private-reason-home'),REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS='6000',REASON_GROQ_MIN_REQUEST_INTERVAL_MS='10000',REASON_GROQ_TOKENS_PER_MINUTE='8000',REASON_GROQ_STRUCTURED_OUTPUT_RETRIES='0',REASON_MISTRAL_RATE_LIMIT_TELEMETRY='1')
    lane=Runner(out,env,protocol['inter_invocation_delay_ms']/1000,bins);model=protocol['models'][a.provider];normal=[];helper=[]
    for index,case in enumerate(corpus['cases']):
        validate();verify_binaries(bins)
        case_dir=out/case['id'];case_dir.mkdir();seed=protocol['base_seed']+index
        order=ARMS if index%2==0 else ARMS[::-1]
        for arm in order:
            path=case_dir/('normal-'+arm);path.mkdir();trace=path/'trace.json';journal=path/'resolver.jsonl';cfg=path/'config.json'
            dump(cfg,config(corpus,case,out/'corpus.json',journal))
            cmd=[str(bins/('reason-'+arm)),case['question'],'--provider',a.provider,'--model',model,'--seed',str(seed),'--max-tokens','1024','--max-model-calls','12','--max-output-tokens','4096','--max-total-tokens','28096','--config',str(cfg),'--format','json','--diagnostic-trace',str(trace)]
            payload,op,wall=lane.invoke(cmd,path,external=True)
            trace_data=read(trace);journal_data=[json.loads(line) for line in journal.read_text().splitlines()] if journal.exists() else []
            row=score_normal(corpus,case,payload,op,trace_data,journal_data);row.update(engine=arm,wall_ms=wall,cost=costs(read(trace)),resolver_calls=len(journal.read_text().splitlines()) if journal.exists() else 0)
            requests=[e.get('request') for e in trace_data.get('events',[]) if e.get('kind')=='model_request']
            row['initial_model_request']=requests[0] if requests else None
            c=row['cost']
            logical_usage=(payload.get('result') or {}).get('usage',{}).get('run',{})
            row['logical_model_operations']=logical_usage.get('model_calls')
            row['provider_http_attempts']=logical_usage.get('provider_attempts')
            row['operational_failures'].extend(normal_budget_failures(payload,c,row['resolver_calls'],wall,model))
            if row['operational_failures']:row['complete_useful_answer']=False;row['safe_withhold']=False
            normal.append(row);dump(path/'score.json',row)
        sources=[]
        for slot,sid in enumerate(case['source_ids']):
            s=copy.deepcopy(corpus['sources'][sid]);s['source_id']+=':slot:'+str(slot);sources.append(s)
        base_input={'id':case['id'],'question':case['helper_question'],'sources':sources,'reference_authorized':case['helper_reference_review_authorized'],'advice_equivalent':False}
        base_dir=case_dir/'helper-0.6.1';base_dir.mkdir();dump(base_dir/'input.json',base_input)
        base,bop,bwall=lane.invoke([str(bins/'probe-0.6.1'),str(base_dir/'input.json')],base_dir,timeout=10)
        candidate_dir=case_dir/'helper-0.7.2';candidate_dir.mkdir()
        same=sources[0]['quote']==sources[1]['quote'] and sources[0]['scope']==sources[1]['scope'] and sources[0]['source_version']==sources[1]['source_version']
        if same:
            advice={'equivalent':None,'model_calls':0,'skip_reason':'identical excerpts; baseline finalizer already determines source-local result','operational_failure':None};aop=None;awall=0
            dump(candidate_dir/'advice-skipped.json',advice)
        else:
            adviser_dir=candidate_dir/'adviser';adviser_dir.mkdir()
            visible=[{k:s[k] for k in ('quote','context_summary','scope','source_version','url','locator','origin_group','synthetic_mutation')} for s in sources]
            dump(adviser_dir/'input.json',{'question':case['helper_question'],'sources':visible})
            advice,aop,awall=lane.invoke([str(bins/'adviser'),'--provider',a.provider,'--model',model,'--seed',str(seed),'--input',str(adviser_dir/'input.json')],adviser_dir,timeout=110,external=True)
        candidate_input={**base_input,'advice_equivalent':advice.get('equivalent') is True};dump(candidate_dir/'input.json',candidate_input)
        candidate,cop,cwall=lane.invoke([str(bins/'probe-0.7.2'),str(candidate_dir/'input.json')],candidate_dir,timeout=10)
        row=score_helper(case,sources,base,candidate,advice,bop or aop or cop);row.update(baseline_wall_ms=bwall,candidate_wall_ms=awall+cwall,advice_usage=advice.get('usage'),advice_model=advice.get('model'));row['advice_provider_attempts']=advice.get('provider_attempts')
        row['operational_failures'].extend(helper_budget_failures(advice))
        if advice.get('model') and advice['model']!=model:row['operational_failures'].append('unexpected_helper_actual_model')
        if row['operational_failures']:row['compatible_display_gain']=0
        helper.append(row);dump(candidate_dir/'score.json',row)
        print(json.dumps({'case':case['id'],'provider':a.provider,'normal_completed':len(normal),'helper_completed':len(helper),'normal_hard_cases':sum(bool(r['hard_violations']) for r in normal),'operational_cases':sum(bool(r['operational_failures']) for r in normal)+sum(bool(r['operational_failures']) for r in helper)}),flush=True)
    summary=summarize(a.provider,normal,helper);dump(out/'normal-rows.json',normal);dump(out/'helper-rows.json',helper);dump(out/'summary.json',summary);print(json.dumps(summary,sort_keys=True),flush=True)
    # NO_GO due to known product blockers is a valid measurement result. Operational
    # incompleteness or actual safety violation is an execution/gate failure.
    if not summary['measurement_gate_pass']:raise SystemExit(1)
if __name__=='__main__':main()
