"""Pre-HTTP reservation/receipt controls; no model or accuracy cases."""
import copy
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch
from preparation_guard import require_callback_admission, validate_reserved_callback


class CallbackAdmission(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.receipt = self.root / 'receipt.json'
        self.digest = 'a' * 64
        self.args = SimpleNamespace(provider='mistral', model='ministral-8b-2512', corpus=self.root / 'corpus.json',
                                    host_policy=self.root / 'host-policy.json', bin_dir=Path('/eval/bin'),
                                    case='engineering', arm='baseline', track='e2e_normal')
        self.argv = ['/python', '/callback', '--provider', self.args.provider]
        self.plan = {'purpose': 'CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE', 'provider': self.args.provider,
                     'model': self.args.model, 'development_receipt_file': str(self.receipt),
                     'artifacts': {str(self.receipt): self.digest}, 'corpus_root': str(self.root),
                     'jobs': [{'id': 'engineering-baseline', 'argv': self.argv, 'contract': 'normal',
                               'budget': {'aggregate_input_tokens': 24000, 'aggregate_output_tokens': 4096}}]}
        self.state = {'provider': self.args.provider, 'model': self.args.model, 'rows': [],
                      'guard': {'inflight': True, 'latch': None}, 'current_reserve': 28096, 'charged_reservations': 28096}

    def test_live_requires_receipt_before_runtime_or_inputs(self):
        with patch('development.checked_receipt') as checked:
            with self.assertRaisesRegex(ValueError, 'callback_requires_frozen_receipt'):
                require_callback_admission(self.args, {})
            checked.assert_not_called()

    def test_wrong_receipt_digest_rejected_before_loading(self):
        self.receipt.write_text('{}')
        env = {'CLI_REPAIR_DEVELOPMENT_RECEIPT': str(self.receipt), 'CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256': '0' * 64}
        with patch('development.checked_receipt') as checked:
            with self.assertRaisesRegex(ValueError, 'callback_receipt_sha_mismatch'):
                require_callback_admission(self.args, env)
            checked.assert_not_called()

    def test_exact_current_durable_scope_passes_without_HTTP(self):
        validate_reserved_callback(self.args, self.plan, self.state, self.root / 'engineering-baseline', self.argv, self.receipt, self.digest)

    def test_changed_or_unreserved_scope_cannot_execute(self):
        mutations = [
            ('old_M0_purpose', lambda p, s: p.update(purpose='fresh_acceptance')),
            ('model', lambda p, s: p.update(model='other')),
            ('provider', lambda p, s: p.update(provider='google')),
            ('receipt', lambda p, s: p['artifacts'].update({str(self.receipt): 'b' * 64})),
            ('not_inflight', lambda p, s: s['guard'].update(inflight=False)),
            ('latched', lambda p, s: s['guard'].update(latch='quota')),
            ('wrong_reserve', lambda p, s: s.update(current_reserve=1)),
            ('not_charged', lambda p, s: s.update(charged_reservations=0)),
            ('finished', lambda p, s: s.update(rows=[{'complete': True}])),
            ('argv', lambda p, s: p['jobs'][0].update(argv=['other'])),
            ('job', lambda p, s: p['jobs'][0].update(id='other'))]
        for name, mutation in mutations:
            with self.subTest(name=name):
                plan, state = copy.deepcopy(self.plan), copy.deepcopy(self.state)
                mutation(plan, state)
                with self.assertRaises(ValueError):
                    validate_reserved_callback(self.args, plan, state, self.root / 'engineering-baseline', self.argv, self.receipt, self.digest)


if __name__ == '__main__':
    unittest.main()
