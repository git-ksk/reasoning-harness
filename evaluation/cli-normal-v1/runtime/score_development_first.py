#!/usr/bin/env python3
"""Exclusive first scoring after external semantic annotation, with runtime gates."""
import argparse,json,math,os,statistics,sys,tempfile
from pathlib import Path
sys.path.insert(1,str(Path(__file__).resolve().parent.parent))
from scorer import frozen_dependencies,sha,validate_review,summarize,percentile95,require
from development import consumed_corpus,INPUT_M1,PURPOSE
from provenance import REPOSITORY,fetch_verified
from collect_normal import reconstruct,binding_hash,new_id

HERE=Path(__file__).resolve().parent;REPO=HERE.parents[2]
# A single persistent mount is required; callers cannot choose another directory
# merely by choosing another output path. Deletion/other hosts remain outside
# this local first-observation guarantee.
FIRST_SCORE_ROOT=Path('/score-reservations')
def read(p):return json.loads(Path(p).read_text())
def dump(p,v):
    with Path(p).open('x') as f:json.dump(v,f,indent=2,ensure_ascii=False);f.write('\n');f.flush();os.fsync(f.fileno())
def verify_observation_files(observations,rebuilt,verified):
    observations=Path(observations);rebuilt=Path(rebuilt)
    require(read(observations/'verified-run.json')==verified,'verified_run_changed')
    actual={str(p.relative_to(observations)):p.read_bytes() for p in observations.rglob('*') if p.is_file() and str(p.relative_to(observations))!='verified-run.json'}
    expected={str(p.relative_to(rebuilt)):p.read_bytes() for p in rebuilt.rglob('*') if p.is_file()}
    require(actual==expected,'observations_not_from_exact_run_artifact')
