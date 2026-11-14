#!/usr/bin/env python3
"""Run private *self-authored* 24-case Dev comparison; NEVER a formal F1 gate.

All original source/labels are GitHub Actions secret input, never in repo or raw logs.
No retry of effectful requests; a failed, timed out or unknown usage stops the shard.
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

CORPUS_HASH = '63da84b7176766c2fec9becf7ad631ff03b0b7c7264f4f120ac05816abd43e1c'
TOTAL_CASES = 24
JOBS = 48
MAX_PRICE = JOBS * v5.TOTAL_CAP * .15 / 1_000_000


def serialize(data):
    return json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + '\n'


def setup(repo, root, corpus_path, build):
    raw = corpus_path.read_bytes()
    v5.checked(hashlib.sha256(raw).hexdigest() == CORPUS_HASH, 'private_corpus_sha256_changed')
    data = json.loads(raw)
    cases = data['cases']
    v5.checked(data['status'] == 'DEVELOPMENT_ONLY_UNREVIEWED_NOT_FORMAL' and len(cases)==24,
               'cohort_classification_changed')
    v5.checked(len(set(row['case_id'] for row in cases))==24 and len(set(row['target'] for row in cases))==24,
               'cohort_duplicate')
    v5.checked([sum(row['gold']['classification']==cat for row in cases) for cat in ('positive','negative','ambiguous')]==[12,8,4],
               'cohort_classes_changed')
    configs = root/'configs';configs.mkdir(mode=0o700,exist_ok=True)
    rows = []
    for row in cases:
        cid, name = row['case_id'], row['target']
        v5.checked(re.fullmatch(r'q(0[1-9]|1[0-9]|2[0-4])', cid) is not None, 'invalid_id')
        v5.checked(len(row['sources'])==1 and row['sources'][0]['synthetic'] is True,'source_class_changed')
        positive = row['gold']['classification']=='positive'
        slug=re.sub('[^a-z0-9]+','-',name.lower()).strip('-')
        cfg=v5.make_config(cid, row['sources'][0]['quote'], positive)
        cfgtext=json.dumps(cfg, ensure_ascii=False).replace('quartz-lantern',slug).replace('synthetic-608:','synthetic-531:')
        if positive: cfgtext=cfgtext.replace('2026-06',row['gold']['launch_month'])
        cfg=json.loads(cfgtext)
        record=cfg['resolution']['investigation']['capabilities'][0]['evidence'][0]
        metadata={'temporal':{'effective_from_unix_seconds':1800000000,'effective_until_unix_seconds':1800086400},'scope':{'project':{'kind':'values','values':[slug]}},'provenance_class':'primary'}
        evidence={'id':record['id'],'source':record['source'],'observation':record['observation'],'facts':dict(sorted(record['facts'].items())),'metadata':metadata}
        evidence_hash=hashlib.sha256(json.dumps(evidence,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
        for ref in cfg['answer_context']['sources']:ref['evidence_sha256']=evidence_hash
        path=configs/(cid+'.json');path.write_text(serialize(cfg));path.chmod(0o600)
        rows.append({'id':cid,'config':str(path),'question':row['question'],'expected_relevant':positive})
    # Never start live before ALL frozen first requests are checked pairwise.
    hashes={}
    for i,row in enumerate(rows):
        plans=[]
        for arm in v5.ARMS:
            plan=root/(row['id']+'-'+arm+'-plan.json')
            cp=subprocess.run(v5.argv(row,arm,build,i,plan=plan),cwd=repo,env={'PATH':os.environ.get('PATH',''),'HOME':os.environ.get('HOME','')},capture_output=True,timeout=35)
            v5.checked(cp.returncode==0 and cp.stdout==b'' and cp.stderr==b'', 'plan_failure_'+row['id']+'_'+arm)
            obj=json.loads(plan.read_text());v5.checked(obj['question']==row['question'] and obj['model']==v5.MODEL,'request_changed')
            plans.append(obj)
        v5.checked(plans[0]['request']==plans[1]['request'] and plans[0]['request_sha256']==plans[1]['request_sha256'], 'pair_mismatch_'+row['id'])
        hashes[row['id']]=plans[0]['request_sha256']
    (root/'preflight.json').write_text(serialize({'dataset_sha256':CORPUS_HASH,'case_count':24,'paired_requests':48,'pair_equality':True,'request_hashes':hashes}))
    return rows


def measure(repo,root,rows,build,shard):
    v5.checked(os.environ.get('REASON_DEV_LITE_FREE_CONFIRMED')=='1','free_budget_confirmation_missing')
    v5.checked(os.environ.get('MISTRAL_API_KEY','')!='','provider_key_missing')
    v5.checked(MAX_PRICE < .21 and MAX_PRICE < 9.98/10, 'price_bound_exceeded')
    sys.path.insert(0,str(repo/'evaluation/dev-lite'))
    import run as lite
    start=shard*6;stop=start+6
    subset=rows[start:stop]
    report={'classification':'SELF_AUTHORED_DEVELOPMENT_ONLY_NO_FORMAL_F1','corpus_sha256':CORPUS_HASH,'shard':shard+1,'model':v5.MODEL,'cli_commit':v5.CLI,'baseline_commit':v5.BASELINE,'candidate_commit':v5.CANDIDATE,'planned_jobs':12,'completed_jobs':0,'observations':[],'run_status':'incomplete','formal_f1':None,'quality_go':False,'adoption_go':False,'maximum_usd_all_48_by_token_cap':round(MAX_PRICE,7)}
    rawdir=root/'raw';rawdir.mkdir(mode=0o700,exist_ok=True)
    status='complete'
    try:
        for i,row in enumerate(subset,start):
            for arm in (v5.ARMS if i%2==0 else v5.ARMS[::-1]):
                st=time.monotonic()
                try:
                    cp=subprocess.run(v5.argv(row,arm,build,i),cwd=repo,stdin=subprocess.DEVNULL,capture_output=True,timeout=1200,env={'PATH':os.environ.get('PATH',''),'HOME':os.environ.get('HOME',''),'MISTRAL_API_KEY':os.environ['MISTRAL_API_KEY'],'NO_COLOR':'1','NO_PROXY':'127.0.0.1','REASON_MISTRAL_RATE_LIMIT_TELEMETRY':'1'})
                except subprocess.TimeoutExpired:
                    status='stopped_timeout_no_retry'
                    break
                stdout=cp.stdout
                p=rawdir/(row['id']+'-'+arm+'.stdout.json');p.write_bytes(stdout);p.chmod(0o600)
                e=rawdir/(row['id']+'-'+arm+'.stderr.txt');e.write_bytes(cp.stderr);e.chmod(0o600)
                observed={'id':row['id'],'arm':arm,'exit_code':cp.returncode,'stdout_sha256':hashlib.sha256(stdout).hexdigest(),'elapsed_seconds':round(time.monotonic()-st,3),'usage':None,'answer_status':None,'cli_status':None}
                report['observations'].append(observed);report['completed_jobs']+=1
                try:
                    usage,answer_status,cli_status=lite.usage_from_cli(stdout)
                    observed.update(usage=usage,answer_status=answer_status,cli_status=cli_status)
                    v5.checked(usage['model_calls']<=v5.MAX_CALLS and usage['total_tokens']<=v5.TOTAL_CAP,'usage_limit_exceeded')
                    v5.checked(cp.returncode==0 and cli_status!='failed','cli_failed')
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
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--build',type=Path,required=True);p.add_argument('--corpus',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--mode',choices=['preflight','live'],required=True);p.add_argument('--shard',type=int,choices=[0,1,2,3],default=0)
    a=p.parse_args();repo=a.repo.resolve(strict=True);build=v5.build_checked(a.build);root=a.output.resolve();v5.checked(not root.exists(),'output_must_be_new');root.mkdir(mode=0o700,parents=True)
    rows=setup(repo,root,a.corpus,build)
    print('24/24_request_pairs_equal=true api_calls_preflight=0',flush=True)
    if a.mode=='live':measure(repo,root,rows,build,a.shard)
    else: print('formal_f1=null model_calls=0',flush=True)

if __name__=='__main__':main()
