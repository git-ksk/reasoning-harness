"""No-provider packet safety tests; fixture-only bytes, no real holdout questions."""
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parent))
import reviewer_handoff as handoff


class ReviewerHandoff(unittest.TestCase):
    @staticmethod
    def inputs():
        payloads=[f'synthetic approved source fixture {i}\n'.encode() for i in range(1,13)]
        records=[]
        for i,payload in enumerate(payloads,1):
            records.append({'repo':f'fixture/synthetic{i}',
                            'commit':'a'*40,'path':f'fixture-{i}.txt',
                            'url':f'https://example.org/fixture/{i}',
                            'sha256':hashlib.sha256(payload).hexdigest(),
                            'bytes':len(payload)})
        manifest={'sources':records}
        return manifest,handoff.canonical(manifest),payloads

    def test_exactly_24_blanks_without_gold_or_promoted_review(self):
        manifest,raw,payloads=self.inputs()
        with tempfile.TemporaryDirectory(prefix='rh531-handoff-',dir='/tmp') as parent:
            root=Path(parent)/'packet'
            result=handoff.make_packet(root,manifest,raw,payloads)
            self.assertFalse(result['approved'])
            self.assertEqual(result['real_question_count'],0)
            self.assertFalse(result['private_gold_present'])
            self.assertEqual(result['model_calls'],0)
            self.assertEqual(result['case_slot_count'],24)
            self.assertEqual(result['blank_case_ids'],[f'ih-{i:02d}' for i in range(1,25)])
            self.assertEqual(len(list((root/'docs').iterdir())),12)
            self.assertEqual(len(list((root/'blank-cases').iterdir())),24)
            self.assertEqual((root/'docs'/'upstream-01.txt').read_bytes(),payloads[0])
            for i in range(1,25):
                slot=json.loads((root/'blank-cases'/f'ih-{i:02d}.json').read_bytes())
                self.assertIsNone(slot['question'])
                self.assertIsNone(slot['answer_or_label'])
                self.assertIsNone(slot['independent_author_id'])
            for item in [root,root/'docs',root/'blank-cases']:
                self.assertEqual(item.stat().st_mode&0o777,0o700)
            for item in [root/'packet.json',root/'source-manifest.json',root/'docs'/'upstream-01.txt']:
                self.assertEqual(item.stat().st_mode&0o777,0o600)
            with self.assertRaises(FileExistsError):
                handoff.make_packet(root,manifest,raw,payloads)

    def test_modified_download_fails_before_any_file_written(self):
        manifest,raw,payloads=self.inputs()
        with tempfile.TemporaryDirectory(prefix='rh531-handoff-',dir='/tmp') as parent:
            root=Path(parent)/'packet'
            tampered=payloads.copy()
            tampered[2]=b'corrupted source'
            with self.assertRaisesRegex(ValueError,'pinned_upstream_document_content_changed'):
                handoff.make_packet(root,manifest,raw,tampered)
            self.assertFalse(root.exists())
            with self.assertRaisesRegex(ValueError,'not_exactly_twelve_sources'):
                handoff.make_packet(root,manifest,raw,payloads[:-1])

    def test_private_audit_rejects_tampering_and_forged_gold_or_reviewer_approval(self):
        manifest,raw,payloads=self.inputs()
        with tempfile.TemporaryDirectory(prefix='rh531-handoff-',dir='/tmp') as parent:
            parent=Path(parent).resolve()
            repo=parent/'fake-source-repo'
            (repo/Path(handoff.PUBLIC_SOURCE).parent).mkdir(parents=True)
            (repo/handoff.PUBLIC_SOURCE).write_bytes(raw)
            root=parent/'packet'
            handoff.make_packet(root,manifest,raw,payloads)
            with patch.object(handoff.source_pool,'validate',return_value=True):
                valid=handoff.audit(repo,root)
                self.assertEqual(valid['source_sha256_verified'],12)
                self.assertEqual(valid['blank_slots_unchanged'],24)
                self.assertFalse(valid['live_allowed'])
                original=(root/'docs'/'upstream-01.txt').read_bytes()
                (root/'docs'/'upstream-01.txt').write_bytes(b'unsupported modified evidence')
                with self.assertRaisesRegex(ValueError,'pinned_upstream_document_content_changed'):
                    handoff.audit(repo,root)
                (root/'docs'/'upstream-01.txt').write_bytes(original)
                blank=root/'blank-cases'/'ih-17.json'
                unchanged=blank.read_bytes()
                slot=json.loads(unchanged)
                slot['question']='illegitimate authored fact'
                blank.write_bytes(handoff.canonical(slot))
                with self.assertRaisesRegex(ValueError,'blank_case_modified_or_independent_gold_leaked'):
                    handoff.audit(repo,root)
                blank.write_bytes(unchanged)
                header=root/'packet.json'
                kept=header.read_bytes()
                fake=json.loads(kept)
                fake['approved']=True
                header.write_bytes(handoff.canonical(fake))
                with self.assertRaisesRegex(ValueError,'false_authoring_reviewer_or_execution_approval'):
                    handoff.audit(repo,root)
                header.write_bytes(kept)
                (root/'blank-cases'/'reviewer-seed.bin').write_bytes(b'garbage')
                with self.assertRaisesRegex(ValueError,'unexpected_packet_inner_file_inventory'):
                    handoff.audit(repo,root)

    def test_packet_read_rejects_symlinks_and_permissive_file_modes(self):
        manifest,raw,payloads=self.inputs()
        with tempfile.TemporaryDirectory(prefix='rh531-handoff-',dir='/tmp') as parent:
            root=Path(parent).resolve()/'packet'
            handoff.make_packet(root,manifest,raw,payloads)
            source=root/'docs'/'upstream-01.txt'
            source.chmod(0o644)
            with self.assertRaisesRegex(ValueError,'packet_private_file_permission_or_identity_mismatch'):
                handoff.read_private_file(source)
            source.chmod(0o600)
            real=root/'docs'/'upstream-02.txt'
            alias=root/'docs'/'source-alias'
            alias.symlink_to(real)
            with self.assertRaises(OSError):
                handoff.read_private_file(alias)

    def test_no_signing_or_model_authority_in_packet(self):
        manifest,raw,payloads=self.inputs()
        with tempfile.TemporaryDirectory(prefix='rh531-handoff-',dir='/tmp') as parent:
            root=Path(parent)/'packet'
            handoff.make_packet(root,manifest,raw,payloads)
            for term in [b'private_key',b'reviewer_seed',b'api_key',b'MISTRAL_API_KEY',
                         b'author_approved',b'generated_gold']:
                self.assertNotIn(term,(root/'packet.json').read_bytes())
            self.assertTrue(handoff.safe_private(root) is None)

if __name__=='__main__':unittest.main()
