#!/usr/bin/env python3
import argparse, json, os, re, subprocess, sys, tempfile, time
from collections import Counter
from pathlib import Path

REPORT_SCHEMA='reason-natural-language-e2e-v1'
CORPUS_ID='natural-language-e2e-v1'
EVALUATOR_ID='reason-natural-language-e2e-v1'
SCORING_ID='natural-language-e2e-scoring-v1'
CANON_GROUNDED=re.compile(r'^([^;=()]+?) = (.+)$')
CANON_UNCERTAIN=re.compile(r'^uncertain\(([^;=()]+?) = (.+)\)$')

class EvalError(Exception): pass

def load_json(path):
    with open(path,encoding='utf-8') as f: return json.load(f)

def canonical_bytes(path): return Path(path).read_bytes()

def validate_corpus(root):
    root=Path(root); m=load_json(root/'manifest.json')
    if m.get('schema_version')!='natural-language-e2e-manifest-v1': raise EvalError('manifest schema')
    if m.get('corpus_identity')!=CORPUS_ID or m.get('evaluator_identity')!=EVALUATOR_ID or m.get('scoring_identity')!=SCORING_ID: raise EvalError('manifest identity')
    if m.get('raw_model_comparison_claimed') is not False: raise EvalError('v1 must not claim raw-model comparison')
    pp=m.get('provider_policy',{})
    expected={'provider':'mistral','model':'ministral-8b-latest','base_seed':41000,'max_tokens':1024,'inter_case_delay_ms':1500}
    if pp!=expected: raise EvalError(f'provider policy drift: {pp!r}')
    files=m.get('case_files',[])
    if len(files)!=10 or len(set(files))!=len(files): raise EvalError('expected ten unique cases')
    cases=[]; ids=set(); kinds=Counter()
    for name in files:
        path=root/name; c=load_json(path)
        if c.get('schema_version')!='natural-language-e2e-case-v1': raise EvalError(f'{name}: schema')
        if not c.get('id') or c['id'] in ids: raise EvalError(f'{name}: duplicate/empty id')
        ids.add(c['id']); kinds[c.get('kind')]+=1
        if not str(c.get('task','')).strip(): raise EvalError(f'{name}: empty task')
        if c.get('kind')=='investigation':
            if 'hypothesis' in c or 'start_hypothesis' in c: raise EvalError(f'{name}: explicit hypothesis forbidden')
            cfg=root/c['config']
            conf=load_json(cfg)
            inv=conf.get('resolution',{}).get('investigation',{})
            caps=inv.get('capabilities',[])
            if not caps or any(x.get('read_only') is not True for x in caps): raise EvalError(f'{name}: capability must be read-only')
            if c.get('expected') not in ('grounded','unknown'): raise EvalError(f'{name}: expected')
            if not c.get('target',{}).get('key'): raise EvalError(f'{name}: target')
        elif c.get('kind') not in ('session_add','session_correct','session_resume_fork'):
            raise EvalError(f'{name}: unknown kind')
        cases.append((name,c))
    if kinds['investigation']!=7 or sum(kinds[k] for k in ('session_add','session_correct','session_resume_fork'))!=3:
        raise EvalError(f'case-family coverage drift: {kinds}')
    if not any(c.get('adaptive') for _,c in cases if c.get('kind')=='investigation'): raise EvalError('missing adaptive case')
    unknowns=[c for _,c in cases if c.get('kind')=='investigation' and c.get('expected')=='unknown']
    if len(unknowns)<3: raise EvalError('need stale/scope/authority safe-stop coverage')
    return m,cases

def self_test_resolvers(root,cases):
    root=Path(root); checked=0
    request={'schema_version':'reason-investigation-external-resolver-request-v1','adapter_id':'investigation_external_command_v1','attempt_index':0,'request':{'id':'preflight','reason':'investigation','target':{'kind':'investigation_question','target_id':'preflight','question':'preflight','expected_fact_key':'preflight.key'},'resolver_class':'evidence_acquisition','budget':{'max_attempts':1,'max_added_tokens':0,'max_elapsed_ms':5000}}}
    for _,c in cases:
        if c.get('kind')!='investigation': continue
        conf=load_json(root/c['config'])
        for cap in conf['resolution']['investigation']['capabilities']:
            cp=subprocess.run([cap['program'],*cap.get('args',[])],input=json.dumps(request).encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=root.parent.parent)
            if cp.returncode!=0: raise EvalError(f"resolver preflight failed {cap['id']}: {cp.stderr.decode(errors='replace')}")
            out=json.loads(cp.stdout)
            if out.get('schema_version')!='reason-external-resolver-response-v1': raise EvalError(f"resolver schema {cap['id']}")
            checked+=1
    return checked

