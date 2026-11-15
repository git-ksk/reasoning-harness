"""Provider-free eight-independent-job partition and frozen-input checks."""
import copy
from pathlib import Path
import unittest
from unittest.mock import patch

import issue613_short_shard_preflight as plan

ROOT=Path(__file__).resolve().parents[2]

class TestIssue613ShortShard(unittest.TestCase):
    def test_eight_preflights_cover_every_frozen_case_once_without_provider(self):
        with patch.dict('os.environ',{'MISTRAL_API_KEY':'','REASON_DEV_LITE_FREE_CONFIRMED':''}):
            result=[plan.preflight(ROOT,shard) for shard in range(8)]
        self.assertEqual([r['planned_cli_runs'] for r in result],[6]*8)
        self.assertEqual([r['pinned_source_records'] for r in result],[6]*8)
        self.assertEqual(sum(r['max_internal_model_calls_if_future_authorized'] for r in result),
                         48*plan.real.v5.MAX_CALLS)
        self.assertEqual(sum(r['max_tokens_if_future_authorized'] for r in result),
                         48*plan.real.v5.TOTAL_CAP)
        self.assertEqual([cid for row in result for cid in row['case_ids']],
                         [f'rw-{i:02d}' for i in range(1,25)])
        for r in result:
            self.assertEqual(r['provider_calls'],0)
            self.assertFalse(r['credentials_read'])
            self.assertFalse(r['live_execution_authorized'])
            self.assertFalse(r['historical_run_replay'])
            self.assertIsNone(r['formal_independent_f1'])
        self.assertEqual(result[7]['case_ids'],['rw-22','rw-23','rw-24'])

    def test_provider_free_workflow_never_resumes_historical_run_or_reads_secrets(self):
        workflow=(ROOT/'.github/workflows/engine073-613-short-shard-preflight.yml').read_text()
        self.assertIn('matrix:',workflow)
        self.assertIn('shard: [0, 1, 2, 3, 4, 5, 6, 7]',workflow)
        self.assertIn('fail-fast: false',workflow)
        self.assertIn('timeout-minutes: 8',workflow)
        self.assertIn('persist-credentials: false',workflow)
        self.assertIn('env -u MISTRAL_API_KEY -u REASON_DEV_LITE_FREE_CONFIRMED',workflow)
        self.assertNotIn('secrets.',workflow)
        self.assertNotIn('refs/tags/eval-531-real-doc24-live-20261115-one-shot',workflow)
        self.assertNotIn('--mode live',workflow)
        self.assertNotIn('retry:',workflow)

    def test_source_hash_tampering_fails_closed_before_any_provider_calls(self):
        path=ROOT/'evaluation/engine073-531-real-doc24-20261115/cases.json'
        original=plan.real.validate_public_manifest(path,ROOT)
        copy_manifest=copy.deepcopy(original)
        copy_manifest['cases'][0]['config_raw_sha256']='0'*64
        with patch.object(plan.real,'validate_public_manifest',return_value=copy_manifest):
            with self.assertRaisesRegex(RuntimeError,'original_config_sha256_changed'):
                plan.preflight(ROOT,0)

    def test_mutated_source_fails_closed_with_valid_config_integrity(self):
        with patch.object(plan.real,'validate_real_source',side_effect=RuntimeError('mutated source')):
            with self.assertRaisesRegex(RuntimeError,'mutated source'):
                plan.preflight(ROOT,0)

if __name__=='__main__':unittest.main()
