#!/usr/bin/env python3
"""Offline development comparison. Never claims provider or ordinary CLI E2E acceptance."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import time

ARMS = {
    '0.6.1': ('engine-v0.6.1', '4abee90f0501f0d8fc09b453a0b45c942fbed69e'),
    '0.7.2': ('engine-v0.7.2', '296690a86e049b75ee693ae5032a71504366137c'),
}
HERE = Path(__file__).resolve().parent

def sha(data):
    return hashlib.sha256(data).hexdigest()

def write_new(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')

def command(argv, cwd=None, env=None):
    return subprocess.run(argv, cwd=cwd, env=env, check=True, capture_output=True).stdout

def validate(corpus):
    if corpus['suite_id'] != 'engine-07-realworld-pilot-v1':
        raise ValueError('wrong suite identity')
    sources = {s['source_id']: s for s in corpus['sources']}
    if len(sources) != len(corpus['sources']):
        raise ValueError('duplicate source IDs')
    for source in sources.values():
        if sha(source['quote'].encode()) != source['quote_sha256']:
            raise ValueError('source excerpt hash mismatch')
        if not source['url'].startswith('https://') or not source['retrieved_at']:
            raise ValueError('source provenance missing')
    ids = set()
    inputs = []
    for case in corpus['cases']:
        if case['id'] in ids or type(case['expected_relevant']) is not bool:
            raise ValueError('duplicate case or invalid label')
        ids.add(case['id'])
        signals = []
        for signal in case['signals']:
            text = sources[signal['source_ref']]['quote']
            if 'append_synthetic_text' in signal:
                if case['family'] != 'synthetic_adversarial_mutation':
                    raise ValueError('unlabeled synthetic mutation')
                text += ' ' + signal['append_synthetic_text']
            signals.append({'kind': signal['kind'], 'text': text})
        inputs.append({
            'id': case['id'],
            'policy': {
                'policy_id': 'realworld-pilot:' + case['id'],
                'target_id': case['id'], 'target_question': case['question'],
                'entity': {'canonical_id': 'realworld:' + case['id'],
                           'canonical_name': case['entity'], 'aliases': case['aliases']},
                'relation': case['relation'], 'identity_requirement': 'require_harness_anchor',
                'assessment_budget': {'max_model_attempts': 2, 'max_tokens': 192, 'max_elapsed_ms': 90000},
            },
            'candidate': {'evidence_id': 'e:' + case['id'], 'source_id': case['signals'][0]['source_ref'], 'signals': signals},
            # Fixed optimistic advisory inputs stress deterministic authority boundaries.
            # They are neither model observations nor expected labels.
            'proposal': {'target_binding': 'exact', 'relation_binding': 'exact'},
            'raw': {'identity_scope': 'exact_target', 'relation_scope': 'requested_relation', 'scope_risk': 'none'},
        })
    return inputs

def build(repo, work, version, tag, commit):
    if command(['git', 'rev-parse', tag + '^{}'], repo).decode().strip() != commit:
        raise ValueError('tag source differs from precommitted arm')
    archive = command(['git', 'archive', commit], repo)
    source = work / ('source-' + version)
    source.mkdir()
    with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
        for member in tar.getmembers():
            if member.name.startswith('/') or '..' in Path(member.name).parts or member.issym() or member.islnk():
                raise ValueError('unexpected archive path or link')
        tar.extractall(source)
    target = work / ('target-' + version)
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(target), CARGO_INCREMENTAL='0', CARGO_PROFILE_DEV_DEBUG='0')
    argv = ['cargo', 'build', '--locked', '--offline', '-p', 'reasoning-harness-core', '--jobs', '2', '--message-format=json']
    lock_before = sha((source / 'Cargo.lock').read_bytes())
    proc = subprocess.run(argv, cwd=source, env=env, capture_output=True)
    (work / ('build-' + version + '.jsonl')).write_bytes(proc.stdout)
    (work / ('build-' + version + '.stderr')).write_bytes(proc.stderr)
    proc.check_returncode()
    artifacts = {}
    for line in proc.stdout.splitlines():
        message = json.loads(line)
        if message.get('reason') == 'compiler-artifact':
            for name in message['filenames']:
                if name.endswith('.rlib'):
                    artifacts[message['target']['name']] = Path(name)
    core, serde = artifacts['reasoning_harness_core'], artifacts['serde_json']
    binary = work / ('probe-' + version)
    rustc_args = ['rustc', '--edition=2024', str(HERE / 'probe.rs'),
                  '--extern', 'reasoning_harness_core=' + str(core),
                  '--extern', 'serde_json=' + str(serde), '-L', 'dependency=' + str(serde.parent),
                  '-o', str(binary)]
    command(rustc_args, repo)
    if sha((source / 'Cargo.lock').read_bytes()) != lock_before:
        raise ValueError('build modified frozen lock')
    identity = {
        'version': version, 'tag': tag, 'commit': commit,
        'core_tree': command(['git', 'rev-parse', commit + ':crates/reasoning-harness-core'], repo).decode().strip(),
        'cargo_lock_sha256': lock_before,
        'evidence_relevance_sha256': sha((source / 'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()),
        'core_rlib_sha256': sha(core.read_bytes()), 'probe_binary_sha256': sha(binary.read_bytes()),
        'archive_sha256': sha(archive), 'cargo_args': argv, 'rustc_args': rustc_args,
    }
    return binary, identity

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--work-dir', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--expected-corpus-sha256', required=True)
    parser.add_argument('--validate-only', action='store_true')
    args = parser.parse_args()
    raw = (HERE / 'corpus.json').read_bytes()
    if sha(raw) != args.expected_corpus_sha256:
        raise ValueError('corpus changed from caller-pinned hash')
    corpus = json.loads(raw)
    inputs = validate(corpus)
    if args.validate_only:
        print(json.dumps({'status': 'validated_no_observation', 'cases': len(inputs), 'model_calls': 0}))
        return
    # Refuse retries into either consumed location. Failed operations remain visible.
    if args.output.exists() or args.work_dir.exists():
        raise ValueError('first-observation output and work locations must both be absent')
    work, output, repo = args.work_dir.resolve(), args.output.resolve(), args.repo.resolve()
    work.mkdir(parents=True)
    output.mkdir(parents=True)
    input_bytes = (json.dumps(inputs, ensure_ascii=False, sort_keys=True) + '\n').encode()
    (work / 'inputs.json').write_bytes(input_bytes)
    write_new(output / 'preflight.json', {
        'suite_id': corpus['suite_id'], 'status': 'reserved_before_any_arm',
        'corpus_sha256': sha(raw), 'expanded_input_sha256': sha(input_bytes),
        'runner_sha256': sha(Path(__file__).read_bytes()),
        'probe_sha256': sha((HERE / 'probe.rs').read_bytes()), 'arms': ARMS,
        'rustc_version': command(['rustc', '-vV']).decode(),
        'cargo_version': command(['cargo', '--version']).decode().strip(),
        'host': platform.platform(), 'machine': platform.machine(),
        'scope': corpus['scope'], 'same_host_and_probe_for_both_arms': True,
    })
    observations = {}
    for version, (tag, commit) in ARMS.items():
        binary, identity = build(repo, work, version, tag, commit)
        write_new(output / ('identity-' + version + '.json'), identity)
        started = time.monotonic_ns()
        data = command([str(binary), str(work / 'inputs.json')])
        elapsed = time.monotonic_ns() - started
        rows = [json.loads(line) for line in data.splitlines()]
        if [r['id'] for r in rows] != [c['id'] for c in corpus['cases']] or any(r['engine_version'] != version for r in rows):
            raise ValueError('observed arm/source/case identity mismatch')
        with (output / ('first-' + version + '.jsonl')).open('xb') as stream:
            stream.write(data)
        write_new(output / ('timing-' + version + '.json'), {'process_elapsed_ns': elapsed,
            'scope': 'single local deterministic probe process, not API/task latency'})
        observations[version] = rows
    paired = []
    for case, old, new in zip(corpus['cases'], observations['0.6.1'], observations['0.7.2']):
        expected = case['expected_relevant']
        old_relevant, new_relevant = old['v30']['disposition'] == 'relevant', new['v30']['disposition'] == 'relevant'
        paired.append({'id': case['id'], 'family': case['family'], 'expected_relevant': expected,
            'baseline': old['v30']['disposition'], 'candidate': new['v30']['disposition'],
            'baseline_mismatch': old_relevant != expected, 'candidate_mismatch': new_relevant != expected,
            'gain': expected and not old_relevant and new_relevant,
            'positive_regression': expected and old_relevant and not new_relevant,
            'candidate_false_relevant': not expected and new_relevant,
            'historical_v29_equal': old['v29'] == new['v29']})
    write_new(output / 'paired.json', paired)
    summary = {'suite_id': corpus['suite_id'], 'cases': len(paired),
        'primary_excerpt_snapshots': len(corpus['sources']),
        'authentic_primary_positives': sum(c['family'] == 'authentic_primary_positive' for c in corpus['cases']),
        'synthetic_mutations': sum(c['family'] == 'synthetic_adversarial_mutation' for c in corpus['cases']),
        'baseline_mismatches': sum(p['baseline_mismatch'] for p in paired),
        'candidate_mismatches': sum(p['candidate_mismatch'] for p in paired),
        'relevance_gains': sum(p['gain'] for p in paired),
        'positive_regressions': sum(p['positive_regression'] for p in paired),
        'candidate_false_relevant': sum(p['candidate_false_relevant'] for p in paired),
        'model_calls': 0, 'api_tokens': None, 'cost': None,
        'adoption_decision': 'NO_GO', 'blockers': ['known unrepaired #520', 'provider arms not run',
            'ordinary CLI E2E not measured', 'fresh independent acceptance not run'],
        'scope': corpus['scope'], 'representative_error_rate': False}
    write_new(output / 'summary.json', summary)
    print(json.dumps(summary, ensure_ascii=False))

if __name__ == '__main__':
    main()
