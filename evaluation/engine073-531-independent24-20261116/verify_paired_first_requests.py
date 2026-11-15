#!/usr/bin/env python3
"""Verify corrected native pair first-request parity with *consumed dev inputs only*.

This verifies the common CLI and frozen source config wiring. It does not
regenerate provider outputs, score historic data or create an independent sample.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
sys.path.insert(0,str(ROOT/'evaluation/dev-lite'))
import issue531_real24_live as dev
import build_corrected_pair as pair


def check_build(build_path):
    proof=json.loads(build_path.read_text())
    if (proof.get('contract')!='engine073-531-corrected-shared-cli-pair-v1'
        or proof.get('cli_commit')!=pair.CLI_COMMIT
        or proof.get('source_identical_except_core_and_lock') is not True
        or proof.get('built') is not True
        or proof.get('provider_generation_authorized') is not False
        or proof.get('formal_independent_f1') is not None):
        raise ValueError('unverified_corrected_cli_pair')
    for arm,commit in pair.ARMS.items():
        record=proof['arms'][arm]
        path=Path(record['binary'])
        if (record['engine_commit']!=commit or not path.is_file()
            or hashlib.sha256(path.read_bytes()).hexdigest()!=record['binary_sha256']):
            raise ValueError('native_pair_binary_changed_or_wrong_arm_'+arm)
    return proof


def verify(repo, build_file, output):
    repo=repo.resolve(strict=True)
    if output.exists():
        raise ValueError('refuse_to_reuse_or_overwrite_paired_plan')
    build=check_build(build_file.resolve(strict=True))
    output.mkdir(mode=0o700,parents=True)
    manifest=repo/'evaluation/engine073-531-real-doc24-20261115/cases.json'
    # dev.setup() performs a deterministic 48-plan binary comparison without
    # any Mistral request or credential, even if a key exists in the shell.
    rows=dev.setup(repo,output,manifest,build)
    result=json.loads((output/'preflight.json').read_text())
    if not (result['case_count']==24 and result['paired_requests']==48
            and result['pair_equality'] is True and result['model_completions']==0
            and len(set(result['request_hashes'].values()))==24):
        raise ValueError('native_pair_first_requests_not_identical')
    return {
        'contract':'engine073-531-fixed-cli-48-first-request-parity-v1',
        'classification':'USED_DEV_INPUTS_REQUEST_PARITY_ONLY_NO_INDEPENDENT_F1',
        'cli_commit':pair.CLI_COMMIT,
        'baseline_engine_commit':pair.ARMS['baseline'],
        'candidate_engine_commit':pair.ARMS['candidate'],
        'case_count':len(rows),
        'paired_first_requests':result['paired_requests'],
        'all_paired_requests_identical':True,
        'model_calls':0,
        'billing_gate_checked':False,
        'formal_independent_f1':None,
    }


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',required=True,type=Path)
    parser.add_argument('--build',required=True,type=Path)
    parser.add_argument('--output',required=True,type=Path)
    args=parser.parse_args()
    print(json.dumps(verify(args.repo,args.build,args.output),indent=2,sort_keys=True))

if __name__=='__main__':main()
