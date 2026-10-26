#!/usr/bin/env python3
"""Exclusive first scoring after external semantic annotation, with runtime gates."""
import argparse,json,math,os,statistics
from pathlib import Path
from scorer import frozen_dependencies,sha,validate_review,summarize,percentile95
from development import consumed_corpus,INPUT_M1,PURPOSE

HERE=Path(__file__).resolve().parent;REPO=HERE.parents[1]
def read(p):return json.loads(Path(p).read_text())
def dump(p,v):
    with Path(p).open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())
def main():
    p=argparse.ArgumentParser();p.add_argument('--development-receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--M1-sha256',required=True);p.add_argument('--observations',type=Path,required=True);p.add_argument('--reviews',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--analysis-freeze',type=Path,required=True);p.add_argument('--analysis-freeze-sha256',required=True);a=p.parse_args()
    receipt_path=a.development_receipt
    # The immutable execution receipt and new annotation manifests are checked
    # before exclusive scoring output is created. No prior experiment is loaded.
    m0=read(receipt_path)
    assert m0['identity']=='cli-repair-paired-v1-development-freeze-v1' and m0['state']=='DEVELOPMENT_FROZEN'
    assert m0['purpose']==PURPOSE and m0['accuracy_acceptance'] is False and m0['fresh_accuracy_cases']==0
    assert a.M1_sha256==m0['consumed_M1_sha256']==INPUT_M1
    assert sha(Path(__file__).read_bytes())==m0['development_scoring_driver_sha256']
    runtime_files={str(p.relative_to(REPO)):sha(p.read_bytes()) for p in REPO.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    assert runtime_files==m0['files']
    assert sha(a.analysis_freeze.read_bytes())==a.analysis_freeze_sha256
    freeze=read(a.analysis_freeze)
    assert freeze['identity']=='cli-repair-paired-real24-first-score-v1' and freeze['reviewer']=='/root/paired_evaluation_independent'
    assert freeze['execution_receipt_sha256']==sha(receipt_path.read_bytes()) and freeze['first_scoring_only'] is True
    for directory,key in [(a.observations,'observations_inventory'),(a.reviews,'reviews_inventory')]:
        assert {str(p.relative_to(directory)):sha(p.read_bytes()) for p in directory.rglob('*') if p.is_file()}==freeze[key]
    a.output.mkdir();dump(a.output/'first-score-started.json',{'M1_sha256':a.M1_sha256,'rescore_or_relabel':False})
    data=consumed_corpus(a.corpus_dir);gold=read(a.corpus_dir/'gold.json');protocol=read(HERE/'protocol.json');deps=frozen_dependencies(REPO)
    records={r['id']:r for r in gold['records']} if isinstance(gold['records'],list) else gold['records'];planned=[];scored=[];obs_index={};errors=[]
    for provider in protocol['providers']:
        for case in data['cases']:
            for track in protocol['tracks']:
                g=records[case['id']+'/'+track]
                for arm in ['baseline','candidate']:
                    opaque=sha((a.M1_sha256+'|'+provider+'|'+case['id']+'|'+track+'|'+arm).encode())[:24]
                    planned.append({'id':case['id'],'row_id':opaque,'provider':provider,'track':track,'arm':arm,'expected':g['label'],'family':g['family'],'required_groups_G':len(g['groups'])})
                    op=a.observations/(opaque+'.json');rp=a.reviews/(opaque+'.json')
                    if not op.exists() or not rp.exists():continue
                    observation=read(op);review=read(rp)
                    if (observation['provider'],observation['track'],observation['arm'],observation['case_id'])!=(provider,track,arm,case['id']):raise ValueError('observation plan mismatch')
                    record={**g,'sha256':sha(json.dumps(g,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())}
                    if review.get('reviewer')!={'role':'agent','id':m0['semantic_reviewer_id']}:raise ValueError('unexpected_semantic_reviewer')
                    try:scored.append(validate_review(record,observation,review,data['sources'],deps));obs_index[(provider,track,arm,case['id'])]=observation
                    except (ValueError,KeyError,TypeError) as e:errors.append({'row_id':opaque,'error':type(e).__name__+':'+str(e)})
    report=summarize(scored,planned,protocol,deps);runtime={};runtime_pass=True
    for provider in protocol['providers']:
        pairs=[]
        for case in data['cases']:
            b=obs_index.get((provider,'e2e_normal','baseline',case['id']));c=obs_index.get((provider,'e2e_normal','candidate',case['id']))
            if b and c and b['operation']==c['operation']=='complete':pairs.append((b,c))
        ratios={}
        for name,extract in [('mean_logical_calls',lambda x:x['usage']['logical_model_operations']),('aggregate_total_tokens',lambda x:x['usage']['total_tokens']),('p95_wall_ratio',lambda x:x['wall_ms'])]:
            reducer=percentile95 if name.startswith('p95') else (statistics.mean if name.startswith('mean') else sum)
            base=reducer([extract(b) for b,c in pairs]) if pairs else None;cand=reducer([extract(c) for b,c in pairs]) if pairs else None
            ratios[name]=(cand-base if name=='mean_logical_calls' else cand/base) if base is not None and base>0 and cand is not None else None
        ok=len(pairs)==24 and ratios['mean_logical_calls'] is not None and ratios['mean_logical_calls']<=1 and ratios['aggregate_total_tokens'] is not None and ratios['aggregate_total_tokens']<=1.25 and ratios['p95_wall_ratio'] is not None and ratios['p95_wall_ratio']<=1.5
        runtime[provider]={'paired_complete_cases':len(pairs),'ratios':ratios,'efficiency_gate_pass':ok};runtime_pass &= ok
    report.update(source_denominators={'cases':len(data['cases']),'unique_documents':len(data['documents']),'unique_products':len({d['product'] for d in data['documents'].values()}),'origin_groups':len({s['origin_group'] for s in data['sources'].values()})},runtime_efficiency=runtime,annotation_failures=errors,semantic_review_authority='independent agent, no human attestation',small_sample='descriptive correlated24-task sample; no population accuracy guarantee')
    report['acceptance']='NOT_ELIGIBLE_CONSUMED_DEVELOPMENT_DIAGNOSTIC'
    report['measurement_scope']=PURPOSE
    report['accuracy_acceptance']=False
    report['fresh_accuracy_cases']=0
    report['declared_quality_gates_pass']=bool(report['semantic_gate_pass'] and runtime_pass and not errors)
    report['execution_receipt_sha256']=sha(receipt_path.read_bytes())
    report['source_merge_release_CLI_distribution']='SEPARATE_GATES_NOT_APPROVED_BY_THIS_REPORT'
    dump(a.output/'rows.json',scored);dump(a.output/'report.json',report)
    print(json.dumps({'acceptance':report['acceptance'],'providers':report['provider_tracks'],'annotation_failures':len(errors)},ensure_ascii=False))
if __name__=='__main__':main()
