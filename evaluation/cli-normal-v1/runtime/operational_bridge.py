"""Normal-profile accounting: nested operational failures stop a durable lane."""
import importlib.util
from pathlib import Path
from bridge import mapping
from guard import FailureClass, Outcome

# Preserve the previous bridge's envelope compatibility and failure accounting;
# it remains byte-for-byte unchanged. This profile adds nested failure checks.
_spec = importlib.util.spec_from_file_location('preserved_operational_bridge', Path(__file__).resolve().parent.parent/'operational_bridge.py')
_legacy = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_legacy)

SAFE_STOPS={'planner_stop','targets_exhausted','action_budget','round_budget','no_progress','resolved'}
SAFE_ATTEMPTS={'applied_evidence','applied_verification','no_result','rejected_untrusted_evidence'}
FAILED_ATTEMPTS={
    'adapter_unavailable','malformed_output','adapter_failed','transport_failure','authentication_failure',
    'permission_denied','negotiation_failure','session_failure','protocol_failure','tool_failed','timed_out',
    'policy_denied','budget_exceeded','applied_candidate_revision','human_review_required',
}
SAFE_TERMINALS={'resolved_supported','resolved_qualified','resolved_refuted','exhausted'}
FAILED_TERMINALS={'unavailable','timed_out','denied','operational_failure','human_review_required'}
SAFE_ACTIONS={'applied_evidence','no_result','rejected_evidence','verification_progress','ambiguous'}

def outcome_from_normal(payload):
    base = _legacy.outcome_from_normal(payload)
    if base.failure_class is not None:
        return base
    result = mapping(mapping(payload).get('result'))
    failures=[]
    def failure(value):
        raw=value.get('failure_class') if isinstance(value,dict) else None
        try:
            failures.append(FailureClass(raw) if isinstance(raw,str) else FailureClass.UNKNOWN)
        except ValueError:
            failures.append(FailureClass.UNKNOWN)
    def stop(value):
        if value is not None and (not isinstance(value,str) or value not in SAFE_STOPS):
            failures.append(FailureClass.UNKNOWN)
    def nested_mapping(value):
        if not isinstance(value,dict):
            failures.append(FailureClass.UNKNOWN)
            return {}
        return value
    if 'rendering_failure' in result:
        failure(result['rendering_failure'])
    if 'investigation' in result:
        investigation=nested_mapping(result['investigation'])
        if 'generation_failure' in investigation:
            failure(investigation['generation_failure'])
        if 'telemetry' in investigation:
            telemetry=nested_mapping(investigation['telemetry'])
            stop(telemetry.get('stop_reason'))
            actions=telemetry.get('actions',[])
            if not isinstance(actions,list):
                failures.append(FailureClass.UNKNOWN)
            else:
                for action in actions:
                    action=nested_mapping(action)
                    if not isinstance(action.get('status'),str) or action['status'] not in SAFE_ACTIONS:
                        failures.append(FailureClass.UNKNOWN)
    if 'acquisition_coverage' in result:
        coverage=nested_mapping(result['acquisition_coverage'])
        stop(coverage.get('stop_reason'))
    rounds=result.get('resolution_rounds',[])
    if not isinstance(rounds,list):
        failures.append(FailureClass.UNKNOWN)
    else:
        for round_ in rounds:
            round_=nested_mapping(round_)
            if not isinstance(round_.get('terminal_status'),str) or round_['terminal_status'] not in SAFE_TERMINALS:
                failures.append(FailureClass.UNKNOWN)
            attempts=round_.get('attempts')
            if not isinstance(attempts,list):
                failures.append(FailureClass.UNKNOWN)
            else:
                for attempt in attempts:
                    attempt=nested_mapping(attempt)
                    if not isinstance(attempt.get('status'),str) or attempt['status'] not in SAFE_ATTEMPTS:
                        failures.append(FailureClass.UNKNOWN)
    if not failures:
        return base
    # Keep an exact quota/cancellation signal when present; all ambiguous or
    # other operational failures are latched by the new lane before another job.
    if FailureClass.QUOTA in failures:
        selected=FailureClass.QUOTA
    elif FailureClass.CANCELLED in failures:
        selected=FailureClass.CANCELLED
    elif FailureClass.UNKNOWN in failures or len(set(failures))!=1:
        selected=FailureClass.UNKNOWN
    else:
        selected=failures[0]
    return Outcome(selected,base.tokens,base.provider_attempts)
