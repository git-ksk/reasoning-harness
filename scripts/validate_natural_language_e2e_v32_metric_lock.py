#!/usr/bin/env python3
"""Validate the prospective v32/v13 metric boundary against immutable v30."""

import ast
import json
import subprocess
from pathlib import Path

try:
    from operational_observability_bounds import BOUND_IDENTITY, HARD_ZERO_FIELDS
except ModuleNotFoundError:
    from scripts.operational_observability_bounds import BOUND_IDENTITY, HARD_ZERO_FIELDS

ROOT = Path(__file__).resolve().parents[1]
PREV = 'natural-language-e2e-v31-freeze-r2'
PREV_COMMIT = '22a035ae7e3c88de679a950ab4429b391f9ebafa'
OLD_CANDIDATE = '597f5ac8ffaef5c4e66f23f301bfc05c9b73ae3f'
NEW_CANDIDATE = 'eafa6015c0d092c629d286708cb745ac3c9d7859'
OLD_SEED = 97362
NEW_SEED = 99584

PRESERVED_RUNNER_FUNCTIONS = [
    'final_artifact',
    'parse_exposed',
    'artifact_supports',
    'unwrap_released_canonical_partial',
    'exposed_metrics',
    'generation_costs',
    'collect_rejections',
    'collect_operational_attempt_failures',
    'continuation_budget_allows',
    'score_investigation',
    'session_artifact',
    'session_events',
    'checkpoint_snapshot',
    'fork_source_snapshot',
    'fork_checkpoint_state_matches',
    'thread_event_payload',
    'thread_event_kind',
    'thread_event_change_kind',
    'aggregate',
    'evaluate_report_gates',
]

PRESERVED_V11 = {
    'followup_observational_cases': 3,
    'trigger_exposed_definition': 'first executed configured cache action returns typed no_result',
    'downstream_utility_scored_separately': True,
    'trigger_miss_is_measurement_data_not_mechanism_failure': True,
    'measurement_validity_does_not_require_utility_success': True,
}
PRESERVED_V12 = {
    'mechanism_denominator': 'continuation_eligible_cases_only',
    'zero_eligible_mechanism_classification': 'inconclusive',
    'mechanism_success_definition': 'immediate next action is the configured registry for the same exact target id and harness_no_result_followup_selections increments for exact-target continuation',
    'round_budget_is_not_continuation_ineligibility': True,
    'control_mechanism_conformance_is_baseline_observation_not_row_validity': True,
    'candidate_mechanism_conformance_required_when_eligible': 1.0,
}


class LockError(Exception):
    pass


def git_show(path: str) -> str:
    cp = subprocess.run(
        ['git', 'show', f'{PREV}:{path}'],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if cp.returncode != 0:
        raise LockError(cp.stderr.strip() or f'cannot read {PREV}:{path}')
    return cp.stdout


def function_map(text: str):
    return {
        node.name: node
        for node in ast.parse(text).body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    }


class IdentityNormalizer(ast.NodeTransformer):
    def visit_Constant(self, node):
        value = node.value
        if isinstance(value, str):
            value = value.replace(OLD_CANDIDATE, '<CANDIDATE_COMMIT>')
            value = value.replace(NEW_CANDIDATE, '<CANDIDATE_COMMIT>')
            value = value.replace('natural_language_e2e_v30', 'natural_language_e2e_vXX')
            value = value.replace('natural_language_e2e_v32', 'natural_language_e2e_vXX')
            value = value.replace('V30', 'VXX').replace('V32', 'VXX')
            value = value.replace('v30', 'vXX').replace('v32', 'vXX')
            return ast.copy_location(ast.Constant(value=value), node)
        if value in (OLD_SEED, NEW_SEED):
            return ast.copy_location(ast.Constant(value='<SEED>'), node)
        return node


def normalized_ast(node) -> str:
    clone = ast.parse(ast.unparse(node)).body[0]
    clone = IdentityNormalizer().visit(clone)
    ast.fix_missing_locations(clone)
    return ast.dump(clone, include_attributes=False)


def compare_functions(previous_text: str, current_text: str, names: list[str]):
    previous = function_map(previous_text)
    current = function_map(current_text)
    missing = [name for name in names if name not in previous or name not in current]
    diffs = [
        name
        for name in names
        if name not in missing and normalized_ast(previous[name]) != normalized_ast(current[name])
    ]
    return {'checked': names, 'missing': missing, 'diffs': diffs}


def assignment_value(text: str, name: str):
    tree = ast.parse(text)
    for node in tree.body:
        if (
            isinstance(node, ast.Assign)
            and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name)
            and node.targets[0].id == name
        ):
            return ast.literal_eval(node.value)
    raise LockError(f'module assignment missing: {name}')