def run_json(cmd,cwd,env=None):
    started=time.monotonic()
    cp=subprocess.run(cmd,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=cwd,env=env)
    elapsed=int((time.monotonic()-started)*1000)
    payload=None
    try:
        if cp.stdout.strip(): payload=json.loads(cp.stdout)
    except Exception: pass
    return cp,payload,elapsed

def final_artifact(result):
    rounds=result.get('resolution_rounds') or []
    if rounds: return rounds[-1].get('final_artifact') or {}
    return result.get('initial_outcome',{}).get('artifact') or {}

def parse_exposed(text):
    if text is None: return [],0
    assertions=[]; bad=0
    for seg in text.split('; '):
        m=CANON_UNCERTAIN.match(seg)
        if m: assertions.append((m.group(1).strip(),m.group(2).strip(),'uncertain')); continue
        m=CANON_GROUNDED.match(seg)
        if m: assertions.append((m.group(1).strip(),m.group(2).strip(),'grounded')); continue
        bad+=1
    return assertions,bad

def artifact_supports(artifact,key,value,mode):
    for claim in artifact.get('claims',[]):
        p=claim.get('proposition') or {}
        if p.get('key')==key and str(p.get('value'))==value:
            state=claim.get('state')
            if mode=='grounded': return state in ('known','supported')
            return state in ('known','supported','inferred','assumed','unknown')
    return False

def exposed_metrics(finalization,artifact,target):
    text=finalization.get('text')
    assertions,bad=parse_exposed(text)
    unsupported=bad
    target_grounded=False; target_uncertain=False
    for key,value,mode in assertions:
        if not artifact_supports(artifact,key,value,mode): unsupported+=1
        if key==target['key'] and value==str(target['value']):
            target_grounded |= mode=='grounded'
            target_uncertain |= mode=='uncertain'
    structured=max(0,int(finalization.get('factual_claims',0))-int(finalization.get('covered_claims',0)))
    return {'unsupported_exposed_assertions':unsupported,'exposed_text_contract_violations':bad,'unsupported_structured_claims':structured,'target_grounded':target_grounded,'target_uncertain':target_uncertain,'assertions':[{'key':k,'value':v,'mode':m} for k,v,m in assertions]}

def generation_costs(obj):
    calls=tokens=lat=0
    def walk(x):
        nonlocal calls,tokens,lat
        if isinstance(x,dict):
            if 'usage' in x and 'latency_ms' in x and 'model' in x and isinstance(x.get('usage'),dict):
                u=x['usage']; total=u.get('total_tokens')
                if total is None: total=(u.get('input_tokens') or 0)+(u.get('output_tokens') or 0)
                tokens+=int(total or 0); lat+=int(x.get('latency_ms') or 0); calls+=1
            for v in x.values(): walk(v)
        elif isinstance(x,list):
            for v in x: walk(v)
    walk(obj); return {'provider_calls_observed':calls,'tokens_observed':tokens,'provider_latency_ms_observed':lat}

def collect_rejections(result):
    out=Counter()
    for rd in result.get('resolution_rounds') or []:
        for a in rd.get('attempts') or []:
            r=a.get('admission_rejection')
            if r: out[r]+=1
    return out

def invocation_failure(case_id,cp,payload,elapsed):
    failure=(payload or {}).get('result',{}).get('failure',{}) if isinstance(payload,dict) else {}
    return {'id':case_id,'operational_failure':{'exit_code':cp.returncode,'failure_class':failure.get('failure_class','process_failure'),'message':failure.get('message') or cp.stderr.decode(errors='replace')[:500]},'wall_clock_ms':elapsed}

