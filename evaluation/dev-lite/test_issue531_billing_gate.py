import json
import unittest
from datetime import datetime, timezone, timedelta

import issue531_billing_gate as gate

T = datetime(2026, 11, 14, 6, 0, tzinfo=timezone.utc)

def raw(**extra):
    d={'verification_method':'authenticated_mistral_console','provider_model':'ministral-8b-2512',
       'pay_as_you_go_enabled':False,'account_or_workspace_identified':True,
       'verified_by_human':True,'included_usage_remaining_usd':9.98,
       'verified_at_utc':T.isoformat(),'verification_nonce':'issue531-20261114-dev24-fixed'}
    d.update(extra)
    return json.dumps(d)

class TestBillingGate(unittest.TestCase):
    def test_missing_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation('',T)
    def test_valid_contract_only_not_actual_account_verification(self):
        o=gate.verify_attestation(raw(),T)
        self.assertFalse(o['formal_quality_authorized'])
        self.assertLess(gate.MAX_ESTIMATED_USD,gate.MIN_FREE_REMAINING_USD)
    def test_payg_enabled_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(pay_as_you_go_enabled=True),T)
    def test_no_rate_proof_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(included_usage_remaining_usd=0.1),T)
    def test_older_than_30m_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(),T+timedelta(minutes=31))
    def test_fake_source_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(verification_method='api_models_list'),T)
    def test_future_attestation_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(),T-timedelta(minutes=3))
    def test_wrong_model_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(provider_model='mistral-small-2512'),T)
    def test_no_review_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation(raw(verified_by_human=False),T)
    def test_malformed_denied(self):
        with self.assertRaises(gate.GateError):gate.verify_attestation('invalid',T)

if __name__=='__main__':unittest.main()
