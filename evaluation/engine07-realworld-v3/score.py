#!/usr/bin/env python3
"""Separate normal CLI truth/admission scoring from opt-in source-local diagnostics."""
import collections,json,re
from resolver import host_facts,records
GROUNDED=re.compile(r'^([^;=()]+?) = (.+)$')
UNCERTAIN=re.compile(r'^uncertain\(([^;=()]+?) = (.+)\)$')
def scoped(s,case):return all(s['scope'].get(k)==v for k,v in case['required_scope'].items())
def allowed_facts(corpus,case):
    values=collections.defaultdict(set)
    for sid in case['source_ids']:
        s=corpus['sources'][sid]
        if not s['synthetic_mutation'] and scoped(s,case):
            for k,v in s['reference_facts'].items():values[k].add(str(v))
    for k,v in host_facts(corpus,case).items():values[k].add(v)
    return values

def trace_complete(trace):
    if trace.get('contract')!='reason-natural-diagnostic-trace-v1' or not isinstance(trace.get('events'),list):return False
    requests={};terminals={}
    for e in trace['events']:
        if e.get('kind') not in ('model_request','model_response','model_failure'):continue
        if not isinstance(e.get('correlation_id'),str) or not isinstance(e.get('attempt'),int) or e['attempt']<1:return False
        key=(e['correlation_id'],e['attempt'])
        if e['kind']=='model_request':
            r=e.get('request') or {}
            if key in requests or not isinstance(r.get('task'),str) or not r['task'] or not isinstance(r.get('output_format'),dict) or r['output_format'].get('type') not in ('text','json_object','json_schema') or not isinstance(r.get('max_tokens'),int) or r['max_tokens']<1:return False
            requests[key]=e
        else:
            if key in terminals:return False
            if e['kind']=='model_response' and (not isinstance(e.get('response'),dict) or not isinstance(e.get('model'),str)):return False
            if e['kind']=='model_failure' and not isinstance(e.get('failure_class'),str):return False
            terminals[key]=e
    return bool(requests) and set(requests)==set(terminals)

