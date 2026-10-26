"""This preparation snapshot only permits inert numeric-loopback callbacks."""
import json
import hashlib
from pathlib import Path
import sys
from urllib.parse import urlsplit


def require_loopback_preparation(model, env):
    protocol = json.loads((Path(__file__).parent / 'protocol.json').read_text())
    if protocol['state'] not in ['PREPARATION_NO_LIVE_ALLOWED', 'DEVELOPMENT_REQUIRES_FROZEN_RECEIPT']:
        raise ValueError('new_execution_admission_not_implemented')
    url = urlsplit(env.get('ENGINE073_LOOPBACK_BASE_URL', ''))
    if not (model == 'fixture-model' and url.scheme == 'http'
            and url.hostname == '127.0.0.1' and url.port is not None
            and url.username is None and url.password is None
            and url.path == '/' and not url.query and not url.fragment):
        raise ValueError('paired_preparation_requires_inert_loopback')
    for name in ['MISTRAL_API_KEY', 'GEMINI_API_KEY', 'GOOGLE_API_KEY', 'GROQ_API_KEY', 'NVIDIA_API_KEY']:
        if env.get(name) not in [None, 'inert-loopback-only']:
            raise ValueError('paired_preparation_disallows_real_credentials')


def validate_reserved_callback(args, plan, state, directory, argv, receipt_path, receipt_sha):
    """A receipt permits only the exact current, durably reserved lane job."""
    from scorer import require
    require(plan.get('purpose') == 'CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE', 'callback_wrong_purpose')
    require(plan.get('provider') == args.provider and plan.get('model') == args.model, 'callback_provider_model_mismatch')
    require(plan.get('development_receipt_file') == str(receipt_path), 'callback_receipt_path_mismatch')
    require(plan.get('artifacts', {}).get(str(receipt_path)) == receipt_sha, 'callback_receipt_not_in_plan')
    require(Path(plan['corpus_root']) / 'corpus.json' == args.corpus.resolve(), 'callback_corpus_mismatch')
    require(args.host_policy.resolve() == Path(plan['corpus_root']) / 'host-policy.json', 'callback_host_policy_mismatch')
    require(args.bin_dir.resolve() == Path('/eval/bin'), 'callback_binary_directory_mismatch')
    require(state.get('provider') == args.provider and state.get('model') == args.model, 'callback_state_identity')
    require(state.get('guard', {}).get('inflight') is True and state['guard'].get('latch') is None, 'callback_not_reserved_inflight')
    rows = state.get('rows', [])
    require(len(rows) < len(plan['jobs']), 'callback_after_finished_lane')
    job = plan['jobs'][len(rows)]
    require(directory.name == job['id'] == args.case + '-' + args.arm, 'callback_wrong_current_job')
    require(argv == job['argv'], 'callback_argv_mismatch')
    require(job['contract'] == ('normal' if args.track == 'e2e_normal' else 'helper'), 'callback_track_mismatch')
    require((args.track == 'e2e_normal' and args.arm in ['baseline', 'candidate'])
            or (args.track == 'helper_opt_in' and args.arm == 'helper'), 'callback_track_arm_mismatch')
    budget = job['budget']
    reserve = budget['aggregate_input_tokens'] + budget['aggregate_output_tokens']
    require(state.get('current_reserve') == reserve, 'callback_reservation_mismatch')
    expected_charged = sum(j['budget']['aggregate_input_tokens'] + j['budget']['aggregate_output_tokens']
                           for j in plan['jobs'][:len(rows) + 1])
    require(state.get('charged_reservations') == expected_charged, 'callback_durable_charge_mismatch')


def require_callback_admission(args, env):
    from scorer import require
    if args.model == 'fixture-model':
        require_loopback_preparation(args.model, env)
        return
    require('ENGINE073_LOOPBACK_BASE_URL' not in env, 'live_callback_cannot_use_engineering_redirect')
    protocol = json.loads((Path(__file__).parent / 'protocol.json').read_text())
    require(protocol['state'] == 'DEVELOPMENT_REQUIRES_FROZEN_RECEIPT', 'paired_runtime_preparation_only')
    require(args.provider in ['mistral', 'google'] and protocol['providers'][args.provider]['new_live_run_allowed'] is True,
            'provider_not_admitted')
    require(args.model == protocol['providers'][args.provider]['model'], 'callback_unfrozen_model')
    name = env.get('CLI_REPAIR_DEVELOPMENT_RECEIPT', '')
    digest = env.get('CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256', '')
    require(bool(name) and len(digest) == 64, 'callback_requires_frozen_receipt')
    receipt_path = Path(name)
    require(receipt_path.is_absolute() and receipt_path.is_file(), 'callback_receipt_missing')
    require(hashlib.sha256(receipt_path.read_bytes()).hexdigest() == digest, 'callback_receipt_sha_mismatch')
    from development import checked_receipt, consumed_corpus
    checked_receipt(receipt_path, args.provider)
    consumed_corpus(args.corpus.resolve().parent)
    directory = Path.cwd().resolve()
    plan = json.loads((directory.parent / 'plan.json').read_bytes())
    state = json.loads((directory.parent / 'state.json').read_bytes())
    from run_lane import check_plan
    check_plan(plan, protocol)
    validate_reserved_callback(args, plan, state, directory, [sys.executable, *sys.argv], receipt_path, digest)
