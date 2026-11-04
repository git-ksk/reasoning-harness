"""Frozen v3 limits: request/usage/clock faults and actual loopback native jobs."""
import copy
import json
import os
from pathlib import Path
import threading
import time
import unittest
import subprocess
from unittest.mock import patch

import authority
from contracts import canonical, digest
from http_limits import FROZEN_HTTP_LIMITS, validate_http_limits
from local_store import initialize_local_store, LocalAtomicStore
from native_http import LIMITED_JOURNAL, normalize_journal
from native_lane import NativeLane
import test_native_http as http_controls
import test_native_admission as admission_controls


def encode(events, binding):
    previous = "0" * 64; rows = []
    for sequence, event in enumerate(events):
        raw = canonical({"contract": LIMITED_JOURNAL, "sequence": sequence, "previous_sha256": previous,
                         "binding_sha256": digest(canonical(binding)), "event": event})
        rows.append(raw + b"\n"); previous = digest(raw)
    return b"".join(rows)


class FrozenLimitsTests(unittest.TestCase):
    def test_limits_equal_original_protocol_without_raising_caps_or_retries(self):
        protocol = json.loads((Path(__file__).resolve().parent.parent / "protocol.json").read_bytes())
        old = protocol["normal_bound"]; new = FROZEN_HTTP_LIMITS
        for current, prior in (("max_request_utf8_json_bytes", "request_UTF8_JSON_bytes"),
                ("max_input_tokens_per_generation", "input_token_upper_bound_per_generation"),
                ("max_output_tokens_per_generation", "output_tokens_per_generation"),
                ("max_input_tokens_total", "observed_input_tokens"), ("max_output_tokens_total", "observed_output_tokens"),
                ("max_generations", "adapter_generations")):
            self.assertEqual(new[current], old[prior])
        self.assertEqual(new["min_generation_start_interval_ms"], protocol["pacing"]["minimum_generation_start_interval_ms"])
        self.assertEqual(old["provider_attempts_per_generation"], 1)
        validate_http_limits(new)

    def test_weakened_or_untyped_limits_and_unbound_internal_retries_rejected(self):
        for key, value in (("max_input_tokens_total", 24001), ("max_generations", 25),
                ("min_generation_start_interval_ms", 10499), ("max_request_utf8_json_bytes", True),
                ("max_output_tokens_total", 4096.0), ("input_overhead_tokens", 511)):
            changed = {**FROZEN_HTTP_LIMITS, key: value}
            with self.assertRaises(ValueError): validate_http_limits(changed)
        control = {**FROZEN_HTTP_LIMITS, "min_generation_start_interval_ms": 0}
        validate_http_limits(control, control=True)
        for attempts in (2, 3):
            with self.assertRaisesRegex(ValueError, "reservation"):
                validate_http_limits(control, control=True, budget={"max_attempts_per_generation": attempts,
                    "max_generations": 24, "max_input_tokens_per_generation": 12512,
                    "max_output_tokens_per_generation": 1024})

    def test_both_arms_must_freeze_the_same_policy_before_receipt(self):
        cases = ["case-" + str(n) for n in range(24)]
        jobs = [{"id": case + "-" + arm, "arm": arm, "contract": "normal",
            "budget": {"max_generations": 24, "max_input_tokens_per_generation": 12512,
                       "max_output_tokens_per_generation": 1024, "max_attempts_per_generation": 1},
            "http_limits": dict(FROZEN_HTTP_LIMITS)} for case in cases for arm in ("baseline", "candidate")]
        plan = {"schema": "engine07-operational-plan-v1", "case_ids": cases,
            "questions": {c: "What can be verified?" for c in cases}, "jobs": jobs,
            "frozen_request_bindings": {j["id"]: "a" * 64 for j in jobs}, "token_budget": 20000000}
        inventory = authority.freeze_jobs(plan)
        self.assertEqual(inventory[0]["http_limits"], FROZEN_HTTP_LIMITS)
        jobs[0]["http_limits"]["max_output_tokens_total"] -= 1
        with self.assertRaisesRegex(ValueError, "paired_http_limits_mismatch"): authority.freeze_jobs(plan)


