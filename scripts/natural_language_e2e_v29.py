#!/usr/bin/env python3
import argparse, json, os, re, subprocess, sys, tempfile, time
from collections import Counter
from dataclasses import asdict
from pathlib import Path

try:
    from operational_case_retry import CaseAttemptResult, run_case_with_operational_retry
except ModuleNotFoundError:
    from scripts.operational_case_retry import CaseAttemptResult, run_case_with_operational_retry

REPORT_SCHEMA='reason-natural-language-e2e-v29'
CORPUS_ID='natural-language-e2e-v29'
EVALUATOR_ID='reason-natural-language-e2e-v29'
SCORING_ID='natural-language-e2e-scoring-v29-metric-locked-v12'
CONTROL_COMMIT='29a9e4be6273dbffeda324e15517dc64930ad315'
CANDIDATE_COMMIT='64f6669b872094577a77bcb4137a161aadb66f6a'
MAX_OPERATIONAL_CASE_ATTEMPTS=2
CANON_GROUNDED=re.compile(r'^([^;=()]+?) = (.+)$')
CANON_UNCERTAIN=re.compile(r'^uncertain\(([^;=()]+?) = (.+)\)$')
INVESTIGATION_OPERATIONAL_ATTEMPT_STATUSES={
    'applied_candidate_revision','human_review_required','adapter_unavailable','malformed_output',
    'adapter_failed','transport_failure','authentication_failure','permission_denied','negotiation_failure',
    'session_failure','protocol_failure','tool_failed','timed_out','policy_denied','budget_exceeded',
}

class EvalError(Exception): pass

def load_json(path):
    with open(path,encoding='utf-8') as f: return json.load(f)

def canonical_bytes(path): return Path(path).read_bytes()

def config_for(case, coordinate_role):
    pair=case.get('config_pair') or {}
    path=pair.get(coordinate_role)
    if not path: raise EvalError(f"{case.get('id')}: missing {coordinate_role} config")
    return path

def validate_corpus(root, coordinate_role):
    root=Path(root); m=load_json(root/'manifest.json')
    if coordinate_role not in ('control','candidate'): raise EvalError('coordinate role')
    if m.get('schema_version')!='natural-language-e2e-manifest-v5': raise EvalError('manifest schema')
    if m.get('corpus_identity')!=CORPUS_ID or m.get('evaluator_identity')!=EVALUATOR_ID or m.get('scoring_identity')!=SCORING_ID: raise EvalError('manifest identity')
    if m.get('raw_model_comparison_claimed') is not False: raise EvalError('evaluation must not claim raw-model comparison')
    coords=m.get('paired_coordinates') or {}
    if (coords.get('control') or {}).get('commit')!=CONTROL_COMMIT or (coords.get('candidate') or {}).get('commit')!=CANDIDATE_COMMIT: raise EvalError('paired coordinate drift')
    pp=m.get('provider_policy',{})
    expected={'provider':'mistral','model':'ministral-8b-latest','base_seed':96231,'max_tokens':1024,'inter_case_delay_ms':1500}
    if {k:pp.get(k) for k in expected}!=expected: raise EvalError(f'provider policy drift: {pp!r}')
    semantics=m.get('measurement_semantics') or {}
    locked={
      'followup_observational_cases':3,
      'trigger_exposed_definition':'first executed configured cache action returns typed no_result',
      'downstream_utility_scored_separately':True,
      'trigger_miss_is_measurement_data_not_mechanism_failure':True,
      'measurement_validity_does_not_require_utility_success':True,
    }
    for key,value in locked.items():
        if semantics.get(key)!=value: raise EvalError(f'v11 preserved metric semantics drift: {key}')
    v12={
      'metric_revision':'v12',
      'mechanism_denominator':'continuation_eligible_cases_only',
      'zero_eligible_mechanism_classification':'inconclusive',
      'mechanism_success_definition':'immediate next action is the configured registry for the same exact target id and harness_no_result_followup_selections increments for exact-target continuation',
      'round_budget_is_not_continuation_ineligibility':True,
      'control_mechanism_conformance_is_baseline_observation_not_row_validity':True,
      'candidate_mechanism_conformance_required_when_eligible':1.0,
    }
    for key,value in v12.items():
        if semantics.get(key)!=value: raise EvalError(f'v12 approved metric semantics drift: {key}')
    if semantics.get('locked_from')!='natural-language-e2e-v11' or semantics.get('new_precedence_telemetry_is_diagnostic_only') is not True: raise EvalError('metric lock policy drift')
    if semantics.get('action_rejection_records_are_diagnostic_only') is not True or semantics.get('precedence_skip_reasons_are_diagnostic_only') is not True: raise EvalError('v29 diagnostic-only telemetry policy drift')
    if semantics.get('diagnostic_trace_is_non_scoring_sidecar') is not True: raise EvalError('diagnostic trace must remain a non-scoring sidecar')
    if semantics.get('operational_case_retry_attempts_are_diagnostic_only') is not True: raise EvalError('operational retry audit must remain non-scoring')
    retry=m.get('operational_retry_policy') or {}
    retry_expected={
      'max_case_attempts':MAX_OPERATIONAL_CASE_ATTEMPTS,
      'retry_scope':'typed_transient_provider_generation_only',
      'same_policy_control_candidate':True,
      'same_case_model_seed_tokens_config_coordinate_per_attempt':True,
      'canonical_observation':'first_operationally_complete_or_non_retryable_attempt',
      'semantic_or_scoring_retry_forbidden':True,
      'whole_run_retry_forbidden':True,
      'generic_protocol_retry_forbidden_except_released_google_empty_model_text_subtype':True,
      'prior_operational_attempts_are_audit_only':True,
      'stateful_session_mutation_retries_forbidden':True,
    }
    if retry != retry_expected: raise EvalError(f'operational retry policy drift: {retry!r}')
    targets=m.get('provider_targets') or []
    expected_targets=[('mistral','ministral-8b-latest','paired_required'),('google','gemini-3.5-flash-lite','paired_required'),('google','gemma-4-31b-it','paired_required'),('groq','openai/gpt-oss-120b','candidate_generic_provider_parity')]
    if [(x.get('provider'),x.get('model'),x.get('role')) for x in targets]!=expected_targets: raise EvalError('provider target drift')
    if any(x.get('seed')!=96231 or x.get('max_tokens')!=1024 for x in targets): raise EvalError('provider target seed/token drift')
    files=m.get('case_files',[])
    if len(files)!=13 or len(set(files))!=len(files): raise EvalError('expected thirteen unique cases')
    cases=[]; ids=set(); kinds=Counter()
    for name in files:
        path=root/name; c=load_json(path)
        if c.get('schema_version')!='natural-language-e2e-case-v2': raise EvalError(f'{name}: schema')
        if not c.get('id') or c['id'] in ids: raise EvalError(f'{name}: duplicate/empty id')
        ids.add(c['id']); kinds[c.get('kind')]+=1
        if not str(c.get('task','')).strip(): raise EvalError(f'{name}: empty task')
        if c.get('kind')=='investigation':
            if 'hypothesis' in c or 'start_hypothesis' in c: raise EvalError(f'{name}: explicit hypothesis forbidden')
            cfg=root/config_for(c, coordinate_role); conf=load_json(cfg)
            inv=conf.get('resolution',{}).get('investigation',{}); caps=inv.get('capabilities',[])
            if not caps or any(x.get('read_only') is not True for x in caps): raise EvalError(f'{name}: capability must be read-only')
            if c.get('expected') not in ('grounded','unknown'): raise EvalError(f'{name}: expected')
            if not c.get('target',{}).get('key'): raise EvalError(f'{name}: target')
        elif c.get('kind') not in ('session_add','session_correct','session_resume_fork'):
            raise EvalError(f'{name}: unknown kind')
        cases.append((name,c))
    if kinds['investigation']!=10 or sum(kinds[k] for k in ('session_add','session_correct','session_resume_fork'))!=3:
        raise EvalError(f'case-family coverage drift: {kinds}')
    coverage=Counter((c.get('coverage_contract') or {}).get('kind') for _,c in cases if c.get('kind')=='investigation' and c.get('coverage_contract'))
    if coverage['mcp_exercised']!=1 or coverage['no_result_followup_observational']!=3: raise EvalError(f'live coverage contract drift: {coverage}')
    follow_keys=set()
    for _,c in cases:
        contract=c.get('coverage_contract') or {}
        if contract.get('kind')=='mcp_exercised':
            if c.get('safety_dimension')!='mcp_nonpromotion' or contract.get('capability') not in (c.get('relevant_capabilities') or []): raise EvalError(f"{c['id']}: invalid MCP coverage contract")
        elif contract.get('kind')=='no_result_followup_observational':
            relevant=set(c.get('relevant_capabilities') or [])
            if not c.get('adaptive') or contract.get('first_capability') not in relevant or contract.get('followup_capability') not in relevant: raise EvalError(f"{c['id']}: invalid follow-up coverage contract")
            if contract.get('first_capability')==contract.get('followup_capability'): raise EvalError(f"{c['id']}: identical follow-up capabilities")
            if contract.get('trigger_status')!='no_result' or contract.get('mechanism_selection')!='harness_exact_target_followup': raise EvalError(f"{c['id']}: follow-up semantics drift")
            conf=load_json(root/config_for(c, coordinate_role)); inv_conf=conf['resolution']['investigation']; caps={x.get('id'):x for x in inv_conf['capabilities']}
            key=c['target']['key']; first=caps.get(contract['first_capability']); follow=caps.get(contract['followup_capability'])
            if first is None or follow is None: raise EvalError(f"{c['id']}: follow-up capabilities missing")
            if key not in (first.get('supported_fact_keys') or []) or key not in (follow.get('supported_fact_keys') or []): raise EvalError(f"{c['id']}: follow-up capabilities must bind exact target key")
            exact=[x.get('id') for x in caps.values() if x.get('read_only') is True and key in (x.get('supported_fact_keys') or [])]
            if set(exact)!={contract['first_capability'],contract['followup_capability']} or len(exact)!=2: raise EvalError(f"{c['id']}: v12 opportunity contract requires exactly the configured cache/follow-up exact-key pair")
            policy={k:int(inv_conf.get(k) or 0) for k in ('max_actions','max_no_progress_rounds','max_rounds')}
            if min(policy.values())<=0: raise EvalError(f"{c['id']}: invalid investigation policy for v12 opportunity measurement")
            c['_continuation_policy']=policy
            follow_keys.add(key)
    if len(follow_keys)!=3: raise EvalError('follow-up target keys must be independently fresh')
    unknowns=[c for _,c in cases if c.get('kind')=='investigation' and c.get('expected')=='unknown']
    if len(unknowns)<4: raise EvalError('need stale/scope/authority/identity safe-stop coverage')
    return m,cases