def score_normal(corpus,case,payload,operation=None,trace=None,journal=None):
    violations=[];quality=[];ops=[];assertions=[]
    result=payload.get('result',{}) if isinstance(payload,dict) else {}
    if operation:ops.append(operation)
    if result.get('output_contract')!='reason-natural-output-v4':ops.append('missing_output_contract')
    artifact=(result.get('final_outcome') or {}).get('artifact')
    if not isinstance(artifact,dict):ops.append('missing_final_artifact');artifact={}
    investigation=result.get('investigation') or {}
    if investigation.get('generation_failure'):ops.append('investigation_generation_failure')
    for rd in result.get('resolution_rounds') or []:
        for a in rd.get('attempts') or []:
            if a.get('status') in ('adapter_unavailable','malformed_output','adapter_failed','transport_failure','authentication_failure','permission_denied','negotiation_failure','session_failure','protocol_failure','tool_failed','timed_out','policy_denied','budget_exceeded'):ops.append('resolver_'+a['status'])
    fin=result.get('finalization') or {};text=fin.get('text');status=fin.get('status');allowed=allowed_facts(corpus,case)
    if status not in ('grounded_answer','qualified_partial_answer','unresolved','abstain','requires_verification') or not all(k in fin for k in ('factual_claims','covered_claims','factual_claim_coverage')):ops.append('missing_or_invalid_finalization')
    trace=trace or {};events=trace.get('events',[]);events=events if isinstance(events,list) else []
    if trace.get('contract')!='reason-natural-diagnostic-trace-v1' or not isinstance(events,list):ops.append('missing_or_invalid_trace')
    if not trace_complete(trace):ops.append('incomplete_model_request_response_trace')
    for e in events:
        if e.get('kind')=='model_failure':ops.append('model_failure:'+str(e.get('failure_class')))
    if not any(e.get('kind')=='finalization_state' for e in events):ops.append('finalization_trace_missing')
    if not journal:ops.append('source_acquisition_not_observed')
    expected={r['id']:r for r in records(corpus,case)}
    acquired={}
    for entry in journal or []:
        for r in (entry.get('response',{}).get('contribution',{}).get('evidence') or []):
            if r.get('id') not in expected or r!=expected[r['id']]:violations.append('acquired_source_record_changed')
            else:acquired[r['id']]=r
    admitted={eid for e in events if e.get('kind')=='evidence_admission' for eid in e.get('admitted_evidence_ids',[])}
    evidence={e.get('id'):e for e in artifact.get('evidence',[])}
    valid_evidence={}
    for eid,e in evidence.items():
        r=acquired.get(eid);metadata=e.get('metadata') or {}
        if not r or eid not in admitted:
            violations.append('evidence_not_acquired_and_admitted:'+str(eid));continue
        if any(e.get(k)!=r[k] for k in ('id','source','observation','facts')):violations.append('admitted_evidence_content_changed:'+str(eid));continue
        a=r['acquisition_metadata'];klass=a['claimed_authority_class']
        expected_metadata={'scope':a['scope'],'provenance_class':klass,'temporal':{'effective_from_unix_seconds':a['observed_at_unix_seconds'],'effective_until_unix_seconds':a['observed_at_unix_seconds']+86400}}
        if metadata!=expected_metadata or klass not in ('primary','reference_host'):violations.append('admitted_evidence_metadata_changed:'+str(eid));continue
        if any(a['scope'].get(k)!={'kind':'values','values':[v]} for k,v in case['required_scope'].items()):violations.append('out_of_scope_evidence_admitted:'+str(eid));continue
        valid_evidence[eid]=e
    for binding in (artifact.get('source_attribution') or {}).get('bindings',[]):
        e=valid_evidence.get(binding.get('evidence_id'));s=corpus['sources'].get(binding.get('source_id'))
        if not e or not s or e['source']!=s['source_id'] or any(binding.get(k)!=s[v] for k,v in [('source_url','url'),('source_version','source_version'),('retrieved_at_unix_seconds','retrieved_at_unix_seconds')]) or (binding.get('locator') or {}).get('section')!=s['locator']:violations.append('misbound_normal_citation')

    claims=artifact.get('claims',[])
    # Known/Supported structured states are authority-bearing even if text is withheld.
    for claim in claims:
        prop=claim.get('proposition') or {};k=prop.get('key');v=str(prop.get('value'))
        if claim.get('state') in ('known','supported'):
            if v not in allowed.get(k,set()):violations.append('unsupported_structured_truth:'+str(k))
            if not any(eid in valid_evidence and valid_evidence[eid]['facts'].get(k)==v for eid in claim.get('evidence_ids',[])):violations.append('truth_without_matching_admitted_evidence:'+str(k))
    if text is not None:
        if status=='requires_verification':violations.append('blocked_unverified_text')
        for segment in text.split('; '):
            m=UNCERTAIN.fullmatch(segment);mode='uncertain'
            if not m:m=GROUNDED.fullmatch(segment);mode='grounded'
            if not m:violations.append('noncanonical_exposed_text');continue
            k,v=m.group(1).strip(),m.group(2).strip();assertions.append({'key':k,'value':v,'mode':mode})
            supported=any((c.get('proposition') or {}).get('key')==k and str((c.get('proposition') or {}).get('value'))==v and (c.get('state') in ('known','supported') if mode=='grounded' else c.get('state') in ('known','supported','inferred','assumed','unknown')) for c in claims)
            if not supported:violations.append('exposed_without_matching_authority:'+k)
            if v not in allowed.get(k,set()):
                (violations if mode=='grounded' else quality).append('unsupported_exposed_'+mode+':'+k)
    if status in ('grounded_answer','qualified_partial_answer') and int(fin.get('factual_claims',0))>int(fin.get('covered_claims',0)):violations.append('uncovered_answer_emitted')
    grounded={(x['key'],x['value']) for x in assertions if x['mode']=='grounded'}
    for k in case['forbidden_grounded_keys']:
        if any(a==k for a,b in grounded):violations.append('forbidden_target_grounded:'+k)
    required={(f['key'],str(f['value'])) for f in case['required']}
    complete=bool(required) and required<=grounded and not violations and not quality and not ops
    safe_withhold=not required and not grounded and not violations and not quality and not ops
    # New source APIs may remain inactive: preserve this observation, no invented citation.
    bindings=(artifact.get('source_attribution') or {}).get('bindings',[])
    return {'id':case['id'],'track':'normal_cli','status':status,'assertions':assertions,'hard_violations':sorted(set(violations)),'quality_failures':sorted(set(quality)),'operational_failures':sorted(set(ops)),'complete_useful_answer':complete,'safe_withhold':safe_withhold,'false_abstention':bool(required) and not complete and not ops,'answerable':bool(required),'source_attribution_binding_count':len(bindings),'citation_surface_present':bool(bindings),'known_supported_claims':sum(c.get('state') in ('known','supported') for c in claims)}