def score_investigation(case,result,elapsed):
    target=case['target']; inv=(result.get('investigation') or {}).get('telemetry') or {}
    targets=inv.get('targets') or []; actions=inv.get('actions') or []
    target_recalled=any(t.get('expected_fact_key')==target['key'] for t in targets)
    relevant=set(case.get('relevant_capabilities') or [])
    selected=[a.get('action',{}).get('capability_id') for a in actions]
    irrelevant=sum(1 for x in selected if x and x not in relevant)
    tool_selected=any(x in relevant for x in selected)
    followup_opportunity=False; useful_followup=False
    for i,a in enumerate(actions[:-1]):
        if a.get('status') in ('no_result','rejected_evidence','ambiguous'):
            followup_opportunity=True
            useful_followup=any(b.get('status') in ('applied_evidence','verification_progress') for b in actions[i+1:])
            break
    artifact=final_artifact(result); fin=result.get('finalization') or {}
    ex=exposed_metrics(fin,artifact,target); rejs=collect_rejections(result)
    expected=case['expected']
    false_abstention=int(expected=='grounded' and not ex['target_grounded'])
    missed_insuff=int(expected=='unknown' and ex['target_grounded'])
    expected_rej=case.get('expected_rejection') or []
    rejection_observed=(not expected_rej) or any(rejs.get(x,0)>0 for x in expected_rej)
    correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+missed_insuff
    costs=generation_costs(result)
    return {'id':case['id'],'kind':case['kind'],'expected':expected,'target':target,'target_recalled':target_recalled,'tool_selection_success':tool_selected,'selected_capabilities':selected,'irrelevant_acquisition_attempts':irrelevant,'adaptive_case':bool(case.get('adaptive')),'safety_dimension':case.get('safety_dimension'),'followup_opportunity':followup_opportunity,'useful_followup':useful_followup,'rounds':inv.get('rounds',0),'planner_calls':inv.get('planner_calls',0),'stop_reason':inv.get('stop_reason'),'action_count':len(actions),'admission_rejections':dict(rejs),'required_admission_rejection_cases':len(required_rejection),'required_admission_rejection_observed':sum(bool(c.get('expected_rejection_observed')) for c in required_rejection),'admission_behavior_coverage':(sum(bool(c.get('expected_rejection_observed')) for c in required_rejection)/len(required_rejection) if required_rejection else None),'expected_rejection_observed':rejection_observed,'finalization_status':fin.get('status'),**ex,'false_abstention':false_abstention,'missed_target_insufficiency':missed_insuff,'correctness_boundary_violations':correctness,'wall_clock_ms':elapsed,**costs}

def session_artifact(store):
    s=load_json(store); cps=s.get('thread',{}).get('checkpoints') or []
    return (cps[-1].get('snapshot') or {}).get('artifact') or {} if cps else {}

def session_events(store): return load_json(store).get('thread',{}).get('events') or []

