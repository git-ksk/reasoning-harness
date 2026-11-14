#!/usr/bin/env python3
"""Run paired *previously-consumed real upstream docs* development check, NOT formal F1.

Upstream source snapshots and questions are public development inputs. Only model
outputs are encrypted. No scoring gold enters the runner. No retry after failures.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

import synthetic_v5_timed_608 as v5

PUBLIC_MANIFEST_SHA256 = '8b910bc4ec497a4bc768cc6c68465c4d2bb8bd6886addfe985c55340265fb542'
ORIGINAL_BUNDLE_SHA256 = '02506cade31f5f0ddd82fd345350f05c08460d1dc3d96682b1a6f2c7f451a6a6'
TOTAL_CASES = 24
JOBS = 48
MAX_PRICE = JOBS * v5.TOTAL_CAP * .15 / 1_000_000


def serialize(data):
    return json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + '\n'


def validate_real_source(observation, cid):
    # Pinned upstream commit + unchanged upstream extraction; an input found in
    # synthetic examples or an editable branch cannot be called a real document.
    v5.checked(type(observation) is dict, 'bad_real_source_object_'+cid)
    url=observation.get('url','')
    v5.checked(isinstance(url,str) and
               url.startswith('https://raw.githubusercontent.com/') and
               re.search(r'/[a-f0-9]{40}/',url) is not None and
               observation.get('synthetic_mutation') is False and
               re.fullmatch(r'sha256:[a-f0-9]{64}',observation.get('source_version','')) is not None and
               isinstance(observation.get('quote'),str) and
               len(observation['quote'].strip())>=12,
               'untrusted_or_mutated_real_source_'+cid)
    return url


def validate_public_manifest(public_manifest, repo):
    checked_path = (repo/'evaluation/engine073-531-real-doc24-20261115/cases.json').resolve(strict=True)
    v5.checked(public_manifest.resolve(strict=True) == checked_path,
               'manifest_must_be_pinned_repo_public_cases')
    raw=checked_path.read_bytes()
    v5.checked(hashlib.sha256(raw).hexdigest() == PUBLIC_MANIFEST_SHA256,
               'public_manifest_sha256_changed')
    data=json.loads(raw)
    v5.checked(data['schema_version']=='reason-531-real-doc24-public-v1' and
               data['classification']=='CONSUMED_2026_10_REAL_UPSTREAM_DOCUMENTS_DEV_ONLY_NOT_INDEPENDENT_NEW_HOLDOUT' and
               data['provider']=='mistral' and data['model']==v5.MODEL and
               data['gold_labels_included'] is False and data['formal_quality_f1'] is None and
               data['initial_external_model_calls']==0 and
               data['frozen_old_private_bundle_sha256']==ORIGINAL_BUNDLE_SHA256 and
               data['source_documents_count']==12 and len(data['source_content_sha256'])==12 and
               len(data['cases'])==24,
               'wrong_public_manifest_contract')
    return data


def setup(repo, root, manifest_path, build):
    data=validate_public_manifest(manifest_path,repo)
    cases=data['cases']
    v5.checked(len(set(row['case_id'] for row in cases))==TOTAL_CASES,
               'real_corpus_duplicate')
    config_output=root/'configs';config_output.mkdir(mode=0o700,exist_ok=True)
    rows=[]
    origins=set()
    for index,row in enumerate(cases):
        cid=row['case_id']
        v5.checked(cid=='rw-'+str(index+1).zfill(2),'real_case_id_order_changed')
        v5.checked(isinstance(row['question'],str) and len(row['question'])>20,
                   'missing_frozen_question')
        relative=f'configs/{cid}.json'
        v5.checked(row['config_path']==relative, 'unexpected_config_path_'+cid)
        unresolved_source=manifest_path.parent/relative
        v5.checked(not unresolved_source.is_symlink(), 'public_config_symlink_'+cid)
        source_path=unresolved_source.resolve(strict=True)
        v5.checked(source_path==(repo/'evaluation/engine073-531-real-doc24-20261115'/relative).resolve(strict=True),
                   'public_config_path_outside_repo_'+cid)
        raw_cfg=source_path.read_bytes()
        v5.checked(hashlib.sha256(raw_cfg).hexdigest()==row['config_raw_sha256'],
                   'original_config_bytes_changed_'+cid)
        cfg=json.loads(raw_cfg)
        v5.checked(cfg['schema_version']=='reason-config-v1', 'real_config_contract_changed')
        caps=cfg['resolution']['investigation']['capabilities']
        v5.checked(len(caps)==2,'original_real_capabilities_changed')
        evidence=[]
        for cap in caps:
            v5.checked(cap['kind']=='captured_evidence' and cap['read_only'] is True,
                       'unexpected_live_source_capability')
            for record in cap['evidence']:
                obj=json.loads(record['observation'])
                url=validate_real_source(obj,cid)
                v5.checked(obj['source_version']==data['source_content_sha256'].get(url),
                           'unpinned_source_content_'+cid)
                evidence.append(record)
                origins.add(url)
        v5.checked(len(evidence)==2,'original_real_source_count_changed')
        v5.checked(2<=len(cfg['answer_context']['sources'])<=3,
                   'real_source_registry_changed')
        output_path=config_output/(cid+'.json')
        output_path.write_bytes(raw_cfg);output_path.chmod(0o600)
        rows.append({'id':cid,'config':str(output_path),'question':row['question']})
    v5.checked(len(origins)==12,'real_upstream_source_count_changed')
    # Both arms must have exactly identical first HTTP request before ANY model
    # call. No answer key, previous output, or semantic gold is on this runner.
    hashes={}
    for i,row in enumerate(rows):
        plans=[]
        for arm in v5.ARMS:
            plan=root/(row['id']+'-'+arm+'-plan.json')
            cp=subprocess.run(v5.argv(row,arm,build,i,plan=plan),cwd=repo,
                              env={'PATH':os.environ.get('PATH',''),
                                   'HOME':os.environ.get('HOME','')},
                              capture_output=True,timeout=35)
            v5.checked(cp.returncode==0 and cp.stdout==b'' and cp.stderr==b'',
                       'real_plan_failure_'+row['id']+'_'+arm)
            obj=json.loads(plan.read_text())
            v5.checked(obj['question']==row['question'] and obj['model']==v5.MODEL,
                       'real_request_changed')
            plans.append(obj)
        v5.checked(plans[0]['request']==plans[1]['request'] and
                   plans[0]['request_sha256']==plans[1]['request_sha256'],
                   'real_pair_mismatch_'+row['id'])
        hashes[row['id']]=plans[0]['request_sha256']
    (root/'preflight.json').write_text(serialize({
        'public_manifest_sha256':PUBLIC_MANIFEST_SHA256,'source_kind':'public_pinned_upstream_repositories',
        'source_files':len(origins),'case_count':24,'paired_requests':48,
        'pair_equality':True,'request_hashes':hashes,'model_completions':0,
        'formal_independent_review':False}))
    return rows


def measure(repo,root,rows,build,shard):
    v5.checked(os.environ.get('REASON_DEV_LITE_FREE_CONFIRMED')=='1','free_budget_confirmation_missing')
    v5.checked(os.environ.get('MISTRAL_API_KEY','')!='','provider_key_missing')
    v5.checked(MAX_PRICE < .21 and MAX_PRICE < 9.98/10, 'price_bound_exceeded')
    sys.path.insert(0,str(repo/'evaluation/dev-lite'))
    import run as lite
    start=shard*6;stop=start+6
    subset=rows[start:stop]
    report={'classification':'REAL_UPSTREAM_DOCUMENTS_CONSUMED_DEV_ONLY_NO_FORMAL_F1','public_manifest_sha256':PUBLIC_MANIFEST_SHA256,'shard':shard+1,'model':v5.MODEL,'cli_commit':v5.CLI,'baseline_commit':v5.BASELINE,'candidate_commit':v5.CANDIDATE,'planned_jobs':12,'completed_jobs':0,'observations':[],'run_status':'incomplete','formal_f1':None,'quality_go':False,'adoption_go':False,'maximum_usd_all_48_by_token_cap':round(MAX_PRICE,7)}
    rawdir=root/'raw';rawdir.mkdir(mode=0o700,exist_ok=True)
    status='complete'
    try:
        for i,row in enumerate(subset,start):
            for arm in (v5.ARMS if i%2==0 else v5.ARMS[::-1]):
                st=time.monotonic()
                try:
                    cp=subprocess.run(v5.argv(row,arm,build,i),cwd=repo,stdin=subprocess.DEVNULL,capture_output=True,timeout=1200,env={'PATH':os.environ.get('PATH',''),'HOME':os.environ.get('HOME',''),'MISTRAL_API_KEY':os.environ['MISTRAL_API_KEY'],'NO_COLOR':'1','NO_PROXY':'127.0.0.1','REASON_MISTRAL_RATE_LIMIT_TELEMETRY':'1'})
                except subprocess.TimeoutExpired as exc:
                    # Provider may already have accepted/charged the request.
                    # Preserve any partial output as unscored private evidence;
                    # never retry this effectful invocation.
                    partial_out = exc.stdout or b''
                    partial_err = exc.stderr or b''
                    if isinstance(partial_out, str): partial_out = partial_out.encode()
                    if isinstance(partial_err, str): partial_err = partial_err.encode()
                    p=rawdir/(row['id']+'-'+arm+'.timeout.stdout.partial')
                    p.write_bytes(partial_out);p.chmod(0o600)
                    e=rawdir/(row['id']+'-'+arm+'.timeout.stderr.partial')
                    e.write_bytes(partial_err);e.chmod(0o600)
                    report['observations'].append({'id':row['id'],'arm':arm,
                        'complete':False,'attempt_outcome':'unknown_timeout_may_have_been_billed',
                        'elapsed_seconds':round(time.monotonic()-st,3),
                        'partial_stdout_sha256':hashlib.sha256(partial_out).hexdigest(),
                        'partial_stderr_sha256':hashlib.sha256(partial_err).hexdigest(),
                        'usage':None,'do_not_retry':True})
                    status='stopped_timeout_no_retry'
                    break
                stdout=cp.stdout
                p=rawdir/(row['id']+'-'+arm+'.stdout.json');p.write_bytes(stdout);p.chmod(0o600)
                e=rawdir/(row['id']+'-'+arm+'.stderr.txt');e.write_bytes(cp.stderr);e.chmod(0o600)
                observed={'id':row['id'],'arm':arm,'exit_code':cp.returncode,'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'elapsed_seconds':round(time.monotonic()-st,3),'usage':None,'answer_status':None,'cli_status':None,'complete':False}
                report['observations'].append(observed);report['completed_jobs']+=1
                try:
                    usage,answer_status,cli_status=lite.usage_from_cli(stdout)
                    observed.update(usage=usage,answer_status=answer_status,cli_status=cli_status)
                    v5.checked(usage['model_calls']<=v5.MAX_CALLS and usage['total_tokens']<=v5.TOTAL_CAP,'usage_limit_exceeded')
                    v5.checked(cp.returncode==0 and cli_status!='failed','cli_failed')
                    observed['complete']=True
                except (ValueError,TypeError,KeyError,RuntimeError,lite.GateError):
                    status='stopped_unknown_usage_or_cli_failure_no_retry'
                    break
                print('observed_case_id='+row['id']+' arm='+arm+' calls='+str(usage['model_calls'])+' total_tokens='+str(usage['total_tokens']),flush=True)
            if status!='complete':break
    finally:
        report['run_status']=status
        (root/'sanitized-summary.json').write_text(serialize(report))
        print('shard='+str(shard+1)+' status='+status+' completed='+str(report['completed_jobs'])+' formal_f1=null',flush=True)
    v5.checked(status=='complete' and report['completed_jobs']==12,'shard_incomplete_or_failed')


def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--build',type=Path,required=True);p.add_argument('--cases',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--mode',choices=['preflight','live'],required=True);p.add_argument('--shard',type=int,choices=[0,1,2,3],default=0)
    a=p.parse_args();repo=a.repo.resolve(strict=True);build=v5.build_checked(a.build);root=a.output.resolve();v5.checked(not root.exists(),'output_must_be_new');root.mkdir(mode=0o700,parents=True)
    rows=setup(repo,root,a.cases,build)
    print('real_docs_12_pinned=true 24/24_request_pairs_equal=true api_calls_preflight=0',flush=True)
    if a.mode=='live':measure(repo,root,rows,build,a.shard)
    else: print('formal_f1=null model_calls=0',flush=True)

if __name__=='__main__':main()
