#!/usr/bin/env python3
import argparse, json
from pathlib import Path

CONTROL='29a9e4be6273dbffeda324e15517dc64930ad315'
CANDIDATE='ced59271b437f23695b01e02bc49c051e99df970'
PAIRED=['mistral/ministral-8b-latest','google/gemini-3.5-flash-lite','google/gemma-4-31b-it']
GROQ='groq/openai/gpt-oss-120b'
ZERO=['correctness_boundary_violations','unsupported_structured_claims','unsupported_exposed_assertions','exposed_text_contract_violations','missed_target_insufficiency','identity_unsafe_admission','mcp_output_authority_self_promotion','session_external_calls_replayed','duplicate_action_rejections']
class AcceptanceError(Exception): pass

def load(p): return json.loads(Path(p).read_text())
def key(r): return f"{r.get('provider')}/{r.get('model')}"
def report_id(r): return f"{r.get('coordinate_role')}:{key(r)}"

def validate_report(r, role, coordinate):
    reasons=[]; a=r.get('aggregate') or {}
    if r.get('schema_version')!='reason-natural-language-e2e-v23' or r.get('corpus_identity')!='natural-language-e2e-v23' or r.get('evaluator_identity')!='reason-natural-language-e2e-v23' or r.get('scoring_identity')!='natural-language-e2e-scoring-v23-metric-locked-v11': reasons.append('report identity mismatch')
    if r.get('coordinate_role')!=role: reasons.append('coordinate role mismatch')
    if (r.get('product_coordinate') or {}).get('commit')!=coordinate: reasons.append('product coordinate mismatch')
    if r.get('seed')!=80000 or r.get('max_tokens')!=1024: reasons.append('seed/max-token mismatch')
    if a.get('completed_cases')!=13 or a.get('operational_failures')!=0 or r.get('operational_completeness_passed') is not True or r.get('report_gate_passed') is not True: reasons.append('row not operationally complete')
    for f in ZERO:
        if int(a.get(f) or 0)!=0: reasons.append(f'{f} must be zero')
    trig=int(a.get('trigger_exposed_cases') or 0)
    if trig>0 and a.get('mechanism_conformance_rate')!=1.0: reasons.append('#249 mechanism conformance must be 1.0 when exposed')
    return reasons

def compare_pair(control,candidate):
    coord=key(control); reasons=[]
    reasons += [f'control: {x}' for x in validate_report(control,'control',CONTROL)]
    reasons += [f'candidate: {x}' for x in validate_report(candidate,'candidate',CANDIDATE)]
    ca=control.get('aggregate') or {}; na=candidate.get('aggregate') or {}
    if na.get('target_recall') is None or ca.get('target_recall') is None or na['target_recall']<ca['target_recall']: reasons.append('target recall regressed vs paired control')
    if na.get('tool_selection_success_rate') is None or ca.get('tool_selection_success_rate') is None or na['tool_selection_success_rate']<ca['tool_selection_success_rate']: reasons.append('tool selection regressed vs paired control')
    if na.get('false_abstentions') is None or ca.get('false_abstentions') is None or int(na['false_abstentions'])>int(ca['false_abstentions']): reasons.append('false abstentions worsened vs paired control')
    cs=int(ca.get('avoidable_followup_stalls') or 0); ns=int(na.get('avoidable_followup_stalls') or 0)
    ct=int(ca.get('trigger_exposed_cases') or 0); nt=int(na.get('trigger_exposed_cases') or 0)
    ceiling=(cs==0 and ct==3)
    if ceiling:
        if ns!=0 or nt!=3: reasons.append('paired control is at utility ceiling; candidate must exactly preserve 0 stalls and 3 triggers')
    else:
        if ns>cs: reasons.append('avoidable follow-up stalls regressed vs paired control')
        if nt<ct: reasons.append('v11-defined trigger reachability regressed vs paired control')
        if not (ns<cs or nt>ct): reasons.append('no strict improvement in locked follow-up utility metrics')
    return {'coordinate':coord,'passed':not reasons,'reasons':reasons,'control_metrics':metrics(ca),'candidate_metrics':metrics(na),'delta':{'avoidable_followup_stalls':ns-cs,'trigger_exposed_cases':nt-ct,'target_recall':None if ca.get('target_recall') is None or na.get('target_recall') is None else na['target_recall']-ca['target_recall'],'tool_selection_success_rate':None if ca.get('tool_selection_success_rate') is None or na.get('tool_selection_success_rate') is None else na['tool_selection_success_rate']-ca['tool_selection_success_rate'],'false_abstentions':None if ca.get('false_abstentions') is None or na.get('false_abstentions') is None else int(na['false_abstentions'])-int(ca['false_abstentions'])}}

