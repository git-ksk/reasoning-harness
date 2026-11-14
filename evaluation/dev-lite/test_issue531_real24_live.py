import json
import os
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

import issue531_real24_live as real

PRIMARY = {
    'url': 'https://raw.githubusercontent.com/curl/curl/' + 'a'*40 + '/docs/cmdline-opts/retry.md',
    'source_version': 'sha256:' + 'b'*64,
    'synthetic_mutation': False,
    'quote': 'This is a real historical upstream quote for a developer test.'
}

class TestRealDocsRunner(unittest.TestCase):
    def test_pin_accepts_actual_shape(self):
        self.assertEqual(real.validate_real_source(PRIMARY,'rw-01'),PRIMARY['url'])

    def test_mutated_source_rejected(self):
        with self.assertRaises(RuntimeError):
            real.validate_real_source({**PRIMARY,'synthetic_mutation':True},'rw-01')

    def test_editable_source_rejected(self):
        with self.assertRaises(RuntimeError):
            real.validate_real_source({**PRIMARY,'url':'https://raw.githubusercontent.com/curl/curl/main/docs/retry.md'},'rw-01')

    def test_nonprimary_domain_rejected(self):
        with self.assertRaises(RuntimeError):
            real.validate_real_source({**PRIMARY,'url':'https://example.invalid/docs/retry.md'},'rw-01')

    def test_unhashed_content_rejected(self):
        with self.assertRaises(RuntimeError):
            real.validate_real_source({**PRIMARY,'source_version':'sha256:unverified'},'rw-01')

    def test_missing_attestation_and_provider_refused_before_first_run(self):
        with tempfile.TemporaryDirectory() as tmp:
            with patch.dict(os.environ, {'REASON_DEV_LITE_FREE_CONFIRMED':'', 'MISTRAL_API_KEY':''}):
                with self.assertRaisesRegex(RuntimeError,'free_budget_confirmation_missing'):
                    real.measure(Path(tmp),Path(tmp),[],{},0)


    def test_public_24_case_source_inventory_and_no_gold(self):
        import hashlib
        from collections import Counter
        repo=Path(__file__).resolve().parents[2]
        path=repo/'evaluation/engine073-531-real-doc24-20261115/cases.json'
        manifest=real.validate_public_manifest(path,repo)
        audit=json.loads((path.parent/'source-audit.json').read_text())
        self.assertEqual(audit['source_count'],12)
        self.assertEqual(audit['total_quote_occurrences'],48)
        self.assertTrue(audit['all_sha256_raw_matched'])
        self.assertFalse(manifest['gold_labels_included'])
        self.assertEqual(len(manifest['cases']),24)
        self.assertEqual(len(manifest['source_content_sha256']),12)
        origin_urls=set()
        quote_count=0
        for i,row in enumerate(manifest['cases'],1):
            cid=f'rw-{i:02d}'
            self.assertEqual(row['case_id'],cid)
            self.assertEqual(row['config_path'],f'configs/{cid}.json')
            self.assertGreater(len(row['question']),20)
            file=path.parent/row['config_path']
            self.assertFalse(file.is_symlink())
            raw=file.read_bytes()
            self.assertEqual(hashlib.sha256(raw).hexdigest(),row['config_raw_sha256'])
            self.assertNotIn(b'\"answerability\"',raw)
            self.assertNotIn(b'\"expected_relevant\"',raw)
            self.assertNotIn(b'/Users/sawadakousuke',raw)
            self.assertNotIn(b'MISTRAL_API_KEY',raw)
            self.assertNotIn(b'-----BEGIN PRIVATE KEY-----',raw)
            config=json.loads(raw)
            self.assertEqual(config['schema_version'],'reason-config-v1')
            for cap in config['resolution']['investigation']['capabilities']:
                self.assertTrue(cap['read_only'])
                self.assertEqual(cap['kind'],'captured_evidence')
                for entry in cap['evidence']:
                    obs=json.loads(entry['observation'])
                    url=real.validate_real_source(obs,cid)
                    self.assertEqual(obs['source_version'],manifest['source_content_sha256'][url])
                    origin_urls.add(url)
                    quote_count+=1
        self.assertEqual(len(origin_urls),12)
        self.assertEqual(quote_count,48)
        self.assertEqual({x['url'] for x in audit['source_documents']},origin_urls)

    def test_price_bound(self):
        self.assertEqual(real.TOTAL_CASES,24)
        self.assertEqual(real.JOBS,48)
        self.assertLess(real.MAX_PRICE,0.21)

if __name__=='__main__':unittest.main()
