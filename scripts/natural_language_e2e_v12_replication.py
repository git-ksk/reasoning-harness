#!/usr/bin/env python3
"""Cross-model replication runner for the immutable v0.4.2 v12 release gate."""
from __future__ import annotations
import argparse, json, os, time
from pathlib import Path
import natural_language_e2e_v12 as v12

REPLICATION_ID="natural-language-e2e-v12-cross-model-v1"
DEFAULT_MANIFEST=Path("fixtures/natural-language-e2e-v12-cross-model-v1/manifest.json")
DEFAULT_FIXTURES=Path("fixtures/natural-language-e2e-v12")
PRODUCT_COMMIT="2d53a27d5ea0e2eb28bba355496db1f2b513f6a7"
FREEZE_TAG="natural-language-e2e-v12-freeze-r2"

def load_replication_manifest(path:Path)->dict:
    m=json.loads(path.read_text())
    if m.get('schema_version')!='natural-language-e2e-v12-cross-model-manifest-v1': raise v12.EvalError('replication manifest schema mismatch')
    if m.get('replication_identity')!=REPLICATION_ID: raise v12.EvalError('replication identity mismatch')
    ref=m.get('reference') or {}
    expected_ref={'corpus_identity':v12.CORPUS_ID,'evaluator_identity':v12.EVALUATOR_ID,'scoring_identity':v12.SCORING_ID,'freeze_tag':FREEZE_TAG,'product_commit':PRODUCT_COMMIT,'canonical_provider':'mistral','canonical_model':'ministral-8b-latest'}
    if {k:ref.get(k) for k in expected_ref}!=expected_ref: raise v12.EvalError('replication reference drift')
    p=m.get('semantic_policy') or {}
    expected={'base_seed':61000,'max_tokens':1024,'inter_case_delay_ms':1500,'reuse_reference_evaluator_and_scoring':True,'precedence_trigger_requires_harness_selection':True,'zero_trigger_denominator_is_inconclusive':True,'provider_specific_transport_pacing_allowed':True,'cross_provider_global_serialisation_forbidden':True}
    if {k:p.get(k) for k in expected}!=expected: raise v12.EvalError('replication semantic policy drift')
    targets=m.get('targets') or []
    expected_targets=[('google','gemma-4-31b-it','google',1),('google','gemini-3.5-flash-lite','google',1),('groq','openai/gpt-oss-120b','groq',1)]
    observed=[(x.get('provider'),x.get('model'),x.get('lane'),x.get('max_parallel_in_lane')) for x in targets]
    if observed!=expected_targets: raise v12.EvalError('replication targets/concurrency drift')
    if (ref['canonical_provider'],ref['canonical_model']) in {(x['provider'],x['model']) for x in targets}: raise v12.EvalError('canonical Mistral coordinate must not be rerun')
    return m

def allowed_target(m,provider,model): return any(x['provider']==provider and x['model']==model for x in m['targets'])

