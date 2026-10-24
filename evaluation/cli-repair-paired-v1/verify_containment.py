#!/usr/bin/env python3
"""Exercise the preserved scheduler on Linux with inert subprocesses only."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path('/repo'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    legacy = args.repo / 'evaluation/engine07-realworld-runner-v1'
    sys.path.insert(0, str(legacy))
    from runner import OperationalRunner, digest, reserve_for
    root = args.output.resolve()
    root.mkdir()
    fixture = root / 'fixture.py'
    original = (legacy / 'fixture_child.py').read_text()
    marker = "    signal.signal(signal.SIGINT, cancelled if a.mode == 'cooperative' else signal.SIG_IGN)\n"
    assert original.count(marker) == 1
    fixture.write_text(original.replace(marker, marker + "    Path('handler-ready').write_text('ready')\n"))
    controls = []

    def plan(name, modes):
        jobs = []
        for index, (mode, contract, arm) in enumerate(modes):
            jobs.append({'id': 'row-' + str(index), 'arm': arm, 'contract': contract,
                         'argv': [sys.executable, str(fixture), '--mode', mode, '--contract', contract,
                                  '--marker', str(root / (name + '-spawns')), '--provider', 'google', '--model', 'offline-fixture'],
                         'budget': {'max_generations': 2, 'max_input_tokens_per_generation': 20,
                                    'max_output_tokens_per_generation': 20, 'max_attempts_per_generation': 2},
                         'timeout_seconds': 2, 'cancel_grace_seconds': 0.5})
        return {'schema': 'engine07-operational-plan-v1', 'provider': 'google', 'model': 'offline-fixture',
                'token_budget': 2000, 'jobs': jobs,
                'artifacts': {sys.executable: digest(sys.executable), str(fixture): digest(fixture)}}

    def execute(name, modes):
        frozen = plan(name, modes)
        output = root / name
        reads = []

        def credentials(provider):
            state = json.loads((output / 'state.json').read_text())
            assert state['guard']['inflight'] and state['current_reserve'] > 0
            reads.append(provider)
            return {}

        result = OperationalRunner(frozen, output, credentials).run()
        return frozen, result, reads

    frozen, result, reads = execute('normal-helper', [('success', 'normal', 'baseline'), ('success', 'normal', 'candidate'), ('success', 'helper', 'helper')])
    assert result['complete'] and len(reads) == 3 and result['known_total_tokens'] == 30
    assert result['charged_reservations'] == 3 * reserve_for(frozen['jobs'][0]['budget'])
    controls.append('normal_candidate_helper_share_durable_reservations')
    for name, mode, latch in [('quota', 'quota', 'typed_quota'), ('unknown-usage', 'missing', 'usage_accounting_unavailable')]:
        frozen, result, reads = execute(name, [(mode, 'normal', 'baseline'), ('success', 'helper', 'helper')])
        assert not result['complete'] and len(reads) == 1 and result['charged_reservations'] > 0
        assert result['rows'][1]['admission']['allowed'] is False
        if name == 'quota':
            assert result['latch'] == latch
        else:
            assert result['unknown_usage_rows'] == 1 and result['rows'][0]['outcome']['tokens'] is None
        controls.append(name + '_stops_following_callbacks_without_refund')
    for name, mode, termination in [('timeout-soft', 'cooperative', 'sigint'), ('timeout-hard', 'hard', 'sigkill_after_grace')]:
        frozen, result, reads = execute(name, [(mode, 'normal', 'baseline'), ('success', 'normal', 'candidate')])
        assert (root / name / 'row-0/handler-ready').read_text() == 'ready'
        assert not result['complete'] and len(reads) == 1 and result['unknown_usage_rows'] == 1
        assert result['rows'][0]['termination'] == termination and result['rows'][0]['operational_failure'] == 'wall_timeout'
        assert result['rows'][1]['admission']['allowed'] is False
        controls.append(name)
    frozen = plan('parent-term', [('hard', 'normal', 'baseline'), ('success', 'helper', 'helper')])
    frozen['jobs'][0]['timeout_seconds'] = 15
    file = root / 'parent-term-plan.json'
    file.write_text(json.dumps(frozen))
    output = root / 'parent-term'
    child = subprocess.Popen([sys.executable, str(legacy / 'runner.py'), '--plan', str(file), '--output', str(output)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    ready = output / 'row-0/handler-ready'
    try:
        deadline = time.monotonic() + 10
        while not ready.exists():
            assert child.poll() is None and time.monotonic() < deadline, 'fixture did not become ready'
            time.sleep(0.01)
        child.send_signal(signal.SIGTERM)
        stdout, stderr = child.communicate(timeout=5)
        assert child.returncode != 0, (stdout, stderr)
        result = json.loads((output / 'summary.json').read_text())
        assert not result['complete'] and result['rows'][0]['operational_failure'] == 'cancelled'
        assert result['rows'][0]['termination'] == 'sigkill_after_grace'
        assert result['rows'][1]['admission']['allowed'] is False
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
    controls.append('parent_sigterm_after_child_ready_contains_child')
    before = (root / 'parent-term-spawns').read_bytes()
    restart = subprocess.run([sys.executable, str(legacy / 'runner.py'), '--plan', str(file), '--output', str(output)], capture_output=True)
    assert restart.returncode != 0 and (root / 'parent-term-spawns').read_bytes() == before
    controls.append('restart_refuses_new_spawn')
    summary = {'all_pass': True, 'controls': controls, 'external_provider_requests': 0, 'new_accuracy_cases': 0,
               'scheduler_sha256': digest(legacy / 'runner.py'), 'original_fixture_sha256': hashlib.sha256(original.encode()).hexdigest(),
               'fixture_sha256': digest(fixture), 'fixture_change': 'readiness marker after installed signal handler',
               'inherited_suite_note': 'Original 120ms startup / early SIGTERM controls failed before child readiness on this Linux runtime; original failure log retained. These controls wait for readiness or allow 2s startup.'}
    (root / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
