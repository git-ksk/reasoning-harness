"""Current CLI accounting adapter; historical bridge bytes remain unchanged."""
from bridge import mapping, outcome_from_normal as legacy_normal
from guard import Outcome

def outcome_from_normal(payload):
    envelope=mapping(payload);result=mapping(envelope.get('result'))
    if (envelope.get('schema_version')=='reason-cli-output-v1'
            and envelope.get('command')=='ask'
            and result.get('output_contract') in ['reason-natural-output-v5','reason-natural-output-v6']
            and not any(key in result for key in ['status','failure','partial_diagnostics'])):
        usage=mapping(mapping(result.get('usage')).get('run'))
        if usage.get('tracked_from_session_start') is not True:usage={}
        return Outcome(None,usage.get('total_tokens'),usage.get('provider_attempts'))
    # Preserve typed failure/partial-usage handling, v4 compatibility and
    # fail-closed handling of unknown/future envelopes.
    return legacy_normal(payload)
