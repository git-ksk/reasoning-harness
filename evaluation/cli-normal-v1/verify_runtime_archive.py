#!/usr/bin/env python3
"""Checkout-side verification before any downloaded image can run or get secrets."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import zipfile

SCRIPT_KEY = 'evaluation/cli-normal-v1/verify_runtime_archive.py'
REVIEWER = '/root/real24_operational_review'
SCOPE = 'cli-normal-development-runtime'
BOUND_FIELDS = ('source_commit', 'paired_host_identity_sha256',
                'runtime_image_id', 'runtime_image_archive_sha256')


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def verify_review(review_path, reference=None, receipt=None):
    require((reference is None) != (receipt is None), 'one_trusted_binding_required')
    review = json.loads(review_path.read_bytes())
    require(review.get('reviewer') == REVIEWER
            and review.get('reviewer_kind') == 'agent_nonhuman'
            and review.get('scope') == SCOPE and review.get('result') == 'PASS'
            and review.get('formal_verdict_issued') is True
            and review.get('verdict_stage') == 'FINAL_INDEPENDENT_OPERATIONAL_REVIEW'
            and review.get('synthetic_offline_integration_input_only') is False
            and review.get('not_formal_operational_verdict') is False,
            'final_independent_operational_review_required')
    require(re.fullmatch('[0-9a-f]{40}', str(review.get('source_commit'))) is not None,
            'invalid_source_identity')
    for key in ('runtime_proof_sha256', 'paired_host_identity_sha256',
                'runtime_image_archive_sha256', 'reviewed_runtime_files_sha256'):
        require(re.fullmatch('[0-9a-f]{64}', str(review.get(key))) is not None,
                'invalid_review_digest:' + key)
    require(re.fullmatch('sha256:[0-9a-f]{64}', str(review.get('runtime_image_id'))) is not None,
            'invalid_image_identity')
    if reference is not None:
        require(reference.get('operational_review_sha256') == sha(review_path),
                'reference_review_changed')
        require(reference.get('archive_verifier_sha256') == sha(Path(__file__)),
                'reference_host_verifier_changed')
        binding = reference
    else:
        require(receipt.get('identity') == 'cli-normal-v1-development-freeze-v1'
                and receipt.get('state') == 'DEVELOPMENT_FROZEN'
                and receipt.get('accuracy_acceptance') is False,
                'ordinary_development_receipt_required')
        require(receipt.get('independent_operational_review', {}).get('verdict_sha256') == sha(review_path),
                'receipt_review_changed')
        require(receipt.get('files', {}).get(SCRIPT_KEY) == sha(Path(__file__)),
                'receipt_host_verifier_changed')
        binding = receipt
    for key in BOUND_FIELDS + ('runtime_proof_sha256',):
        require(binding.get(key) == review[key], 'trusted_binding_changed:' + key)
    return review


def extract_verified(zip_path, output, review_path, reference=None, receipt=None):
    review = verify_review(review_path, reference, receipt)
    require(not output.exists() or not any(output.iterdir()), 'output_must_be_empty')
    with zipfile.ZipFile(zip_path) as archive:
        entries = archive.infolist()
        require(len(entries) <= 1000, 'too_many_archive_entries')
        names = set()
        for item in entries:
            name = PurePosixPath(item.filename)
            mode = item.external_attr >> 16
            require(item.filename not in names and not name.is_absolute()
                    and '..' not in name.parts and '\\' not in item.filename
                    and stat.S_IFMT(mode) in (0, stat.S_IFREG, stat.S_IFDIR)
                    and not item.flag_bits & 1, 'unsafe_archive_entry')
            names.add(item.filename)
        proofs = [item for item in entries if not item.is_dir()
                  and PurePosixPath(item.filename).name == 'runtime-proof.json']
        images = [item for item in entries if not item.is_dir()
                  and PurePosixPath(item.filename).name == 'runtime-image.tar.gz']
        require(len(proofs) == len(images) == 1, 'unique_runtime_pair_required')
        proof_entry, image_entry = proofs[0], images[0]
        require(PurePosixPath(proof_entry.filename).parent == PurePosixPath(image_entry.filename).parent,
                'runtime_pair_directory_mismatch')
        require(proof_entry.file_size <= 4 * 1024 * 1024
                and image_entry.file_size <= 4 * 1024 ** 3, 'runtime_archive_too_large')
        proof_bytes = archive.read(proof_entry)
        require(hashlib.sha256(proof_bytes).hexdigest() == review['runtime_proof_sha256'],
                'runtime_proof_changed')
        proof = json.loads(proof_bytes)
        require(proof.get('profile') == 'cli-normal-real24-development-v1'
                and proof.get('normal_only') is True
                and proof.get('state') == 'BUILT_OFFLINE_NOT_EXECUTION_ADMISSION'
                and proof.get('API_execution_admitted') is False,
                'ordinary_offline_proof_required')
        for key in BOUND_FIELDS:
            require(proof.get(key) == review[key], 'proof_review_mismatch:' + key)
        files = proof['runtime_files']
        require(hashlib.sha256(json.dumps(files, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
                == review['reviewed_runtime_files_sha256'], 'reviewed_inventory_changed')
        require(files.get(SCRIPT_KEY) == sha(Path(__file__)), 'proof_host_verifier_changed')
        output.mkdir(parents=True, exist_ok=True)
        partial = output / 'runtime-image.unverified'
        with archive.open(image_entry) as source, partial.open('xb') as target:
            shutil.copyfileobj(source, target, length=1024 * 1024)
        require(sha(partial) == review['runtime_image_archive_sha256'], 'runtime_image_archive_changed')
        partial.rename(output / 'runtime-image.tar.gz')
        (output / 'runtime-proof.json').write_bytes(proof_bytes)
    return proof


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact-zip', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--review', type=Path, required=True)
    binding = parser.add_mutually_exclusive_group(required=True)
    binding.add_argument('--reference', type=Path)
    binding.add_argument('--receipt', type=Path)
    args = parser.parse_args()
    read = lambda path: json.loads(path.read_bytes()) if path else None
    proof = extract_verified(args.artifact_zip, args.output, args.review,
                             read(args.reference), read(args.receipt))
    print(json.dumps({'host_archive_verification': 'PASS',
                      'runtime_image_id': proof['runtime_image_id'],
                      'docker_started': False, 'provider_requests': 0}))


if __name__ == '__main__':
    main()
