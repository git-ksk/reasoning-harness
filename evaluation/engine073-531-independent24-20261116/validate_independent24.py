#!/usr/bin/env python3
"""Issue #616 provider-free author-draft integrity auditor. NO model/billing calls.

Public mode reads *only* Git-tracked challenge questions and fixed source hashes.
Private mode reads locally owner-restricted gold; NEVER outputs gold contents.
Review approval, first observations, billing and live execution always stay HOLD.
"""
import argparse,collections,hashlib,json,os,re,stat,sys
from pathlib import Path

HERE=Path(__file__).resolve().parent
PUBLIC='independent24-public-cases.json'
LOCK='independent24-author-freeze.json'
SOURCE='provisional-new-sources.json'
DEV=HERE.parent/'engine073-531-real-doc24-20261115'/'cases.json'
PACKET=Path.home()/'Library/Application Support/reasoning-harness/issue531-independent24-20261116/independent-author-source-packet-v1'
REQUIRED_CASE={'case_id','source_id','question','question_sha256','official_url','upstream_commit','source_sha256'}
FORBIDDEN=('answer','label','gold','expected','negative_reason','required_facts','classification','citation_quote')
sha=lambda b:hashlib.sha256(b).hexdigest()
canonical=lambda o:(json.dumps(o,sort_keys=True,ensure_ascii=False,indent=2,allow_nan=False)+'\n').encode()

def checked(ok,why):
    if not ok:raise ValueError(why)

def read_json(path):
    checked(path.is_file() and not path.is_symlink(),'file_missing_or_symlinked')
    raw=path.read_bytes()
    checked(len(raw)<2500000,'oversize_input')
    obj=json.loads(raw)
    return obj,raw

def shingles(t):
    words=re.findall(r'[a-z0-9_]+',t.casefold())
    return set(zip(words,words[1:],words[2:],words[3:],words[4:]))

