#!/usr/bin/env python3
"""Prove that v29 introduces no scoring-semantic change from immutable v28."""

import ast
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PREV = 'natural-language-e2e-v28-freeze'
PREV_COMMIT = 'ccce3e56b3093746450db05a07ac2dacc473fa1b'
OLD_CANDIDATE = '756b63b5a5cbe024f89797b6c7da48be6700bc36'
NEW_CANDIDATE = '64f6669b872094577a77bcb4137a161aadb66f6a'
OLD_SEED = 95100
NEW_SEED = 96231

FILES = {
    'runner': ('scripts/natural_language_e2e_v28.py', ROOT / 'scripts/natural_language_e2e_v29.py'),
    'acceptance': ('scripts/natural_language_e2e_v28_acceptance.py', ROOT / 'scripts/natural_language_e2e_v29_acceptance.py'),
    'pair': ('scripts/validate_natural_language_e2e_v28_pair.py', ROOT / 'scripts/validate_natural_language_e2e_v29_pair.py'),
}

RUNNER_SCORING_FUNCTIONS = [
    'final_artifact', 'parse_exposed', 'artifact_supports', 'unwrap_released_canonical_partial',
    'exposed_metrics', 'generation_costs', 'collect_rejections',
    'collect_operational_attempt_failures', 'invocation_failure', 'continuation_budget_allows',
    'score_investigation', 'session_artifact', 'session_events', 'checkpoint_snapshot',
    'fork_source_snapshot', 'fork_checkpoint_state_matches', 'thread_event_payload',
    'thread_event_kind', 'thread_event_change_kind', 'run_session_case', 'aggregate',
    'evaluate_report_gates',
]
ACCEPTANCE_SCORING_FUNCTIONS = ['validate_report', 'compare_pair', 'metrics', 'groq_row']
PAIR_SCORING_FUNCTIONS = ['scrub']


class LockError(Exception):
    pass


def git_show(path: str) -> str:
    cp = subprocess.run(
        ['git', 'show', f'{PREV}:{path}'], cwd=ROOT,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    )
    if cp.returncode != 0:
        raise LockError(cp.stderr.strip() or f'cannot read {PREV}:{path}')
    return cp.stdout


def function_map(text: str):
    return {
        node.name: node for node in ast.parse(text).body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    }


class IdentityNormalizer(ast.NodeTransformer):
    def visit_Constant(self, node):
        value = node.value
        if isinstance(value, str):
            value = value.replace(OLD_CANDIDATE, '<CANDIDATE_COMMIT>')
            value = value.replace(NEW_CANDIDATE, '<CANDIDATE_COMMIT>')
            value = value.replace('natural_language_e2e_v28', 'natural_language_e2e_vXX')
            value = value.replace('natural_language_e2e_v29', 'natural_language_e2e_vXX')
            value = value.replace('V28', 'VXX').replace('V29', 'VXX')
            value = value.replace('v28', 'vXX').replace('v29', 'vXX')
            return ast.copy_location(ast.Constant(value=value), node)
        if value in (OLD_SEED, NEW_SEED):
            return ast.copy_location(ast.Constant(value='<SEED>'), node)
        return node


def normalized_ast(node) -> str:
    clone = ast.parse(ast.unparse(node)).body[0]
    clone = IdentityNormalizer().visit(clone)
    ast.fix_missing_locations(clone)
    return ast.dump(clone, include_attributes=False)


def compare_functions(label: str, previous_text: str, current_text: str, names: list[str]):
    prev = function_map(previous_text)
    cur = function_map(current_text)
    missing = [name for name in names if name not in prev or name not in cur]
    diffs = [
        name for name in names
        if name not in missing and normalized_ast(prev[name]) != normalized_ast(cur[name])
    ]
    return {'label': label, 'checked': names, 'missing': missing, 'diffs': diffs}


def normalized_assignment(text: str, name: str):
    for node in ast.parse(text).body:
        if (
            isinstance(node, ast.Assign) and len(node.targets) == 1
            and isinstance(node.targets[0], ast.Name) and node.targets[0].id == name
        ):
            clone = ast.parse(ast.unparse(node)).body[0]
            clone = IdentityNormalizer().visit(clone)
            ast.fix_missing_locations(clone)
            return ast.dump(clone, include_attributes=False)
    raise LockError(f'module assignment missing: {name}')


def main():
    resolved = subprocess.check_output(['git', 'rev-list', '-n', '1', PREV], cwd=ROOT, text=True).strip()
    if resolved != PREV_COMMIT:
        raise LockError(f'predecessor freeze moved: {resolved}')

    texts = {
        key: (git_show(previous), current.read_text(encoding='utf-8'))
        for key, (previous, current) in FILES.items()
    }
    checks = [
        compare_functions('runner', *texts['runner'], RUNNER_SCORING_FUNCTIONS),
        compare_functions('acceptance', *texts['acceptance'], ACCEPTANCE_SCORING_FUNCTIONS),
        compare_functions('pair', *texts['pair'], PAIR_SCORING_FUNCTIONS),
    ]

    assignment_checks = {}
    for label, names in {
        'acceptance': ['ZERO'],
        'pair': ['PRESERVED', 'V12'],
    }.items():
        prev_text, cur_text = texts[label]
        for name in names:
            assignment_checks[f'{label}.{name}'] = (
                normalized_assignment(prev_text, name) == normalized_assignment(cur_text, name)
            )

    valid = all(not check['missing'] and not check['diffs'] for check in checks) and all(assignment_checks.values())
    out = {
        'schema_version': 'natural-language-e2e-v29-metric-lock-validation-v1',
        'valid': valid,
        'predecessor_freeze': PREV,
        'predecessor_commit': PREV_COMMIT,
        'locked_from': 'natural-language-e2e-v11',
        'metric_revision': 'v12',
        'semantic_delta': 'none',
        'normalization_allowlist': [
            'v28/v29 identity and path tokens', 'base seed identity', 'candidate coordinate commit',
        ],
        'function_checks': checks,
        'module_assignment_checks': assignment_checks,
    }
    print(json.dumps(out, indent=2, sort_keys=True))
    if not valid:
        raise SystemExit(2)


if __name__ == '__main__':
    try:
        main()
    except (LockError, subprocess.CalledProcessError) as error:
        print(json.dumps({
            'schema_version': 'natural-language-e2e-v29-metric-lock-validation-v1',
            'valid': False,
            'error': str(error),
        }, indent=2, sort_keys=True))
        raise SystemExit(2)
