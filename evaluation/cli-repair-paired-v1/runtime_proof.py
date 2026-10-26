#!/usr/bin/env python3
"""Record built runtime identity and offline controls; never grants API access."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tarfile


def sha(path):
    digest = hashlib.sha256()
    with path.open('rb') as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--prepared', type=Path, required=True)
    parser.add_argument('--bin-dir', type=Path, required=True)
    parser.add_argument('--controls', type=Path, required=True)
    parser.add_argument('--image-id', required=True)
    parser.add_argument('--image-archive', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    read = lambda path: json.loads(path.read_bytes())
    prepared = read(args.prepared / 'preparation.json')
    runtime = args.prepared / 'repo'
    files = {str(path.relative_to(runtime)): sha(path) for path in sorted(runtime.rglob('*'))
             if path.is_file() and '__pycache__' not in path.parts}
    assert files == prepared['files']
    protocol = read(runtime / 'evaluation/cli-repair-paired-v1/protocol.json')
    assert protocol['state'] in ['PREPARATION_NO_LIVE_ALLOWED', 'DEVELOPMENT_REQUIRES_FROZEN_RECEIPT']
    assert protocol['providers']['groq']['new_live_run_allowed'] is False
    identity = read(args.bin_dir / 'identity.json')
    actual_image = read(args.controls / 'image-inventory.json')
    assert actual_image['runtime_files'] == files
    assert actual_image['binary_files'] == {path.name: sha(path) for path in args.bin_dir.iterdir() if path.is_file()}
    assert identity['host_commit'] == protocol['common_cli_host']
    assert identity['build_script_sha256'] == sha(runtime / 'evaluation/cli-repair-paired-v1/build.py')
    assert identity['registry_source_sha256'] == sha(runtime / 'evaluation/cli-repair-paired-v1/registry.rs')
    for arm in ['baseline', 'candidate']:
        record = identity['arms'][arm]
        assert record['engine_commit'] == protocol[arm]
        original, patched = record['original_core_files'], record['patched_core_files']
        assert original.keys() == patched.keys()
        assert [key for key in original if original[key] != patched[key]] == ['src/resolution.rs']
        assert set(record['binaries']) == {'reason', 'adviser', 'source-host', 'audit-probe', 'registry'}
        for name, digest in record['binaries'].items():
            assert sha(args.bin_dir / (name + '-' + arm)) == digest
    baseline, candidate = identity['arms']['baseline'], identity['arms']['candidate']
    assert baseline['provider_patch'] == candidate['provider_patch']
    assert baseline['common_resolution_patch_sha256'] == candidate['common_resolution_patch_sha256']
    controls = {}
    for name, count in [('provider', 18), ('cli', 17), ('containment', 7)]:
        path = args.controls / name / 'summary.json'
        data = read(path)
        assert data['all_pass'] is True and len(data['controls']) == count
        assert data['external_provider_requests'] == 0
        controls[name] = {'count': count, 'summary_sha256': sha(path)}
    registry = read(args.controls / 'registry-parity.json')
    assert registry['pass'] is True and registry['identical_output'] is True
    unit = (args.controls / 'unit.log').read_text()
    expected_units = protocol.get('offline_validation', {}).get('expected_unit_controls', 51)
    assert 'Ran ' + str(expected_units) + ' tests' in unit and unit.rstrip().endswith('OK')
    assert args.image_id.startswith('sha256:') and len(args.image_id) == 71
    with tarfile.open(args.image_archive, 'r:gz') as archive:
        manifest = json.load(archive.extractfile('manifest.json'))
        assert len(manifest) == 1
        config = archive.extractfile(manifest[0]['Config']).read()
        config_id = 'sha256:' + hashlib.sha256(config).hexdigest()
        archive_ids = [config_id]
        if 'index.json' in archive.getnames():
            index = json.load(archive.extractfile('index.json'))
            assert len(index['manifests']) == 1
            descriptor = index['manifests'][0]
            digest = descriptor['digest']
            assert digest.startswith('sha256:') and len(digest) == 71
            blob = archive.extractfile('blobs/sha256/' + digest[7:]).read()
            assert 'sha256:' + hashlib.sha256(blob).hexdigest() == digest
            image_manifest = json.loads(blob)
            assert image_manifest['config']['digest'] == config_id
            archive_ids.append(digest)
        # Docker stores may expose a config ID or an OCI manifest ID. Both
        # accepted identities are derived from archive bytes and linked to its
        # one actual config; a declared index digest alone is insufficient.
        assert args.image_id in archive_ids
    proof = {'state': 'BUILT_OFFLINE_NOT_EXECUTION_ADMISSION',
             'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=args.repo).decode().strip(),
             'runtime_image_id': args.image_id, 'runtime_image_archive_sha256': sha(args.image_archive),
             'archive_verified_image_ids': archive_ids,
             'paired_host_identity_sha256': sha(args.bin_dir / 'identity.json'),
             'preparation_sha256': sha(args.prepared / 'preparation.json'),
             'protocol_sha256': sha(runtime / 'evaluation/cli-repair-paired-v1/protocol.json'),
             'runtime_files': files, 'controls': controls, 'registry_parity_sha256': sha(args.controls / 'registry-parity.json'),
             'actual_image_inventory_sha256': sha(args.controls / 'image-inventory.json'),
             'unit_controls': expected_units, 'unit_log_sha256': sha(args.controls / 'unit.log'),
             'independent_operational_review_complete': False, 'new_external_provider_requests': 0,
             'new_accuracy_cases': 0, 'F1_recomputed': False, 'API_execution_admitted': False,
             'runtime_proof_script_sha256': sha(Path(__file__))}
    with args.output.open('x') as handle:
        json.dump(proof, handle, indent=2)
        handle.write('\n')
    print(json.dumps({'state': proof['state'], 'runtime_image_id': args.image_id, 'API_execution_admitted': False}))


if __name__ == '__main__':
    main()
