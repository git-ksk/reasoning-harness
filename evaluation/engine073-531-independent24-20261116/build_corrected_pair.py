#!/usr/bin/env python3
"""Build provider-free paired native CLIs with identical #552 and #613 Core patches.

Historical commits and evaluator contract are immutable.  Never calls a model,
never uses billing credentials or modifies the original one-shot experiment.
"""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

REPO_PART='evaluation/cli-repair-paired-v1/normal_v3'
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/REPO_PART))
import build_native_pair as original

CLI_COMMIT='b85a4a0b2ab03397b361a8620d7a97dd26b2eb36'
ARMS={
    'baseline':'4abee90f0501f0d8fc09b453a0b45c942fbed69e',
    'candidate':'7466c6f90d420a98c99713b96d6a0623e2aeef52',
}
COMMON_FROM=original.PATCH_FROM
COMMON_TO=original.PATCH_TO
FIX_FROM='170c4fa73cdd967b0637da9e9e47a0cd272f5983'
FIX_TO='a538403954588549602e6e2671c059c72e415b11'
CORE_PATH='crates/reasoning-harness-core/src/resolution.rs'
COMMON_PATCH_SHA256='f0cbea7b4ca7f2d265468d1dab7fb0155fb2dc06bd393178eacbe10627c42c39'
FIX_PATCH_SHA256='e6688e0d16a312e77cc7c2d1e0d132220a7a679ee97ae9437cc6186889a2c019'


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def require_commit(repo, commit):
    original.require(re.fullmatch('[0-9a-f]{40}',commit) is not None,
                     'mutable_commit_not_allowed')
    original.require(original.git(repo,'rev-parse',commit+'^{commit}').decode().strip()==commit,
                     'unknown_pinned_commit')


def patches(repo):
    for commit in [CLI_COMMIT, *ARMS.values(), COMMON_FROM, COMMON_TO, FIX_FROM, FIX_TO]:
        require_commit(repo,commit)
    # Reproducible across operating systems and Git releases. Git may change
    # hunk-header context, whitespace or diff algorithms without changing the
    # underlying commit. Bundle exact previously reviewed patch bytes instead
    # of treating `git diff` textual output as a portable content hash.
    files=Path(__file__).resolve().parent/'patches'
    old=base64.b64decode(b''.join((files/'common552.patch.b64').read_bytes().split()),validate=True)
    fix=base64.b64decode(b''.join((files/'issue613-reject-acquisition.patch.b64').read_bytes().split()),validate=True)
    original.require(sha(old)==COMMON_PATCH_SHA256,'common_core_patch_drift')
    original.require(sha(fix)==FIX_PATCH_SHA256,'issue613_core_patch_drift')
    original.require(sum(line.startswith(b'@@ ') for line in fix.splitlines())==2,
                     'issue613_expected_two_function_hunks_only')
    original.require(b'run_with_pending_investigation' not in fix,
                     'unexpected_run_api_change')
    return old,fix


