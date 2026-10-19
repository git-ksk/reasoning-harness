#!/usr/bin/env python3
"""Exclusive first scoring after external semantic annotation, with runtime gates."""
import argparse,json,math,os,statistics
from pathlib import Path
from scorer import frozen_dependencies,sha,validate_review,summarize,percentile95
from corpus import validate,verify_provenance

HERE=Path(__file__).resolve().parent;REPO=HERE.parents[1]
def read(p):return json.loads(Path(p).read_text())
def dump(p,v):
    with Path(p).open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())
def main():
    p=argparse.ArgumentParser();p.add_argument('--M0-receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--M1-sha256',required=True);p.add_argument('--observations',type=Path,required=True);p.add_argument('--reviews',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    a.output.mkdir();dump(a.output/'first-score-started.json',{'M1_sha256':a.M1_sha256,'rescore_or_relabel':False})
    m0=verify_provenance(a.corpus_dir,a.M0_receipt);data,gold,_=validate(a.corpus_dir,a.M1_sha256);protocol=read(HERE/'protocol.json');deps=frozen_dependencies(REPO)
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
    report['acceptance']='GO_FOR_DECLARED_QUALITY_SCOPE_ONLY' if report['semantic_gate_pass'] and runtime_pass and not errors else 'NO_GO'
    report['source_merge_release_CLI_distribution']='SEPARATE_GATES_NOT_APPROVED_BY_THIS_REPORT'
    dump(a.output/'rows.json',scored);dump(a.output/'report.json',report)
    print(json.dumps({'acceptance':report['acceptance'],'providers':report['provider_tracks'],'annotation_failures':len(errors)},ensure_ascii=False))
if __name__=='__main__':main()
