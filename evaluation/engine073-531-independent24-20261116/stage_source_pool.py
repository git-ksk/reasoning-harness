#!/usr/bin/env python3
"""Acquire 12 *new* pinned public upstream source fingerprints, never labels.

Input documents were not used by the 2026-11-15 dev24 corpus. This is source
preparation only: NO questions, NO independent reviewer approval, NO F1 and
NO model calls.  GitHub public HTTP GETs only; no credentials are read.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request

SOURCES={
  'python/cpython':('main',[
      'Doc/library/tomllib.rst','Doc/library/pathlib.rst','Doc/library/zipfile.rst']),
  'nodejs/node':('main',[
      'doc/api/fs.md','doc/api/process.md','doc/api/url.md']),
  'rust-lang/book':('main',[
      'src/ch04-01-what-is-ownership.md','src/ch09-02-recoverable-errors-with-result.md']),
  'kubernetes/website':('main',[
      'content/en/docs/concepts/configuration/secret.md',
      'content/en/docs/concepts/workloads/controllers/deployment.md']),
  'postgres/postgres':('master',[
      'doc/src/sgml/ref/psql-ref.sgml','doc/src/sgml/backup.sgml']),
}
CONTRACT='reason-531-independent24-provisional-source-pool-v1'
MAX_SOURCE_BYTES=400_000
MAX_RESPONSE_BYTES=262_144
HEADERS={'User-Agent':'reasoning-harness-independent-source-pool-no-model-requests',
         'Accept':'application/vnd.github+json'}


def sha256(content):
    return hashlib.sha256(content).hexdigest()


def fetch(url,limit):
    req=urllib.request.Request(url,headers=HEADERS)
    with urllib.request.urlopen(req,timeout=25) as resp:
        if resp.status!=200 or resp.geturl()!=url:
            raise ValueError('source_not_exact_unredirected_success')
        content=resp.read(limit+1)
        if not content or len(content)>limit:
            raise ValueError('source_too_large_or_empty')
        return content


def prior_provenance(repo):
    path=repo/'evaluation/engine073-531-real-doc24-20261115/source-audit.json'
    raw=path.read_bytes()
    d=json.loads(raw)
    urls={r['url'] for r in d['source_documents']}
    repos={u.split('/')[3]+'/'+u.split('/')[4] for u in urls}
    if len(urls)!=12:
        raise ValueError('original_development_source_count_drift')
    return sha256(raw),repos


def stage(repo):
    prior_hash,prior_repos=prior_provenance(repo)
    if prior_repos.intersection(SOURCES):
        raise ValueError('new_source_pool_reuses_consumed_dev_repository')
    records=[]
    for owner_repo,(branch,paths) in SOURCES.items():
        endpoint=f'https://api.github.com/repos/{owner_repo}/commits/{branch}'
        result=json.loads(fetch(endpoint,MAX_RESPONSE_BYTES))
        commit=result['sha']
        if not re.fullmatch('[a-f0-9]{40}',commit):
            raise ValueError('unverified_public_upstream_commit')
        for path in paths:
            url=f'https://raw.githubusercontent.com/{owner_repo}/{commit}/{path}'
            payload=fetch(url,MAX_SOURCE_BYTES)
            payload.decode('utf-8','strict')
            records.append({'repo':owner_repo,'path':path,'commit':commit,
                            'url':url,'sha256':sha256(payload),'bytes':len(payload)})
    if len(records)!=12 or len({r['url'] for r in records})!=12:
        raise ValueError('wrong_new_source_inventory')
    return {
        'contract':CONTRACT,
        'date_utc':'2026-11-15',
        'classification':'PROVISIONAL_NEW_SOURCE_POOL_NOT_AUTHORITATIVE_HOLDOUT',
        'prior_consumed_dev_source_audit_sha256':prior_hash,
        'sources':records,
        'source_count':len(records),
        'case_count':0,
        'gold_labels_included':False,
        'fresh_independent_reviewer_approved':False,
        'first_observation_authorized':False,
        'model_calls':0,
        'formal_independent_f1':None,
    }


def validate(repo,manifest):
    if manifest.get('contract')!=CONTRACT or manifest.get('source_count')!=12:
        raise ValueError('provisional_source_pool_contract_mismatch')
    prior_hash,prior_repos=prior_provenance(repo)
    if manifest.get('prior_consumed_dev_source_audit_sha256')!=prior_hash:
        raise ValueError('prior_dev_source_audit_changed')
    for field,value in [('case_count',0),('gold_labels_included',False),
                        ('fresh_independent_reviewer_approved',False),
                        ('first_observation_authorized',False),('model_calls',0),
                        ('formal_independent_f1',None)]:
        if manifest.get(field)!=value:
            raise ValueError('unauthorized_early_score_or_run_'+field)
    records=manifest.get('sources',[])
    if len(records)!=12 or len({r.get('url') for r in records})!=12:
        raise ValueError('public_source_pool_duplicate_or_count_drift')
    if len({r.get('repo') for r in records})!=5 or prior_repos.intersection(r.get('repo') for r in records):
        raise ValueError('source_pool_repository_reuse')
    expected={(repo,path) for repo,(_,paths) in SOURCES.items() for path in paths}
    if {(r.get('repo'),r.get('path')) for r in records}!=expected:
        raise ValueError('unrecognized_candidate_source')
    for record in records:
        commit=record['commit']
        if not re.fullmatch('[a-f0-9]{40}',commit):
            raise ValueError('mutable_upstream_source_revision')
        if record['url']!=f'https://raw.githubusercontent.com/{record["repo"]}/{commit}/{record["path"]}':
            raise ValueError('noncanonical_pinned_upstream_url')
        if not re.fullmatch('[a-f0-9]{64}',record['sha256']):
            raise ValueError('invalid_raw_document_sha')
        if not 1<=record['bytes']<=MAX_SOURCE_BYTES:
            raise ValueError('unbounded_upstream_source')
    return True


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    repo=args.repo.resolve(strict=True)
    out=args.output.resolve()
    if out.exists():
        raise ValueError('never_overwrite_frozen_source_pool')
    manifest=stage(repo)
    validate(repo,manifest)
    out.write_text(json.dumps(manifest,indent=2,sort_keys=True)+'\n')
    print(json.dumps({'contract':CONTRACT,'source_count':12,
                      'distinct_new_repositories':5,
                      'manifest_sha256':sha256(out.read_bytes()),'model_calls':0,
                      'formal_independent_f1':None},sort_keys=True))


if __name__=='__main__':main()
