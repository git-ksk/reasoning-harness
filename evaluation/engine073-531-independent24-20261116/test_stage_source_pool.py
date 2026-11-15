"""Public-source freshness/immutability controls; never request a model."""
import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parent))
import stage_source_pool as pool

ROOT=Path(__file__).resolve().parents[2]
FILE=ROOT/'evaluation/engine073-531-independent24-20261116/provisional-new-sources.json'

class NewSourcePool(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.manifest=json.loads(FILE.read_text())

    def test_public_new_source_pool_frozen_and_disjoint_from_old_development(self):
        self.assertTrue(pool.validate(ROOT,self.manifest))
        self.assertEqual(len(pool.SOURCES),5)
        self.assertEqual(len(self.manifest['sources']),12)
        self.assertIsNone(self.manifest['formal_independent_f1'])
        self.assertFalse(self.manifest['gold_labels_included'])
        self.assertFalse(self.manifest['fresh_independent_reviewer_approved'])
        self.assertFalse(self.manifest['first_observation_authorized'])
        self.assertEqual(self.manifest['model_calls'],0)

    def test_source_pool_is_not_a_completed_holdout(self):
        for field in ['gold_labels_included','fresh_independent_reviewer_approved',
                      'first_observation_authorized']:
            mutated=copy.deepcopy(self.manifest)
            mutated[field]=True
            with self.subTest(field=field),self.assertRaisesRegex(ValueError,'unauthorized_early_score_or_run'):
                pool.validate(ROOT,mutated)
        mutated=copy.deepcopy(self.manifest)
        mutated['case_count']=24
        with self.assertRaisesRegex(ValueError,'unauthorized_early_score_or_run'):
            pool.validate(ROOT,mutated)

    def test_source_repost_revision_and_hash_tampering_fail_closed(self):
        def assert_rejected(mutator):
            mutated=copy.deepcopy(self.manifest)
            mutator(mutated)
            with self.assertRaises(ValueError):
                pool.validate(ROOT,mutated)
        assert_rejected(lambda m: m['sources'].append(copy.deepcopy(m['sources'][0])))
        assert_rejected(lambda m: m['sources'][0].update(commit='main'))
        assert_rejected(lambda m: m['sources'][0].update(sha256='0'*40))
        assert_rejected(lambda m: m['sources'][0].update(url='https://example.org/unpinned'))
        assert_rejected(lambda m: m['sources'][0].update(repo='curl/curl'))
        assert_rejected(lambda m: m.update(prior_consumed_dev_source_audit_sha256='0'*64))
        assert_rejected(lambda m: m['sources'][0].update(bytes=500_000))

    def test_manifest_bytes_are_stable_across_checks(self):
        before=hashlib.sha256(FILE.read_bytes()).hexdigest()
        pool.validate(ROOT,self.manifest)
        after=hashlib.sha256(FILE.read_bytes()).hexdigest()
        self.assertEqual(before,after)
        self.assertEqual(before,'f5b814e66d715c7c7b5e413348e7a0526c98956c03079c976254518381fdf527')

if __name__=='__main__':unittest.main()
