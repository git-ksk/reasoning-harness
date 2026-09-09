#!/usr/bin/env python3
import argparse, json
from pathlib import Path

CONTROL='29a9e4be6273dbffeda324e15517dc64930ad315'
CANDIDATE='c804551b381f48f14809f2ab37d0609c33006ddf'
LOCKED={
  'followup_observational_cases':3,
  'trigger_exposed_definition':'first executed configured cache action returns typed no_result',
  'mechanism_denominator':'trigger_exposed_cases_only',
  'zero_trigger_mechanism_classification':'inconclusive',
  'mechanism_success_definition':'immediate next action is configured registry and harness_no_result_followup_selections increments for exact-target continuation',
  'downstream_utility_scored_separately':True,
  'trigger_miss_is_measurement_data_not_mechanism_failure':True,
  'measurement_validity_does_not_require_utility_success':True,
}
class PairError(Exception): pass

def load(p): return json.loads(Path(p).read_text())
def scrub(obj):
    obj=json.loads(json.dumps(obj))
    for cap in obj.get('resolution',{}).get('investigation',{}).get('capabilities',[]):
        cap.pop('selection_priority',None)
        if cap.get('kind')=='mcp_readonly':
            fixed=cap.get('fixed_arguments') or {}
            if fixed.get('ref') not in (CONTROL,CANDIDATE): raise PairError('bad MCP ref')
            fixed['ref']='<COORDINATE_COMMIT>'
    return obj

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--fixtures',default='fixtures/natural-language-e2e-v25'); args=ap.parse_args()
    root=Path(args.fixtures); m=load(root/'manifest.json')
    if m.get('schema_version')!='natural-language-e2e-manifest-v5': raise PairError('manifest schema')
    sem=m.get('measurement_semantics') or {}
    for k,v in LOCKED.items():
        if sem.get(k)!=v: raise PairError(f'v11 metric semantic drift: {k}')
    if sem.get('locked_from')!='natural-language-e2e-v11': raise PairError('metric lock origin drift')
    if sem.get('action_rejection_records_are_diagnostic_only') is not True or sem.get('precedence_skip_reasons_are_diagnostic_only') is not True: raise PairError('v25 diagnostic-only telemetry policy drift')
    if sem.get('diagnostic_trace_is_non_scoring_sidecar') is not True: raise PairError('diagnostic trace must remain non-scoring')
    coords=m.get('paired_coordinates') or {}
    if (coords.get('control') or {}).get('commit')!=CONTROL or (coords.get('candidate') or {}).get('commit')!=CANDIDATE: raise PairError('coordinate drift')
    checked=0; priority_fields=0; mcp_pairs=0
    for name in m.get('case_files') or []:
        c=load(root/name)
        if c.get('kind')!='investigation': continue
        pair=c.get('config_pair') or {}
        ctrl=load(root/pair['control']); cand=load(root/pair['candidate'])
        if scrub(ctrl)!=scrub(cand): raise PairError(f'{c["id"]}: semantic config drift beyond allowlist')
        ccaps=ctrl['resolution']['investigation']['capabilities']; dcaps=cand['resolution']['investigation']['capabilities']
        if len(ccaps)!=len(dcaps): raise PairError(f'{c["id"]}: capability count drift')
        for a,b in zip(ccaps,dcaps):
            if a.get('id')!=b.get('id'): raise PairError(f'{c["id"]}: capability identity drift')
            if 'selection_priority' in a: raise PairError(f'{c["id"]}: control unexpectedly contains selection_priority')
            if 'selection_priority' in b: priority_fields+=1
            if a.get('kind')=='mcp_readonly':
                if a['fixed_arguments']['ref']!=CONTROL or b['fixed_arguments']['ref']!=CANDIDATE: raise PairError(f'{c["id"]}: MCP coordinate ref mismatch')
                mcp_pairs+=1
        checked+=1
    if priority_fields!=6: raise PairError(f'expected six candidate-only priorities, got {priority_fields}')
    if mcp_pairs!=1: raise PairError(f'expected one paired MCP coordinate, got {mcp_pairs}')
    print(json.dumps({'schema_version':'natural-language-e2e-v25-pair-validation-v1','valid':True,'paired_investigation_cases':checked,'candidate_only_selection_priority_fields':priority_fields,'paired_mcp_coordinate_refs':mcp_pairs,'metric_semantics_locked_from':'natural-language-e2e-v11'},indent=2,sort_keys=True))

if __name__=='__main__':
    try: main()
    except PairError as e:
        print(json.dumps({'schema_version':'natural-language-e2e-v25-pair-validation-v1','valid':False,'error':str(e)}))
        raise SystemExit(2)