def verify_public_data(public,lock,source,raw_bytes,source_bytes,dev):
    checked(set(public)=={'contract','classification','source_manifest_sha256','cases','reviewer_approved','model_calls','formal_f1'},'public_schema_or_label_leak')
    checked(public['contract']=='engine073-531-independent24-public-questions-v1' and
            public['classification']=='UNREVIEWED_AUTHOR_DRAFT_NOT_EXECUTION_AUTHORITY' and
            public['reviewer_approved'] is False and public['formal_f1'] is None and public['model_calls']==0,
            'public_false_authority')
    checked(isinstance(public['cases'],list) and len(public['cases'])==24,'not_24_cases')
    checked(len(source.get('sources',[]))==12,'not_12_sources')
    checked(public['source_manifest_sha256']==sha(source_bytes),'public_source_hash_drift')
    questions=[];dev_q=[r['question'] for r in dev['cases']]
    old_sets=[shingles(q) for q in dev_q]
    similarities=[]
    for i,row in enumerate(public['cases'],1):
        ref=source['sources'][(i-1)//2]
        checked(set(row)==REQUIRED_CASE,'public_case_extra_field_or_leak')
        checked(row['case_id']==f'ih-{i:02d}' and row['source_id']==f'source-{(i+1)//2:02d}','id_or_per_source_count_wrong')
        checked(row['official_url']==ref['url'] and row['source_sha256']==ref['sha256'] and row['upstream_commit']==ref['commit'],'official_source_provenance_drift')
        q=row['question']
        checked(isinstance(q,str) and 65<=len(q)<=600 and '?' in q,'invalid_question')
        checked(sha(q.encode())==row['question_sha256'],'question_hash_drift')
        checked(q not in questions and q not in dev_q,'dev_or_internal_exact_question_reuse')
        grams=shingles(q)
        sim=max((len(grams&b)/len(grams|b) if grams|b else 0 for b in old_sets),default=0)
        checked(sim<0.55,'too_similar_to_consumed_dev_question')
        similarities.append(sim)
        questions.append(q)
    checked(lock['contract']=='engine073-531-independent24-author-freeze-v1' and
            lock['phase']=='AUTHOR_DRAFT_SEALED_AWAITING_INDEPENDENT_REVIEW','false_lock_phase')
    checked(lock['public_cases_sha256']==sha(raw_bytes) and lock['source_manifest_sha256']==sha(source_bytes),'public_lock_hash_changed')
    checked(re.fullmatch(r'[0-9a-f]{64}',lock['private_gold_sha256']) is not None,'gold_sha_missing')
    checked(lock['case_count']==24 and lock['source_count']==12 and lock['gold_count_private_only']==24,'lock_counts_changed')
    checked(lock['baseline_core_commit']=='4abee90f0501f0d8fc09b453a0b45c942fbed69e' and
        lock['candidate_core_commit']=='7466c6f90d420a98c99713b96d6a0623e2aeef52' and
        lock['shared_corrected_cli_commit']=='b85a4a0b2ab03397b361a8620d7a97dd26b2eb36','core_cli_commit_drift')
    checked(lock['provider']=='mistral' and lock['model']=='ministral-8b-2512' and
        lock['sampling_parameters']=={
         'first_request_random_seed_base':49200,
         'per_case_seed_rule':'base + zero_based_case_index',
         'first_request_max_tokens':1024,
         'first_request_temperature':'not specified',
         'first_request_top_p':'not specified',
         'first_request_output':'structured candidate',
         'later_calls':'identical pinned CLI implementation; bounded by 12 calls / 28096 tokens per CLI',
         'source':'provider-free prior corrected-pair request plans, not independently reproduced for new cases'
        },'sampling_protocol_changed')
    checked(lock['first_observation_count_required']==48 and lock['max_model_calls_per_cli']==12 and
        lock['max_tokens_per_cli']==28096,'first_observation_or_token_cap_changed')
    for key,val in {'case_f1_min':0.90,'precision_min':0.95,'recall_min':0.85,
        'unsafe_negative_assertions_max':0,'invalid_citations_max':0,'no_candidate_regression':True}.items():
        checked(lock.get(key)==val,'pre_registered_threshold_drift_'+key)
    for key in ('independent_reviewer_approved','ready_to_execute','billing_safety_verified',
                'paired_first_requests_checked','live_workflow_created','quality_go','release_go'):
        checked(lock.get(key) is False,'premature_go_'+key)
    for key in ('reviewer_signature','formal_independent_f1'):
        checked(lock.get(key) is None,'forged_review_or_score_'+key)
    checked('final user-visible answer' in lock['scoring_policy'] and 'timeout' in lock['scoring_policy'],
            'unsafe_scoring_policy')
    return {'public_case_count':24,'source_count':12,'dev_exact_overlap':0,
            'max_dev_5gram_jaccard':round(max(similarities),4),
            'public_cases_sha256':sha(raw_bytes),'private_gold_sha256':lock['private_gold_sha256']}
def verify_gold_data(gold,public,packet):
    checked(set(gold)=={'contract','classification','cases','reviewer_approved',
                      'reviewer_signature','first_observations','formal_f1'},'private_gold_schema')
    checked(gold['contract']=='engine073-531-independent24-private-gold-v2'
      and gold['classification']=='AUTHOR_SELF_CHECK_NOT_INDEPENDENT_APPROVAL'
      and gold['reviewer_approved'] is False and gold['reviewer_signature'] is None
      and gold['first_observations']==0 and gold['formal_f1'] is None,'private_gold_false_authority')
    rows=gold['cases']
    checked(isinstance(rows,list) and len(rows)==24,'gold_not_24')
    kinds=collections.Counter()
    facts=0;quotes=0
    for i,(g,c) in enumerate(zip(rows,public['cases']),1):
        src_id=(i+1)//2
        source=(packet/'docs'/f'upstream-{src_id:02d}.txt')
        lines=source.read_text().splitlines()
        checked(g['case_id']==c['case_id'] and g['question_sha256']==c['question_sha256'],
                'gold_case_question_mismatch')
        for key in ('official_url','upstream_commit','source_sha256'):
            checked(g[key]==c[key],'gold_source_mismatch_'+key)
        checked(isinstance(g['scope'],str) and len(g['scope'])>=15 and
            isinstance(g['acceptable_paraphrases'],str) and len(g['acceptable_paraphrases'])>=15 and
            isinstance(g['incorrect_answers'],str) and len(g['incorrect_answers'])>=15,'poor_gold_review_attributes')
        checked(g['classification'] in ('answerable','negative'),'invalid_label')
        kinds[g['classification']]+=1
        if g['classification']=='answerable':
            checked(g['negative_reason'] is None and g['guardrail_evidence']==[] and
                isinstance(g['required_facts'],list) and len(g['required_facts'])>=2,
                'positive_wrong_form')
            ev=[f['evidence'] for f in g['required_facts']]
            checked(all(f['fact_id']==f'ih-{i:02d}-f{j}' and len(f['fact'])>10
                        for j,f in enumerate(g['required_facts'],1)),'invalid_fact_id_or_text')
            facts+=len(ev)
        else:
            checked(g['required_facts']==[] and isinstance(g['negative_reason'],str) and
                len(g['negative_reason'])>=70 and len(g['guardrail_evidence'])>=1,
                'negative_label_or_reason_missing')
            ev=g['guardrail_evidence']
        for e in ev:
            checked(e['source_id']==c['source_id'],'quote_source_wrong')
            n=e['line_start']
            checked(isinstance(n,int) and n>=1 and n<=e['line_end']<=min(len(lines),n+4),
                'quote_position_wrong')
            checked(e['exact_quote']=='\n'.join(lines[n-1:e['line_end']]).strip() and
                e['quote_sha256']==sha(e['exact_quote'].encode()),
                'quote_not_exact_or_modified')
            quotes+=1
    checked(kinds=={'answerable':16,'negative':8},'positive_negative_distribution')
    return {'private_gold_cases':24,'answerable':16,'negative':8,
            'required_facts':facts,'exact_evidence_lines':quotes}

def private_file(path,repo):
    path=path.expanduser().absolute()
    checked(not path.is_symlink() and path.is_file() and not path.resolve().is_relative_to(repo.resolve()),
            'private_gold_not_outside_repo')
    st=path.stat()
    checked(st.st_uid==os.getuid() and stat.S_IMODE(st.st_mode)==0o600,
            'gold_permissions_not_0600')
    parent=path.parent
    checked(not parent.is_symlink() and parent.stat().st_uid==os.getuid() and
            stat.S_IMODE(parent.stat().st_mode)==0o700,'gold_parent_not_0700')
    return path

def verify(repo,gold_path=None,packet_path=PACKET):
    repo=repo.resolve(strict=True)
    checked((repo/'evaluation/engine073-531-independent24-20261116').resolve()==HERE,
            'wrong_eval_repository')
    pub,pubraw=read_json(HERE/PUBLIC)
    lock,_=read_json(HERE/LOCK)
    src,srcraw=read_json(HERE/SOURCE)
    dev,_=read_json(DEV)
    checked(canonical(pub)==pubraw,'noncanonical_public_payload')
    out=verify_public_data(pub,lock,src,pubraw,srcraw,dev)
    import validate_readiness
    validate_readiness.verify(repo)
    older,_=read_json(HERE.parent/'engine073-531-real-doc24-20261115'/'source-audit.json')
    old_urls={x['url'] for x in older['source_documents']}
    old_shas={x['sha256'].removeprefix('sha256:') for x in older['source_documents']}
    checked(not any(x['url'] in old_urls or x['sha256'] in old_shas for x in src['sources']),
            'consumed_dev_source_reused')
    out['dev_source_overlap']=0
    if gold_path is not None:
        from reviewer_handoff import audit as audit_packet
        packet_path=packet_path.expanduser().absolute()
        audit_receipt=audit_packet(repo,packet_path)
        checked(audit_receipt['source_sha256_verified']==12 and
                audit_receipt['blank_slots_unchanged']==24,'source_packet_tampered')
        path=private_file(Path(gold_path),repo)
        gold,raw=read_json(path)
        checked(canonical(gold)==raw and sha(raw)==lock['private_gold_sha256'],
                'private_gold_sha_or_canonical_mismatch')
        out.update(verify_gold_data(gold,pub,packet_path))
        out['packet_sha_verified']=12
    out.update({'author_self_check':'PASS','independent_reviewer_approval':False,
                'live_authorized':False,'formal_f1':None,'provider_calls':0})
    return out

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--repo',required=True,type=Path)
    p.add_argument('--gold',type=Path)
    p.add_argument('--packet',type=Path,default=PACKET)
    args=p.parse_args()
    print(json.dumps(verify(args.repo,args.gold,args.packet),indent=2,sort_keys=True))
if __name__=='__main__':main()
