"""Strict adapters for the two recorded host failure envelopes; no prose inference."""
from guard import FailureClass, Outcome

HELPER_FAILURES = {
    'provider_Quota': FailureClass.QUOTA,
    'provider_RateLimit': FailureClass.RATE_LIMIT,
    'provider_ProviderUnavailable': FailureClass.PROVIDER_UNAVAILABLE,
    'provider_Timeout': FailureClass.TIMEOUT,
    'provider_Credentials': FailureClass.CREDENTIALS,
    'provider_Transport': FailureClass.TRANSPORT,
    'provider_Provider': FailureClass.PROVIDER_ERROR,
    'provider_Protocol': FailureClass.PROTOCOL,
    'provider_UnsupportedCapability': FailureClass.UNSUPPORTED_CAPABILITY,
    'invalid_advice_json': FailureClass.PROTOCOL,
    'timeout_100_seconds': FailureClass.TIMEOUT,
}

def mapping(value):
    return value if isinstance(value, dict) else {}

def outcome_from_normal(payload):
    result = mapping(mapping(payload).get('result'))
    if result.get('status') == 'failed':
        raw = mapping(result.get('failure')).get('failure_class')
        try:
            failure = FailureClass(raw) if isinstance(raw, str) else FailureClass.UNKNOWN
        except ValueError:
            failure = FailureClass.UNKNOWN
        partial = mapping(result.get('partial_diagnostics'))
        # Accept measured partial usage only with the explicit completeness flags.
        measured = (failure is not FailureClass.CANCELLED and partial.get('complete') is False
                    and partial.get('usage_is_lower_bound') is False)
        usage = mapping(mapping(partial.get('usage')).get('run')) if measured else {}
    elif (result.get('output_contract') == 'reason-natural-output-v4'
          and not any(key in result for key in ('status', 'failure', 'partial_diagnostics'))):
        failure = None
        usage = mapping(mapping(result.get('usage')).get('run'))
    else:
        failure = FailureClass.UNKNOWN
        usage = {}
    return Outcome(failure, usage.get('total_tokens'), usage.get('provider_attempts'))

def outcome_from_helper(payload):
    result = mapping(payload)
    raw = result.get('operational_failure')
    if isinstance(raw, str):
        failure = HELPER_FAILURES.get(raw, FailureClass.UNKNOWN)
    elif ('operational_failure' in result and raw is None and type(result.get('equivalent')) is bool
          and not any(key in result for key in ('status', 'failure', 'partial_diagnostics'))):
        failure = None
    else:
        failure = FailureClass.UNKNOWN
    usage = mapping(result.get('usage'))
    return Outcome(failure, usage.get('total_tokens'), result.get('provider_attempts'))
