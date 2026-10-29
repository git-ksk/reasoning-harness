#!/usr/bin/env python3
"""Derive the ordinary v2 preparation from a pinned, immutable v1 tree (offline)."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

HERE = Path(__file__).resolve().parent
PIN = '7073471d7490c64841200ae9ddaf252317c34721'
OLD_HOST = 'dbad2dc25d3487819206582f1fad778d9af75da1'


def sha(data):
    return hashlib.sha256(data).hexdigest()


def require(ok, message):
    if not ok:
        raise ValueError(message)


def substitutions(host):
    return [('cli-normal-real24-development-v1','cli-normal-real24-development-v2'),
            ('cli-normal-v1','cli-normal-v2'), ('normal_v1','normal_v2'),
            ('cli-normal-runtime.yml','cli-normal-runtime-v2.yml'),
            ('cli-normal-real24-development.yml','cli-normal-real24-development-v2.yml'),
            (OLD_HOST,host), ('cli-normal-development-runtime','cli-normal-v2-development-runtime')]


def transform(path, data, expected, host):
    text = data.decode('utf-8')
    for old, new in substitutions(host):
        count = text.count(old)
        require(count == expected.get(old, 0), f'unexpected_replacement_count:{path}:{old}:{count}')
        text = text.replace(old, new)
    if path == '.github/workflows/cli-normal-runtime.yml':
        old_test = "-p 'test_runtime_archive.py'"
        require(text.count(old_test) == 1, 'workflow_test_scope_mismatch')
        text = text.replace(old_test, "-p 'test_*.py'")
        old_group = 'group: cli-normal-runtime\n'
        require(text.count(old_group) == 1, 'workflow_concurrency_group_mismatch')
        text = text.replace(old_group, 'group: cli-normal-runtime-v2\n')
        old_build = 'docker build -f "$stage/normal_v2/Dockerfile.runtime" -t paired-runtime target/paired-context'
        require(text.count(old_build) == 1, 'runtime_build_export_flags_mismatch')
        text = text.replace(old_build, 'docker build --platform linux/amd64 --provenance=false --sbom=false -f "$stage/normal_v2/Dockerfile.runtime" -t paired-runtime target/paired-context')
    return text.encode()


def destination(path):
    return path.replace('evaluation/cli-normal-v1/', 'evaluation/cli-normal-v2/').replace(
        '.github/workflows/cli-normal-runtime.yml', '.github/workflows/cli-normal-runtime-v2.yml').replace(
        '.github/workflows/cli-normal-real24-development.yml', '.github/workflows/cli-normal-real24-development-v2.yml')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--write-derived', action='store_true', help='create v2 checkout-side workflow and verifier files; never overwrite')
    args = parser.parse_args()
    require((args.output is None) != (not args.write_derived), 'choose_output_or_write_derived')
    repo = args.repo.resolve()
    config = json.loads((HERE/'config.json').read_bytes())
    require(config['source_commit'] == PIN and config['execution_admitted'] is False, 'v2_source_or_safety_changed')
    host = config['common_cli_host']
    require(host == 'PENDING' or re.fullmatch('[0-9a-f]{40}', host), 'invalid_common_host')
    counts = json.loads((HERE/'replacement-counts.json').read_bytes())
    require(subprocess.check_output(['git','rev-parse',PIN+'^{commit}'],cwd=repo,text=True).strip() == PIN,'source_commit_missing')
    blob = subprocess.check_output(['git','archive',PIN], cwd=repo)
    with tempfile.TemporaryDirectory(prefix='normal-v2-frozen-source-') as temporary:
        staging = Path(temporary)
        with tarfile.open(fileobj=io.BytesIO(blob)) as archive:
            for member in archive.getmembers():
                require(not member.name.startswith('/') and '..' not in Path(member.name).parts and (member.isfile() or member.isdir()), 'unsafe_archive_entry')
            archive.extractall(staging, filter='data')
        expected_sources = {
            'evaluation/cli-normal-v1/prepare_runtime.py',
            'evaluation/cli-normal-v1/verify_runtime_archive.py',
            'evaluation/cli-normal-v1/test_runtime_archive.py',
            '.github/workflows/cli-normal-runtime.yml',
            '.github/workflows/cli-normal-real24-development.yml',
        }
        runtime_dir = staging/'evaluation/cli-normal-v1/runtime'
        expected_sources.update(str(p.relative_to(staging)) for p in runtime_dir.iterdir()
                                if p.is_file() and (p.suffix in ('.py','.json','.md') or p.name=='Dockerfile.runtime'))
        require(set(counts)==expected_sources, 'derived_source_inventory_changed')
        provenance = {}
        for source, expected in sorted(counts.items()):
            src = staging/source
            require(src.is_file(), 'missing_pinned_source:'+source)
            before = src.read_bytes()
            after = transform(source, before, expected, host)
            dest = destination(source)
            target = staging/dest
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(after)
            provenance[dest] = {'source':source,'before_sha256':sha(before),'after_sha256':sha(after),'replacement_counts':expected}
        derived = ['evaluation/cli-normal-v2/verify_runtime_archive.py', 'evaluation/cli-normal-v2/test_runtime_archive.py', 'evaluation/cli-normal-v2/runtime/tag_admission.py', '.github/workflows/cli-normal-runtime-v2.yml', '.github/workflows/cli-normal-real24-development-v2.yml']
        for name in derived:
            target = repo/name
            if args.write_derived:
                target.parent.mkdir(parents=True, exist_ok=True)
                with target.open('xb') as stream:
                    stream.write((staging/name).read_bytes())
            else:
                require(target.is_file() and target.read_bytes() == (staging/name).read_bytes(), 'checkout_derived_file_changed:'+name)
        if args.write_derived:
            print(json.dumps({'state':'GENERATED_NOT_ADMITTED','files':derived,'common_cli_host':host}))
            return
        # The pinned inherited preparation uses its own fixed git archive as input,
        # but overlays files relative to the staged snapshot. Never use cwd's mutable overlay.
        inherited = staging/'evaluation/cli-repair-paired-v1/prepare.py'
        before = inherited.read_bytes()
        text = before.decode()
        require(text.count('import io\n') == 1 and text.count('cwd=args.repo)') == 1, 'inherited_prepare_patch_mismatch')
        text = text.replace('import io\n', 'import io\nimport os\n').replace('cwd=args.repo)', "cwd=Path(os.environ['NORMAL_V2_GIT_OBJECT_REPO']))")
        inherited.write_text(text)
        provenance['evaluation/cli-repair-paired-v1/prepare.py']={'source':'evaluation/cli-repair-paired-v1/prepare.py','before_sha256':sha(before),'after_sha256':sha(inherited.read_bytes()),'patch':'fixed git object store separated from frozen overlay'}
        env = {**os.environ, 'NORMAL_V2_GIT_OBJECT_REPO':str(repo)}
        subprocess.run([sys.executable,str(staging/'evaluation/cli-normal-v2/prepare_runtime.py'),'--repo',str(staging),'--output',str(args.output.resolve())],check=True,env=env)
        manifest_path=args.output.resolve()/'preparation.json'
        manifest=json.loads(manifest_path.read_bytes())
        manifest['v2_derivation']={'source_commit':PIN,'common_cli_host':host,'execution_admitted':False,
            'generator_sha256':sha(Path(__file__).read_bytes()),'config_sha256':sha((HERE/'config.json').read_bytes()),
            'replacement_counts_sha256':sha((HERE/'replacement-counts.json').read_bytes()),'files':provenance}
        manifest['frozen_execution_receipt']=False
        manifest['independent_review_complete']=False
        manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
        print(json.dumps({'state':'PREPARATION_ONLY','profile':manifest['profile'],'inventory_files':len(manifest['files']), 'common_cli_host':host,'live_execution_admitted':False}))


if __name__=='__main__':
    main()
