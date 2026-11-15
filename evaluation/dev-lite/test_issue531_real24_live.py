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

    def test_issue613_eight_independent_shards_cover_each_frozen_case_once(self):
        rows=[{'id':f'rw-{i:02d}'} for i in range(1,25)]
        short=[real.selected_shard(rows,shard,8) for shard in range(8)]
        self.assertEqual([len(chunk) for chunk in short],[3]*8)
        self.assertEqual([r['id'] for chunk in short for r in chunk],
                         [r['id'] for r in rows])
        self.assertEqual([r['id'] for r in short[-1]],['rw-22','rw-23','rw-24'])
        historical=[real.selected_shard(rows,shard,4) for shard in range(4)]
        self.assertEqual([len(chunk) for chunk in historical],[6]*4)
        self.assertEqual([r['id'] for r in historical[-1]],
                         ['rw-19','rw-20','rw-21','rw-22','rw-23','rw-24'])

    def test_issue613_shard_rejects_wrong_shape_duplicates_and_out_of_range(self):
        rows=[{'id':f'rw-{i:02d}'} for i in range(1,25)]
        for n,shard in [(8,-1),(8,8),(4,4),(9,0),(0,0)]:
            with self.subTest(n=n,shard=shard),self.assertRaisesRegex(RuntimeError,'shard'):
                real.selected_shard(rows,shard,n)
        for broken in [rows[:-1],rows+[rows[-1]],rows[::-1],rows[:1]+[rows[0]]+rows[2:]]:
            with self.assertRaisesRegex(RuntimeError,'shard'):
                real.selected_shard(broken,0,8)

    def test_issue613_offline_only_partition_checks_require_no_provider(self):
        repo=Path(__file__).resolve().parents[2]
        path=repo/'evaluation/engine073-531-real-doc24-20261115/cases.json'
        corpus=real.validate_public_manifest(path,repo)
        rows=[{'id':r['case_id']} for r in corpus['cases']]
        with patch.dict(os.environ,{'MISTRAL_API_KEY':'', 'REASON_DEV_LITE_FREE_CONFIRMED':''}):
            short=[real.selected_shard(rows,shard,8) for shard in range(8)]
            self.assertEqual(sum(len(x) for x in short),24)
            self.assertEqual(len(set(r['id'] for chunk in short for r in chunk)),24)
            self.assertEqual(real.MAX_PRICE,48 * real.v5.TOTAL_CAP * .15 / 1_000_000)

    def test_price_bound(self):
        self.assertEqual(real.TOTAL_CASES,24)
        self.assertEqual(real.JOBS,48)
        self.assertLess(real.MAX_PRICE,0.21)

if __name__=='__main__':unittest.main()