def main():
    resolved = subprocess.check_output(
        ['git', 'rev-list', '-n', '1', PREV], cwd=ROOT, text=True
    ).strip()
    if resolved != PREV_COMMIT:
        raise LockError(f'predecessor freeze moved: {resolved}')

    previous_runner = git_show('scripts/natural_language_e2e_v31.py')
    current_runner = (ROOT / 'scripts/natural_language_e2e_v32.py').read_text(encoding='utf-8')
    runner_check = compare_functions(previous_runner, current_runner, PRESERVED_RUNNER_FUNCTIONS)

    previous_pair = git_show('scripts/validate_natural_language_e2e_v31_pair.py')
    current_pair = (ROOT / 'scripts/validate_natural_language_e2e_v32_pair.py').read_text(encoding='utf-8')
    pair_scrub = compare_functions(previous_pair, current_pair, ['scrub'])

    previous_manifest = json.loads(git_show('fixtures/natural-language-e2e-v31/manifest.json'))
    current_manifest = json.loads(
        (ROOT / 'fixtures/natural-language-e2e-v32/manifest.json').read_text(encoding='utf-8')
    )
    previous_semantics = previous_manifest['measurement_semantics']
    current_semantics = current_manifest['measurement_semantics']
    v11_preserved = all(
        previous_semantics.get(key) == value == current_semantics.get(key)
        for key, value in PRESERVED_V11.items()
    )
    v12_preserved = all(
        previous_semantics.get(key) == value == current_semantics.get(key)
        for key, value in PRESERVED_V12.items()
    )

    policy = json.loads(
        (ROOT / 'config/natural-language-e2e-metric-v13.json').read_text(encoding='utf-8')
    )
    v13_policy = (
        policy.get('metric_revision') == 'v13'
        and policy.get('bound_identity') == BOUND_IDENTITY
        and policy.get('prospective_only') is True
        and policy.get('first_allowed_successor') == 'natural-language-e2e-v30'
        and policy.get('historical_rescore_forbidden') is True
        and tuple(policy.get('hard_zero_fields') or ()) == HARD_ZERO_FIELDS
    )
    acceptance_text = (ROOT / 'scripts/natural_language_e2e_v32_acceptance.py').read_text(encoding='utf-8')
    acceptance_boundary = all(
        token in acceptance_text
        for token in [
            'report_bounds',
            'compare_bound',
            'BOUND_IDENTITY',
            "metric_revision': 'v13'",
            "INCONCLUSIVE",
        ]
    )
    intentional_delta = (
        current_semantics.get('metric_revision') == 'v13'
        and current_semantics.get('operational_observability_identity') == BOUND_IDENTITY
        and current_semantics.get('complete_case_deletion_forbidden') is True
        and current_semantics.get('favorable_imputation_forbidden') is True
        and current_semantics.get('candidate_operational_failure_is_hard_gate') is True
        and current_semantics.get('inconclusive_release_allowed') is False
        and current_semantics.get('asymmetric_diagnostics_scoring_input') is False
    )

    valid = (
        not runner_check['missing']
        and not runner_check['diffs']
        and not pair_scrub['missing']
        and not pair_scrub['diffs']
        and v11_preserved
        and v12_preserved
        and v13_policy
        and acceptance_boundary
        and intentional_delta
    )
    out = {
        'schema_version': 'natural-language-e2e-v32-metric-lock-validation-v1',
        'valid': valid,
        'predecessor_freeze': PREV,
        'predecessor_commit': PREV_COMMIT,
        'locked_from': 'natural-language-e2e-v11',
        'preserved_metric_revision': 'v12',
        'metric_revision': 'v13',
        'bound_identity': BOUND_IDENTITY,
        'semantic_delta': 'operational observability frontier + conservative partial-identification bounds only',
        'runner_preserved_function_check': runner_check,
        'pair_scrub_check': pair_scrub,
        'v11_semantics_preserved': v11_preserved,
        'v12_semantics_preserved': v12_preserved,
        'v13_policy_identity_valid': v13_policy,
        'v13_acceptance_boundary_valid': acceptance_boundary,
        'v13_intentional_delta_valid': intentional_delta,
        'normalization_allowlist': [
            'v30/v32 identity and path tokens',
            'fresh base seed identity',
            'candidate coordinate commit',
        ],
        'explicit_non_ast_deltas': [
            'operational failure case envelope retains semantic kind/target/coverage metadata',
            'canonical reports add metric_observability and conservative_bounds derived from common telemetry',
            'acceptance uses v13 interval comparison and tri-state release classification',
        ],
    }
    print(json.dumps(out, indent=2, sort_keys=True))
    if not valid:
        raise SystemExit(2)


if __name__ == '__main__':
    try:
        main()
    except (LockError, subprocess.CalledProcessError, ValueError, SyntaxError) as error:
        print(json.dumps({
            'schema_version': 'natural-language-e2e-v32-metric-lock-validation-v1',
            'valid': False,
            'error': str(error),
        }, indent=2, sort_keys=True))
        raise SystemExit(2)
