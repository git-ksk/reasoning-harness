"""First-observation subprocess runner; no credentials or network on import.

A plan schedules normal baseline/candidate and helper commands through ONE arm.
The semantic corpus/scorer and host/binaries must be frozen separately before use.
"""
from dataclasses import asdict
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

OPERATIONS = Path(__file__).resolve().parents[1] / 'engine07-realworld-operations-v1'
sys.path.insert(0, str(OPERATIONS))
from bridge import outcome_from_helper, outcome_from_normal
from guard import CAPACITY_FAILURES, FailureClass, Outcome, ProviderArmGuard


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def write_new(path, value):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump(value, stream, sort_keys=True, indent=2)
        stream.write('\n')
        stream.flush()
        os.fsync(stream.fileno())
    sync_directory(Path(path).parent)


def positive(value):
    return type(value) is int and value > 0


def reserve_for(budget):
    keys = ('max_generations', 'max_input_tokens_per_generation',
            'max_output_tokens_per_generation', 'max_attempts_per_generation')
    if not isinstance(budget, dict) or not all(positive(budget.get(k)) for k in keys):
        raise ValueError('freeze complete callback generation/input/output/retry bounds')
    return (budget[keys[0]] * (budget[keys[1]] + budget[keys[2]]) * budget[keys[3]])


