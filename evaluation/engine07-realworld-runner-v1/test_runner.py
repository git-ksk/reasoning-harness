import copy
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest

from runner import OperationalRunner, digest, reserve_for

HERE = Path(__file__).resolve().parent


class RunnerIntegration(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.fixture = self.root / 'fixture.py'
        shutil.copyfile(HERE / 'fixture_child.py', self.fixture)
        self.marker = self.root / 'spawns.txt'
        self.output = self.root / 'first'
        self.credential_reads = 0

    def tearDown(self):
        self.temp.cleanup()

    def job(self, index, mode='success', contract='normal', arm='baseline'):
        return {'id': f'row-{index}', 'arm': arm, 'contract': contract,
                'argv': [sys.executable, str(self.fixture), '--mode', mode,
                         '--contract', contract, '--marker', str(self.marker),
                         '--provider', 'google', '--model', 'offline-fixture'],
                'budget': {'max_generations': 2, 'max_input_tokens_per_generation': 20,
                           'max_output_tokens_per_generation': 20, 'max_attempts_per_generation': 2},
                'timeout_seconds': 2, 'cancel_grace_seconds': 0.2}

    def plan(self, jobs):
        return {'schema': 'engine07-operational-plan-v1', 'provider': 'google', 'model': 'offline-fixture',
                'token_budget': 2000, 'jobs': jobs,
                'artifacts': {sys.executable: digest(sys.executable), str(self.fixture): digest(self.fixture)}}

    def credentials(self, provider):
        self.credential_reads += 1
        state = json.loads((self.output / 'state.json').read_text())
        self.assertTrue(state['guard']['inflight'])
        self.assertGreater(state['current_reserve'], 0)
        self.assertEqual(state['charged_reservations'], state['current_reserve'] * self.credential_reads)
        return {}

    def run_jobs(self, jobs, configure=None):
        plan = self.plan(jobs)
        if configure:
            configure(plan)
        return OperationalRunner(plan, self.output, self.credentials).run()

    def spawn_count(self):
        return len(self.marker.read_text().splitlines()) if self.marker.exists() else 0

    def test_normal_baseline_candidate_and_helper_share_durable_reservations(self):
        jobs = [self.job(1), self.job(2, arm='candidate'), self.job(3, contract='helper', arm='helper')]
        result = self.run_jobs(jobs)
        self.assertTrue(result['complete'])
        self.assertEqual((self.spawn_count(), self.credential_reads), (3, 3))
        self.assertEqual(result['known_total_tokens'], 30)
        self.assertEqual(result['charged_reservations'], 3 * reserve_for(jobs[0]['budget']))
        self.assertEqual(result['semantic_acceptance'], 'NOT_SCORED')
        self.assertFalse(json.loads((self.output / 'state.json').read_text())['guard']['inflight'])

    def test_quota_stops_all_normal_helper_and_engine_transitions(self):
        for first_contract, second_contract in [('normal', 'helper'), ('helper', 'normal'), ('normal', 'normal')]:
            with self.subTest(first=first_contract, second=second_contract):
                self.output = self.root / f'{first_contract}-{second_contract}'
                self.marker = self.root / f'{first_contract}-{second_contract}.txt'
                self.credential_reads = 0
                result = self.run_jobs([self.job(1, 'quota', first_contract), self.job(2, contract=second_contract, arm='candidate')])
                self.assertEqual((self.spawn_count(), self.credential_reads), (1, 1))
                self.assertEqual(result['latch'], 'typed_quota')
                self.assertFalse(any(row['complete'] for row in result['rows']))
                self.assertFalse(result['rows'][1]['admission']['allowed'])

    def test_known_groq_exhaustion_blocks_before_credentials_for_any_model(self):
        for model in ('openai/gpt-oss-120b', 'another-model'):
            self.output = self.root / model.replace('/', '-')
            plan = self.plan([self.job(1), self.job(2, contract='helper')])
            plan['provider'] = 'groq'
            plan['model'] = model
            for job in plan['jobs']:
                job['argv'][-3] = 'groq'
                job['argv'][-1] = model
            result = OperationalRunner(plan, self.output, self.credentials).run()
            self.assertEqual((self.spawn_count(), self.credential_reads), (0, 0))
            self.assertEqual(result['latch'], 'known_daily_quota_exhausted')
            self.assertFalse(result['complete'])
            self.assertEqual(len(result['rows']), 2)

    def test_unknown_usage_is_unavailable_and_reservation_never_refunded(self):
        result = self.run_jobs([self.job(1, 'missing'), self.job(2)])
        self.assertFalse(result['complete'])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['unknown_usage_rows'], 1)
        self.assertEqual(result['known_total_tokens'], 0)  # known subtotal, NOT a total-cost claim
        self.assertIsNone(result['rows'][0]['outcome']['tokens'])
        self.assertGreater(result['charged_reservations'], 0)
        self.assertIsNotNone(result['rows'][0]['operational_failure'])

    def test_measured_success_does_not_refund_conservative_callback_reserve(self):
        jobs = [self.job(1), self.job(2)]
        result = self.run_jobs(jobs, lambda p: p.update(token_budget=reserve_for(jobs[0]['budget'])))
        self.assertTrue(result['rows'][0]['complete'])
        self.assertFalse(result['rows'][1]['complete'])
        self.assertEqual((self.spawn_count(), result['known_total_tokens']), (1, 10))
        self.assertEqual(result['latch'], 'arm_reserved_budget_exhausted')

    def test_capacity_failures_carry_across_candidate_and_helper(self):
        jobs = [self.job(1, 'capacity'), self.job(2, 'capacity', arm='candidate'), self.job(3, contract='helper')]
        result = self.run_jobs(jobs)
        self.assertEqual(self.spawn_count(), 2)
        self.assertEqual(result['known_total_tokens'], 14)
        self.assertEqual(result['latch'], 'two_consecutive_capacity_failures')
        self.assertFalse(result['rows'][2]['admission']['allowed'])

    def test_timeout_requests_sigint_and_preserves_lower_bound_trace(self):
        job = self.job(1, 'cooperative')
        job.update(timeout_seconds=0.15, cancel_grace_seconds=0.5)
        result = self.run_jobs([job, self.job(2, contract='helper')])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['rows'][0]['termination'], 'sigint')
        self.assertEqual(result['rows'][0]['operational_failure'], 'wall_timeout')
        self.assertTrue((self.output / 'row-1' / 'partial-trace.json').exists())
        self.assertEqual(result['unknown_usage_rows'], 1)
        self.assertIsNone(result['rows'][0]['outcome']['tokens'])
        self.assertTrue(result['rows'][0]['partial_diagnostics_observed']['usage_is_lower_bound'])

    def test_hard_kill_only_after_cancel_grace_remains_incomplete(self):
        job = self.job(1, 'hard')
        job.update(timeout_seconds=0.12, cancel_grace_seconds=0.08)
        result = self.run_jobs([job, self.job(2)])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['rows'][0]['termination'], 'sigkill_after_grace')
        self.assertEqual(result['rows'][0]['returncode'], -signal.SIGKILL)
        self.assertGreaterEqual(result['rows'][0]['wall_ms'], 190)
        self.assertFalse(result['complete'])

    def test_child_crash_and_unknown_failure_stop_following_requests(self):
        result = self.run_jobs([self.job(1, 'crash'), self.job(2)])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['latch'], 'unknown_failure_class')
        self.assertEqual(result['unknown_usage_rows'], 1)
        self.assertFalse(result['complete'])

    def test_credential_exception_is_durable_and_does_not_leak_secret_text(self):
        plan = self.plan([self.job(1), self.job(2)])
        def broken(provider):
            self.credentials(provider)
            raise RuntimeError('SECRET_MUST_NOT_LEAK')
        result = OperationalRunner(plan, self.output, broken).run()
        self.assertEqual((self.credential_reads, self.spawn_count()), (1, 0))
        self.assertEqual(result['latch'], 'interrupted_accounting')
        self.assertFalse(result['complete'])
        self.assertFalse(any('SECRET_MUST_NOT_LEAK' in p.read_text() for p in self.output.glob('*.json')))

    def test_restart_refuses_even_completed_or_corrupt_previous_state(self):
        plan = self.plan([self.job(1)])
        OperationalRunner(plan, self.output, self.credentials).run()
        (self.output / 'state.json').write_text('broken')
        with self.assertRaises(FileExistsError):
            OperationalRunner(plan, self.output, self.credentials)
        self.assertEqual((self.credential_reads, self.spawn_count()), (1, 1))

    def test_frozen_artifact_change_suppresses_every_planned_row(self):
        runner = OperationalRunner(self.plan([self.job(1), self.job(2)]), self.output, self.credentials)
        self.fixture.write_text('changed')
        result = runner.run()
        self.assertEqual((self.credential_reads, self.spawn_count()), (0, 0))
        self.assertEqual(result['latch'], 'frozen_artifact_changed')
        self.assertEqual(len(result['rows']), 2)

    def test_model_option_switch_or_duplicate_is_rejected_before_output_and_auth(self):
        for extra in (['--model', 'another-model'], ['--model=another-model'], ['--provider=groq']):
            plan = self.plan([self.job(1)])
            plan['jobs'][0]['argv'].extend(extra)
            with self.assertRaises(ValueError):
                OperationalRunner(plan, self.output, self.credentials)
            self.assertFalse(self.output.exists())
            self.assertEqual((self.credential_reads, self.spawn_count()), (0, 0))

    def test_typed_protocol_failure_stops_even_with_measured_usage(self):
        result = self.run_jobs([self.job(1, 'protocol'), self.job(2)])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['known_total_tokens'], 7)
        self.assertEqual(result['latch'], 'typed_operational_failure:protocol')
        self.assertFalse(result['complete'])

    def test_observed_callback_overshoot_stops_next_invocation(self):
        result = self.run_jobs([self.job(1, 'overshoot'), self.job(2)])
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(result['rows'][0]['operational_failure'], 'reservation_exceeded')
        self.assertFalse(result['complete'])

    def launch_runner(self, job):
        plan = self.plan([job, self.job(2, contract='helper')])
        plan_path = self.root / 'plan.json'
        plan_path.write_text(json.dumps(plan))
        return subprocess.Popen([sys.executable, str(HERE / 'runner.py'), '--plan', str(plan_path),
                                 '--output', str(self.output)], stdout=subprocess.PIPE, stderr=subprocess.PIPE), plan_path

    def wait_for_spawn(self):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            try:
                return json.loads((self.output / 'row-1' / 'spawn.json').read_text())['pid']
            except (OSError, ValueError):
                time.sleep(0.01)
        self.fail('fixture child did not start')

    def test_sigterm_parent_cancels_child_and_records_all_suppressed_rows(self):
        job = self.job(1, 'cooperative')
        job.update(timeout_seconds=5, cancel_grace_seconds=0.5)
        proc, _ = self.launch_runner(job)
        self.wait_for_spawn()
        time.sleep(0.08)  # fixture has installed its cooperative handler
        proc.send_signal(signal.SIGTERM)
        proc.communicate(timeout=3)
        result = json.loads((self.output / 'summary.json').read_text())
        self.assertEqual(proc.returncode, 1)
        self.assertEqual(self.spawn_count(), 1)
        self.assertEqual(len(result['rows']), 2)
        self.assertEqual(result['rows'][0]['operational_failure'], 'cancelled')
        self.assertEqual(result['rows'][0]['termination'], 'sigint')
        self.assertFalse(result['complete'])

    def test_killed_parent_keeps_inflight_and_restart_spawns_nothing(self):
        job = self.job(1, 'hard')
        job.update(timeout_seconds=5)
        proc, plan_path = self.launch_runner(job)
        child = self.wait_for_spawn()
        try:
            state = json.loads((self.output / 'state.json').read_text())
            self.assertTrue(state['guard']['inflight'])
            self.assertGreater(state['current_reserve'], 0)
            proc.kill()
            proc.communicate(timeout=2)
            replay = subprocess.run([sys.executable, str(HERE / 'runner.py'), '--plan', str(plan_path),
                                     '--output', str(self.output)], capture_output=True, timeout=2)
            self.assertNotEqual(replay.returncode, 0)
            self.assertEqual(self.spawn_count(), 1)
            self.assertTrue(json.loads((self.output / 'state.json').read_text())['guard']['inflight'])
        finally:
            if proc.poll() is None:
                proc.kill()
                proc.communicate(timeout=2)
            try:
                os.killpg(child, signal.SIGKILL)
            except ProcessLookupError:
                pass


if __name__ == '__main__':
    unittest.main()