def prepare(repo, output, compile_binaries=False):
    repo=Path(repo).resolve(strict=True)
    output=Path(output).resolve()
    original.require(not output.exists(), 'must_use_fresh_private_output_root')
    old_patch,fix_patch=patches(repo)
    original.private_new_directory(output)
    if compile_binaries:
        original.private_new_directory(output/'bin')
    proof={
        'contract':'engine073-531-corrected-shared-cli-pair-v1',
        'classification':'PROVIDER_FREE_EVALUATION_PREPARATION_NOT_F1',
        'cli_commit':CLI_COMMIT,
        'baseline_engine_commit':ARMS['baseline'],
        'candidate_engine_commit':ARMS['candidate'],
        'common_core_patch_sha256':sha(old_patch),
        'issue613_core_patch_sha256':sha(fix_patch),
        'source_identical_except_core_and_lock':False,
        'api_calls':0,
        'provider_generation_authorized':False,
        'historical_run_replayed':False,
        'formal_independent_f1':None,
        'built':compile_binaries,
        'arms':{},
    }
    for arm,core_commit in ARMS.items():
        source=output/('source-'+arm)
        original.private_new_directory(source)
        original.export(repo,CLI_COMMIT,
            (*original.SOURCE_FILES,*original.SOURCE_DIRS),source)
        core=source/'crates/reasoning-harness-core'
        shutil.rmtree(core)
        original.export(repo,core_commit,('crates/reasoning-harness-core',),source)
        before={str(p.relative_to(core)):sha(p.read_bytes())
                for p in core.rglob('*') if p.is_file()}
        for patch in (old_patch,fix_patch):
            subprocess.run(['patch','-p1','--batch','--forward'],input=patch,
                           cwd=source,check=True,capture_output=True,timeout=20)
        after={str(p.relative_to(core)):sha(p.read_bytes())
               for p in core.rglob('*') if p.is_file()}
        original.require(before.keys()==after.keys() and
                         [k for k in before if before[k]!=after[k]]==['src/resolution.rs'],
                         'unexpected_core_patch_surface')
        resolution=(core/'src/resolution.rs').read_text()
        original.require('let pending = if pending_investigation && attempts.is_empty()' in resolution and
                         '&& pending.as_ref().is_none_or(Vec::is_empty)' in resolution,
                         'post_reject_readonly_acquisition_not_in_both_arms')
        version=re.search(r'^version = "([^"]+)"',
                          (core/'Cargo.toml').read_text(),re.M).group(1)
        lock=source/'Cargo.lock'
        locked,count=re.subn(r'(name = "reasoning-harness-core"\nversion = ")[^"]+("\n)',
                             lambda match: match[1]+version+match[2],lock.read_text())
        original.require(count==1,'native_core_lock_inventory_changed')
        lock.write_text(locked)
        if compile_binaries:
            env={k:v for k,v in os.environ.items()
                 if k not in ('MISTRAL_API_KEY','GROQ_API_KEY','GEMINI_API_KEY','OPENAI_API_KEY')}
            env.update(CARGO_TARGET_DIR=str(output/'target'),
                       CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
            with (output/(arm+'-build.log')).open('xb') as log:
                subprocess.run(['cargo','build','--locked','--offline','-p',
                                'reasoning-harness-cli','--bin','reason','--jobs','2'],
                               cwd=source,env=env,stdout=log,
                               stderr=subprocess.STDOUT,check=True)
            src=output/'target/debug/reason'
            dst=output/'bin'/('reason-'+arm)
            shutil.copyfile(src,dst)
            dst.chmod(0o700)
        inventory={name:sha((source/name).read_bytes())
                   for name in original.source_inventory(source)}
        proof['arms'][arm]={
            'engine_commit':core_commit,
            'core_package_version':version,
            'patched_core_file_sha256':after['src/resolution.rs'],
            'source_file_sha256':inventory,
            'binary_sha256':sha(dst.read_bytes()) if compile_binaries else None,
            'binary':str(dst) if compile_binaries else None,
            'source_root':str(source),
        }
    first=proof['arms']['baseline']['source_file_sha256']
    second=proof['arms']['candidate']['source_file_sha256']
    without_core=lambda d:{k:v for k,v in d.items()
                            if not k.startswith('crates/reasoning-harness-core/')
                            and k!='Cargo.lock'}
    original.require(without_core(first)==without_core(second),
                     'corrected_pair_host_drift')
    proof['source_identical_except_core_and_lock']=True
    # Private on-machine build manifest, never history/answer gold or API secrets.
    report=output/'pair-preflight.json'
    report.write_text(json.dumps(proof,sort_keys=True,indent=2)+'\n')
    report.chmod(0o600)
    return {'contract':proof['contract'], 'validated':True,
            'built':compile_binaries, 'cli_commit':CLI_COMMIT,
            'arms':{arm:{'engine_commit':entry['engine_commit'],
                         'core_package_version':entry['core_package_version'],
                         'binary_sha256':entry['binary_sha256']}
                    for arm,entry in proof['arms'].items()},
            'host_parity':True,'provider_calls':0,
            'formal_independent_f1':None}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--mode',choices=['validate-only','build'],default='validate-only')
    args=parser.parse_args()
    print(json.dumps(prepare(args.repo,args.output,args.mode=='build'),
                     sort_keys=True,indent=2))


if __name__=='__main__':
    main()
