#!/usr/bin/env python3
"""Fail-closed authorization for billable 531 development observations.

This gate verifies the *shape, freshness, model and claimed no-overage setting*
of a human-confirmed Mistral console attestation. It is not a remote billing
query and cannot make an unsupported assertion that the account was inspected.
"""
from __future__ import annotations

from datetime import datetime, timezone, timedelta
from decimal import Decimal, InvalidOperation
import json
import os

MODEL = 'ministral-8b-2512'
MAX_TOTAL_TOKENS = 48 * 28096
LIST_PRICE_USD_PER_MILLION = Decimal('0.15')
MAX_ESTIMATED_USD = Decimal(MAX_TOTAL_TOKENS) * LIST_PRICE_USD_PER_MILLION / Decimal(1_000_000)
MIN_FREE_REMAINING_USD = Decimal('0.50')

class GateError(Exception):
    pass


def verify_attestation(raw: str, now: datetime | None = None) -> dict:
    if not raw:
        raise GateError('billing_attestation_missing_no_provider_calls')
    try:
        data = json.loads(raw)
        if type(data) is not dict:
            raise GateError('billing_attestation_not_an_object')
        if data.get('verification_method') != 'authenticated_mistral_console':
            raise GateError('billing_source_not_authorized')
        if data.get('provider_model') != MODEL:
            raise GateError('model_mismatch')
        if data.get('pay_as_you_go_enabled') is not False:
            raise GateError('provider_overage_not_proven_disabled')
        if data.get('account_or_workspace_identified') is not True:
            raise GateError('account_not_identified')
        if data.get('verified_by_human') is not True:
            raise GateError('human_review_missing')
        if data.get('verification_nonce') != 'issue531-20261114-dev24-fixed':
            raise GateError('wrong_run_attestation')
        timestamp = datetime.fromisoformat(data['verified_at_utc'].replace('Z', '+00:00'))
        if timestamp.tzinfo is None:
            raise GateError('timezone_missing')
        now = now or datetime.now(timezone.utc)
        age = now - timestamp.astimezone(timezone.utc)
        if age < timedelta(seconds=-60) or age > timedelta(minutes=30):
            raise GateError('stale_billing_attestation')
        remaining = Decimal(str(data['included_usage_remaining_usd']))
        if not remaining.is_finite() or remaining < MIN_FREE_REMAINING_USD:
            raise GateError('insufficient_included_free_usage')
        if MAX_ESTIMATED_USD >= MIN_FREE_REMAINING_USD:
            raise GateError('model_cap_beyond_remaining_margin')
    except (KeyError, ValueError, TypeError, InvalidOperation) as exc:
        raise GateError('invalid_billing_attestation_fields') from exc
    return {'verified_for_dev_only': True,
            'no_overage_policy_from_human_attestation': True,
            'maximum_estimated_usd': str(MAX_ESTIMATED_USD),
            'unspent_included_usage_at_last_review_at_least_usd': str(MIN_FREE_REMAINING_USD),
            'formal_quality_authorized': False}


def main() -> None:
    try:
        output=verify_attestation(os.environ.get('MISTRAL_BILLING_ATTESTATION_JSON',''))
        print(json.dumps(output,sort_keys=True))
    except GateError as exc:
        print('billing_gate=HOLD reason='+str(exc))
        raise SystemExit(2)

if __name__=='__main__':
    main()