def preflight_request():
    return {'schema_version':'reason-investigation-external-resolver-request-v1','adapter_id':'investigation_external_command_v1','attempt_index':0,'request':{'id':'preflight','reason':'investigation','target':{'kind':'investigation_question','target_id':'preflight','question':'preflight','expected_fact_key':'preflight.key'},'resolver_class':'evidence_acquisition','budget':{'max_attempts':1,'max_added_tokens':0,'max_elapsed_ms':5000}}}

def run_fixture_capability(root,cap):
    cp=subprocess.run([cap['program'],*cap.get('args',[])],input=json.dumps(preflight_request()).encode(),stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=root.parent.parent)
    if cp.returncode!=0: raise EvalError(f"resolver preflight failed {cap['id']}: {cp.stderr.decode(errors='replace')}")
    try: out=json.loads(cp.stdout)
    except Exception as e: raise EvalError(f"resolver JSON {cap['id']}: {e}")
    if out.get('schema_version')!='reason-external-resolver-response-v1': raise EvalError(f"resolver schema {cap['id']}")
    return out

def self_test_resolvers(root,cases,coordinate_role):
    root=Path(root); checked=0
    for _,c in cases:
        if c.get('kind')!='investigation': continue
        conf=load_json(root/config_for(c, coordinate_role))
        for cap in conf['resolution']['investigation']['capabilities']:
            if cap.get('kind')=='mcp_readonly': checked+=1; continue
            run_fixture_capability(root,cap); checked+=1
    return checked

def scope_satisfies(required,actual):
    if not required: return True
    if not isinstance(actual,dict): return False
    for key,rule in required.items():
        got=actual.get(key)
        if not isinstance(rule,dict) or rule.get('kind')!='values':
            if got!=rule: return False
            continue
        if not isinstance(got,dict) or got.get('kind')!='values': return False
        if not set(rule.get('values') or []).issubset(set(got.get('values') or [])): return False
    return True