def verify_review_binding(review,observation,binding,reviewer):
    observed_sha=sha(json.dumps(observation,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())
    require(review.get('row_id')==observation.get('row_id') and review.get('observation_sha256')==observed_sha
            and review.get('execution_provenance_sha256')==binding
            and review.get('reviewer')=={'role':'agent','id':reviewer},'review_not_for_exact_artifact')
def reserve_first_score(run_id,receipt_sha,artifact_id):
    require(FIRST_SCORE_ROOT.is_dir(),'persistent_score_reservation_mount_required')
    name=sha((REPOSITORY+'|'+str(run_id)+'|'+receipt_sha).encode())+'.json'
    path=FIRST_SCORE_ROOT/name
    with path.open('x') as f:
        json.dump({'repository':REPOSITORY,'run_id':run_id,'receipt_sha256':receipt_sha,'artifact_id':artifact_id},f,sort_keys=True)
        f.write('\n');f.flush();os.fsync(f.fileno())
    fd=os.open(FIRST_SCORE_ROOT,os.O_RDONLY)
    try:os.fsync(fd)
    finally:os.close(fd)
    return path

def main():
    p=argparse.ArgumentParser();p.add_argument('--development-receipt',type=Path,required=True);p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--M1-sha256',required=True);p.add_argument('--observations',type=Path,required=True);p.add_argument('--reviews',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--analysis-freeze',type=Path,required=True);p.add_argument('--analysis-freeze-sha256',required=True);p.add_argument('--run-id',type=int,required=True);p.add_argument('--artifact-id',type=int,required=True);a=p.parse_args()
    receipt_path=a.development_receipt
    # The immutable execution receipt and new annotation manifests are checked
    # before exclusive scoring output is created. No prior experiment is loaded.
    m0=read(receipt_path)
    require(m0.get('identity')=='cli-normal-v1-development-freeze-v1' and m0.get('state')=='DEVELOPMENT_FROZEN','wrong_frozen_receipt')
    require(m0.get('purpose')==PURPOSE and m0.get('accuracy_acceptance') is False and m0.get('fresh_accuracy_cases')==0,'receipt_scope_mismatch')
    require(a.M1_sha256==m0.get('consumed_M1_sha256')==INPUT_M1,'wrong_corpus')
    require(sha(Path(__file__).read_bytes())==m0['development_scoring_driver_sha256'],'scoring_driver_changed')
    runtime_files={str(p.relative_to(REPO)):sha(p.read_bytes()) for p in REPO.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    require(runtime_files==m0['files'],'runtime_inventory_changed')
    require(sha(a.analysis_freeze.read_bytes())==a.analysis_freeze_sha256,'analysis_freeze_changed')
    freeze=read(a.analysis_freeze)
    require(freeze.get('identity')=='cli-normal-real24-first-score-v1' and freeze.get('reviewer')=='/root/paired_evaluation_independent','analysis_freeze_identity')
    receipt_sha=sha(receipt_path.read_bytes())
    require(freeze.get('execution_receipt_sha256')==receipt_sha and freeze.get('first_scoring_only') is True,'analysis_receipt_mismatch')
    for directory,key in [(a.observations,'observations_inventory'),(a.reviews,'reviews_inventory')]:
        require({str(p.relative_to(directory)):sha(p.read_bytes()) for p in directory.rglob('*') if p.is_file()}==freeze[key],'analysis_inventory_changed')
    # Re-read the exact immutable Actions artifact from GitHub and reconstruct
    # every observation from its original lane. Caller-provided provenance or
    # a relabeled v2 observation can never substitute for this reconstruction.
    with tempfile.TemporaryDirectory(prefix='cli-normal-score-verify-') as temporary:
        root,verified=fetch_verified(REPOSITORY,a.run_id,a.artifact_id,receipt_path,Path(temporary)/'github')
        binding=binding_hash(verified)
        require(freeze.get('verified_run_sha256')==binding,'analysis_run_mismatch')
        rebuilt=Path(temporary)/'reconstructed'
        require(reconstruct(root,verified,a.corpus_dir,rebuilt)==binding,'reconstruction_binding_mismatch')
        verify_observation_files(a.observations,rebuilt,verified)
    data=consumed_corpus(a.corpus_dir);gold=read(a.corpus_dir/'gold.json');protocol=read(HERE/'protocol.json');deps=frozen_dependencies(REPO)
    require(protocol['required_providers']==['mistral'] and protocol['tracks']==['e2e_normal'],'wrong_profile')
    require(protocol['providers']['google']['new_live_run_allowed'] is False and protocol['providers']['groq']['new_live_run_allowed'] is False,'unapproved_provider')
    expected_ids={new_id(binding,sha((INPUT_M1+'|mistral|'+case['id']+'|e2e_normal|'+arm).encode())[:24])
                  for case in data['cases'] for arm in ['baseline','candidate']}
    review_files={str(path.relative_to(a.reviews)) for path in a.reviews.rglob('*') if path.is_file()}
    require(review_files <= {row_id+'.json' for row_id in expected_ids},'review_inventory_has_foreign_rows')
    for name in review_files:
        review=read(a.reviews/name);observation=read(a.observations/name)
        verify_review_binding(review,observation,binding,m0['semantic_reviewer_id'])
    require(not a.output.exists(),'score_output_exists')
    reserve_first_score(a.run_id,receipt_sha,a.artifact_id)
    a.output.mkdir();dump(a.output/'first-score-started.json',{'M1_sha256':a.M1_sha256,'verified_run_sha256':binding,'rescore_or_relabel':False})
    records={r['id']:r for r in gold['records']} if isinstance(gold['records'],list) else gold['records'];planned=[];scored=[];obs_index={};errors=[]
    for provider in protocol['required_providers']:
        for case in data['cases']:
            for track in protocol['tracks']:
                g=records[case['id']+'/'+track]
                for arm in ['baseline','candidate']:
                    old_id=sha((a.M1_sha256+'|'+provider+'|'+case['id']+'|'+track+'|'+arm).encode())[:24]
                    opaque=new_id(binding,old_id)
                    planned.append({'id':case['id'],'row_id':opaque,'provider':provider,'track':track,'arm':arm,'expected':g['label'],'family':g['family'],'required_groups_G':len(g['groups'])})
                    op=a.observations/(opaque+'.json');rp=a.reviews/(opaque+'.json')
                    if not op.exists() or not rp.exists():continue
                    observation=read(op);review=read(rp)
                    if (observation['provider'],observation['track'],observation['arm'],observation['case_id'])!=(provider,track,arm,case['id']):raise ValueError('observation plan mismatch')
                    record={**g,'sha256':sha(json.dumps(g,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())}
                    if review.get('reviewer')!={'role':'agent','id':m0['semantic_reviewer_id']}:raise ValueError('unexpected_semantic_reviewer')
                    if review.get('execution_provenance_sha256')!=binding:raise ValueError('review_not_for_exact_artifact')
                    try:scored.append(validate_review(record,observation,review,data['sources'],deps));obs_index[(provider,track,arm,case['id'])]=observation
                    except (ValueError,KeyError,TypeError) as e:errors.append({'row_id':opaque,'error':type(e).__name__+':'+str(e)})
    report=summarize(scored,planned,protocol,deps);runtime={};runtime_pass=True
    for provider in protocol['required_providers']:
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
    report['deferred_providers']={'google':'NO_NEW_RUN_PRIOR_23_OF_24_PARTIAL_F1_REFERENCE_ONLY','groq':'NOT_RUN_NO_F1_NO_ADOPTION_JUDGMENT'}
    report['accuracy_acceptance']=False
    report['fresh_accuracy_cases']=0
    report['declared_quality_gates_pass']=bool(report['semantic_gate_pass'] and runtime_pass and not errors)
    report['execution_receipt_sha256']=receipt_sha
    report['verified_run_sha256']=binding
    report['run_id']=a.run_id
    report['artifact_id']=a.artifact_id
    report['source_merge_release_CLI_distribution']='SEPARATE_GATES_NOT_APPROVED_BY_THIS_REPORT'
    dump(a.output/'rows.json',scored);dump(a.output/'report.json',report)
    print(json.dumps({'acceptance':report['acceptance'],'providers':report['provider_tracks'],'annotation_failures':len(errors)},ensure_ascii=False))
if __name__=='__main__':main()