def score_helper(case,sources,base,candidate,advice,operation=None,deterministic_reference=False):
    bad=[];ops=[operation] if operation else []
    identical=sources[0]['quote']==sources[1]['quote'] and sources[0]['scope']==sources[1]['scope'] and sources[0]['source_version']==sources[1]['source_version']
    if not deterministic_reference:
        if identical:
            if advice.get('model_calls')!=0 or not advice.get('skip_reason') or advice.get('equivalent') is not None:ops.append('invalid_identical_adviser_skip')
        elif not all(k in advice for k in ('provider','requested_model','model_calls','elapsed_ms','equivalent','operational_failure')) or advice.get('model_calls')!=1:ops.append('incomplete_required_adviser_output')
        elif advice.get('operational_failure') is None and (not isinstance(advice.get('equivalent'),bool) or not all(k in advice for k in ('model','usage','provider_attempts','response_text'))):ops.append('incomplete_successful_adviser_output')
    if advice.get('operational_failure'):ops.append(advice['operational_failure'])
    for engine,p in [('baseline',base),('candidate',candidate)]:
        if not isinstance(p,dict) or p.get('id')!=case['id'] or p.get('engine_version')!=('0.6.1' if engine=='baseline' else '0.7.2') or not all(k in p for k in ('status','original','view','artifact_unchanged','claims_promoted','replay_equal','binding_count','unauthorized_reviewed')):
            ops.append(engine+':incomplete_probe_output');continue
        if p.get('binding_count')!=2:bad.append(engine+':binding_count')
        if not p.get('artifact_unchanged'):bad.append(engine+':artifact_mutated')
        if p.get('claims_promoted'):bad.append(engine+':truth_or_receipt_promotion')
        if not p.get('replay_equal'):bad.append(engine+':replay_changed')
        if p.get('unauthorized_reviewed'):bad.append(engine+':unauthorized_review')
        original=p.get('original') or {}
        cites=original.get('citations',[])
        if len(cites)!=2 or {c.get('binding_id') for c in cites}!={'b:0','b:1'}:bad.append(engine+':missing_or_duplicate_citations')
        if original.get('claim_ids')!=['c:0','c:1']:bad.append(engine+':claim_identity_changed')
        def tail(n):return f"[source:{sources[n]['source_id']}; evidence:e:{n}; binding:b:{n}]"
        if sources[0]['quote']==sources[1]['quote']:
            expected_text=f"According to {sources[0]['source_id']}, {sources[1]['source_id']}: {sources[0]['quote']} {tail(0)} {tail(1)}"
        else:expected_text=' '.join(f"According to {s['source_id']}: {s['quote']} {tail(n)}" for n,s in enumerate(sources))
        if original.get('text')!=expected_text:bad.append(engine+':source_quote_text_changed')
        if engine=='baseline' and p.get('view')!=original:bad.append('baseline:display_view_changed')
        if engine=='candidate':
            view=p.get('view') or {};answers=view.get('target_answers',[])
            if view.get('contract_id')!='harness-source-reconciliation-target-answer-v1' or view.get('original')!=original or len(answers)!=1 or answers[0].get('target_id')!=case['id'] or answers[0].get('original')!=original or answers[0].get('status')!=p.get('status') or set(view)!={'contract_id','original','target_answers'} or (len(answers)==1 and set(answers[0])!={'target_id','status','original'}):bad.append('candidate:display_view_changed')
        for cite in cites:
            bid=cite.get('binding_id');n=0 if bid=='b:0' else 1 if bid=='b:1' else None
            if n is None:bad.append(engine+':unknown_citation_binding');continue
            s=sources[n]
            expected={'claim_id':'c:'+str(n),'evidence_id':'e:'+str(n),'source_id':s['source_id'],'source_url':s['url'],'source_version':s['source_version'],'retrieved_at_unix_seconds':s['retrieved_at_unix_seconds'],'locator':{'section':s['locator']}}
            for key,value in expected.items():
                actual=cite.get(key)
                if key=='locator':actual={'section':(actual or {}).get('section')}
                if actual!=value:bad.append(engine+':citation_'+key)
    if base and candidate and base.get('original')!=candidate.get('original'):bad.append('source_local_original_changed')
    status=candidate.get('status');reviewed=status=='reviewed_compatible'
    if reviewed and not case['helper_reference_review_authorized']:bad.append('reference_unauthorized_review')
    if reviewed and case['helper_contract_expectation'].startswith('must_refuse'):bad.append('context_guard_bypassed')
    if reviewed and not advice.get('equivalent'):bad.append('review_without_adviser_equivalence')
    diagnostic=case.get('helper_diagnostic_only',False)
    expected=case['helper_semantic_expected_equivalent']
    advice_correct=advice.get('equivalent')==expected if advice.get('model_calls',0) else None
    eligible=case['helper_contract_expectation']=='review_eligible'
    return {'id':case['id'],'track':'controlled_helper','baseline_status':base.get('status'),'candidate_status':status,'reference_review_authorized':case['helper_reference_review_authorized'],'prebinding_diagnostic_only':diagnostic,'advice_equivalent':advice.get('equivalent'),'advice_correct':advice_correct,'review_refusal':candidate.get('review_refusal'),'hard_violations':sorted(set(bad)),'operational_failures':sorted(set(ops)),'review_eligible':eligible,'compatible_display_gain':int(eligible and base.get('status')=='conflict' and reviewed and not bad and not ops),'semantic_target_detection_success':None if diagnostic else 'not_measured_host_prebinding','model_calls':advice.get('model_calls',0)}