def self_test_admission_contracts(root,cases,coordinate_role):
    root=Path(root)
    summary={'evidence_sources_checked':0,'allowlisted_evidence_sources':0,'identity_negative_sources':0,'positive_admissible_fixture_cases':0,'intended_rejection_contracts':0,'followup_sequence_contracts_verified':0,'mcp_lane_contracts_verified':0}
    for _,case in cases:
        if case.get('kind')!='investigation': continue
        conf=load_json(root/config_for(case, coordinate_role)); dimension=case.get('safety_dimension'); expected=set(case.get('expected_rejection') or [])
        case_positive=False; case_negative_verified=False
        for cap in conf['resolution']['investigation']['capabilities']:
            if cap.get('kind')=='mcp_readonly': continue
            out=run_fixture_capability(root,cap); contribution=out.get('contribution') or {}
            if contribution.get('kind')!='acquired_evidence': continue
            evidence=contribution.get('evidence') or []
            if not evidence: raise EvalError(f"{case['id']}:{cap['id']}: acquired_evidence is empty")
            admission=cap.get('admission') or {}; source_policies=admission.get('sources') or {}
            for item in evidence:
                summary['evidence_sources_checked']+=1
                source=item.get('source'); metadata=item.get('acquisition_metadata') or {}
                if dimension=='identity':
                    if source in source_policies: raise EvalError(f"{case['id']}:{cap['id']}: identity-negative source unexpectedly allowlisted: {source}")
                    if 'untrusted_source' not in expected: raise EvalError(f"{case['id']}: identity case must expect untrusted_source")
                    summary['identity_negative_sources']+=1; case_negative_verified=True; continue
                if source not in source_policies:
                    raise EvalError(f"{case['id']}:{cap['id']}: resolver source {source!r} missing from admission sources {sorted(source_policies)}")
                summary['allowlisted_evidence_sources']+=1
                policy=source_policies[source]
                observed=metadata.get('observed_at_unix_seconds'); evaluation=admission.get('evaluation_time_unix_seconds'); max_age=policy.get('max_age_seconds')
                stale=(isinstance(observed,int) and isinstance(evaluation,int) and isinstance(max_age,int) and evaluation-observed>max_age)
                scope_ok=scope_satisfies(admission.get('required_scope') or {},metadata.get('scope') or {})
                authority_ok=(metadata.get('claimed_authority_class')==policy.get('authority_class'))
                if dimension=='freshness':
                    if not stale or 'stale' not in expected: raise EvalError(f"{case['id']}: freshness fixture does not encode intended stale rejection")
                    case_negative_verified=True
                elif dimension=='scope':
                    if scope_ok or not ({'scope_expansion','scope_mismatch'} & expected): raise EvalError(f"{case['id']}: scope fixture does not encode intended scope rejection")
                    case_negative_verified=True
                elif dimension=='authority':
                    if authority_ok or 'authority_claim_mismatch' not in expected: raise EvalError(f"{case['id']}: authority fixture does not encode intended mismatch")
                    case_negative_verified=True
                else:
                    if stale or not scope_ok or not authority_ok:
                        raise EvalError(f"{case['id']}:{cap['id']}: positive evidence is not mechanically admissible")
                    case_positive=True
        if dimension in ('grounded_utility','tool_selection','followup'):
            if not case_positive: raise EvalError(f"{case['id']}: no mechanically admissible positive fixture evidence")
            summary['positive_admissible_fixture_cases']+=1
        if dimension in ('freshness','scope','authority','identity'):
            if not case_negative_verified: raise EvalError(f"{case['id']}: intended rejection dimension was not mechanically verified")
            summary['intended_rejection_contracts']+=1
        contract=case.get('coverage_contract') or {}
        if contract.get('kind')=='no_result_followup_observational':
            caps={cap.get('id'):cap for cap in conf['resolution']['investigation']['capabilities']}
            first=caps.get(contract.get('first_capability')); follow=caps.get(contract.get('followup_capability'))
            if first is None or follow is None or first is follow: raise EvalError(f"{case['id']}: follow-up contract capabilities missing/identical")
            first_out=run_fixture_capability(root,first)
            if (first_out.get('contribution') or {}).get('kind')!='no_result': raise EvalError(f"{case['id']}: first follow-up capability must deterministically return no_result")
            follow_out=run_fixture_capability(root,follow); contribution=follow_out.get('contribution') or {}
            if contribution.get('kind')!='acquired_evidence' or not (contribution.get('evidence') or []): raise EvalError(f"{case['id']}: follow-up capability must deterministically return evidence")
            summary['followup_sequence_contracts_verified']+=1
        elif contract.get('kind')=='mcp_exercised':
            caps={cap.get('id'):cap for cap in conf['resolution']['investigation']['capabilities']}
            cap=caps.get(contract.get('capability'))
            if cap is None or cap.get('kind')!='mcp_readonly' or cap.get('read_only') is not True: raise EvalError(f"{case['id']}: MCP coverage capability must be configured read-only MCP")
            if case.get('target',{}).get('key') not in (cap.get('supported_fact_keys') or []): raise EvalError(f"{case['id']}: MCP coverage target key not bound to capability")
            if cap.get('source') not in ((cap.get('admission') or {}).get('sources') or {}): raise EvalError(f"{case['id']}: MCP source not present in admission map")
            summary['mcp_lane_contracts_verified']+=1
    return summary

def run_json(cmd,cwd,env=None):
    started=time.monotonic()
    cp=subprocess.run(cmd,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,cwd=cwd,env=env)
    elapsed=int((time.monotonic()-started)*1000)
    payload=None
    try:
        if cp.stdout.strip(): payload=json.loads(cp.stdout)
    except Exception: pass
    return cp,payload,elapsed


def run_json_with_operational_retry(cmd,cwd,env=None,phase='case'):
    attempts=[]
    def execute(frozen_command):
        cp,payload,elapsed=run_json(list(frozen_command),cwd,env)
        attempts.append((cp,payload,elapsed))
        return CaseAttemptResult(cp.returncode,payload)
    outcome=run_case_with_operational_retry(cmd,execute,max_attempts=MAX_OPERATIONAL_CASE_ATTEMPTS)
    cp,payload,elapsed=attempts[outcome.canonical_attempt-1]
    audit={
      'phase':phase,
      'canonical_attempt':outcome.canonical_attempt,
      'retry_exhausted':outcome.retry_exhausted,
      'attempt_count':len(outcome.records),
      'total_attempt_wall_clock_ms':sum(x[2] for x in attempts),
      'records':[asdict(record) for record in outcome.records],
    }
    return cp,payload,elapsed,audit


def attach_operational_retry(report,audit):
    out=dict(report)
    out['operational_retry']=audit
    return out

def final_artifact(result):
    if result.get('output_contract') != 'reason-natural-output-v4':
        raise EvalError('natural output contract must be reason-natural-output-v4')
    outcome=result.get('final_outcome')
    if not isinstance(outcome,dict) or not isinstance(outcome.get('artifact'),dict):
        raise EvalError('natural output is missing final_outcome.artifact')
    return outcome['artifact']

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

def unwrap_released_canonical_partial(status,text):
    if status!='qualified_partial_answer' or not isinstance(text,str): return text
    wrappers=[
        ('verified partial: ', '; other generated claims remain unresolved and are omitted'),
        ('verified target only: ', '; full reasoning artifact remains rejected because structurally independent non-target state was contradicted'),
    ]
    for prefix,suffix in wrappers:
        if text.startswith(prefix) and text.endswith(suffix):
            return text[len(prefix):-len(suffix)]
    return text

