"""HTTP-zero admission controls. No check stubs, monkeypatches or model entry points."""
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from types import SimpleNamespace

HERE = Path('/repo/evaluation/cli-repair-paired-v1/normal_v1')
sys.path.insert(0, str(HERE))
import development
import run_lane
import run_task
from image_identity import verify_loaded_image

OUT = Path('/offline')
FROZEN = Path('/frozen')
RECEIPT = FROZEN / 'development.json'
PROVIDER = 'mistral'
ACTUAL_ID = 'sha256:ea084e99e3a31784f3aa3b755b232cd0e7673107a79064bc434923c666ee9e69'

def require(value, message):
    if not value:
        raise AssertionError(message)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def dump(path, value):
    with path.open('x') as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write('\n')

def expect_rejection(name, operation, expected):
    try:
        operation()
    except (ValueError, FileExistsError) as error:
        text = str(error)
        require(isinstance(error, FileExistsError) if expected == 'FileExistsError' else expected in text,
                name + ': unexpected rejection ' + type(error).__name__ + ':' + text)
        return {'name': name, 'rejected': True, 'exception_type': type(error).__name__,
                'reason': expected}
    raise AssertionError(name + ': negative control was admitted')

def main():
    keys = ['MISTRAL_API_KEY', 'GOOGLE_API_KEY', 'GEMINI_API_KEY', 'GROQ_API_KEY', 'NVIDIA_API_KEY']
    require(not any(os.environ.get(key) for key in keys), 'provider credential present')
    frozen_before = {str(p.relative_to(FROZEN)): sha(p) for p in FROZEN.rglob('*') if p.is_file()}
    receipt = development.checked_receipt(RECEIPT, PROVIDER)
    require(sha(RECEIPT) == '7a9eba8cffaab549aabde40995ba3d0d106c4b2a4b697ce2e0d992a2f8e61dc6',
            'wrong genuine candidate receipt')
    aliases = verify_loaded_image(Path('/image/runtime-image.tar.gz'),
                                  receipt['runtime_image_archive_sha256'],
                                  receipt['runtime_image_id'], ACTUAL_ID)
    corpus = development.consumed_corpus(FROZEN / 'corpus')
    protocol = json.loads((HERE / 'protocol.json').read_bytes())
    plan_path = OUT / 'generated-plan.json'
    command = [sys.executable, '-B', str(HERE / 'make_development_plan.py'),
               '--receipt', str(RECEIPT), '--corpus-dir', str(FROZEN / 'corpus'),
               '--provider', PROVIDER, '--output', str(plan_path)]
    result = subprocess.run(command, capture_output=True)
    (OUT / 'make-plan.stdout').write_bytes(result.stdout)
    (OUT / 'make-plan.stderr').write_bytes(result.stderr)
    require(result.returncode == 0, 'actual make_development_plan failed')
    plan = json.loads(plan_path.read_bytes())
    reservation_usd = run_lane.check_plan(plan, protocol)
    require(len(plan['jobs']) == 48 and plan['provider'] == PROVIDER, 'not 48 Mistral jobs')
    require(plan['model'] == 'ministral-8b-2512', 'wrong model')
    expected = []
    for index, case in enumerate(corpus['cases']):
        for arm in (['baseline', 'candidate'] if index % 2 == 0 else ['candidate', 'baseline']):
            expected.append((case['id'] + '-' + arm, arm, 'normal'))
    require([(j['id'], j['arm'], j['contract']) for j in plan['jobs']] == expected,
            'counterbalanced normal plan changed')
    require(plan['token_budget'] == 48 * 28096 and reservation_usd == '0.2022912',
            'reservation total changed')
    require(all(j['budget']['aggregate_input_tokens'] == 24000
                and j['budget']['aggregate_output_tokens'] == 4096
                and j['argv'][j['argv'].index('--track') + 1] == 'e2e_normal'
                for j in plan['jobs']), 'helper or token scope changed')
    lane = OUT / 'offline-reservation-only'
    runner = run_lane.AuditedRunner(plan, lane, protocol)
    job = runner.plan['jobs'][0]
    directory = lane / job['id']
    directory.mkdir(mode=0o700)
    values = {name: job['argv'][job['argv'].index('--' + name.replace('_', '-')) + 1]
              for name in ['provider', 'model', 'corpus', 'host_policy', 'case', 'track', 'arm', 'bin_dir']}
    for name in ['corpus', 'host_policy', 'bin_dir']:
        values[name] = Path(values[name])
    args = SimpleNamespace(**values)
    env = {'CLI_REPAIR_DEVELOPMENT_RECEIPT': str(RECEIPT),
           'CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256': sha(RECEIPT)}
    require(run_task.preparation_guard.require_callback_admission is run_task.require_ordinary_callback,
            'frozen normal dispatch disconnected')
    os.chdir(directory)
    sys.argv = job['argv'][1:]
    negatives = [expect_rejection('genuine receipt without reservation',
                                 lambda: run_task.require_ordinary_callback(args, env),
                                 'callback_not_reserved_inflight')]
    # Use the exact production guard/reserve/persist methods, without entering
    # runner.run(), process(), credential loading, or paired.main().
    run_lane.scheduler.validate_plan(runner.plan)
    reserve = run_lane.scheduler.reserve_for(job['budget'])
    decision = runner.guard.before_invocation(reserve)
    require(decision.allowed and reserve == 28096, 'offline reservation denied')
    runner.charged_reservations += reserve
    runner.current_reserve = reserve
    runner.persist('reserved_inflight')
    reserved_state = json.loads((lane / 'state.json').read_bytes())
    run_task.require_ordinary_callback(args, env)
    positive = {'current_job': job['id'], 'arm': job['arm'], 'accepted': True,
                'durable_sequence': reserved_state['sequence'],
                'current_reserve': reserved_state['current_reserve'],
                'charged_reservations': reserved_state['charged_reservations'],
                'inflight': reserved_state['guard']['inflight'], 'rows': len(reserved_state['rows'])}
    for name, changed, reason in [
        ('helper prohibited', {'track': 'helper_opt_in', 'arm': 'helper'}, 'ordinary_only'),
        ('Google prohibited', {'provider': 'google'}, 'provider_not_admitted'),
        ('wrong current arm', {'arm': 'candidate'}, 'callback_wrong_current_job')]:
        modified = SimpleNamespace(**(vars(args) | changed))
        negatives.append(expect_rejection(name, lambda modified=modified:
                         run_task.require_ordinary_callback(modified, env), reason))
    sys.argv = job['argv'][1:] + ['--unexpected']
    negatives.append(expect_rejection('argv must be byte exact',
                     lambda: run_task.require_ordinary_callback(args, env), 'callback_argv_mismatch'))
    sys.argv = job['argv'][1:]
    for index, (name, mutate, reason) in enumerate([
        ('reservation charge changed', lambda s: s.update(charged_reservations=0), 'callback_durable_charge_mismatch'),
        ('reservation bound changed', lambda s: s.update(current_reserve=1), 'callback_reservation_mismatch'),
        ('inflight lost', lambda s: s['guard'].update(inflight=False), 'callback_not_reserved_inflight'),
        ('quota latch active', lambda s: s['guard'].update(latch='typed_quota'), 'callback_not_reserved_inflight')]):
        negative_lane = OUT / ('negative-state-' + str(index))
        negative_lane.mkdir()
        shutil.copyfile(lane / 'plan.json', negative_lane / 'plan.json')
        state = copy.deepcopy(reserved_state)
        mutate(state)
        dump(negative_lane / 'state.json', state)
        negative_directory = negative_lane / job['id']
        negative_directory.mkdir()
        os.chdir(negative_directory)
        negatives.append(expect_rejection(name, lambda:
                         run_task.require_ordinary_callback(args, env), reason))
    os.chdir(directory)
    negatives.append(expect_rejection('offline ledger restart is refused',
                     lambda: run_lane.AuditedRunner(plan, lane, protocol), 'FileExistsError'))
    require((lane / 'state.json').read_bytes() == json.dumps(reserved_state, sort_keys=True, indent=2).encode() + b'\n',
            'original offline reservation ledger changed')
    forbidden = [p for p in OUT.rglob('*') if p.name in ['spawn.json', 'model-journal.jsonl',
                  'stdout.json', 'actual-cli.stdout', 'summary.json']]
    require(not forbidden, 'model process or lane run output appeared')
    frozen_after = {str(p.relative_to(FROZEN)): sha(p) for p in FROZEN.rglob('*') if p.is_file()}
    require(frozen_before == frozen_after, 'genuine frozen inputs changed')
    report = {'reviewer': '/root/real24_operational_review', 'reviewer_kind': 'agent_nonhuman',
              'result': 'PASS_GENUINE_RECEIPT_PLAN_RESERVED_CALLBACK_HTTP_ZERO',
              'receipt_sha256': sha(RECEIPT), 'actual_image_id': ACTUAL_ID,
              'archive_derived_aliases': sorted(aliases), 'checked_receipt': 'PASS',
              'actual_plan_jobs': len(plan['jobs']), 'normal_jobs': 48, 'helper_jobs': 0,
              'providers': [PROVIDER], 'model': plan['model'],
              'counterbalanced_order_verified': True, 'token_reservation': plan['token_budget'],
              'frozen_nominal_USD_reservation': reservation_usd,
              'plan_sha256': sha(plan_path), 'positive_reserved_callback': positive,
              'negative_controls': negatives, 'check_stubs_or_monkeypatches': False,
              'runner_run_or_process_called': False, 'paired_main_called': False,
              'provider_credentials_loaded': False, 'external_HTTP_requests': 0,
              'model_generations': 0, 'actual_live_lane_created': False,
              'offline_ledger_is_unexecuted_reserved_only': True,
              'genuine_frozen_inputs_unchanged': True, 'formal_runtime_verdict_changed': False,
              'quota_headroom': 'UNKNOWN', 'full_cohort_completion_guaranteed': False,
              'readiness_valid_until_unix_seconds': receipt['required_provider_readiness'][PROVIDER]['valid_until_unix_seconds'],
              'checked_at_unix_seconds': int(time.time())}
    dump(OUT / 'admission-result.json', report)
    print(json.dumps(report))

if __name__ == '__main__':
    main()
