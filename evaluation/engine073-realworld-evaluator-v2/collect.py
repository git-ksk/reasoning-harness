#!/usr/bin/env python3
"""Collect new first-run bytes; never inspect or rescore preserved experiments."""
import argparse,hashlib,json
from pathlib import Path
from capture import bind_normal,bind_helper
from scorer import sha

def read(p):return json.loads(Path(p).read_text())
def lines(p):return [json.loads(x) for x in Path(p).read_text().splitlines()] if Path(p).exists() else []
def dump(p,v):
    with Path(p).open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n')
def main():
    p=argparse.ArgumentParser();p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--provider',required=True);p.add_argument('--lane',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    a.output.mkdir();blob=(a.corpus_dir/'corpus.json').read_bytes();corpus=json.loads(blob);corpus['_capture_sha256']=sha(blob)
    gold=read(a.corpus_dir/'gold.json');plan=read(a.lane/'plan.json');summary=read(a.lane/'summary.json') if (a.lane/'summary.json').exists() else (read(a.lane/'state.json') if (a.lane/'state.json').exists() else {'rows':[]});rows={r['id']:r for r in summary['rows']}
    if a.provider!=plan['provider']:raise ValueError('collector_provider_differs_from_frozen_lane')
    observations=[];failures=[]
    for job in plan['jobs']:
        row=rows.get(job['id'],{'complete':False,'operational_failure':'parent_terminated_before_observation'});directory=a.lane/job['id'];cid=job['argv'][job['argv'].index('--case')+1];track='e2e_normal' if job['contract']=='normal' else 'helper_opt_in'
        arms=[job['arm']] if track=='e2e_normal' else ['baseline','candidate']
        try:output=read(directory/'stdout.json') if (directory/'stdout.json').exists() else {}
        except (ValueError,OSError):output={}
        bound=None;reason=None
        corpus['_expected_case_id']=cid
        if row['complete']:
            try:
                if track=='e2e_normal':bound={job['arm']:bind_normal(output,read(directory/'trace.json'),lines(directory/'resolver.jsonl'),read(directory/'config.json'),corpus,lines(directory/'model-journal.jsonl'),plan['provider'],plan['model'])}
                else:bound=bind_helper(output,corpus,plan['provider'],plan['model'])
            except (KeyError,ValueError,TypeError,FileNotFoundError) as e:
                reason='capture_binding_failed:'+type(e).__name__+':'+str(e);failures.append({'job_id':job['id'],'failure':reason})
        else:reason=row.get('operational_failure') or row.get('admission',{}).get('reason') or 'operational_incomplete'
        for arm in arms:
            opaque=sha((plan['M1_sha256']+'|'+a.provider+'|'+cid+'|'+track+'|'+arm).encode())[:24]
            partial=output
            if not bound:
                raw_file=directory/('actual-cli.stdout' if track=='e2e_normal' else 'source-host-'+arm+'.stdout')
                if raw_file.exists():
                    try:partial=read(raw_file)
                    except (ValueError,OSError):pass
                if track=='e2e_normal':
                    text=partial.get('result',{}).get('finalization',{}).get('text') or ''
                else:
                    raw=partial.get(arm,partial)
                    text=(raw.get('original',{}) if arm=='baseline' else next(iter(raw.get('view',{}).get('target_answers',[])),{}).get('original',{})).get('text') or ''
                if not isinstance(text,str):text=''
                observed={'display_artifacts':[{'id':'answer','kind':'final_answer','text':text}],
                    'display_citations':[],'acquired_source_ids':[],'admitted_source_ids':[],
                    'terminal_status':'unavailable','usage':None,'unvalidated_partial_payload':partial,
                    'source_route_verification':'UNAVAILABLE_DO_NOT_INFER_UNSUPPORTED_FROM_UNKNOWN'}
            else:observed=bound[arm]
            operation='complete' if bound else ('not_run' if row.get('admission',{}).get('allowed') is False else 'incomplete')
            hashes={str(f.relative_to(directory)):sha(f.read_bytes()) for f in sorted(directory.rglob('*')) if f.is_file() and 'private-reason-home' not in f.parts}
            obs={**observed,'row_id':opaque,'case_id':cid,'provider':a.provider,'track':track,'arm':arm,'operation':operation,'unavailable_reason':reason,
                 'raw_file_sha256':hashes,'wall_ms':row.get('wall_ms'),'failure_partial_display_audit_required':not bool(bound)}
            dump(a.output/(opaque+'.json'),obs);observations.append(obs)
            g=gold['records'][cid+'/'+track] if isinstance(gold['records'],dict) else next(x for x in gold['records'] if x['id']==cid+'/'+track)
            # Review packet omits provider/arm/build metadata. The observation
            # digest still binds the complete private record, and raw files are
            # available if semantic adjudication requires an unblinded audit.
            record={k:v for k,v in g.items()};record['sha256']=sha(json.dumps(g,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
            packet={'row_id':opaque,'observation_sha256':sha(json.dumps(obs,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()),'gold':record,
                    'display_artifacts':obs['display_artifacts'],'display_citations':obs['display_citations'],'admitted_source_ids':obs['admitted_source_ids'],
                    'acquired_source_ids':obs['acquired_source_ids'],'terminal_status':obs['terminal_status'],'operation':operation,
                    'admission_routes':obs.get('admission_routes',{}),'full_sources':{sid:corpus['sources'][sid] for sid in next(c for c in corpus['cases'] if c['id']==cid)['source_ids']},'unvalidated_partial_payload':obs.get('unvalidated_partial_payload')}
            dump(a.output/(opaque+'.review-packet.json'),packet)
    dump(a.output/'observation-index.json',{'provider':a.provider,'observations':len(observations),'binding_failures':failures,'actual_answer_f1':None,'semantic_review':'PENDING'})
    print(json.dumps({'observations':len(observations),'binding_failures':len(failures),'semantic_score':'NOT_YET_SCORED'}))
if __name__=='__main__':main()
