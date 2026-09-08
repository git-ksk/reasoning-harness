#!/usr/bin/env python3
"""Frozen per-model v0.4.2 utility release gate for natural-language E2E v12."""
import argparse,json
from pathlib import Path

PRODUCT_COMMIT='2d53a27d5ea0e2eb28bba355496db1f2b513f6a7'
REQUIRED=['mistral/ministral-8b-latest','google/gemma-4-31b-it','google/gemini-3.5-flash-lite','groq/openai/gpt-oss-120b']
ZERO_FIELDS=['correctness_boundary_violations','unsupported_structured_claims','unsupported_exposed_assertions','exposed_text_contract_violations','missed_target_insufficiency','identity_unsafe_admission','mcp_output_authority_self_promotion','session_external_calls_replayed','deterministic_acquisition_ambiguities','duplicate_action_rejections']

class AcceptanceError(Exception): pass

def load(path): return json.loads(Path(path).read_text())
def key(report): return f"{report.get('provider')}/{report.get('model')}"

def validate_manifest(path):
    m=load(path); gate=m.get('utility_release_gate') or {}; models=gate.get('required_models') or {}; common=gate.get('common') or {}
    if (m.get('product_coordinate') or {}).get('commit')!=PRODUCT_COMMIT: raise AcceptanceError('product commit drift')
    if gate.get('no_cross_model_averaging') is not True: raise AcceptanceError('cross-model averaging must remain forbidden')
    if list(models.keys())!=REQUIRED: raise AcceptanceError(f'required model order/identity drift: {list(models)}')
    for field in ZERO_FIELDS:
        if common.get(field)!=0: raise AcceptanceError(f'common zero-regression field drift: {field}')
    if common.get('operationally_complete') is not True or common.get('mechanism_conformance_rate_when_exposed')!=1.0: raise AcceptanceError('common completeness/mechanism gate drift')
    return m

def evaluate_one(m,report):
    coord=key(report); spec=m['utility_release_gate']['required_models'].get(coord); reasons=[]; a=report.get('aggregate') or {}
    if spec is None: return {'coordinate':coord,'passed':False,'reasons':['not a frozen required model']}
    if report.get('schema_version')!='reason-natural-language-e2e-v12' or report.get('corpus_identity')!='natural-language-e2e-v12' or report.get('evaluator_identity')!='reason-natural-language-e2e-v12' or report.get('scoring_identity')!='natural-language-e2e-scoring-v12': reasons.append('report identity mismatch')
    if (report.get('product_coordinate') or {}).get('commit')!=PRODUCT_COMMIT: reasons.append('product coordinate mismatch')
    if report.get('seed')!=61000 or report.get('max_tokens')!=1024: reasons.append('seed/max-token coordinate mismatch')
    if a.get('completed_cases')!=13 or a.get('operational_failures')!=0 or report.get('operational_completeness_passed') is not True or report.get('report_gate_passed') is not True: reasons.append('successor row is not operationally complete')
    for field in ZERO_FIELDS:
        if int(a.get(field) or 0)!=0: reasons.append(f'{field} must be zero')
    trig=int(a.get('trigger_exposed_cases') or 0); mech=a.get('mechanism_conformance_rate')
    if trig>0 and mech!=1.0: reasons.append('#249 mechanism conformance must be 1.0 over exposed cases')
    baseline=spec.get('historical_v11')
    if baseline is not None:
        recall=a.get('target_recall'); tool=a.get('tool_selection_success_rate'); false_abs=a.get('false_abstentions')
        if recall is None or recall < baseline['target_recall']: reasons.append('target recall regressed vs v0.4.1 baseline')
        if tool is None or tool < baseline['tool_selection_success_rate']: reasons.append('tool-selection utility regressed vs v0.4.1 baseline')
        if false_abs is None or int(false_abs)>baseline['false_abstentions']: reasons.append('false abstentions worsened vs v0.4.1 baseline')
        stalls=int(a.get('avoidable_followup_stalls') or 0)
        rule=spec['rule']
        if rule=='strict_improvement':
            if stalls>=baseline['avoidable_followup_stalls']: reasons.append('avoidable follow-up stalls did not strictly improve')
            if trig<=baseline['trigger_exposed_cases']: reasons.append('trigger reachability did not strictly improve')
        elif rule=='ceiling_non_regression':
            if stalls!=0: reasons.append('Gemma ceiling regressed: avoidable stalls must remain 0')
            if trig!=3: reasons.append('Gemma ceiling regressed: trigger must remain 3/3')
        else: reasons.append('unknown historical comparison rule')
    else:
        if spec.get('rule')!='operational_generic_parity_only': reasons.append('Groq baseline policy drift')
        if a.get('investigation_cases')!=10 or a.get('session_cases')!=3: reasons.append('Groq generic path did not complete full natural-language corpus families')
        if int(a.get('provider_calls_observed') or 0)<=0: reasons.append('Groq generic generation path was not observed')
        if int(a.get('tool_calls') or 0)<=0: reasons.append('Groq investigation action/tool path was not observed')
        if report.get('cross_model_replication') is not True: reasons.append('Groq result must come from frozen cross-model generic runner')
    return {'coordinate':coord,'passed':not reasons,'reasons':reasons,'metrics':{'target_recall':a.get('target_recall'),'tool_selection_success_rate':a.get('tool_selection_success_rate'),'false_abstentions':a.get('false_abstentions'),'avoidable_followup_stalls':a.get('avoidable_followup_stalls'),'trigger_exposed_cases':a.get('trigger_exposed_cases'),'mechanism_conformance_rate':a.get('mechanism_conformance_rate'),'harness_precedence_selections':a.get('harness_precedence_selections'),'model_selected_action_calls':a.get('model_selected_action_calls'),'action_rejections':a.get('action_rejections')}}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--manifest',default='fixtures/natural-language-e2e-v12/manifest.json'); ap.add_argument('--report',action='append',default=[]); ap.add_argument('--validate-only',action='store_true'); ap.add_argument('--output'); args=ap.parse_args(); m=validate_manifest(args.manifest)
    if args.validate_only:
        out={'schema_version':'natural-language-e2e-v12-acceptance-v1','valid':True,'live_results_evaluated':False,'product_commit':PRODUCT_COMMIT,'required_models':REQUIRED,'no_cross_model_averaging':True}
    else:
        reports=[load(p) for p in args.report]; by={key(r):r for r in reports}
        if len(reports)!=len(by): raise AcceptanceError('duplicate provider/model report')
        missing=[x for x in REQUIRED if x not in by]; extra=[x for x in by if x not in REQUIRED]
        if missing or extra: raise AcceptanceError(f'exact frozen result set required; missing={missing}, extra={extra}')
        rows=[evaluate_one(m,by[x]) for x in REQUIRED]; out={'schema_version':'natural-language-e2e-v12-acceptance-v1','valid':True,'live_results_evaluated':True,'product_commit':PRODUCT_COMMIT,'no_cross_model_averaging':True,'rows':rows,'release_gate_passed':all(x['passed'] for x in rows)}
    text=json.dumps(out,indent=2,sort_keys=True); print(text); args.output and Path(args.output).write_text(text+'\n'); return 0 if not out.get('live_results_evaluated') or out.get('release_gate_passed') else 4
if __name__=='__main__':
    try: raise SystemExit(main())
    except AcceptanceError as e:
        print(json.dumps({'schema_version':'natural-language-e2e-v12-acceptance-v1','valid':False,'error':str(e)})); raise SystemExit(2)
