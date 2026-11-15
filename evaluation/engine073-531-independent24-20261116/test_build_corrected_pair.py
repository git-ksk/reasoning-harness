"""Offline control checks on both historical Engines with equal corrected CLI."""
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parent))
import build_corrected_pair as evaluated

REPO=Path(__file__).resolve().parents[2]

class CorrectedPairBuild(unittest.TestCase):
    def test_immutable_patch_proofs_and_expected_function_change(self):
        old,fix=evaluated.patches(REPO)
        self.assertEqual(hashlib.sha256(old).hexdigest(),evaluated.COMMON_PATCH_SHA256)
        self.assertEqual(hashlib.sha256(fix).hexdigest(),evaluated.FIX_PATCH_SHA256)
        self.assertIn(b'let pending = if pending_investigation && attempts.is_empty()',fix)
        self.assertNotIn(b'issue613_readonly_investigation_after_reject',fix)

    def test_both_historical_core_sources_accept_same_patches_no_provider(self):
        with tempfile.TemporaryDirectory(prefix='rh531-corrected-') as tmp:
            with patch.dict('os.environ',{'MISTRAL_API_KEY':'',
                                        'GROQ_API_KEY':'', 'GEMINI_API_KEY':''}):
                output=evaluated.prepare(REPO,Path(tmp)/'paired',False)
            self.assertTrue(output['validated'])
            self.assertFalse(output['built'])
            self.assertTrue(output['host_parity'])
            self.assertEqual(output['provider_calls'],0)
            self.assertIsNone(output['formal_independent_f1'])
            self.assertEqual(output['cli_commit'],evaluated.CLI_COMMIT)
            self.assertEqual(output['arms']['baseline']['engine_commit'],evaluated.ARMS['baseline'])
            self.assertEqual(output['arms']['candidate']['engine_commit'],evaluated.ARMS['candidate'])

    def test_rejects_mutable_sha_and_tampered_patch(self):
        with self.assertRaises(Exception):
            evaluated.require_commit(REPO,'main')
        with patch.object(evaluated,'FIX_PATCH_SHA256','0'*64):
            with self.assertRaisesRegex(Exception,'issue613_core_patch_drift'):
                evaluated.patches(REPO)

if __name__=='__main__':unittest.main()