class LimitedJournalTests(unittest.TestCase):
    def setUp(self):
        legacy = http_controls.JournalValidationTests(); legacy.setUp()
        self.binding, self.trace = legacy.binding, legacy.trace
        limits = {**FROZEN_HTTP_LIMITS, "max_generations": 2, "min_generation_start_interval_ms": 20}
        self.binding["job"].update(reserve_tokens=20000, http_limits=limits)
        first, response = self.trace["events"]
        size = len(canonical(first["request"]))
        first.update(request_utf8_json_bytes=size, requested_output_tokens=10)
        second, final = copy.deepcopy(first), copy.deepcopy(response)
        second.update(sequence=2, attempt=2); final.update(sequence=3, attempt=2)
        self.trace["events"] += [second, final]
        self.events = [{"kind": "binding", "max_attempts": 2, "token_reserve": 20000, "limits": limits}]
        for index in range(2):
            rid = "http-control-" + str(index + 1)
            context = {"correlation_id": "control", "logical_attempt": index + 1, "request_sha256": "a" * 64}
            self.events += [
                {"kind": "preflight", "request_id": rid, "context": context, "request_utf8_json_bytes": size,
                 "input_reserve_tokens": size + 512, "requested_output_tokens": 10,
                 "used_input_tokens": index * 2, "used_output_tokens": index * 3},
                {"kind": "request", "request_id": rid, "context": context, "provider": "mistral", "model": "fixture-model",
                 "wire_request_sha256": "c" * 64, "start_elapsed_ns": 1000 + index * 20000000},
                {"kind": "response", "request_id": rid, "http_status": 200, "response_sha256": "d" * 64,
                 "usage": {"input_tokens": 2, "output_tokens": 3, "total_tokens": 5}, "failure_class": None},
                {"kind": "completed", "request_id": rid}]

    def normalized(self, events=None, binding=None, trace=None):
        return normalize_journal(encode(self.events if events is None else events, self.binding),
                                 self.binding if binding is None else binding, self.trace if trace is None else trace)

    def test_v2_preflight_and_actual_start_times_normalize_without_semantic_credit(self):
        events = self.normalized()
        self.assertEqual(len(events), 2); self.assertEqual(sum(e["total_tokens"] for e in events), 10)
        self.assertNotIn("continuation_authorized", events[0])

    def test_missing_preflight_changed_limits_or_start_time_are_rejected(self):
        mutations = [lambda x: x.pop(1), lambda x: x[0]["limits"].update(max_input_tokens_total=23999),
                     lambda x: x[6].update(start_elapsed_ns=1001), lambda x: x[2].pop("start_elapsed_ns"),
                     lambda x: x[1].update(input_reserve_tokens=1), lambda x: x[5].update(used_input_tokens=0),
                     lambda x: x[1].update(request_utf8_json_bytes=1),
                     lambda x: x[3]["usage"].update(output_tokens=11, total_tokens=13)]
        for mutate in mutations:
            changed = copy.deepcopy(self.events); mutate(changed)
            with self.assertRaises(ValueError): self.normalized(events=changed)

    def test_trace_original_request_metrics_and_policy_bound_receipt_are_required(self):
        for key in ("request_utf8_json_bytes", "requested_output_tokens"):
            trace = copy.deepcopy(self.trace); trace["events"][0].pop(key)
            with self.assertRaises(ValueError): self.normalized(trace=trace)
        binding = copy.deepcopy(self.binding); binding["job"].pop("http_limits")
        with self.assertRaises(ValueError): self.normalized(binding=binding)