def metrics(a):
    return {k:a.get(k) for k in ['target_recall','tool_selection_success_rate','false_abstentions','avoidable_followup_stalls','trigger_exposed_cases','mechanism_conformance_rate','harness_precedence_selections','model_selected_action_calls','action_rejections','action_rejection_count','action_rejection_diagnostic_records','precedence_skip_reasons']}

def groq_row(r):
    reasons=validate_report(r,'candidate',CANDIDATE); a=r.get('aggregate') or {}
    if key(r)!=GROQ: reasons.append('wrong Groq model')
    if a.get('investigation_cases')!=10 or a.get('session_cases')!=3: reasons.append('Groq generic path did not cover 10 investigation + 3 session cases')
    if int(a.get('provider_calls_observed') or 0)<=0: reasons.append('Groq generic generation path not observed')
    if int(a.get('tool_calls') or 0)<=0: reasons.append('Groq investigation action path not observed')
    return {'coordinate':GROQ,'passed':not reasons,'reasons':reasons,'candidate_metrics':metrics(a)}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--report',action='append',default=[]); ap.add_argument('--pair-control'); ap.add_argument('--pair-candidate'); ap.add_argument('--validate-only',action='store_true'); ap.add_argument('--output'); args=ap.parse_args()
    if args.validate_only:
        out={'schema_version':'natural-language-e2e-v23-acceptance-v1','valid':True,'live_results_evaluated':False,'control_commit':CONTROL,'candidate_commit':CANDIDATE,'paired_models':PAIRED,'candidate_only_provider_parity_model':GROQ,'metric_semantics_locked_from':'natural-language-e2e-v11','no_cross_model_averaging':True}
    elif args.pair_control or args.pair_candidate:
        if not (args.pair_control and args.pair_candidate) or args.report: raise AcceptanceError('pair-only mode requires exactly --pair-control and --pair-candidate')
        control=load(args.pair_control); candidate=load(args.pair_candidate)
        if key(control)!=key(candidate) or key(control) not in PAIRED: raise AcceptanceError('pair-only reports must be the same required paired model')
        row=compare_pair(control,candidate)
        out={'schema_version':'natural-language-e2e-v23-acceptance-v1','valid':True,'live_results_evaluated':True,'pair_only':True,'control_commit':CONTROL,'candidate_commit':CANDIDATE,'metric_semantics_locked_from':'natural-language-e2e-v11','no_cross_model_averaging':True,'paired_row':row,'paired_gate_passed':row['passed']}
    else:
        reports=[load(p) for p in args.report]; by={report_id(r):r for r in reports}
        if len(reports)!=len(by): raise AcceptanceError('duplicate report identity')
        required=[f'control:{x}' for x in PAIRED]+[f'candidate:{x}' for x in PAIRED]+[f'candidate:{GROQ}']
        missing=[x for x in required if x not in by]; extra=[x for x in by if x not in required]
        if missing or extra: raise AcceptanceError(f'exact frozen result set required; missing={missing}, extra={extra}')
        rows=[compare_pair(by[f'control:{x}'],by[f'candidate:{x}']) for x in PAIRED]
        groq=groq_row(by[f'candidate:{GROQ}'])
        out={'schema_version':'natural-language-e2e-v23-acceptance-v1','valid':True,'live_results_evaluated':True,'control_commit':CONTROL,'candidate_commit':CANDIDATE,'metric_semantics_locked_from':'natural-language-e2e-v11','no_cross_model_averaging':True,'paired_rows':rows,'groq_row':groq,'release_gate_passed':all(x['passed'] for x in rows) and groq['passed']}
    text=json.dumps(out,indent=2,sort_keys=True); print(text)
    if args.output: Path(args.output).write_text(text+'\n')
    return 0 if not out.get('live_results_evaluated') or out.get('release_gate_passed') or out.get('paired_gate_passed') else 4
if __name__=='__main__':
    try: raise SystemExit(main())
    except AcceptanceError as e:
        print(json.dumps({'schema_version':'natural-language-e2e-v23-acceptance-v1','valid':False,'error':str(e)})); raise SystemExit(2)
