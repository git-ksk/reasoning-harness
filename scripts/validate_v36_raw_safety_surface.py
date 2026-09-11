#!/usr/bin/env python3
import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

FREEZE_TAG = 'natural-language-e2e-v36-freeze'
FREEZE_SHA = '57bea659d472a103cc48d86ddee7dfe4a41de790'
CANDIDATE = '9497b563ad914fada13d33e0c1a7fee549a1f1de'
CASE_FILES = [
    '03_stale-drovanel-window.json',
    '04_scope-hexaria-index.json',
    '08_authority-ivelorn-release.json',
    '09_identity-cythera-registry.json',
    '10_mcp-v36-belqorin-coordinate-cargo-nonpromotion.json',
]


def git_show(ref: str, path: str) -> str:
    cp = subprocess.run(
        ['git', 'show', f'{ref}:{path}'],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if cp.returncode:
        raise RuntimeError(cp.stderr.strip() or f'git show failed: {ref}:{path}')
    return cp.stdout


def load_at(ref: str, path: str):
    return json.loads(git_show(ref, path))


def exact_resolver_response(capability, target_key: str):
    args = capability.get('args') or []
    if not args or args[0] != 'scripts/natural_e2e_fixture_resolver.py':
        raise RuntimeError(f'unexpected resolver argv: {args!r}')
    script_text = git_show(FREEZE_TAG, 'scripts/natural_e2e_fixture_resolver.py')
    request = {
        'schema_version': 'reason-investigation-external-resolver-request-v1',
        'adapter_id': 'investigation_external_command_v1',
        'attempt_index': 0,
        'request': {
            'id': 'v36-raw-safety-reconstruction',
            'reason': 'investigation',
            'target': {
                'kind': 'investigation_question',
                'target_id': 'v36-raw-safety-target',
                'question': 'frozen supplemental reconstruction',
                'expected_fact_key': target_key,
            },
            'resolver_class': 'evidence_acquisition',
            'budget': {'max_attempts': 1, 'max_added_tokens': 0, 'max_elapsed_ms': 5000},
        },
    }
    with tempfile.TemporaryDirectory(prefix='v36-raw-safety-') as td:
        script = Path(td) / 'natural_e2e_fixture_resolver.py'
        script.write_text(script_text)
        cp = subprocess.run(
            [capability['program'], str(script), *args[1:]],
            input=json.dumps(request).encode(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    if cp.returncode:
        raise RuntimeError(
            f"resolver reconstruction failed for {capability['id']}: "
            + cp.stderr.decode(errors='replace')
        )
    try:
        payload = json.loads(cp.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"resolver reconstruction JSON failed for {capability['id']}: {error}") from error
    if payload.get('schema_version') != 'reason-external-resolver-response-v1':
        raise RuntimeError(f"resolver reconstruction schema drift for {capability['id']}")
    return payload


def build_surface():
    actual = subprocess.check_output(['git', 'rev-list', '-n', '1', FREEZE_TAG], text=True).strip()
    if actual != FREEZE_SHA:
        raise RuntimeError(f'v36 freeze drift: {actual}')
    manifest = load_at(FREEZE_TAG, 'fixtures/natural-language-e2e-v36/manifest.json')
    mcp_policy = manifest.get('mcp_policy') or {}
    if mcp_policy.get('generic_output_authority') != 'non_promoting':
        raise RuntimeError('v36 MCP non-promotion policy drift')
    cases = []
    for name in CASE_FILES:
        base = f'fixtures/natural-language-e2e-v36/{name}'
        case = load_at(FREEZE_TAG, base)
        if case.get('expected') != 'unknown' or case.get('kind') != 'investigation':
            raise RuntimeError(f'{name}: safety slice must be investigation/unknown')
        cfg_rel = case['config_pair']['candidate']
        cfg_path = f'fixtures/natural-language-e2e-v36/{cfg_rel}'
        cfg = load_at(FREEZE_TAG, cfg_path)
        caps = cfg['resolution']['investigation']['capabilities']
        relevant = case.get('relevant_capabilities') or []
        if len(relevant) != 1:
            raise RuntimeError(f'{name}: expected one relevant capability')
        matches = [cap for cap in caps if cap.get('id') == relevant[0]]
        if len(matches) != 1:
            raise RuntimeError(f'{name}: relevant capability not unique')
        cap = matches[0]
        if cap.get('read_only') is not True:
            raise RuntimeError(f'{name}: capability must be read-only')
        if cap.get('kind') == 'external_command':
            observation = {
                'capability_id': cap['id'],
                'kind': cap['kind'],
                'resolver_response': exact_resolver_response(cap, case['target']['key']),
            }
        elif cap.get('kind') == 'mcp_readonly':
            fixed = cap.get('fixed_arguments') or {}
            if fixed.get('ref') != CANDIDATE or fixed.get('path') != 'Cargo.toml':
                raise RuntimeError(f'{name}: unexpected MCP coordinate')
            observation = {
                'capability_id': cap['id'],
                'kind': cap['kind'],
                'source': cap.get('source'),
                'server_id': cap.get('server_id'),
                'tool': cap.get('tool'),
                'fixed_arguments': fixed,
                'generic_content': git_show(FREEZE_TAG, 'Cargo.toml'),
                'mcp_policy': mcp_policy,
            }
        else:
            raise RuntimeError(f'{name}: unsupported capability kind {cap.get("kind")}')
        cases.append(
            {
                'id': case['id'],
                'kind': case['kind'],
                'task': case['task'],
                'safety_dimension': case['safety_dimension'],
                'target_key': case['target']['key'],
                'expected_rejection': case.get('expected_rejection') or [],
                'context': {
                    'type': 'v36_raw_safety_context',
                    'admission_policy': cap.get('admission') or {},
                    'observation': observation,
                },
                'source_files': {'case': base, 'candidate_config': cfg_path},
            }
        )
    dimensions = [case['safety_dimension'] for case in cases]
    if dimensions != ['freshness', 'scope', 'authority', 'identity', 'mcp_nonpromotion']:
        raise RuntimeError(f'unexpected safety dimensions: {dimensions}')
    return {
        'schema_version': 'v36-raw-safety-supplement-v1',
        'source': {
            'freeze_tag': FREEZE_TAG,
            'freeze_sha': FREEZE_SHA,
            'candidate_commit': CANDIDATE,
            'seed': 738214,
            'max_tokens': 1024,
            'case_count': 5,
        },
        'comparison_contract': {
            'id': 'v36-post-acquisition-unknown-preservation-v1',
            'scope': 'safety_only',
            'expected_unknown_cases': 5,
            'planner_metrics_not_compared': True,
            'final_answer_utility_not_compared': True,
            'expected_target_values_not_exposed_as_evaluator_labels': True,
            'raw_receives_harness_admission_policy': True,
        },
        'cases': cases,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--surface', default='evaluation/v36-raw-baseline/surface-v1.json')
    parser.add_argument('--write', action='store_true')
    args = parser.parse_args()
    expected = build_surface()
    path = Path(args.surface)
    if args.write:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(expected, indent=2, ensure_ascii=False) + '\n')
        print(json.dumps({'valid': True, 'written': str(path), 'cases': 5}))
        return
    actual = json.loads(path.read_text())
    if actual != expected:
        print(
            'surface does not match deterministic reconstruction from frozen v36 sources',
            file=sys.stderr,
        )
        sys.exit(1)
    print(
        json.dumps(
            {'valid': True, 'surface': str(path), 'cases': 5, 'freeze_sha': FREEZE_SHA}
        )
    )


if __name__ == '__main__':
    main()
