#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

try:
    from operational_observability_bounds import (
        FAIL,
        HIGHER_IS_BETTER,
        INCONCLUSIVE,
        LOWER_IS_BETTER,
        NON_WORSE,
        PASS,
        STRICT_IMPROVEMENT,
        BOUND_IDENTITY,
        compare_bound,
        report_bounds,
        row_safety_gate,
    )
except ModuleNotFoundError:
    from scripts.operational_observability_bounds import (
        FAIL,
        HIGHER_IS_BETTER,
        INCONCLUSIVE,
        LOWER_IS_BETTER,
        NON_WORSE,
        PASS,
        STRICT_IMPROVEMENT,
        BOUND_IDENTITY,
        compare_bound,
        report_bounds,
        row_safety_gate,
    )

CONTROL = '29a9e4be6273dbffeda324e15517dc64930ad315'
CANDIDATE = 'c34610863d23bb014c4b0cf49f801ab873f0f67f'
PAIRED = [
    'mistral/ministral-8b-latest',
    'google/gemini-3.5-flash-lite',
    'google/gemma-4-31b-it',
]
GROQ = 'groq/openai/gpt-oss-120b'
ZERO = [
    'correctness_boundary_violations',
    'unsupported_structured_claims',
    'unsupported_exposed_assertions',
    'exposed_text_contract_violations',
    'missed_target_insufficiency',
    'identity_unsafe_admission',
    'mcp_output_authority_self_promotion',
    'session_external_calls_replayed',
    'duplicate_action_rejections',
]
REPORT_SCHEMA = 'reason-natural-language-e2e-v33'
CORPUS_ID = 'natural-language-e2e-v33'
EVALUATOR_ID = 'reason-natural-language-e2e-v33'
SCORING_ID = 'natural-language-e2e-scoring-v33-metric-locked-v13'
SEED = 323518
MAX_TOKENS = 1024


class AcceptanceError(Exception):
    pass