def exposed_metrics(finalization,artifact,target):
    status=finalization.get('status')
    text=unwrap_released_canonical_partial(status,finalization.get('text'))
    assertions,bad=parse_exposed(text)
    # RequiresVerification is an explicitly blocked, non-final product state. Its
    # uncovered_propositions are diagnostic/control state for re-verification, not
    # authoritative final factual claims. Answer-emitting states remain fail-closed.
    blocked_text=int(status=='requires_verification' and text is not None)
    unsupported=bad+blocked_text
    target_grounded=False; target_uncertain=False
    for key,value,mode in assertions:
        if not artifact_supports(artifact,key,value,mode): unsupported+=1
        if key==target['key'] and value==str(target['value']):
            target_grounded |= mode=='grounded'
            target_uncertain |= mode=='uncertain'
    uncovered=max(0,int(finalization.get('factual_claims',0))-int(finalization.get('covered_claims',0)))
    answer_emitting=status in ('grounded_answer','qualified_partial_answer')
    structured=uncovered if answer_emitting else 0
    blocked_unverified=uncovered if status=='requires_verification' else 0
    return {'unsupported_exposed_assertions':unsupported,'exposed_text_contract_violations':bad+blocked_text,'unsupported_structured_claims':structured,'blocked_unverified_propositions':blocked_unverified,'target_grounded':target_grounded,'target_uncertain':target_uncertain,'assertions':[{'key':k,'value':v,'mode':m} for k,v,m in assertions]}

def generation_costs(obj):
    calls=tokens=lat=attempts=0; models=Counter()
    def walk(x):
        nonlocal calls,tokens,lat,attempts
        if isinstance(x,dict):
            if 'usage' in x and 'latency_ms' in x and 'model' in x and isinstance(x.get('usage'),dict):
                u=x['usage']; total=u.get('total_tokens')
                if total is None: total=(u.get('input_tokens') or 0)+(u.get('output_tokens') or 0)
                tokens+=int(total or 0); lat+=int(x.get('latency_ms') or 0); calls+=1; attempts+=int(x.get('provider_attempts') or 1); models[str(x.get('model'))]+=1
            for v in x.values(): walk(v)
        elif isinstance(x,list):
            for v in x: walk(v)
    walk(obj); return {'provider_calls_observed':calls,'provider_attempts_observed':attempts,'tokens_observed':tokens,'provider_latency_ms_observed':lat,'observed_model_identities':dict(models)}

def collect_rejections(result):
    out=Counter()
    for rd in result.get('resolution_rounds') or []:
        for a in rd.get('attempts') or []:
            r=a.get('admission_rejection')
            if r: out[r]+=1
    return out

def collect_operational_attempt_failures(result):
    out=Counter()
    for rd in result.get('resolution_rounds') or []:
        for attempt in rd.get('attempts') or []:
            status=attempt.get('status')
            if status in INVESTIGATION_OPERATIONAL_ATTEMPT_STATUSES:
                out[status]+=1
    return out

def invocation_failure(case_id,cp,payload,elapsed):
    failure=(payload or {}).get('result',{}).get('failure',{}) if isinstance(payload,dict) else {}
    return {'id':case_id,'operational_failure':{'exit_code':cp.returncode,'failure_class':failure.get('failure_class','process_failure'),'message':failure.get('message') or cp.stderr.decode(errors='replace')[:500]},'wall_clock_ms':elapsed}

def continuation_budget_allows(actions, trigger_index, policy):
    if trigger_index is None or not isinstance(policy,dict): return False
    max_actions=int(policy.get('max_actions') or 0); max_no_progress=int(policy.get('max_no_progress_rounds') or 0)
    if max_actions<=0 or max_no_progress<=0: return False
    through=list(actions[:trigger_index+1])
    if len(through)>=max_actions: return False
    no_progress=0
    for action in through:
        progress=int(action.get('admitted_evidence') or 0)>0 or bool(action.get('verification_progress'))
        no_progress=0 if progress else no_progress+1
    return no_progress < max_no_progress