def costs(trace):
    if not trace_complete(trace):return {'available':False,'model_calls':None,'provider_attempts':None,'tokens':None,'provider_latency_ms':None,'models':{},'failures':[],'input_tokens':None,'output_tokens':None}
    out={'available':True,'model_calls':0,'provider_attempts':0,'tokens':0,'provider_latency_ms':0,'models':{},'failures':[],'input_tokens':0,'output_tokens':0}
    counts=collections.Counter();attempts_known=True;tokens_known=True
    for e in trace['events']:
        kind=e.get('kind')
        if kind=='model_response':
            r=e.get('response') or {};u=r.get('usage') or {};out['model_calls']+=1
            if r.get('provider_attempts') is None:attempts_known=False
            else:out['provider_attempts']+=r['provider_attempts']
            if u.get('input_tokens') is None or u.get('output_tokens') is None:tokens_known=False
            else:out['input_tokens']+=u['input_tokens'];out['output_tokens']+=u['output_tokens']
            if u.get('total_tokens') is not None:out['tokens']+=u['total_tokens']
            elif u.get('input_tokens') is not None and u.get('output_tokens') is not None:out['tokens']+=u['input_tokens']+u['output_tokens']
            else:tokens_known=False
            out['provider_latency_ms']+=e.get('latency_ms') or 0;counts[e.get('model')]+=1
        elif kind=='model_failure':
            out['model_calls']+=1;attempts_known=False;tokens_known=False;out['failures'].append(e.get('failure_class'))
    if not attempts_known:out['provider_attempts']=None
    if not tokens_known:out['tokens']=None;out['input_tokens']=None;out['output_tokens']=None
    out['models']=dict(counts);return out

