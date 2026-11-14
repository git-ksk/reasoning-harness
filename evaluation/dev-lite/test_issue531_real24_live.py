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

    def test_price_bound(self):
        self.assertEqual(real.TOTAL_CASES,24)
        self.assertEqual(real.JOBS,48)
        self.assertLess(real.MAX_PRICE,0.21)

if __name__=='__main__':unittest.main()