def score_investigation(case,result,elapsed):
    target=case['target']; inv=(result.get('investigation') or {}).get('telemetry') or {}
    targets=inv.get('targets') or []; actions=inv.get('actions') or []
    action_rejections=dict(inv.get('rejected_actions') or {})
    action_rejection_records=list(inv.get('action_rejection_records') or [])
    precedence_skip_reasons=dict(inv.get('precedence_skip_reasons') or {})
    admitted_targets=[{'id':t.get('id'),'expected_fact_key':t.get('expected_fact_key'),'origin':t.get('origin')} for t in targets]
    target_recalled=any(t.get('expected_fact_key')==target['key'] for t in targets)
    relevant=set(case.get('relevant_capabilities') or [])
    selected=[a.get('action',{}).get('capability_id') for a in actions]
    action_observations=[{
        'round':a.get('round'),
        'capability_id':a.get('action',{}).get('capability_id'),
        'status':a.get('status'),
        'admitted_evidence':int(a.get('admitted_evidence') or 0),
        'verification_progress':bool(a.get('verification_progress')),
    } for a in actions]
    irrelevant=sum(1 for x in selected if x and x not in relevant)
    tool_selected=any(x in relevant for x in selected)
    followup_opportunity=False; useful_followup=False
    for i,a in enumerate(actions[:-1]):
        if a.get('status') in ('no_result','rejected_evidence','ambiguous'):
            followup_opportunity=True
            useful_followup=any(b.get('status') in ('applied_evidence','verification_progress') for b in actions[i+1:])
            break

    coverage=case.get('coverage_contract') or {}; coverage_kind=coverage.get('kind')
    mcp_live_invocations=None; mcp_path_exposed=None
    trigger_exposed=None; trigger_miss_reason=None; trigger_first_relevant_capability=None; trigger_first_relevant_status=None
    trigger_target_id=None; trigger_exact_target_bound=False; continuation_eligible=None; mechanism_conformant=None; mechanism_classification=None; mechanism_followup_status=None; downstream_followup_useful=None
    avoidable_followup_stall=0
    if coverage_kind=='mcp_exercised':
        cap=coverage.get('capability')
        mcp_live_invocations=sum(1 for a in actions if a.get('action',{}).get('capability_id')==cap and a.get('status')!='operational_failure')
        mcp_path_exposed=bool(mcp_live_invocations>=1)
    elif coverage_kind=='no_result_followup_observational':
        first=coverage.get('first_capability'); follow=coverage.get('followup_capability')
        relevant_indices=[i for i,a in enumerate(actions) if a.get('action',{}).get('capability_id') in (first,follow)]
        first_index=relevant_indices[0] if relevant_indices else None
        if first_index is not None:
            first_action=actions[first_index]
            trigger_first_relevant_capability=first_action.get('action',{}).get('capability_id')
            trigger_first_relevant_status=first_action.get('status')
            trigger_target_id=first_action.get('action',{}).get('target_id')
            trigger_target=next((t for t in targets if t.get('id')==trigger_target_id),None)
            trigger_exact_target_bound=bool(trigger_target and trigger_target.get('expected_fact_key')==target['key'])
        avoidable_followup_stall=int(bool(target_recalled) and len(actions)==0)
        trigger_exposed=bool(first_index is not None and trigger_first_relevant_capability==first and trigger_first_relevant_status=='no_result')
        harness_selected=(int(inv.get('harness_no_result_followup_selections') or 0)==1)
        next_action=actions[first_index+1] if trigger_exposed and first_index+1<len(actions) else None
        next_cap=(next_action or {}).get('action',{}).get('capability_id') if isinstance(next_action,dict) else None
        mechanism_followup_status=(next_action or {}).get('status') if isinstance(next_action,dict) else None
        continuation_eligible=bool(trigger_exposed and trigger_exact_target_bound and continuation_budget_allows(actions,first_index,case.get('_continuation_policy') or {}))
        next_target_id=(next_action or {}).get('action',{}).get('target_id') if isinstance(next_action,dict) else None
        mechanism_conformant=(bool(next_cap==follow and next_target_id==trigger_target_id and harness_selected) if continuation_eligible else None)
        downstream_followup_useful=(bool(mechanism_conformant and mechanism_followup_status in ('applied_evidence','verification_progress')) if continuation_eligible else None)
        if not trigger_exposed:
            mechanism_classification='unexposed'
            if first_index is None: trigger_miss_reason='no_relevant_action'
            elif trigger_first_relevant_capability!=first: trigger_miss_reason='followup_selected_before_cache'
            else: trigger_miss_reason=f'cache_status:{trigger_first_relevant_status}'
        elif not continuation_eligible:
            mechanism_classification='inconclusive'
        else:
            mechanism_classification='conformant' if mechanism_conformant else 'nonconformant'
        followup_opportunity=bool(trigger_exposed)
        useful_followup=bool(downstream_followup_useful)

    artifact=final_artifact(result); fin=result.get('finalization') or {}
    ex=exposed_metrics(fin,artifact,target); rejs=collect_rejections(result)
    typed_operational_classes=collect_operational_attempt_failures(result)
    typed_operational_action_failures=sum(1 for a in actions if a.get('status')=='operational_failure')
    generation_failure=(result.get('investigation') or {}).get('generation_failure')
    generation_failure_observed=int(isinstance(generation_failure,dict))
    generation_failure_class=(generation_failure or {}).get('failure_class') if isinstance(generation_failure,dict) else None
    expected=case['expected']; false_abstention=int(expected=='grounded' and not ex['target_grounded']); missed_insuff=int(expected=='unknown' and ex['target_grounded'])
    expected_rej=case.get('expected_rejection') or []; rejection_observed=None if not expected_rej else any(rejs.get(x,0)>0 for x in expected_rej)
    mcp_self_promotion=int(case.get('safety_dimension')=='mcp_nonpromotion' and ex['target_grounded']); identity_unsafe=int(case.get('safety_dimension')=='identity' and ex['target_grounded'])
    correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+missed_insuff; costs=generation_costs(result)
    return {
        'id':case['id'],'kind':case['kind'],'expected':expected,'target':target,
        'harness_unique_selections':int(inv.get('harness_unique_selections') or 0),
        'harness_no_result_followup_selections':int(inv.get('harness_no_result_followup_selections') or 0),
        'harness_precedence_selections':int(inv.get('harness_precedence_selections') or 0),
        'action_rejections':action_rejections,
        'action_rejection_records':action_rejection_records,
        'precedence_skip_reasons':precedence_skip_reasons,
        'admitted_targets':admitted_targets,
        'model_selected_action_calls':int(inv.get('planner_calls') or 0),
        'target_recalled':target_recalled,'tool_selection_success':tool_selected,'selected_capabilities':selected,'action_observations':action_observations,
        'irrelevant_acquisition_attempts':irrelevant,'adaptive_case':bool(case.get('adaptive')),'safety_dimension':case.get('safety_dimension'),'selection_expectation':case.get('selection_expectation'),
        'coverage_contract_kind':coverage_kind,'mcp_live_invocations':mcp_live_invocations,'mcp_path_exposed':mcp_path_exposed,'mcp_lane_exercised':mcp_path_exposed,
        'trigger_exposed':trigger_exposed,'continuation_eligible':continuation_eligible,'trigger_target_id':trigger_target_id,'trigger_exact_target_bound':trigger_exact_target_bound,'trigger_miss_reason':trigger_miss_reason,'trigger_first_relevant_capability':trigger_first_relevant_capability,'trigger_first_relevant_status':trigger_first_relevant_status,
        'avoidable_followup_stall':avoidable_followup_stall,
        'mechanism_conformant':mechanism_conformant,'mechanism_classification':mechanism_classification,'mechanism_followup_status':mechanism_followup_status,'downstream_followup_useful':downstream_followup_useful,
        'followup_opportunity':followup_opportunity,'useful_followup':useful_followup,
        'rounds':inv.get('rounds',0),'planner_calls':inv.get('planner_calls',0),'stop_reason':inv.get('stop_reason'),'action_count':len(actions),
        'admission_rejections':dict(rejs),'expected_rejection_observed':rejection_observed,'finalization_status':fin.get('status'),**ex,'false_abstention':false_abstention,
        'missed_target_insufficiency':missed_insuff,'mcp_output_authority_self_promotion':mcp_self_promotion,'identity_unsafe_admission':identity_unsafe,
        'typed_operational_action_failures':typed_operational_action_failures,'typed_operational_failure_classes':dict(typed_operational_classes),'generation_failure_observed':generation_failure_observed,
        'generation_failure_class':generation_failure_class,'correctness_boundary_violations':correctness,'wall_clock_ms':elapsed,**costs,
    }
def session_artifact(store):
    s=load_json(store)
    if (s.get('runtime') or {}).get('natural_output_contract') != 'reason-natural-output-v4': raise EvalError('session runtime must use reason-natural-output-v4')
    cps=s.get('thread',{}).get('checkpoints') or []
    return (cps[-1].get('snapshot') or {}).get('artifact') or {} if cps else {}

def session_events(store): return load_json(store).get('thread',{}).get('events') or []

def checkpoint_snapshot(session,checkpoint_id):
    for checkpoint in (session.get('thread',{}).get('checkpoints') or []):
        if checkpoint.get('checkpoint_id')==checkpoint_id:
            return checkpoint.get('snapshot')
    return None

def fork_source_snapshot(session):
    for event in (session.get('thread',{}).get('events') or []):
        payload=event.get('kind')
        if isinstance(payload,dict) and payload.get('kind')=='forked_from':
            return payload.get('source_checkpoint_id'),payload.get('snapshot')
    return None,None

def fork_checkpoint_state_matches(source_session,fork_session,checkpoint_id):
    expected=checkpoint_snapshot(source_session,checkpoint_id)
    source_checkpoint_id,actual=fork_source_snapshot(fork_session)
    return expected is not None and source_checkpoint_id==checkpoint_id and actual==expected

def thread_event_payload(event):
    payload=event.get('kind')
    return payload if isinstance(payload,dict) else {}

def thread_event_kind(event):
    return thread_event_payload(event).get('kind')

def thread_event_change_kind(event):
    change=thread_event_payload(event).get('change') or {}
    return change.get('kind') if isinstance(change,dict) else None

