#!/usr/bin/env python3
"""Offline readiness receipt for prospective unseen independent 24-case evaluation.

Never generates or scores model answers. No live mode. Never trusts an unreviewed
source inventory as an approved/frozen benchmark or inferred cost entitlement.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import stage_source_pool as source_pool
import build_corrected_pair as pair

CONTRACT='reason-engine073-531-fresh-independent24-readiness-v1'
SOURCE_MANIFEST='evaluation/engine073-531-independent24-20261116/provisional-new-sources.json'


def verify(repo):
    repo=repo.resolve(strict=True)
    protocol=json.loads((HERE/'holdout-readiness.json').read_text())
    if protocol.get('contract')!=CONTRACT or protocol.get('phase')!='source_pool_prepared_only':
        raise ValueError('holdout_readiness_wrong_phase')
    if (protocol.get('source_pool_path')!=SOURCE_MANIFEST or
        protocol.get('corrected_shared_cli_commit')!=pair.CLI_COMMIT or
        protocol.get('baseline_core_commit')!=pair.ARMS['baseline'] or
        protocol.get('candidate_core_commit')!=pair.ARMS['candidate']):
        raise ValueError('holdout_readiness_unpinned_source_or_engine')
    full=repo/SOURCE_MANIFEST
    content=full.read_bytes()
    if hashlib.sha256(content).hexdigest()!=protocol['source_pool_sha256']:
        raise ValueError('holdout_source_pool_digest_drift')
    manifest=json.loads(content)
    source_pool.validate(repo,manifest)
    expected_counts={
        'planned_source_files':12,'planned_question_count':24,
        'planned_answerable_count':16,'planned_unanswerable_count':8,
        'first_requests_per_arm':24,'planned_independent_shards':8,
        'questions_per_shard':3,'paired_cli_runs_per_shard':6,
        'per_cli_max_model_calls':12,'per_cli_max_tokens':28096,
        'maximum_total_model_calls':576,'maximum_total_tokens':1348608,
        'model_calls_performed':0,
    }
    for key,expected in expected_counts.items():
        if protocol.get(key)!=expected:
            raise ValueError('holdout_readiness_case_or_budget_drift_'+key)
    if protocol.get('provider')!='mistral' or protocol.get('model')!='ministral-8b-2512':
        raise ValueError('holdout_provider_or_model_drift')
    if protocol.get('source_authorship_independent_review_required') is not True:
        raise ValueError('holdout_reviewer_requirement_removed')
    for key in [
        'reviewed_frozen_public_cases_present','independent_gold_privately_frozen',
        'independent_reviewer_approval','independent_reviewer_key_enrolled',
        'same_account_no_overage_billing_checked',
        'paired_first_requests_for_fresh_cases_checked',
        'one_shot_live_workflow_created','one_shot_live_authorized',
        'development_f1_source_reused','quality_go','release_go',
    ]:
        if protocol.get(key) is not False:
            raise ValueError('unauthorized_holdout_approval_or_release_'+key)
    if protocol.get('formal_independent_f1') is not None:
        raise ValueError('independent_f1_not_yet_observed')
    threshold={
        'case_f1_min':0.9,'case_precision_min':0.95,'case_recall_min':0.85,
        'unsafe_negative_assertions_max':0,'invalid_citations_max':0,
        'all_planned_arms_complete_required':True,
        'paired_candidate_regression_forbidden':True,
        'unknown_ambiguous_or_timeout_not_scored_as_success':True,
    }
    if protocol.get('score_thresholds')!=threshold:
        raise ValueError('pre_registered_quality_threshold_changed')
    return {
        'contract':CONTRACT,'source_files_pinned':12,
        'source_repositories_new_and_disjoint':5,
        'corrected_pair_commits_pinned':True,
        'fresh_questions_authored':False,
        'fresh_independent_reviewer_approved':False,
        'fresh_gold_private_and_frozen':False,
        'account_billing_gate_checked':False,
        'provider_calls':0,
        'live_execution_allowed':False,
        'formal_independent_f1':None,
        'quality_go':False,'release_go':False,
    }


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',type=Path,required=True)
    args=parser.parse_args()
    print(json.dumps(verify(args.repo),indent=2,sort_keys=True))

if __name__=='__main__':main()
