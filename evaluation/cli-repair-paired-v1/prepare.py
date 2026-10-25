#!/usr/bin/env python3
"""Materialize the pinned evaluator plus this preparation overlay, offline."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shutil
import subprocess
import tarfile

HERE = Path(__file__).resolve().parent
PIN = '6165de77c1544a7fe58fc1e675e7cb3a8ec7cc98'
FOLDERS = ['engine073-realworld-evaluator-v2', 'engine073-answer-metrics-development-v1',
           'engine07-quality-gates-v1', 'engine07-realworld-runner-v1',
           'engine07-realworld-operations-v1']
WORKFLOWS = ['engine073-realworld-independent-v2.yml', 'engine073-realworld-development-v2.yml']
OVERLAY = ['assemble_image.py', 'image_inventory.py', 'image_identity.py', 'verify_development.py', 'build.py', 'registry.rs', 'resolver.py', 'capture.py', 'terminal_chain.py',
           'display_projection.py', 'development.py', 'operational_bridge.py', 'test_operational_bridge.py', 'provider_readiness.py', 'protocol.json', 'run_task.py',
           'run_lane.py', 'preparation_guard.py', 'test_review_boundaries.py', 'test_plan_admission.py',
           'preflight_cli.py', 'collect.py', 'plan_inputs.py', 'score_development_first.py', 'test_callback_admission.py', 'tag_admission.py', 'test_tag_admission.py', 'prepare_consumed_sources.py', 'test_projection_boundary.py', 'verify_containment.py', 'README.md', 'PROTOCOL.md']


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    paths = ['evaluation/' + name for name in FOLDERS]
    paths += ['.github/workflows/' + name for name in WORKFLOWS]
    blob = subprocess.check_output(['git', 'archive', PIN, *paths], cwd=args.repo)
    source_manifest = json.loads((HERE / 'upstream-files.json').read_text())
    imported = {}
    contents = {}
    with tarfile.open(fileobj=io.BytesIO(blob)) as archive:
        for member in archive.getmembers():
            path = Path(member.name)
            if not member.isfile():
                continue
            if path.parts[0] == 'evaluation':
                if len(path.parts) != 3 or not (path.suffix in ['.py', '.rs', '.json', '.md'] or path.name.startswith('Dockerfile')):
                    continue
                if path.name == 'runtime-reuse-v1.json':
                    continue  # a historical runtime receipt cannot admit this host
            if member.name not in source_manifest:
                raise ValueError('undeclared upstream file: ' + member.name)
            data = archive.extractfile(member).read()
            if sha(data) != source_manifest[member.name]:
                raise ValueError('upstream digest mismatch: ' + member.name)
            destination = member.name.replace('engine073-realworld-evaluator-v2', 'cli-repair-paired-v1')
            transformed = data.replace(b'engine073-realworld-evaluator-v2', b'cli-repair-paired-v1')
            imported[member.name] = {'upstream_sha256': sha(data), 'destination': destination,
                                     'path_rewritten_sha256': sha(transformed)}
            contents[destination] = transformed
    if set(imported) != set(source_manifest):
        raise ValueError('incomplete upstream inventory')
    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.mkdir()  # never overwrite an earlier preparation or observation
    repo = output / 'repo'
    for destination, data in contents.items():
        path = repo / destination
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    for name in OVERLAY:
        shutil.copyfile(HERE / name, repo / 'evaluation/cli-repair-paired-v1' / name)
    for name in ['cli-repair-paired-runtime.yml', 'cli-repair-paired-development.yml']:
        shutil.copyfile(HERE.parents[1] / '.github/workflows' / name, repo / '.github/workflows' / name)
    inventory = {str(path.relative_to(repo)): sha(path.read_bytes())
                 for path in sorted(repo.rglob('*')) if path.is_file()}
    proof = {'state': json.loads((HERE / 'protocol.json').read_bytes())['state'], 'upstream_commit': PIN,
             'prepare_script_sha256': sha(Path(__file__).read_bytes()), 'imported': imported,
             'overlay': {name: sha((HERE / name).read_bytes()) for name in OVERLAY},
             'files': inventory, 'frozen_execution_receipt': False,
             'independent_review_complete': False, 'accuracy_cases': 0}
    (output / 'preparation.json').write_text(json.dumps(proof, indent=2) + '\n')
    print(json.dumps({'state': proof['state'], 'files': len(inventory), 'output': str(output)}))


if __name__ == '__main__':
    main()
