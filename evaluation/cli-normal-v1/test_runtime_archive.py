"""Inert host-boundary controls. No real review, Docker, credential or model use."""
import copy
import hashlib
import json
import os
from pathlib import Path
import shlex
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile

import verify_runtime_archive as gate


class HostArchiveBoundary(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.image = b'INERT FIXTURE BYTES, NOT A DOCKER IMAGE'
        self.files = {gate.SCRIPT_KEY: gate.sha(Path(gate.__file__))}
        self.proof = {'source_commit': 'a' * 40, 'paired_host_identity_sha256': 'b' * 64,
                      'runtime_image_id': 'sha256:' + 'c' * 64,
                      'archive_verified_image_ids': ['sha256:' + 'c' * 64, 'sha256:' + 'd' * 64],
                      'runtime_image_archive_sha256': hashlib.sha256(self.image).hexdigest(),
                      'runtime_files': self.files, 'normal_only': True,
                      'profile': 'cli-normal-real24-development-v1',
                      'state': 'BUILT_OFFLINE_NOT_EXECUTION_ADMISSION', 'API_execution_admitted': False}
        self.proof_bytes = json.dumps(self.proof).encode()
        self.review = {key: self.proof[key] for key in gate.BOUND_FIELDS}
        self.review.update(reviewer=gate.REVIEWER, reviewer_kind='agent_nonhuman', scope=gate.SCOPE,
                           result='PASS', formal_verdict_issued=True,
                           verdict_stage='FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                           synthetic_offline_integration_input_only=False, not_formal_operational_verdict=False,
                           runtime_proof_sha256=hashlib.sha256(self.proof_bytes).hexdigest(),
                           reviewed_runtime_files_sha256=hashlib.sha256(json.dumps(self.files, sort_keys=True, separators=(',', ':')).encode()).hexdigest())
        self.review_path = self.root / 'inert-review.json'
        self.review_path.write_text(json.dumps(self.review))
        self.reference = {key: self.review[key] for key in gate.BOUND_FIELDS + ('runtime_proof_sha256',)}
        self.reference.update(operational_review_sha256=gate.sha(self.review_path),
                              archive_verifier_sha256=gate.sha(Path(gate.__file__)))
        self.receipt = {key: self.review[key] for key in gate.BOUND_FIELDS + ('runtime_proof_sha256',)}
        self.receipt.update(identity='cli-normal-v1-development-freeze-v1', state='DEVELOPMENT_FROZEN',
                            accuracy_acceptance=False, files=self.files,
                            independent_operational_review={'verdict_sha256': gate.sha(self.review_path)})
        self.zip = self.root / 'inert.zip'
        self.write_zip()

    def write_zip(self, image=None, proof=None, extra=None):
        with zipfile.ZipFile(self.zip, 'w') as archive:
            archive.writestr('paired-artifact/runtime-proof.json', self.proof_bytes if proof is None else proof)
            archive.writestr('paired-artifact/runtime-image.tar.gz', self.image if image is None else image)
            if extra is not None:
                archive.writestr(*extra)

    def test_trusted_review_proof_archive_and_host_code_all_bind(self):
        for mode in ('reference', 'receipt'):
            binding = {mode: getattr(self, mode)}
            gate.extract_verified(self.zip, self.root / ('valid-' + mode), self.review_path, **binding)
            for index, (key, value) in enumerate([
                ('result', 'FAIL'), ('scope', 'source-only'), ('reviewer', '/root/author'),
                ('reviewer_kind', 'human'), ('formal_verdict_issued', False),
                ('verdict_stage', 'PROVISIONAL'), ('synthetic_offline_integration_input_only', True),
                ('not_formal_operational_verdict', True), ('source_commit', 'd' * 40),
                ('runtime_proof_sha256', 'd' * 64), ('paired_host_identity_sha256', 'd' * 64),
                ('runtime_image_id', 'sha256:' + 'd' * 64), ('runtime_image_archive_sha256', 'd' * 64)]):
                changed = copy.deepcopy(self.review); changed[key] = value
                self.review_path.write_text(json.dumps(changed))
                # Even updating a claimed review digest cannot turn a provisional
                # review or different image/proof into the independently bound one.
                altered_binding = copy.deepcopy(binding)
                if mode == 'reference':
                    altered_binding[mode]['operational_review_sha256'] = gate.sha(self.review_path)
                else:
                    altered_binding[mode]['independent_operational_review']['verdict_sha256'] = gate.sha(self.review_path)
                with self.subTest(mode=mode, key=key), self.assertRaises(ValueError):
                    gate.extract_verified(self.zip, self.root / f'invalid-{mode}-{index}', self.review_path, **altered_binding)
            self.review_path.write_text(json.dumps(self.review))
        for index, extra in enumerate([('../escape', 'x'), ('elsewhere/runtime-proof.json', '{}')]):
            self.write_zip(extra=extra)
            with self.assertRaises(ValueError):
                gate.extract_verified(self.zip, self.root / f'unsafe-{index}', self.review_path, reference=self.reference)
        link = zipfile.ZipInfo('link'); link.create_system = 3; link.external_attr = (stat.S_IFLNK | 0o777) << 16
        self.write_zip(extra=(link, 'target'))
        with self.assertRaises(ValueError):
            gate.extract_verified(self.zip, self.root / 'symlink', self.review_path, reference=self.reference)

    def test_changed_image_cannot_reach_docker_in_either_workflow(self):
        reference_path = self.root / 'reference.json'; reference_path.write_text(json.dumps(self.reference))
        receipt_path = self.root / 'receipt.json'; receipt_path.write_text(json.dumps(self.receipt))
        fake_bin = self.root / 'bin'; fake_bin.mkdir()
        sentinel = self.root / 'docker-was-started'
        docker = fake_bin / 'docker'
        docker.write_text('#!/bin/sh\ntouch ' + shlex.quote(str(sentinel)) + '\n'); docker.chmod(0o755)
        for mode, path in [('reference', reference_path), ('receipt', receipt_path)]:
            for valid in (False, True):
                self.write_zip(image=self.image if valid else b'UNREVIEWED IMAGE WITH FAKE VERIFY-ONLY')
                output = self.root / f'cli-{mode}-{valid}'
                args = [sys.executable, str(Path(gate.__file__)), '--artifact-zip', str(self.zip),
                        '--output', str(output), '--review', str(self.review_path), '--' + mode, str(path)]
                command = shlex.join(args) + ' && docker load -i ' + shlex.quote(str(output / 'runtime-image.tar.gz'))
                result = subprocess.run(['/bin/sh', '-c', command], capture_output=True,
                                        env={**os.environ, 'PATH': str(fake_bin) + ':' + os.defpath, 'PYTHONOPTIMIZE': '1'})
                self.assertEqual(result.returncode == 0, valid)
                self.assertEqual(sentinel.exists(), valid)
                if sentinel.exists(): sentinel.unlink()
        repo = Path(gate.__file__).resolve().parents[2]
        for name in ('cli-normal-runtime.yml', 'cli-normal-real24-development.yml'):
            workflow = (repo / '.github/workflows' / name).read_text()
            self.assertLess(workflow.index('verify_runtime_archive.py --artifact-zip'), workflow.index('docker load'))
            self.assertNotIn('unzip -q /tmp/runtime.zip', workflow)

    def test_loaded_tag_must_resolve_to_a_reviewed_immutable_id_before_run(self):
        reference_path = self.root / 'reference.json'; reference_path.write_text(json.dumps(self.reference))
        proof_path = self.root / 'runtime-proof.json'; proof_path.write_bytes(self.proof_bytes)
        fake_bin = self.root / 'bin'; fake_bin.mkdir()
        sentinel = self.root / 'untrusted-container-started'
        docker = fake_bin / 'docker'
        docker.write_text('#!/bin/sh\ntouch ' + shlex.quote(str(sentinel)) + '\n'); docker.chmod(0o755)
        for image_id in ['sha256:' + 'e' * 64] + self.proof['archive_verified_image_ids']:
            args = [sys.executable, str(Path(gate.__file__)), '--loaded-proof', str(proof_path),
                    '--actual-image-id', image_id, '--review', str(self.review_path),
                    '--reference', str(reference_path)]
            result = subprocess.run(['/bin/sh', '-c', shlex.join(args) + ' && docker run --pull=never ' + shlex.quote(image_id)],
                                    capture_output=True, env={**os.environ, 'PATH': str(fake_bin) + ':' + os.defpath, 'PYTHONOPTIMIZE': '1'})
            admitted = image_id in self.proof['archive_verified_image_ids']
            self.assertEqual(result.returncode == 0, admitted)
            self.assertEqual(sentinel.exists(), admitted)
            if sentinel.exists(): sentinel.unlink()
        repo = Path(gate.__file__).resolve().parents[2]
        for name in ('cli-normal-runtime.yml', 'cli-normal-real24-development.yml'):
            workflow = (repo / '.github/workflows' / name).read_text()
            if name == 'cli-normal-runtime.yml':
                workflow = workflow[workflow.index('  provider-metadata-readiness:'):]
            self.assertLess(workflow.index('--loaded-proof'), workflow.index('docker run'))
            self.assertNotIn('paired-runtime python3', workflow)
            for line in workflow.splitlines():
                if 'docker run' in line:
                    self.assertIn('--pull=never', line)
                    self.assertIn('"$PAIRED_RUNTIME_IMAGE"', line)


if __name__ == '__main__':
    unittest.main()
