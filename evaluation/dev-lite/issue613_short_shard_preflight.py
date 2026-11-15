#!/usr/bin/env python3
"""Issue #613 prospective eight-shard offline contract, never a live provider runner.

Requires no CLI build, provider credential, network, or private evaluation gold.
Prints only public case IDs, manifest hashes, and upper-bound counts. Deliberately
has no --live mode and does not invoke issue531_real24_live.measure().
"""
import argparse
import hashlib
import json
from pathlib import Path

import issue531_real24_live as real


def preflight(repo, shard):
    manifest_path = repo/'evaluation/engine073-531-real-doc24-20261115/cases.json'
    manifest = real.validate_public_manifest(manifest_path,repo)
    # The original 24-case manifest is frozen.  The partition helper validates
    # ordered unique *id* values; use a separate projection, not mutable rows.
    rows = [{'id':r['case_id']} for r in manifest['cases']]
    selected = real.selected_shard(rows,shard,8)
    original = {r['case_id']:r for r in manifest['cases']}
    pinned_records = 0
    for case in selected:
        row = original[case['id']]
        cfg_path = (manifest_path.parent/row['config_path']).resolve(strict=True)
        assert cfg_path.is_relative_to(manifest_path.parent.resolve(strict=True))
        raw=cfg_path.read_bytes()
        real.v5.checked(hashlib.sha256(raw).hexdigest()==row['config_raw_sha256'],
                        'original_config_sha256_changed_'+case['id'])
        config=json.loads(raw)
        real.v5.checked(config['schema_version']=='reason-config-v1',
                        'wrong_config_schema_'+case['id'])
        caps=config['resolution']['investigation']['capabilities']
        real.v5.checked(len(caps)==2,'wrong_source_capabilities_'+case['id'])
        for cap in caps:
            real.v5.checked(cap['kind']=='captured_evidence' and cap['read_only'] is True,
                            'not_frozen_readonly_evidence_'+case['id'])
            for record in cap['evidence']:
                observation=json.loads(record['observation'])
                source=real.validate_real_source(observation,case['id'])
                real.v5.checked(observation['source_version']==manifest['source_content_sha256'][source],
                                'unpinned_upstream_source_'+case['id'])
                pinned_records+=1
    real.v5.checked(pinned_records==6,'shard_source_count_not_six')
    return {
        'contract':'issue613-eight-independent-jobs-offline-preflight-v1',
        'classification':'REUSED_DEV_INPUTS_ONLY_NO_FORMAL_F1',
        'shard_index_zero_based':shard,
        'shard_count':8,
        'case_ids':[row['id'] for row in selected],
        'planned_cli_runs':2*len(selected),
        'max_internal_model_calls_if_future_authorized':2*len(selected)*real.v5.MAX_CALLS,
        'max_tokens_if_future_authorized':2*len(selected)*real.v5.TOTAL_CAP,
        'pinned_source_records':pinned_records,
        'public_manifest_sha256':real.PUBLIC_MANIFEST_SHA256,
        'provider_calls':0,
        'credentials_read':False,
        'live_execution_authorized':False,
        'historical_run_replay':False,
        'formal_independent_f1':None,
    }


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',required=True,type=Path)
    parser.add_argument('--shard',required=True,type=int,choices=range(8))
    args=parser.parse_args()
    print(json.dumps(preflight(args.repo.resolve(strict=True),args.shard),
                     indent=2,sort_keys=True))


if __name__=='__main__':
    main()