def run(args):
    repl=load_replication_manifest(Path(args.replication_manifest)); root=Path(args.fixtures)
    reference,cases=v12.validate_corpus(root)
    if (reference.get('product_coordinate') or {}).get('commit')!=PRODUCT_COMMIT: raise v12.EvalError('v12 product coordinate drift')
    if args.validate_only:
        return {'schema_version':'reason-natural-language-e2e-v12-cross-model-v1','replication_identity':REPLICATION_ID,'valid':True,'live_observation_performed':False,'reference_evaluator_identity':v12.EVALUATOR_ID,'reference_scoring_identity':v12.SCORING_ID,'reference_freeze_tag':FREEZE_TAG,'reference_product_commit':PRODUCT_COMMIT,'cases':len(cases),'targets':repl['targets']}
    if args.preflight:
        return {'schema_version':'reason-natural-language-e2e-v12-cross-model-v1','replication_identity':REPLICATION_ID,'valid':True,'live_observation_performed':False,'model_used':False,'resolver_capabilities_checked':v12.self_test_resolvers(root,cases),'admission_contracts':v12.self_test_admission_contracts(root,cases),'cases':len(cases)}
    provider=args.provider; model=args.model
    if not provider or not model or not allowed_target(repl,provider,model): raise v12.EvalError('provider/model is not a frozen replication target')
    pol=repl['semantic_policy']; seed=args.seed if args.seed is not None else pol['base_seed']; max_tokens=args.max_tokens if args.max_tokens is not None else pol['max_tokens']; delay=args.inter_case_delay_ms if args.inter_case_delay_ms is not None else pol['inter_case_delay_ms']
    if (seed,max_tokens,delay)!=(pol['base_seed'],pol['max_tokens'],pol['inter_case_delay_ms']): raise v12.EvalError('replication semantic coordinate mismatch')
    if not args.reason_bin or not Path(args.reason_bin).exists(): raise v12.EvalError('--reason-bin must reference built reason executable')
    reason_bin=Path(args.reason_bin).resolve(); cwd=Path.cwd(); env=os.environ.copy(); reports=[]; boundary=False
    for idx,(_,case) in enumerate(cases):
        case_seed=seed+idx
        if not boundary:
            marker={'schema_version':'reason-natural-language-e2e-v12-cross-model-v1','replication_identity':REPLICATION_ID,'corpus_identity':v12.CORPUS_ID,'live_case_launch_boundary_entered':True,'case_id':case['id'],'provider':provider,'model':model,'seed':case_seed,'product_coordinate':reference['product_coordinate'],'reference_freeze_tag':FREEZE_TAG,'github_run_id':os.environ.get('GITHUB_RUN_ID'),'github_run_attempt':os.environ.get('GITHUB_RUN_ATTEMPT')}
            if args.attempt_marker: Path(args.attempt_marker).write_text(json.dumps(marker,indent=2,sort_keys=True)+'\n')
            boundary=True
        if case['kind']=='investigation':
            cmd=[str(reason_bin),case['task'],'--provider',provider,'--model',model,'--max-tokens',str(max_tokens),'--seed',str(case_seed),'--config',str((root/case['config']).resolve()),'--format','json']
            cp,payload,elapsed=v12.run_json(cmd,cwd,env)
            reports.append(v12.invocation_failure(case['id'],cp,payload,elapsed) if cp.returncode!=0 or not isinstance(payload,dict) or 'result' not in payload else v12.score_investigation(case,payload['result'],elapsed))
        else:
            reports.append(v12.run_session_case(case,reason_bin,provider,model,max_tokens,case_seed,cwd,env,delay))
        if idx+1<len(cases): time.sleep(delay/1000)
    agg=v12.aggregate(reports); gates=v12.evaluate_report_gates(reference,agg,len(reports),len(cases))
    return {'schema_version':v12.REPORT_SCHEMA,'replication_identity':REPLICATION_ID,'cross_model_replication':True,'corpus_identity':v12.CORPUS_ID,'evaluator_identity':v12.EVALUATOR_ID,'scoring_identity':v12.SCORING_ID,'live_observation_performed':True,'provider':provider,'model':model,'seed':seed,'max_tokens':max_tokens,'inter_case_delay_ms':delay,'raw_model_comparison_claimed':False,'product_coordinate':reference['product_coordinate'],'reference_freeze_tag':FREEZE_TAG,'reference_provider_policy':reference['provider_policy'],'mcp_policy':reference['mcp_policy'],'measurement_semantics':reference['measurement_semantics'],'cases':reports,'aggregate':agg,**gates}

def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--replication-manifest',default=str(DEFAULT_MANIFEST)); ap.add_argument('--fixtures',default=str(DEFAULT_FIXTURES)); ap.add_argument('--reason-bin'); ap.add_argument('--provider'); ap.add_argument('--model'); ap.add_argument('--seed',type=int); ap.add_argument('--max-tokens',type=int); ap.add_argument('--inter-case-delay-ms',type=int); ap.add_argument('--attempt-marker'); ap.add_argument('--validate-only',action='store_true'); ap.add_argument('--preflight',action='store_true'); ap.add_argument('--output'); args=ap.parse_args(); result=run(args); text=json.dumps(result,indent=2,sort_keys=True); print(text); args.output and Path(args.output).write_text(text+'\n'); return 0 if not result.get('live_observation_performed') or result.get('report_gate_passed') else 3

if __name__=='__main__':
    try: raise SystemExit(main())
    except v12.EvalError as exc:
        print(json.dumps({'schema_version':'reason-natural-language-e2e-v12-cross-model-v1','valid':False,'error':str(exc)})); raise SystemExit(2)