@unittest.skipUnless(os.environ.get("REASON_V3_NATIVE_CLI"), "set REASON_V3_NATIVE_CLI for actual native limits controls")
class ActualNativeLimitsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls): admission_controls.ActualNativeAdmissionTests.setUpClass()
    @classmethod
    def tearDownClass(cls): admission_controls.ActualNativeAdmissionTests.tearDownClass()

    def setUp(self):
        self.base = admission_controls.ActualNativeAdmissionTests(); self.base.setUp()
        self.addCleanup(self.base.doCleanups)

    def configure(self, *, changes=None, include=True):
        base = self.base
        plan = copy.deepcopy(base.plan)
        if include:
            for job in plan["jobs"]:
                job["http_limits"] = {**FROZEN_HTTP_LIMITS, "max_generations": 4,
                    "max_output_tokens_per_generation": 32, "max_output_tokens_total": 128,
                    "min_generation_start_interval_ms": 0, **(changes or {})}
                job["timeout_seconds"] = 30
        profile = initialize_local_store(base.root / "limited-authority")
        base.store = LocalAtomicStore(profile, role="executor")
        base.evaluator_store = LocalAtomicStore(profile, role="evaluator")
        _, receipt = base.template.fixture("native-581", plan=plan, binding=base.binding)
        receipt["contract"] = authority.NATIVE_RECEIPT
        receipt["native_admission"] = {"bundle_sha256": base.bundle.sha256,
            "policy_sha256": digest(base.reviewer.policy_bytes), "sources_sha256": digest(base.catalog_bytes)}
        LocalAtomicStore(profile, role="issuer").create(f"{authority.PREFIX}/receipts/native-581.json", canonical(receipt))
        base.session = authority.TrustedAuthority(base.store).start("native-581", plan, base.template.local_receipt, base.binding)
        base.plan = plan
        return base

    def test_actual_v2_http_binding_and_signed_advance(self):
        base = self.configure(); lane, server = base.lane()
        packet, expected = lane.run_next()
        rows = [json.loads(line) for line in (lane.output / expected["job_id"] / "http-journal.jsonl").read_bytes().splitlines()]
        self.assertTrue(all(row["contract"] == LIMITED_JOURNAL for row in rows))
        self.assertEqual(len([r for r in rows if r["event"]["kind"] == "preflight"]), len(server.requests))
        self.assertIn("pacing_sha256", packet)
        lane.accept_review(base.sign(packet, expected)); self.assertEqual(lane.next, 1)

    def test_request_size_preflight_halts_with_full_reserve_and_zero_actual_http(self):
        base = self.configure(changes={"max_request_utf8_json_bytes": 1}); lane, server = base.lane()
        with self.assertRaises(ValueError): lane.run_next()
        self.assertEqual(server.requests, []); self.assertGreater(base.session._reserved, 0)
        with self.assertRaisesRegex(ValueError, "halted"): lane.run_next()
        raw = (lane.output / base.plan["jobs"][0]["id"] / "http-journal.jsonl").read_bytes()
        self.assertIn(b'"kind":"blocked"', raw)

    def test_output_total_blocks_second_send_while_preserving_first_usage(self):
        base = self.configure(changes={"max_output_tokens_total": 32}); lane, server = base.lane()
        with self.assertRaises(ValueError): lane.run_next()
        self.assertEqual(len(server.requests), 1); self.assertGreater(base.session._reserved, 0)
        raw = (lane.output / base.plan["jobs"][0]["id"] / "http-journal.jsonl").read_bytes()
        self.assertIn(b'"total_tokens":5', raw)
        self.assertIn(b'"kind":"blocked"', raw)

    def test_between_job_pacing_and_tampered_pacing_record_cannot_advance(self):
        base = self.configure(changes={"min_generation_start_interval_ms": 30}); lane, _ = base.lane()
        packet, expected = lane.run_next(); lane.accept_review(base.sign(packet, expected))
        packet, expected = lane.run_next()
        folder = lane.output / expected["job_id"]
        pacing = json.loads((folder / "pacing.json").read_bytes())
        self.assertGreaterEqual(pacing["wait_elapsed_ns"], 30000000)
        signed = base.sign(packet, expected); (folder / "pacing.json").write_bytes(b"{}")
        with self.assertRaisesRegex(ValueError, "pacing_changed"): lane.accept_review(signed)
        with self.assertRaisesRegex(ValueError, "halted"): lane.run_next()

    def test_cancel_during_initial_job_pacing_spawns_nothing_and_keeps_reservation(self):
        base = self.configure(changes={"min_generation_start_interval_ms": 2000}); lane, server = base.lane()
        errors = []
        def run():
            try: lane.run_next()
            except BaseException as error: errors.append(error)
        thread = threading.Thread(target=run); thread.start()
        folder = lane.output / base.plan["jobs"][0]["id"]
        deadline = time.monotonic() + 120
        while not (folder / "binding.json").exists() and time.monotonic() < deadline: time.sleep(.01)
        self.assertTrue((folder / "binding.json").exists())
        lane.cancel(); thread.join(5)
        self.assertFalse(thread.is_alive()); self.assertTrue(errors)
        self.assertEqual(server.requests, []); self.assertFalse((folder / "execution.json").exists())
        self.assertGreater(base.session._reserved, 0)

    def test_live_profile_without_limits_and_plan_policy_tampering_are_rejected(self):
        base = self.configure(include=False)
        with self.assertRaisesRegex(ValueError, "http_limits_required"):
            NativeLane(session=base.session, plan=base.plan, bundle=base.bundle,
                       policy_bytes=base.reviewer.policy_bytes, catalog_bytes=base.catalog_bytes, output=base.root / "no-live")
        plan = copy.deepcopy(base.plan)
        plan["jobs"][0]["http_limits"] = dict(FROZEN_HTTP_LIMITS)
        with self.assertRaisesRegex(ValueError, "receipt_identity_mismatch"):
            NativeLane(session=base.session, plan=plan, bundle=base.bundle,
                       policy_bytes=base.reviewer.policy_bytes, catalog_bytes=base.catalog_bytes, output=base.root / "changed")

    def test_full_48_v2_children_complete_signed_advance_and_first_score_claim(self):
        base = self.configure()
        lane, server = base.lane(); hashes = {}
        for _ in range(48):
            packet, expected = lane.run_next(); lane.accept_review(base.sign(packet, expected))
            hashes[expected["job_id"]] = expected["observation_sha256"]
        self.assertEqual(lane.next, 48); self.assertGreaterEqual(len(server.requests), 48)
        self.assertIsNotNone(base.store.read(authority._key("native-581", "completed")))
        evaluator = authority.TrustedAuthority(base.evaluator_store)
        args = dict(observation_index_sha256="1" * 64, review_manifest_sha256="2" * 64, observation_hashes=hashes)
        evaluator.claim_score("native-581", base.plan, base.template.local_receipt, base.binding, **args)
        with self.assertRaises(FileExistsError):
            evaluator.claim_score("native-581", base.plan, base.template.local_receipt, base.binding, **args)

    def test_mutating_controller_plan_after_enrollment_latches_before_spawn(self):
        base = self.configure(); lane, server = base.lane()
        lane.plan["jobs"][0]["http_limits"]["min_generation_start_interval_ms"] = 10
        with self.assertRaisesRegex(ValueError, "frozen_plan_changed"): lane.run_next()
        self.assertEqual(server.requests, []); self.assertTrue(base.session._halted)

    def test_plan_mutation_while_pacing_cannot_change_spawn_timeout_grace_or_argv(self):
        base = self.configure(changes={"min_generation_start_interval_ms": 2000}); lane, server = base.lane()
        errors = []
        def run():
            try: lane.run_next()
            except BaseException as error: errors.append(error)
        with patch("native_lane.capture_process", side_effect=AssertionError("must stop before spawn")) as spawn:
            thread = threading.Thread(target=run); thread.start()
            folder = lane.output / base.plan["jobs"][0]["id"]
            deadline = time.monotonic() + 120
            while not (folder / "binding.json").exists() and time.monotonic() < deadline: time.sleep(.005)
            self.assertTrue((folder / "binding.json").exists())
            job = lane.plan["jobs"][0]
            job["argv"][job["argv"].index("--max-model-calls") + 1] = "99"
            job.update(timeout_seconds=600, cancel_grace_seconds=5); job.pop("http_limits")
            thread.join(5)
            self.assertFalse(thread.is_alive()); spawn.assert_not_called()
        self.assertTrue(errors); self.assertIn("frozen_plan_changed", str(errors[0]))
        self.assertTrue(base.session._halted); self.assertEqual(server.requests, [])

    def test_direct_cli_refuses_missing_live_limits_before_credentials_or_journal_write(self):
        root = self.base.root
        fd = os.open(root / "no-live.jsonl", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            argv = [str(Path(os.environ["REASON_V3_NATIVE_CLI"]).resolve()), "--no-config", "--plain",
                "--provider", "mistral", "--model", "fixture-model", "--format", "json",
                "--diagnostic-trace", str(root / "no-live-trace.json"), "--evaluation-http-journal-fd", str(fd),
                "--evaluation-http-binding-sha256", "a" * 64, "--evaluation-http-max-attempts", "1",
                "--evaluation-http-token-reserve", "1000", "What is verified?"]
            result = subprocess.run(argv, env={"PATH": "/usr/bin:/bin"}, pass_fds=(fd,),
                                    capture_output=True, timeout=20)
        finally: os.close(fd)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b"frozen HTTP limits", result.stdout + result.stderr)
        self.assertEqual((root / "no-live.jsonl").stat().st_size, 0)