def load(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def key(report):
    return f"{report.get('provider')}/{report.get('model')}"


def report_id(report):
    return f"{report.get('coordinate_role')}:{key(report)}"


def validate_report(report, role, coordinate):
    reasons = []
    aggregate = report.get('aggregate') or {}
    if (
        report.get('schema_version') != REPORT_SCHEMA
        or report.get('corpus_identity') != CORPUS_ID
        or report.get('evaluator_identity') != EVALUATOR_ID
        or report.get('scoring_identity') != SCORING_ID
    ):
        reasons.append('report identity mismatch')
    if report.get('coordinate_role') != role:
        reasons.append('coordinate role mismatch')
    if (report.get('product_coordinate') or {}).get('commit') != coordinate:
        reasons.append('product coordinate mismatch')
    if report.get('seed') != SEED or report.get('max_tokens') != MAX_TOKENS:
        reasons.append('seed/max-token mismatch')
    cases = report.get('cases')
    if not isinstance(cases, list) or len(cases) != 13 or aggregate.get('total_cases') != 13:
        reasons.append('exact thirteen-case canonical envelope required')
    if report.get('measurement_observability_passed') is not True:
        reasons.append('canonical case-envelope observability gate failed')

    safety = row_safety_gate(report)
    if not safety['passed']:
        for field, value in safety['violations'].items():
            if value:
                reasons.append(f'{field} must be zero')

    if role == 'candidate':
        if (
            aggregate.get('completed_cases') != 13
            or aggregate.get('operational_failures') != 0
            or report.get('operational_completeness_passed') is not True
            or report.get('report_gate_passed') is not True
        ):
            reasons.append('candidate row not operationally complete')
        eligible = int(aggregate.get('continuation_eligible_cases') or 0)
        if eligible > 0 and aggregate.get('mechanism_conformance_rate') != 1.0:
            reasons.append('#249 candidate mechanism conformance must be 1.0 when legally executable')
    return reasons


def metric_summary(aggregate):
    return {
        name: aggregate.get(name)
        for name in [
            'target_recall',
            'tool_selection_success_rate',
            'false_abstentions',
            'avoidable_followup_stalls',
            'trigger_exposed_cases',
            'continuation_eligible_cases',
            'continuation_ineligible_trigger_cases',
            'mechanism_conformance_rate',
            'harness_precedence_selections',
            'model_selected_action_calls',
            'action_rejections',
            'action_rejection_count',
            'action_rejection_diagnostic_records',
            'precedence_skip_reasons',
        ]
    }


def _exact_candidate_value(candidate_bounds, metric):
    bound = candidate_bounds['bounds'][metric]
    lower = bound.get('lower_bound')
    upper = bound.get('upper_bound')
    if lower is None or upper is None or bound.get('undefined_possible') is True or lower != upper:
        return None
    return lower


def _comparison(candidate_bounds, control_bounds, metric, direction, requirement):
    value = _exact_candidate_value(candidate_bounds, metric)
    if value is None:
        return {
            'metric': metric,
            'classification': FAIL,
            'reason': 'candidate metric is not exactly observed despite candidate hard-gate completion',
        }
    return {
        'metric': metric,
        **compare_bound(
            value,
            control_bounds['bounds'][metric],
            direction=direction,
            requirement=requirement,
        ),
    }


def compare_pair(control, candidate):
    coordinate = key(control)
    reasons = []
    reasons += [f'control: {reason}' for reason in validate_report(control, 'control', CONTROL)]
    reasons += [f'candidate: {reason}' for reason in validate_report(candidate, 'candidate', CANDIDATE)]
    if reasons:
        return {
            'coordinate': coordinate,
            'classification': FAIL,
            'passed': False,
            'reasons': reasons,
            'control_metrics': metric_summary(control.get('aggregate') or {}),
            'candidate_metrics': metric_summary(candidate.get('aggregate') or {}),
        }

    try:
        control_bounds = report_bounds(control)
        candidate_bounds = report_bounds(candidate)
    except ValueError as error:
        return {
            'coordinate': coordinate,
            'classification': FAIL,
            'passed': False,
            'reasons': [f'v13 observability derivation failed: {error}'],
            'control_metrics': metric_summary(control.get('aggregate') or {}),
            'candidate_metrics': metric_summary(candidate.get('aggregate') or {}),
        }

    comparisons = [
        _comparison(candidate_bounds, control_bounds, 'target_recall', HIGHER_IS_BETTER, NON_WORSE),
        _comparison(candidate_bounds, control_bounds, 'tool_selection_success_rate', HIGHER_IS_BETTER, NON_WORSE),
        _comparison(candidate_bounds, control_bounds, 'false_abstentions', LOWER_IS_BETTER, NON_WORSE),
        _comparison(candidate_bounds, control_bounds, 'avoidable_followup_stalls', LOWER_IS_BETTER, NON_WORSE),
        _comparison(candidate_bounds, control_bounds, 'trigger_exposed_cases', HIGHER_IS_BETTER, NON_WORSE),
    ]

    control_stalls = control_bounds['bounds']['avoidable_followup_stalls']
    control_triggers = control_bounds['bounds']['trigger_exposed_cases']
    exact_ceiling = (
        control_stalls.get('lower_bound') == 0
        and control_stalls.get('upper_bound') == 0
        and control_triggers.get('lower_bound') == 3
        and control_triggers.get('upper_bound') == 3
    )
    utility_strict = []
    if not exact_ceiling:
        utility_strict = [
            _comparison(
                candidate_bounds,
                control_bounds,
                'avoidable_followup_stalls',
                LOWER_IS_BETTER,
                STRICT_IMPROVEMENT,
            ),
            _comparison(
                candidate_bounds,
                control_bounds,
                'trigger_exposed_cases',
                HIGHER_IS_BETTER,
                STRICT_IMPROVEMENT,
            ),
        ]

    if any(item['classification'] == FAIL for item in comparisons):
        classification = FAIL
        reasons.append('definite regression against a conservative control bound')
    elif any(item['classification'] == INCONCLUSIVE for item in comparisons):
        classification = INCONCLUSIVE
        reasons.append('non-regression cannot be proven against conservative control bounds')
    elif exact_ceiling:
        classification = PASS
    elif any(item['classification'] == PASS for item in utility_strict):
        classification = PASS
    elif all(item['classification'] == FAIL for item in utility_strict):
        classification = FAIL
        reasons.append('strict utility improvement is impossible against conservative control bounds')
    else:
        classification = INCONCLUSIVE
        reasons.append('strict utility improvement cannot be proven against conservative control bounds')

    return {
        'coordinate': coordinate,
        'classification': classification,
        'passed': classification == PASS,
        'reasons': reasons,
        'bound_identity': BOUND_IDENTITY,
        'control_operational_failures': int((control.get('aggregate') or {}).get('operational_failures') or 0),
        'control_bounds': control_bounds['bounds'],
        'candidate_bounds': candidate_bounds['bounds'],
        'non_regression_comparisons': comparisons,
        'strict_utility_comparisons': utility_strict,
        'control_exact_utility_ceiling': exact_ceiling,
        'control_metrics': metric_summary(control.get('aggregate') or {}),
        'candidate_metrics': metric_summary(candidate.get('aggregate') or {}),
    }


def groq_row(report):
    reasons = validate_report(report, 'candidate', CANDIDATE)
    aggregate = report.get('aggregate') or {}
    if key(report) != GROQ:
        reasons.append('wrong Groq model')
    if aggregate.get('investigation_cases') != 10 or aggregate.get('session_cases') != 3:
        reasons.append('Groq generic path did not cover 10 investigation + 3 session cases')
    if int(aggregate.get('provider_calls_observed') or 0) <= 0:
        reasons.append('Groq generic generation path not observed')
    if int(aggregate.get('tool_calls') or 0) <= 0:
        reasons.append('Groq investigation action path not observed')
    return {
        'coordinate': GROQ,
        'classification': PASS if not reasons else FAIL,
        'passed': not reasons,
        'reasons': reasons,
        'candidate_metrics': metric_summary(aggregate),
    }


def _overall(rows, groq=None):
    classifications = [row['classification'] for row in rows]
    if groq is not None:
        classifications.append(groq['classification'])
    if FAIL in classifications:
        return FAIL
    if INCONCLUSIVE in classifications:
        return INCONCLUSIVE
    return PASS


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--report', action='append', default=[])
    parser.add_argument('--pair-control')
    parser.add_argument('--pair-candidate')
    parser.add_argument('--validate-only', action='store_true')
    parser.add_argument('--output')
    args = parser.parse_args()

    if args.validate_only:
        out = {
            'schema_version': 'natural-language-e2e-v33-acceptance-v2',
            'valid': True,
            'live_results_evaluated': False,
            'control_commit': CONTROL,
            'candidate_commit': CANDIDATE,
            'paired_models': PAIRED,
            'candidate_only_provider_parity_model': GROQ,
            'metric_semantics_locked_from': 'natural-language-e2e-v11',
            'metric_revision': 'v13',
            'bound_identity': BOUND_IDENTITY,
            'release_classifications': [PASS, FAIL, INCONCLUSIVE],
            'inconclusive_releases': False,
            'no_cross_model_averaging': True,
        }
    elif args.pair_control or args.pair_candidate:
        if not (args.pair_control and args.pair_candidate) or args.report:
            raise AcceptanceError('pair-only mode requires exactly --pair-control and --pair-candidate')
        control = load(args.pair_control)
        candidate = load(args.pair_candidate)
        if key(control) != key(candidate) or key(control) not in PAIRED:
            raise AcceptanceError('pair-only reports must be the same required paired model')
        row = compare_pair(control, candidate)
        out = {
            'schema_version': 'natural-language-e2e-v33-acceptance-v2',
            'valid': True,
            'live_results_evaluated': True,
            'pair_only': True,
            'control_commit': CONTROL,
            'candidate_commit': CANDIDATE,
            'metric_semantics_locked_from': 'natural-language-e2e-v11',
            'metric_revision': 'v13',
            'bound_identity': BOUND_IDENTITY,
            'no_cross_model_averaging': True,
            'paired_row': row,
            'paired_gate_classification': row['classification'],
            'paired_gate_passed': row['classification'] == PASS,
        }
    else:
        reports = [load(path) for path in args.report]
        by = {report_id(report): report for report in reports}
        if len(reports) != len(by):
            raise AcceptanceError('duplicate report identity')
        required = [f'control:{model}' for model in PAIRED] + [f'candidate:{model}' for model in PAIRED] + [f'candidate:{GROQ}']
        missing = [name for name in required if name not in by]
        extra = [name for name in by if name not in required]
        if missing or extra:
            raise AcceptanceError(f'exact frozen result set required; missing={missing}, extra={extra}')
        rows = [compare_pair(by[f'control:{model}'], by[f'candidate:{model}']) for model in PAIRED]
        groq = groq_row(by[f'candidate:{GROQ}'])
        classification = _overall(rows, groq)
        out = {
            'schema_version': 'natural-language-e2e-v33-acceptance-v2',
            'valid': True,
            'live_results_evaluated': True,
            'control_commit': CONTROL,
            'candidate_commit': CANDIDATE,
            'metric_semantics_locked_from': 'natural-language-e2e-v11',
            'metric_revision': 'v13',
            'bound_identity': BOUND_IDENTITY,
            'no_cross_model_averaging': True,
            'paired_rows': rows,
            'groq_row': groq,
            'release_gate_classification': classification,
            'release_gate_passed': classification == PASS,
        }

    text = json.dumps(out, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + '\n', encoding='utf-8')
    if not out.get('live_results_evaluated'):
        return 0
    classification = out.get('release_gate_classification') or out.get('paired_gate_classification')
    return 0 if classification == PASS else 4


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except AcceptanceError as error:
        print(json.dumps({
            'schema_version': 'natural-language-e2e-v33-acceptance-v2',
            'valid': False,
            'error': str(error),
        }))
        raise SystemExit(2)