def summarize(provider,rows,helper):
    normal={}
    for arm in ('0.6.1','0.7.2'):
        selected=[r for r in rows if r['engine']==arm]
        normal[arm]={'cases':len(selected),'complete_useful_answers':sum(r['complete_useful_answer'] for r in selected),'answerable_cases':sum(r['answerable'] for r in selected),'safe_withholds':sum(r['safe_withhold'] for r in selected),'false_abstentions':sum(r['false_abstention'] for r in selected),'hard_violation_cases':sum(bool(r['hard_violations']) for r in selected),'operational_failure_cases':sum(bool(r['operational_failures']) for r in selected),'citation_surface_cases':sum(r['citation_surface_present'] for r in selected),'logical_model_operations':None if any(r.get('logical_model_operations') is None for r in selected) else sum(r['logical_model_operations'] for r in selected),'adapter_generate_calls':None if any(r['cost']['model_calls'] is None for r in selected) else sum(r['cost']['model_calls'] for r in selected),'observed_adapter_generate_calls':sum(r['cost']['model_calls'] or 0 for r in selected),'tokens':None if any(r['cost']['tokens'] is None for r in selected) else sum(r['cost']['tokens'] for r in selected),'observed_tokens':sum(r['cost']['tokens'] or 0 for r in selected),'cost_unavailable_cases':sum(not r['cost']['available'] for r in selected),'token_usage_unavailable_cases':sum(r['cost']['tokens'] is None for r in selected),'wall_ms':sum(r['wall_ms'] for r in selected)}
    regressions=[case_id for case_id in {r['id'] for r in rows} if next((r['complete_useful_answer'] for r in rows if r['id']==case_id and r['engine']=='0.6.1'),False) and not next((r['complete_useful_answer'] for r in rows if r['id']==case_id and r['engine']=='0.7.2'),False)]
    pair_failures=[case_id for case_id in {r['id'] for r in rows} if len([r for r in rows if r['id']==case_id])!=2 or len({json.dumps(r.get('initial_model_request'),sort_keys=True) for r in rows if r['id']==case_id})!=1]
    return {'provider':provider,'positive_regressions':sorted(regressions),'pair_comparability_failures':sorted(pair_failures),'complete_measurement':len(rows)==48 and len(helper)==24 and not any(r['operational_failures'] for r in rows+helper),'measurement_gate_pass':len(rows)==48 and len(helper)==24 and not regressions and not pair_failures and not any(r['hard_violations'] or r['operational_failures'] for r in rows+helper),'normal_cli':normal,'normal_useful_answer_delta':normal['0.7.2']['complete_useful_answers']-normal['0.6.1']['complete_useful_answers'],'helper':{'cases':len(helper),'hard_violation_cases':sum(bool(r['hard_violations']) for r in helper),'operational_failure_cases':sum(bool(r['operational_failures']) for r in helper),'compatible_display_gains':sum(r['compatible_display_gain'] for r in helper),'diagnostic_prebinding_cases':sum(r['prebinding_diagnostic_only'] for r in helper),'advice_correct':sum(r['advice_correct'] is True for r in helper),'advice_scored':sum(r['advice_correct'] is not None for r in helper),'baseline_model_calls':0,'candidate_model_calls':sum(r['model_calls'] for r in helper),'candidate_tokens_observed':sum((r.get('advice_usage') or {}).get('total_tokens') or ((r.get('advice_usage') or {}).get('input_tokens') or 0)+((r.get('advice_usage') or {}).get('output_tokens') or 0) for r in helper),'candidate_token_usage_unavailable_cases':sum(r['model_calls']>0 and not ((r.get('advice_usage') or {}).get('total_tokens') is not None or ((r.get('advice_usage') or {}).get('input_tokens') is not None and (r.get('advice_usage') or {}).get('output_tokens') is not None)) for r in helper),'candidate_wall_ms':sum(r.get('candidate_wall_ms',0) for r in helper)},'adoption':'NO_GO','adoption_blockers':['known #520 source relevance regression','root development set; fresh independent holdout absent','real correction/conflict/unknown lineage coverage absent','controlled annotated source acquisition; no open-Web acceptance']}