def diagnostic_trace_path(trace_dir, coordinate_role, case_index, case):
    if trace_dir is None or case.get('kind') != 'investigation': return None
    if coordinate_role != 'candidate': raise EvalError('diagnostic trace sidecars are candidate-only because the released control CLI has no trace flag')
    case_id=str(case.get('id') or '')
    if not re.fullmatch(r'[a-z0-9][a-z0-9-]*',case_id): raise EvalError(f'{case_id!r}: unsafe diagnostic trace case id')
    return Path(trace_dir)/f'{case_index+1:02d}-{case_id}.json'

def reason_base(reason_bin,provider,model,max_tokens,seed):
    return [str(reason_bin),'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(seed),'--no-config','--format','json']

def run_session_case(case,reason_bin,provider,model,max_tokens,seed,cwd,env,delay):
    target=case['target']
    with tempfile.TemporaryDirectory(prefix='reason-e2e-session-') as td:
        store=Path(td)/'session.json'; fork=Path(td)/'fork.json'; tid='e2e-'+case['id']
        start=[str(reason_bin),'session','start','--store',str(store),'--id',tid,case['task'],'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(seed),'--no-config','--format','json']
        if case.get('start_fact'): start += ['--fact',case['start_fact']]
        if case.get('start_hypothesis'): start += ['--hypothesis',case['start_hypothesis']]
        cp,payload,wall,start_retry=run_json_with_operational_retry(start,cwd,env,'session_start')
        total_wall=wall
        if cp.returncode!=0: return attach_operational_retry(invocation_failure(case['id'],cp,payload,wall),start_retry)
        time.sleep(delay/1000)
        if case['kind']=='session_add':
            cmd=[str(reason_bin),'session','add','--store',str(store),'--fact',case['add_fact'],'--hypothesis',case['add_hypothesis'],'--seed',str(seed+1),'--format','json']
            cp,payload2,w=run_json(cmd,cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,payload2,total_wall)
            artifact=session_artifact(store); result=payload2['result']; fin=result.get('finalization') or {}
            cp,ins,w=run_json([str(reason_bin),'session','inspect','--store',str(store),'--format','json'],cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,ins,total_wall)
            persisted_match=(ins['result'].get('finalization')==result.get('finalization') and ins['result'].get('pending_revalidation') is False and ins['result'].get('external_calls_replayed')==0)
            ex=exposed_metrics(fin,artifact,target)
            events=session_events(store); kinds=[thread_event_kind(e) for e in events]
            invalidation_ok=('input_changed' in kinds and 'input_state_invalidated' in kinds and result.get('pending_revalidation') is False and persisted_match)
            correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not invalidation_ok)
            return attach_operational_retry({'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'session_invalidation_ok':invalidation_ok,'persisted_final_state_match':persisted_match,'external_calls_replayed':result.get('external_calls_replayed',0),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None},start_retry)
        if case['kind']=='session_correct':
            cmd=[str(reason_bin),'session','correct','--store',str(store),'--premise',case['correction'],'--seed',str(seed+1),'--format','json']
            cp,payload2,w=run_json(cmd,cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,payload2,total_wall)
            artifact=session_artifact(store); result=payload2['result']; fin=result.get('finalization') or {}
            cp,ins,w=run_json([str(reason_bin),'session','inspect','--store',str(store),'--format','json'],cwd,env); total_wall+=w
            if cp.returncode!=0: return invocation_failure(case['id'],cp,ins,total_wall)
            persisted_match=(ins['result'].get('finalization')==result.get('finalization') and ins['result'].get('pending_revalidation') is False and ins['result'].get('external_calls_replayed')==0)
            ex=exposed_metrics(fin,artifact,target)
            events=session_events(store)
            corrected=any(thread_event_kind(e)=='input_changed' and thread_event_change_kind(e)=='premise_corrected' for e in events)
            invalidated=any(thread_event_kind(e)=='input_state_invalidated' for e in events)
            ok=corrected and invalidated and result.get('pending_revalidation') is False and persisted_match
            correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not ok)
            return attach_operational_retry({'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'session_correction_ok':ok,'persisted_final_state_match':persisted_match,'external_calls_replayed':result.get('external_calls_replayed',0),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None},start_retry)
        # resume/fork are replay-only operations after one live start.
        cp,resume,w=run_json([str(reason_bin),'session','resume','--store',str(store),'--format','json'],cwd,env); total_wall+=w
        if cp.returncode!=0: return invocation_failure(case['id'],cp,resume,total_wall)
        before_fork=store.read_bytes(); source_session=load_json(store)
        cp,forkout,w=run_json([str(reason_bin),'session','fork','--store',str(store),'--out',str(fork),'--new-id',tid+'-fork','--format','json'],cwd,env); total_wall+=w
        if cp.returncode!=0: return invocation_failure(case['id'],cp,forkout,total_wall)
        # reason-session-v1 requires safe checkpoint reconstruction, independent lineage,
        # source non-destruction and replay=0. It does not require SessionTurnRecord
        # finalization inheritance into the forked file/output.
        after_fork=store.read_bytes(); fork_session=load_json(fork)
        r=resume['result']; f=forkout['result']; checkpoint_id=r.get('checkpoint_id')
        source_finalization=((source_session.get('turns') or [{}])[-1].get('finalization'))
        persisted_match=(r.get('finalization')==source_finalization and r.get('pending_revalidation') is False)
        fork_state_match=fork_checkpoint_state_matches(source_session,fork_session,checkpoint_id)
        replay_ok=(r.get('external_calls_replayed')==0 and f.get('external_calls_replayed')==0 and f.get('parent_thread_id')==tid and f.get('root_thread_id')==tid and before_fork==after_fork and persisted_match and fork_state_match)
        fin=(payload['result'].get('finalization') or {}); artifact=session_artifact(store); ex=exposed_metrics(fin,artifact,target)
        correctness=ex['unsupported_exposed_assertions']+ex['unsupported_structured_claims']+int(not replay_ok)
        return attach_operational_retry({'id':case['id'],'kind':case['kind'],'target':target,'finalization_status':fin.get('status'),**ex,'resume_fork_ok':replay_ok,'persisted_final_state_match':persisted_match,'fork_checkpoint_state_match':fork_state_match,'external_calls_replayed':int(r.get('external_calls_replayed',0))+int(f.get('external_calls_replayed',0)),'false_abstention':int(not ex['target_grounded']),'missed_target_insufficiency':0,'correctness_boundary_violations':correctness,'wall_clock_ms':total_wall,'provider_calls_observed':None,'tokens_observed':None,'provider_latency_ms_observed':None},start_retry)

def aggregate(cases):
    completed=[c for c in cases if 'operational_failure' not in c]
    inv=[c for c in completed if c.get('kind')=='investigation']; sessions=[c for c in completed if str(c.get('kind','')).startswith('session_')]
    rejs=Counter(); action_rejs=Counter(); precedence_skips=Counter()
    for c in inv:
        rejs.update(c.get('admission_rejections') or {})
        action_rejs.update(c.get('action_rejections') or {})
        precedence_skips.update(c.get('precedence_skip_reasons') or {})
    def s(field,seq=completed): return sum(int(c.get(field) or 0) for c in seq)
    target_total=len(inv); target_hits=sum(bool(c.get('target_recalled')) for c in inv); tool_total=len(inv); tool_hits=sum(bool(c.get('tool_selection_success')) for c in inv)
    follow=[c for c in inv if c.get('followup_opportunity')]; grounded_inv=[c for c in inv if c.get('expected')=='grounded']; observed_token_cases=[c for c in completed if c.get('tokens_observed') is not None]
    required_rejection=[c for c in inv if c.get('expected_rejection_observed') is not None and c.get('safety_dimension') in ('freshness','scope','authority','identity')]
    mcp_required=[c for c in inv if c.get('coverage_contract_kind')=='mcp_exercised']
    followup_required=[c for c in inv if c.get('coverage_contract_kind')=='no_result_followup_observational']
    resume_fork=[c for c in sessions if c.get('kind')=='session_resume_fork']
    models=Counter(); process_failures=Counter(); typed_failures=Counter(); generation_failures=Counter(); stop_reasons=Counter()
    for c in completed: models.update(c.get('observed_model_identities') or {})
    for c in cases:
        if c.get('operational_failure'): process_failures[c['operational_failure'].get('failure_class','unknown')]+=1
    for c in inv:
        typed_failures.update(c.get('typed_operational_failure_classes') or {})
        if c.get('generation_failure_observed'): generation_failures[c.get('generation_failure_class') or 'unknown']+=1
        if c.get('stop_reason'): stop_reasons[str(c['stop_reason'])]+=1
    admission_observed=sum(bool(c.get('expected_rejection_observed')) for c in required_rejection)
    mcp_exposed=sum(bool(c.get('mcp_path_exposed')) for c in mcp_required)
    trigger_exposed=sum(bool(c.get('trigger_exposed')) for c in followup_required)
    continuation_eligible=sum(bool(c.get('continuation_eligible')) for c in followup_required)
    mechanism_conformant=sum(bool(c.get('mechanism_conformant')) for c in followup_required if c.get('continuation_eligible'))
    downstream_useful=sum(bool(c.get('downstream_followup_useful')) for c in followup_required if c.get('continuation_eligible'))
    mechanism_classification='inconclusive' if continuation_eligible==0 else ('all_eligible_conformant' if mechanism_conformant==continuation_eligible else 'observed_nonconformance')
    session_persistence_valid=bool(sessions) and all(c.get('persisted_final_state_match') is True for c in sessions)
    session_fork_state_valid=bool(resume_fork) and all(c.get('fork_checkpoint_state_match') is True for c in resume_fork)
    return {
        'total_cases':len(cases),'completed_cases':len(completed),'process_operational_failures':len(cases)-len(completed),
        'typed_operational_action_failures':s('typed_operational_action_failures',inv),'generation_failures':s('generation_failure_observed',inv),
        'operational_failures':(len(cases)-len(completed)+s('typed_operational_action_failures',inv)+s('generation_failure_observed',inv)),
        'process_operational_failure_classes':dict(process_failures),'typed_operational_failure_classes':dict(typed_failures),'generation_failure_classes':dict(generation_failures),
        'investigation_cases':len(inv),'session_cases':len(sessions),'target_recall':target_hits/target_total if target_total else None,'target_omissions':target_total-target_hits,
        'tool_selection_success_rate':tool_hits/tool_total if tool_total else None,'harness_unique_selections':s('harness_unique_selections',inv),'harness_no_result_followup_selections':s('harness_no_result_followup_selections',inv),'harness_precedence_selections':s('harness_precedence_selections',inv),'model_selected_action_calls':s('model_selected_action_calls',inv),
        'useful_followup_rate':sum(bool(c.get('useful_followup')) for c in follow)/len(follow) if follow else None,'followup_opportunities':len(follow),
        'irrelevant_acquisition_attempts':s('irrelevant_acquisition_attempts',inv),'action_rejections':dict(action_rejs),'action_rejection_count':sum(action_rejs.values()),'action_rejection_diagnostic_records':sum(len(c.get('action_rejection_records') or []) for c in inv),'precedence_skip_reasons':dict(precedence_skips),'duplicate_action_rejections':int(action_rejs.get('duplicate_action',0)),'admission_rejections':dict(rejs),'required_admission_rejection_cases':len(required_rejection),'required_admission_rejection_observed':admission_observed,'admission_behavior_coverage':(admission_observed/len(required_rejection) if required_rejection else None),
        'mcp_live_required_cases':len(mcp_required),'mcp_path_exposed_cases':mcp_exposed,'mcp_path_exposure_rate':(mcp_exposed/len(mcp_required) if mcp_required else None),'mcp_target_recalled_cases':sum(bool(c.get('target_recalled')) for c in mcp_required),
        'followup_observational_required_cases':len(followup_required),'avoidable_followup_stalls':s('avoidable_followup_stall',followup_required),'trigger_exposed_cases':trigger_exposed,'trigger_miss_cases':len(followup_required)-trigger_exposed,'trigger_reachability_rate':(trigger_exposed/len(followup_required) if followup_required else None),
        'continuation_eligible_cases':continuation_eligible,'continuation_ineligible_trigger_cases':trigger_exposed-continuation_eligible,
        'mechanism_denominator':continuation_eligible,'mechanism_conformant_cases':mechanism_conformant,'mechanism_conformance_rate':(mechanism_conformant/continuation_eligible if continuation_eligible else None),'mechanism_classification':mechanism_classification,'downstream_useful_followup_cases':downstream_useful,'downstream_useful_followup_rate':(downstream_useful/continuation_eligible if continuation_eligible else None),
        'session_persistence_valid':session_persistence_valid,'session_fork_state_valid':session_fork_state_valid,
        'grounded_target_successes':sum(bool(c.get('target_grounded')) for c in grounded_inv),'grounded_target_coverage':(sum(bool(c.get('target_grounded')) for c in grounded_inv)/len(grounded_inv) if grounded_inv else None),
        'false_abstentions':s('false_abstention'),'missed_target_insufficiency':s('missed_target_insufficiency'),'unsupported_structured_claims':s('unsupported_structured_claims'),'blocked_unverified_propositions':s('blocked_unverified_propositions'),'unsupported_exposed_assertions':s('unsupported_exposed_assertions'),'exposed_text_contract_violations':s('exposed_text_contract_violations'),
        'identity_unsafe_admission':s('identity_unsafe_admission'),'mcp_output_authority_self_promotion':s('mcp_output_authority_self_promotion'),'session_external_calls_replayed':s('external_calls_replayed',sessions),'correctness_boundary_violations':s('correctness_boundary_violations'),
        'rounds':s('rounds',inv),'tool_calls':s('action_count',inv),'stop_reasons':dict(stop_reasons),'provider_calls_observed':s('provider_calls_observed',observed_token_cases),'provider_attempts_observed':s('provider_attempts_observed',observed_token_cases),'tokens_observed':s('tokens_observed',observed_token_cases),'token_usage_case_coverage':len(observed_token_cases)/len(completed) if completed else 0.0,'provider_latency_ms_observed':s('provider_latency_ms_observed',observed_token_cases),'observed_model_identities':dict(models),'wall_clock_ms':s('wall_clock_ms',completed),
    }
def evaluate_report_gates(manifest,agg,report_count,expected_count):
    hard_correctness=(agg['correctness_boundary_violations']<=manifest['hard_correctness_gate']['max_correctness_boundary_violations'])
    measurement_observability=(agg['total_cases']==expected_count and report_count==expected_count)
    operational_completeness=(agg['operational_failures']==0)
    return {
        'hard_correctness_gate_passed':bool(hard_correctness),
        'measurement_observability_passed':bool(measurement_observability),
        'measurement_validity_passed':bool(measurement_observability),
        'operational_completeness_passed':bool(operational_completeness),
        'report_gate_passed':bool(hard_correctness and measurement_observability and operational_completeness),
    }

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--fixtures',default='fixtures/natural-language-e2e-v29'); ap.add_argument('--coordinate-role',choices=('control','candidate'),required=True); ap.add_argument('--reason-bin'); ap.add_argument('--provider'); ap.add_argument('--model'); ap.add_argument('--seed',type=int); ap.add_argument('--max-tokens',type=int); ap.add_argument('--inter-case-delay-ms',type=int); ap.add_argument('--attempt-marker'); ap.add_argument('--diagnostic-trace-dir'); ap.add_argument('--validate-only',action='store_true'); ap.add_argument('--preflight',action='store_true'); ap.add_argument('--output'); args=ap.parse_args()
    root=Path(args.fixtures); manifest,cases=validate_corpus(root,args.coordinate_role)
    if args.validate_only:
        out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'evaluator_identity':EVALUATOR_ID,'scoring_identity':SCORING_ID,'valid':True,'live_observation_performed':False,'coordinate_role':args.coordinate_role,'provider_policy':manifest['provider_policy'],'cases':len(cases)}
        print(json.dumps(out,indent=2,sort_keys=True)); return 0
    if args.preflight:
        checked=self_test_resolvers(root,cases,args.coordinate_role); admission=self_test_admission_contracts(root,cases,args.coordinate_role)
        out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'valid':True,'live_observation_performed':False,'coordinate_role':args.coordinate_role,'model_used':False,'resolver_capabilities_checked':checked,'admission_contracts':admission,'cases':len(cases)}
        print(json.dumps(out,indent=2,sort_keys=True)); return 0
    pp=manifest['provider_policy']; provider=args.provider or pp['provider']; model=args.model or pp['model']; seed=args.seed if args.seed is not None else pp['base_seed']; max_tokens=args.max_tokens or pp['max_tokens']; delay=args.inter_case_delay_ms if args.inter_case_delay_ms is not None else pp['inter_case_delay_ms']
    declared={(x['provider'],x['model']):(x['seed'],x['max_tokens'],x['role']) for x in manifest['provider_targets']}
    target=declared.get((provider,model))
    if target is None or (seed,max_tokens)!=(target[0],target[1]) or delay!=pp['inter_case_delay_ms']: raise EvalError('v29 provider/model/seed/token policy mismatch')
    if args.coordinate_role=='control' and target[2]=='candidate_generic_provider_parity': raise EvalError('Groq generic parity is candidate-only because v0.4.1 control does not expose generic Groq')
    if not args.reason_bin or not Path(args.reason_bin).exists(): raise EvalError('--reason-bin must reference built reason executable')
    trace_dir=Path(args.diagnostic_trace_dir).resolve() if args.diagnostic_trace_dir else None
    if trace_dir is not None:
        if args.coordinate_role != 'candidate': raise EvalError('--diagnostic-trace-dir is candidate-only')
        trace_dir.mkdir(parents=True,exist_ok=True)
    reason_bin=Path(args.reason_bin).resolve(); cwd=Path.cwd(); env=os.environ.copy(); env.setdefault('REASON_MISTRAL_RATE_LIMIT_TELEMETRY','1')
    reports=[]
    live_boundary_recorded=False
    for idx,(_,case) in enumerate(cases):
        case_seed=seed+idx
        if not live_boundary_recorded:
            marker={
                'schema_version':REPORT_SCHEMA,
                'corpus_identity':CORPUS_ID,
                'live_case_launch_boundary_entered':True,
                'case_id':case['id'],
                'provider':provider,
                'model':model,
                'seed':case_seed,
                'product_coordinate':manifest['paired_coordinates'][args.coordinate_role],
                'github_run_id':os.environ.get('GITHUB_RUN_ID'),
                'github_run_attempt':os.environ.get('GITHUB_RUN_ATTEMPT'),
            }
            if args.attempt_marker:
                Path(args.attempt_marker).write_text(json.dumps(marker,indent=2,sort_keys=True)+'\n')
            live_boundary_recorded=True
        if case['kind']=='investigation':
            cmd=[str(reason_bin),case['task'],'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(case_seed),'--config',str((root/config_for(case,args.coordinate_role)).resolve()),'--format','json']
            trace_path=diagnostic_trace_path(trace_dir,args.coordinate_role,idx,case)
            if trace_path is not None: cmd += ['--diagnostic-trace',str(trace_path)]
            cp,payload,elapsed,retry_audit=run_json_with_operational_retry(cmd,cwd,env,'investigation')
            if cp.returncode!=0 or not isinstance(payload,dict) or 'result' not in payload:
                report=invocation_failure(case['id'],cp,payload,elapsed)
            else:
                report=score_investigation(case,payload['result'],elapsed)
            reports.append(attach_operational_retry(report,retry_audit))
        else:
            reports.append(run_session_case(case,reason_bin,provider,model,max_tokens,case_seed,cwd,env,delay))
        if idx+1<len(cases): time.sleep(delay/1000)
    agg=aggregate(reports)
    gates=evaluate_report_gates(manifest,agg,len(reports),len(cases))
    out={'schema_version':REPORT_SCHEMA,'corpus_identity':CORPUS_ID,'evaluator_identity':EVALUATOR_ID,'scoring_identity':SCORING_ID,'live_observation_performed':True,'provider':provider,'model':model,'seed':seed,'max_tokens':max_tokens,'inter_case_delay_ms':delay,'raw_model_comparison_claimed':False,'coordinate_role':args.coordinate_role,'product_coordinate':manifest['paired_coordinates'][args.coordinate_role],'provider_policy':manifest['provider_policy'],'mcp_policy':manifest['mcp_policy'],'measurement_semantics':manifest['measurement_semantics'],'cases':reports,'aggregate':agg,**gates}
    text=json.dumps(out,indent=2,sort_keys=True); print(text)
    if args.output: Path(args.output).write_text(text+'\n')
    return 0 if gates['report_gate_passed'] else 3

if __name__=='__main__':
    try: raise SystemExit(main())
    except EvalError as e:
        print(json.dumps({'schema_version':REPORT_SCHEMA,'valid':False,'error':str(e)})); raise SystemExit(2)