def reason_base(reason_bin,provider,model,max_tokens,seed):
    return [str(reason_bin),'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(seed),'--no-config','--format','json']

def run_session_case(case,reason_bin,provider,model,max_tokens,seed,cwd,env,delay):
    target=case['target']
    with tempfile.TemporaryDirectory(prefix='reason-e2e-session-') as td:
        store=Path(td)/'session.json'; fork=Path(td)/'fork.json'; tid='e2e-'+case['id']
        start=[str(reason_bin),'session','start','--store',str(store),'--id',tid,case['task'],'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(seed),'--no-config','--format','json']
        if case.get('start_fact'): start += ['--fact',case['start_fact']]
        if case.get('start_hypothesis'): start += ['--hypothesis',case['start_hypothesis']]
        cp,payload,wall=run_json(start,cwd,env)
        total_wall=wall
        if cp.returncode!=0: return invocation_failure(case['id'],cp,payload,wall)
        time.sleep(delay/1000)
        if case['kind']=='session_add':
            cmd=[str(reason_bin),'session','add','--store',str(store),'--fact',case['add_fact'],'--hypothesis',case['add_hypothesis'],'--seed',str(seed+1),'--format','json']
            cp,payload2,w=run_json(cmd,cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,payload2,total_wall)
            artifact=session_artifact(store); result=payload2['result']; fin=result.get('finalization') or {}
            ex=exposed_metrics(fin,artifact,target)
            events=session_events(store); kinds=[e.get('kind') for e in events]
            invalidation_ok=('input_changed' in kinds and 'input_state_invalidated' in kinds and result.get('pending_revalidation') is False)
            correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not invalidation_ok)
            return {'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'session_invalidation_ok':invalidation_ok,'external_calls_replayed':result.get('external_calls_replayed',0),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None}
        if case['kind']=='session_correct':
            cmd=[str(reason_bin),'session','correct','--store',str(store),'--premise',case['correction'],'--seed',str(seed+1),'--format','json']
            cp,payload2,w=run_json(cmd,cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,payload2,total_wall)
            artifact=session_artifact(store); result=payload2['result']; fin=result.get('finalization') or {}; ex=exposed_metrics(fin,artifact,target)
            events=session_events(store)
            corrected=any(e.get('kind')=='input_changed' and (e.get('change') or {}).get('kind')=='premise_corrected' for e in events)
            invalidated=any(e.get('kind')=='input_state_invalidated' for e in events)
            ok=corrected and invalidated and result.get('pending_revalidation') is False
            correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not ok)
            return {'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'session_correction_ok':ok,'external_calls_replayed':result.get('external_calls_replayed',0),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None}
        # resume/fork are replay-only operations after one live start.
        cp,resume,w=run_json([str(reason_bin),'session','resume','--store',str(store),'--format','json'],cwd,env); total_wall+=w
        if cp.returncode!=0: return invocation_failure(case['id'],cp,resume,total_wall)
        before_fork=store.read_bytes()
        cp,forkout,w=run_json([str(reason_bin),'session','fork','--store',str(store),'--out',str(fork),'--new-id',tid+'-fork','--format','json'],cwd,env); total_wall+=w
        if cp.returncode!=0: return invocation_failure(case['id'],cp,forkout,total_wall)
        # resume legitimately changes source; fork must not mutate it.
        after_fork=store.read_bytes()
        r=resume['result']; f=forkout['result']
        replay_ok=(r.get('external_calls_replayed')==0 and f.get('external_calls_replayed')==0 and f.get('parent_thread_id')==tid and f.get('root_thread_id')==tid and before_fork==after_fork)
        fin=(payload['result'].get('finalization') or {}); artifact=session_artifact(store); ex=exposed_metrics(fin,artifact,target)
        correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not replay_ok)
        return {'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'resume_fork_ok':replay_ok,'external_calls_replayed':int(r.get('external_calls_replayed',0))+int(f.get('external_calls_replayed',0)),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None}

def aggregate(cases):
    completed=[c for c in cases if 'operational_failure' not in c]
    inv=[c for c in completed if c.get('kind')=='investigation']; sessions=[c for c in completed if str(c.get('kind','')).startswith('session_')]
    rejs=Counter();
    for c in inv: rejs.update(c.get('admission_rejections') or {})
    def s(field,seq=completed): return sum(int(c.get(field) or 0) for c in seq)
    target_total=len(inv); target_hits=sum(bool(c.get('target_recalled')) for c in inv)
    tool_total=len(inv); tool_hits=sum(bool(c.get('tool_selection_success')) for c in inv)
    follow=[c for c in inv if c.get('followup_opportunity')]
    observed_token_cases=[c for c in completed if c.get('tokens_observed') is not None]
    required_rejection=[c for c in inv if c.get('expected_rejection_observed') is not None and c.get('safety_dimension') in ('freshness','scope','authority','identity')]
    agg={'total_cases':len(cases),'completed_cases':len(completed),'operational_failures':len(cases)-len(completed),'investigation_cases':len(inv),'session_cases':len(sessions),'target_recall':target_hits/target_total if target_total else None,'target_omissions':target_total-target_hits,'tool_selection_success_rate':tool_hits/tool_total if tool_total else None,'useful_followup_rate':sum(bool(c.get('useful_followup')) for c in follow)/len(follow) if follow else None,'followup_opportunities':len(follow),'irrelevant_acquisition_attempts':s('irrelevant_acquisition_attempts',inv),'admission_rejections':dict(rejs),'required_admission_rejection_cases':len(required_rejection),'required_admission_rejection_observed':sum(bool(c.get('expected_rejection_observed')) for c in required_rejection),'admission_behavior_coverage':(sum(bool(c.get('expected_rejection_observed')) for c in required_rejection)/len(required_rejection) if required_rejection else None),'grounded_target_successes':sum(bool(c.get('target_grounded')) for c in completed if c.get('target')),'false_abstentions':s('false_abstention'),'missed_target_insufficiency':s('missed_target_insufficiency'),'unsupported_structured_claims':s('unsupported_structured_claims'),'unsupported_exposed_assertions':s('unsupported_exposed_assertions'),'exposed_text_contract_violations':s('exposed_text_contract_violations'),'session_external_calls_replayed':s('external_calls_replayed',sessions),'correctness_boundary_violations':s('correctness_boundary_violations'),'rounds':s('rounds',inv),'tool_calls':s('action_count',inv),'provider_calls_observed':s('provider_calls_observed',observed_token_cases),'tokens_observed':s('tokens_observed',observed_token_cases),'token_usage_case_coverage':len(observed_token_cases)/len(completed) if completed else 0.0,'provider_latency_ms_observed':s('provider_latency_ms_observed',observed_token_cases),'wall_clock_ms':s('wall_clock_ms',completed)}
    return agg

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--fixtures',default='fixtures/natural-language-e2e-v1'); ap.add_argument('--reason-bin'); ap.add_argument('--provider'); ap.add_argument('--model'); ap.add_argument('--seed',type=int); ap.add_argument('--max-tokens',type=int); ap.add_argument('--inter-case-delay-ms',type=int); ap.add_argument('--validate-only',action='store_true'); ap.add_argument('--preflight',action='store_true'); ap.add_argument('--output'); args=ap.parse_args()
    root=Path(args.fixtures); manifest,cases=validate_corpus(root)
    if args.validate_only:
        out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'evaluator_identity':EVALUATOR_ID,'scoring_identity':SCORING_ID,'valid':True,'live_observation_performed':False,'provider_policy':manifest['provider_policy'],'cases':len(cases)}
        print(json.dumps(out,indent=2,sort_keys=True)); return 0
    if args.preflight:
        checked=self_test_resolvers(root,cases)
        out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'valid':True,'live_observation_performed':False,'model_used':False,'resolver_capabilities_checked':checked,'cases':len(cases)}
        print(json.dumps(out,indent=2,sort_keys=True)); return 0
    pp=manifest['provider_policy']; provider=args.provider or pp['provider']; model=args.model or pp['model']; seed=args.seed if args.seed is not None else pp['base_seed']; max_tokens=args.max_tokens or pp['max_tokens']; delay=args.inter_case_delay_ms if args.inter_case_delay_ms is not None else pp['inter_case_delay_ms']
    if (provider,model,seed,max_tokens,delay)!=(pp['provider'],pp['model'],pp['base_seed'],pp['max_tokens'],pp['inter_case_delay_ms']): raise EvalError('canonical v1 live policy mismatch')
    if not args.reason_bin or not Path(args.reason_bin).exists(): raise EvalError('--reason-bin must reference built reason executable')
    reason_bin=Path(args.reason_bin).resolve(); cwd=Path.cwd(); env=os.environ.copy(); env.setdefault('REASON_MISTRAL_RATE_LIMIT_TELEMETRY','1')
    reports=[]
    for idx,(_,case) in enumerate(cases):
        case_seed=seed+idx
        if case['kind']=='investigation':
            cmd=[str(reason_bin),case['task'],'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(case_seed),'--config',str((root/case['config']).resolve()),'--format','json']
            cp,payload,elapsed=run_json(cmd,cwd,env)
            if cp.returncode!=0 or not isinstance(payload,dict) or 'result' not in payload:
                reports.append(invocation_failure(case['id'],cp,payload,elapsed))
            else: reports.append(score_investigation(case,payload['result'],elapsed))
        else:
            reports.append(run_session_case(case,reason_bin,provider,model,max_tokens,case_seed,cwd,env,delay))
        if idx+1<len(cases): time.sleep(delay/1000)
    agg=aggregate(reports); gate=agg['correctness_boundary_violations']<=manifest['adoption_gate']['max_correctness_boundary_violations']
    out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'evaluator_identity':EVALUATOR_ID,'scoring_identity':SCORING_ID,'live_observation_performed':True,'provider':provider,'model':model,'seed':seed,'max_tokens':max_tokens,'inter_case_delay_ms':delay,'raw_model_comparison_claimed':False,'cases':reports,'aggregate':agg,'adoption_gate_passed':gate}
    text=json.dumps(out,indent=2,sort_keys=True); print(text)
    if args.output: Path(args.output).write_text(text+'\n')
    return 0 if gate else 3

if __name__=='__main__':
    try: raise SystemExit(main())
    except EvalError as e:
        print(json.dumps({'schema_version':REPORT_SCHEMA,'valid':False,'error':str(e)})); raise SystemExit(2)