def validate_plan(plan):
    if plan.get('schema') != 'engine07-operational-plan-v1':
        raise ValueError('unknown plan contract')
    if plan.get('provider') not in ('mistral', 'google', 'groq') or not plan.get('model'):
        raise ValueError('freeze provider/model')
    if not positive(plan.get('token_budget')):
        raise ValueError('positive arm self-budget required')
    if not isinstance(plan.get('jobs'), list) or not plan['jobs']:
        raise ValueError('freeze scheduled normal/helper rows')
    artifacts = plan.get('artifacts')
    if not isinstance(artifacts, dict) or not artifacts:
        raise ValueError('pin executables and static inputs by SHA256')
    for path, sha in artifacts.items():
        if not Path(path).is_absolute() or digest(path) != sha:
            raise ValueError('frozen artifact mismatch')
    ids = set()
    for job in plan['jobs']:
        job_id = job.get('id', '')
        if (not job_id or job_id in ids or len(job_id) > 100
                or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_' for c in job_id)):
            raise ValueError('unique safe row id required')
        ids.add(job_id)
        if job.get('contract') not in ('normal', 'helper') or job.get('arm') not in ('baseline', 'candidate', 'helper'):
            raise ValueError('explicit output contract and arm required')
        reserve_for(job.get('budget'))
        for key in ('timeout_seconds', 'cancel_grace_seconds'):
            if type(job.get(key)) not in (int, float) or not 0 < job[key] <= 600:
                raise ValueError('bounded deadline and grace required')
        argv = job.get('argv')
        if not isinstance(argv, list) or not argv or not all(isinstance(s, str) for s in argv):
            raise ValueError('argument vector required')
        if argv[0] not in artifacts:
            raise ValueError('executable not pinned')
        for key, expected in (('--provider', plan['provider']), ('--model', plan['model'])):
            if any(arg.startswith(key + '=') for arg in argv):
                raise ValueError('ambiguous inline provider/model option')
            if argv.count(key) != 1 or argv.index(key) + 1 == len(argv) or argv[argv.index(key) + 1] != expected:
                raise ValueError('command provider/model differs from arm')
    return plan


class OperationalRunner:
    def __init__(self, plan, output, credential_loader):
        validate_plan(plan)
        self.plan = plan
        self.output = Path(output).resolve()
        self.load_credentials = credential_loader
        self.rows = []
        self.charged_reservations = 0
        self.known_total_tokens = 0
        self.known_provider_attempts = 0
        self.unknown_usage_rows = 0
        self.current_reserve = None
        self.sequence = 0
        self.cancelled = False
        # create-new is the restart barrier, even if a previous state is corrupt,
        # complete, empty or left between journal/state persistence steps.
        self.output.mkdir(mode=0o700)
        sync_directory(self.output.parent)
        write_new(self.output / 'plan.json', plan)
        self.guard = ProviderArmGuard(token_budget=plan['token_budget'])
        self.groq_state_sha256 = None
        if plan['provider'] == 'groq':
            # Keep the existing provider-wide prohibition, including another
            # model/account. Recovery is a separate verified policy change.
            state_path = OPERATIONS / 'groq-state.json'
            self.groq_state_sha256 = digest(state_path)
            state = json.loads(state_path.read_text())
            if state.get('provider') != 'groq' or state.get('new_live_run_allowed') is not False:
                self.guard.latch = 'groq_recovery_policy_unverified'
            else:
                self.guard.latch = 'known_daily_quota_exhausted'
        self.persist('created')

    def persist(self, event):
        self.sequence += 1
        state = {
            'schema': 'engine07-durable-arm-v1', 'sequence': self.sequence,
            'provider': self.plan['provider'], 'model': self.plan['model'],
            'guard': self.guard.snapshot(), 'charged_reservations': self.charged_reservations,
            'current_reserve': self.current_reserve, 'known_total_tokens': self.known_total_tokens,
            'known_provider_attempts': self.known_provider_attempts,
            'unknown_usage_rows': self.unknown_usage_rows, 'rows': self.rows,
            'groq_state_sha256': self.groq_state_sha256,
        }
        # Each event is exclusive and durable before state points at it. Atomic
        # replace + directory fsync prevents a torn latest state. Existing output
        # ALWAYS refuses restart, including any interruption between these steps.
        event_path = self.output / f'event-{self.sequence:06}.json'
        write_new(event_path, {'event': event, 'state': state})
        temporary = self.output / f'state-{self.sequence:06}.tmp'
        write_new(temporary, state)
        os.replace(temporary, self.output / 'state.json')
        sync_directory(self.output)

    def stop_child(self, proc, grace):
        try:
            os.killpg(proc.pid, signal.SIGINT)
        except ProcessLookupError:
            pass
        try:
            proc.wait(timeout=grace)
            return 'sigint'
        except subprocess.TimeoutExpired:
            try:
                os.killpg(proc.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            proc.wait()
            return 'sigkill_after_grace'

    def process(self, job, env, directory):
        argv = [s.replace('{RUN_DIR}', str(directory)) for s in job['argv']]
        started = time.monotonic()
        failure = None
        termination = None
        proc = None
        with (directory / 'stdout.json').open('xb') as stdout, (directory / 'stderr.txt').open('xb') as stderr:
            try:
                if self.cancelled:
                    raise KeyboardInterrupt()
                proc = subprocess.Popen(argv, env=env, cwd=directory, stdout=stdout, stderr=stderr,
                                        start_new_session=True)
                # Persist the child identity as well; reserve/inflight was already
                # durable BEFORE credential retrieval and before this spawn.
                write_new(directory / 'spawn.json', {'pid': proc.pid, 'at_unix': time.time()})
                deadline = started + job['timeout_seconds']
                while proc.poll() is None:
                    if self.cancelled or time.monotonic() >= deadline:
                        failure = FailureClass.CANCELLED if self.cancelled else FailureClass.WALL_TIMEOUT
                        termination = self.stop_child(proc, job['cancel_grace_seconds'])
                        break
                    try:
                        proc.wait(timeout=min(0.05, max(0.001, deadline - time.monotonic())))
                    except subprocess.TimeoutExpired:
                        pass
            except BaseException:
                self.cancelled = True
                failure = FailureClass.CANCELLED
                if proc is not None:
                    termination = self.stop_child(proc, job['cancel_grace_seconds'])
            finally:
                stdout.flush()
                stderr.flush()
                os.fsync(stdout.fileno())
                os.fsync(stderr.fileno())
        try:
            payload = json.loads((directory / 'stdout.json').read_text())
        except (OSError, ValueError):
            payload = {}
        bridge = outcome_from_normal if job['contract'] == 'normal' else outcome_from_helper
        outcome = bridge(payload)
        partial = (payload.get('result', {}).get('partial_diagnostics') if isinstance(payload, dict) else None)
        if failure is not None:
            outcome = Outcome(failure, None, None)
        elif proc is None or (proc.returncode != 0 and outcome.failure_class is None):
            outcome = Outcome(FailureClass.UNKNOWN, None, None)
        return outcome, {
            'returncode': proc.returncode if proc is not None else None,
            'termination': termination, 'wall_ms': round((time.monotonic() - started) * 1000),
            'partial_diagnostics_observed': partial,
            'stdout_sha256': digest(directory / 'stdout.json'),
            'stderr_sha256': digest(directory / 'stderr.txt'),
        }

    def run(self):
        previous_handlers = {}
        def cancel(signum, frame):
            # Finish durable writes, then cancel the active child with grace.
            self.cancelled = True
        for sig in (signal.SIGINT, signal.SIGTERM):
            previous_handlers[sig] = signal.signal(sig, cancel)
        try:
            for job in self.plan['jobs']:
                reserve = reserve_for(job['budget'])
                directory = self.output / job['id']
                directory.mkdir(mode=0o700)
                if self.cancelled:
                    self.guard.latch = self.guard.latch or 'cancelled_operation'
                try:
                    validate_plan(self.plan)
                except (OSError, ValueError):
                    self.guard.latch = self.guard.latch or 'frozen_artifact_changed'
                if self.guard.latch is None and self.charged_reservations + reserve > self.plan['token_budget']:
                    self.guard.latch = 'arm_reserved_budget_exhausted'
                decision = self.guard.before_invocation(reserve)
                row = {'id': job['id'], 'arm': job['arm'], 'contract': job['contract'],
                       'admission': asdict(decision), 'complete': False, 'reserve': reserve}
                if not decision.allowed:
                    row['operational_failure'] = decision.reason
                    self.rows.append(row)
                    write_new(directory / 'row.json', row)
                    self.persist('suppressed')
                    continue
                self.charged_reservations += reserve  # no speculative refunds
                self.current_reserve = reserve
                self.persist('reserved_inflight')
                try:
                    credentials = self.load_credentials(self.plan['provider'])
                    env = os.environ.copy()
                    env.update(credentials)
                    env['REASON_HOME'] = str(directory / 'private-reason-home')
                    outcome, diagnostics = self.process(job, env, directory)
                    self.guard.observe(outcome)
                    if outcome.failure_class is not None and outcome.failure_class not in CAPACITY_FAILURES:
                        self.guard.latch = self.guard.latch or 'typed_operational_failure:' + outcome.failure_class.value
                    row.update(diagnostics)
                    row['outcome'] = asdict(outcome)
                    row['operational_failure'] = outcome.failure_class
                    known = (type(outcome.tokens) is int and outcome.tokens >= 0
                             and type(outcome.provider_attempts) is int and outcome.provider_attempts >= 0)
                    if known:
                        self.known_total_tokens += outcome.tokens
                        self.known_provider_attempts += outcome.provider_attempts
                    else:
                        self.unknown_usage_rows += 1
                        row['operational_failure'] = row['operational_failure'] or 'usage_accounting_unavailable'
                    caps = job['budget']
                    if known and (outcome.tokens > reserve or outcome.provider_attempts > caps['max_generations'] * caps['max_attempts_per_generation']):
                        self.guard.latch = self.guard.latch or 'reservation_exceeded'
                        row['operational_failure'] = 'reservation_exceeded'
                    row['complete'] = outcome.failure_class is None and known and row['operational_failure'] is None
                except BaseException as error:
                    self.guard.inflight = False
                    self.guard.latch = self.guard.latch or 'interrupted_accounting'
                    self.unknown_usage_rows += 1
                    row['operational_failure'] = 'interrupted_accounting'
                    row['host_exception_type'] = type(error).__name__  # never exception text/secrets
                self.current_reserve = None
                self.rows.append(row)
                write_new(directory / 'row.json', row)
                self.persist('observed')
            result = {'complete': not self.cancelled and all(row['complete'] for row in self.rows), 'rows': self.rows,
                      'known_total_tokens': self.known_total_tokens,
                      'known_provider_attempts': self.known_provider_attempts,
                      'unknown_usage_rows': self.unknown_usage_rows,
                      'charged_reservations': self.charged_reservations,
                      'semantic_acceptance': 'NOT_SCORED', 'latch': self.guard.latch}
            write_new(self.output / 'summary.json', result)
            self.persist('finished')
            return result
        finally:
            for sig, handler in previous_handlers.items():
                signal.signal(sig, handler)


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--plan', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    plan = json.loads(args.plan.read_text())
    # CLI/keyring credential resolution remains owned by each pinned child. No
    # credential command occurs here, especially for a suppressed Groq arm.
    runner = OperationalRunner(plan, args.output, lambda provider: {})
    result = runner.run()
    print(json.dumps({k: v for k, v in result.items() if k != 'rows'}))
    return 0 if result['complete'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
