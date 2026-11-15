"""Holdout cannot be represented as accepted while independent authoring is absent."""
import copy
import json
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parent))
import validate_readiness as readiness

ROOT=Path(__file__).resolve().parents[2]

class HoldoutReadiness(unittest.TestCase):
    def test_offline_prepared_state_never_claims_score_or_authorization(self):
        result=readiness.verify(ROOT)
        self.assertTrue(result['corrected_pair_commits_pinned'])
        self.assertEqual(result['source_files_pinned'],12)
        self.assertFalse(result['fresh_questions_authored'])
        self.assertFalse(result['fresh_independent_reviewer_approved'])
        self.assertFalse(result['live_execution_allowed'])
        self.assertEqual(result['provider_calls'],0)
        self.assertIsNone(result['formal_independent_f1'])

    def test_any_fake_review_quality_or_live_approval_fails_closed(self):
        file=readiness.HERE/'holdout-readiness.json'
        base=json.loads(file.read_text())
        for key in ['reviewed_frozen_public_cases_present',
                    'independent_gold_privately_frozen','independent_reviewer_approval',
                    'same_account_no_overage_billing_checked','one_shot_live_authorized',
                    'quality_go','release_go']:
            updated=copy.deepcopy(base)
            updated[key]=True
            with patch('pathlib.Path.read_text',return_value=json.dumps(updated)):
                with self.subTest(key=key),self.assertRaises(ValueError):
                    readiness.verify(ROOT)

    def test_any_relaxed_metric_threshold_fails_closed(self):
        file=readiness.HERE/'holdout-readiness.json'
        updated=json.loads(file.read_text())
        updated['score_thresholds']['case_f1_min']=0.7
        with patch('pathlib.Path.read_text',return_value=json.dumps(updated)):
            with self.assertRaisesRegex(ValueError,'pre_registered_quality_threshold_changed'):
                readiness.verify(ROOT)

if __name__=='__main__':unittest.main()
