"""One-host native v3 controller with mandatory pre-advance review (#582).

No implicit run, enrollment, retry, model approval, refund or semantic grading.
Each run_next returns a packet; only accept_review can advance the fixed run.
"""
import json
import os
from pathlib import Path
import selectors
import signal
import socket
import subprocess
import threading
import time
import math

from collect import collect_row
from contracts import (FILES, canonical, digest, exclusive_json, frozen_request_digest,
                       private_new_directory, require, _open_trusted_directory)
from native_admission import ADMISSION, capture_display, source_catalog, verify_pre_advance
from native_http import NativeHttpCapture, _unique
from review_auth import validate_policy
from native_bundle import MAC_PROCESS_POLICY

CONTROL = "NATIVE_V3_LOOPBACK_CONTROL_NO_PROVIDER_EXECUTION"
MAX_STREAM = 8 * 1024 * 1024


def private_snapshot(path):
    from local_store import _read
    path = Path(path)
    parent = _open_trusted_directory(path.parent)
    try:
        fd = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
        try: return _read(fd)
        finally: os.close(fd)
    finally: os.close(parent)


def private_bytes(path, raw):
    path = Path(path)
    parent = _open_trusted_directory(path.parent)
    try:
        fd = os.open(path.name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=parent)
        with os.fdopen(fd, "wb") as stream:
            os.fchmod(stream.fileno(), 0o600)
            stream.write(raw); stream.flush(); os.fsync(stream.fileno())
        os.fsync(parent)
    finally: os.close(parent)


def validate_argv(argv, binary, question, bundle=None, budget=None):
    require(type(argv) is list and len(argv) > 2 and all(type(x) is str for x in argv)
            and argv[0] == binary and argv[-1] == question, "native_argv_identity_mismatch")
    require(argv.count("--format") == 1 and argv[argv.index("--format") + 1] == "json"
            and "--plain" in argv and ("--no-config" in argv or "--config" in argv), "native_json_profile_required")
    require(not any(x.startswith(("--evaluation-", "--diagnostic-trace", "--continue", "--resume", "--ephemeral"))
                    for x in argv[1:-1]), "unfrozen_native_control_argument")
    require(not any(x.startswith(("--file=", "--config=", "--resolver-command=")) for x in argv[1:-1]),
            "native_requires_canonical_path_options")
    if bundle is not None:
        if bundle.data.get("process_policy") == MAC_PROCESS_POLICY:
            require("--resolver-command" not in argv, "native_no_fork_external_program_rejected")
            if "--config" in argv:
                configured = json.loads(Path(argv[argv.index("--config") + 1]).read_bytes(), object_pairs_hook=_unique)
                def no_program(value):
                    if isinstance(value, dict):
                        require("program" not in value, "native_no_fork_external_program_rejected")
                        for child in value.values(): no_program(child)
                    elif isinstance(value, list):
                        for child in value: no_program(child)
                no_program(configured)
        paths = {v["path"] for v in bundle.data["files"].values()}
        for index, flag in enumerate(argv[:-1]):
            if flag in ("--config", "--file", "--resolver-command"):
                require(index + 1 < len(argv) - 1 and argv[index + 1] in paths, "native_unfrozen_config_or_executable")
    if budget is not None:
        limits = {"--max-tokens": budget["max_output_tokens_per_generation"],
                  "--max-model-calls": budget["max_generations"],
                  "--max-total-tokens": budget["max_generations"] *
                      (budget["max_input_tokens_per_generation"] + budget["max_output_tokens_per_generation"])}
        for flag, ceiling in limits.items():
            require(argv.count(flag) == 1 and argv[argv.index(flag) + 1].isdigit()
                    and 0 < int(argv[argv.index(flag) + 1]) <= ceiling, "native_argv_budget_mismatch")


def plan_first_request(bundle, arm, argv, destination):
    """Uses the actual first-request builder; no key, model, response or HTTP."""
    validate_argv(argv, bundle.binary(arm), argv[-1], bundle)
    bundle.verify()
    isolation = ["--evaluation-isolated-config"] if "--config" in argv else []
    directory = Path(str(destination) + ".process"); private_new_directory(directory)
    code, complete = capture_process(argv + isolation + ["--evaluation-request-plan", str(destination)],
        directory=directory, environment={"PATH": "/usr/bin:/bin"}, pass_fds=(), timeout=30,
        supervised=True, no_fork=bundle.data["process_policy"] == MAC_PROCESS_POLICY)
    require(complete and code == 0 and not private_snapshot(directory / "cli.json")
            and not private_snapshot(directory / "stderr.txt"), "native_request_planning_failed")
    bundle.verify()
    raw = private_snapshot(destination)
    planned = json.loads(raw, object_pairs_hook=_unique)
    require(planned.get("contract") == "reason-native-first-request-plan-v1"
            and planned.get("question") == argv[-1], "native_request_plan_mismatch")
    planned.update(native_bundle_sha256=bundle.sha256, argv_sha256=digest(canonical(argv)), arm=arm)
    return planned


class NativeCancelled(KeyboardInterrupt):
    pass


def capture_process(argv, *, directory, environment, pass_fds, timeout, cancel_grace=1,
                    supervised=False, no_fork=False, stop_event=None):
    """Bounded complete non-TTY streams. Overflow/timeout/cancel cannot admit."""
    require(type(timeout) in (int, float) and math.isfinite(timeout) and 0 < timeout <= 600
            and type(cancel_grace) in (int, float) and math.isfinite(cancel_grace) and 0 < cancel_grace <= 5,
            "native_invalid_process_deadline")
    parent = child = None
    process = None
    streams = {"stdout": bytearray(), "stderr": bytearray()}
    complete = True
    reason = None
    selector = selectors.DefaultSelector()
    deadline = time.monotonic() + timeout
    handlers = {}
    def cancelled(signum, _frame): raise NativeCancelled("native_controller_signal_" + str(signum))
    def read_streams(wait):
        nonlocal complete, reason
        for key, _ in selector.select(wait):
            raw = os.read(key.fileobj.fileno(), 65536)
            if not raw: selector.unregister(key.fileobj); continue
            output = streams[key.data]
            room = MAX_STREAM - len(output)
            output.extend(raw[:room])
            if len(raw) > room: complete, reason = False, reason or "native_stream_overflow"
    sent_interrupt = hard_killed = False
    try:
        if stop_event is not None and stop_event.is_set(): raise NativeCancelled("native_cancelled_before_spawn")
        # Install before spawn; SIGTERM must not abandon a child while we can
        # still perform cooperative cancellation and persist partial streams.
        if threading.current_thread() is threading.main_thread():
            for sig in (signal.SIGINT, signal.SIGTERM):
                handlers[sig] = signal.signal(sig, cancelled)
        actual = list(argv)
        inherited = tuple(pass_fds)
        if supervised:
            parent, child = socket.socketpair()
            actual += ["--evaluation-lifeline-fd", str(child.fileno())]
            if no_fork: actual += ["--evaluation-no-fork"]
            inherited += (child.fileno(),)
        process = subprocess.Popen(actual, env=environment, stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            pass_fds=inherited, umask=0o077, start_new_session=True)
        if child is not None: child.close(); child = None
        for name, pipe in (("stdout", process.stdout), ("stderr", process.stderr)):
            os.set_blocking(pipe.fileno(), False)
            selector.register(pipe, selectors.EVENT_READ, name)
        if parent is not None:
            handshake_deadline = min(deadline, time.monotonic() + 5)
            ready = b"reason-evaluation-lifeline-v1\n"
            received = b""
            while len(received) < len(ready):
                parent.settimeout(max(.001, handshake_deadline - time.monotonic()))
                chunk = parent.recv(len(ready) - len(received))
                require(bool(chunk), "native_lifeline_missing")
                received += chunk
            require(received == ready, "native_lifeline_wrong_handshake")
        exclusive_json(Path(directory) / "supervision.json", {
            "contract": "reason-native-process-supervision-v1", "pid": process.pid,
            "argv": actual, "lifeline": supervised, "no_fork": no_fork,
            "timeout_seconds": timeout, "cancel_grace_seconds": cancel_grace})
        if stop_event is not None and stop_event.is_set(): raise NativeCancelled("native_cancelled_before_go")
        if parent is not None: parent.sendall(b"G")
        while selector.get_map():
            if stop_event is not None and stop_event.is_set(): raise NativeCancelled("native_cancelled")
            if time.monotonic() >= deadline:
                complete, reason = False, "native_process_timeout"
                break
            read_streams(min(0.05, max(0, deadline - time.monotonic())))
            if not complete: break
        # A child can close its streams yet remain alive. It still must exit
        # within the same wall-clock bound before any admission.
        while complete and process.poll() is None:
            if stop_event is not None and stop_event.is_set(): raise NativeCancelled("native_cancelled")
            if time.monotonic() >= deadline:
                complete, reason = False, "native_process_timeout"
                break
            time.sleep(.01)
    except BaseException:
        complete, reason = False, "native_process_interrupted"
        raise
    finally:
        # A second signal cannot skip cleanup. Restore prior handlers only
        # after process reaping and durable output records.
        for sig in handlers: signal.signal(sig, signal.SIG_IGN)
        try:
            if process is not None:
                if process.poll() is None or not complete:
                    try: os.killpg(process.pid, signal.SIGINT); sent_interrupt = True
                    except ProcessLookupError: pass
                    grace_deadline = time.monotonic() + cancel_grace
                    while process.poll() is None and time.monotonic() < grace_deadline:
                        read_streams(min(.05, max(0, grace_deadline - time.monotonic())))
                    if not no_fork or process.poll() is None:
                        try: os.killpg(process.pid, signal.SIGKILL); hard_killed = True
                        except ProcessLookupError: pass
                process.wait(timeout=10)
                drain_deadline = time.monotonic() + .5
                while selector.get_map() and time.monotonic() < drain_deadline:
                    read_streams(.01)
        except BaseException:
            complete, reason = False, "native_cleanup_failed"
            raise
        finally:
            # Even kill/read failures revoke the socket. The Mac native child
            # then self-terminates; never abandon an open lifeline on error.
            if parent is not None: parent.close()
            if child is not None: child.close()
            selector.close()
            if process is not None:
                process.stdout.close(); process.stderr.close()
            try:
                for name, raw in streams.items():
                    private_bytes(Path(directory) / ("cli.json" if name == "stdout" else "stderr.txt"), bytes(raw))
                exclusive_json(Path(directory) / "execution.json", {"exit_code": process.returncode if process else None,
                    "streams_complete": complete, "operational_failure": reason, "sigint_sent": sent_interrupt,
                    "sigkill_sent": hard_killed, "process_reaped": process is not None and process.returncode is not None,
                    "stdout_sha256": digest(streams["stdout"]), "stderr_sha256": digest(streams["stderr"])})
            finally:
                for sig, previous in handlers.items(): signal.signal(sig, previous)
    return process.returncode, complete


class NativeLane:
    def __init__(self, *, session, plan, bundle, policy_bytes, catalog_bytes, output, control_url=None, provider_environment=None):
        self.session, self.plan, self.bundle = session, json.loads(canonical(plan)), bundle
        self.policy_bytes, self.catalog_bytes = policy_bytes, catalog_bytes
        validate_policy(policy_bytes)
        self.catalog = source_catalog(catalog_bytes)
        receipt = session._receipt  # same trusted controller package, not receipt-selected enrollment
        from authority import NATIVE_RECEIPT
        require(receipt["contract"] == NATIVE_RECEIPT, "native_authority_receipt_required")
        binding = bundle.binding(receipt["executor_id"], receipt["evaluator_id"])
        require(all(receipt[k] == getattr(binding, k) for k in ("source_sha256", "image_sha256", "cli_build_sha256", "runtime_sha256"))
                and digest(canonical(plan)) == receipt["plan_sha256"], "native_lane_receipt_identity_mismatch")
        for job in plan["jobs"]:
            jid, arm = job["id"], job["arm"]
            question = plan["questions"][jid.rsplit("-", 1)[0]]
            validate_argv(job["argv"], bundle.binary(arm), question, bundle, job["budget"])
            for index, flag in enumerate(job["argv"][:-1]):
                if flag == "--config":
                    configured = json.loads(Path(job["argv"][index + 1]).read_bytes(), object_pairs_hook=_unique)
                    def check_programs(value):
                        if isinstance(value, dict):
                            for key, child in value.items():
                                if key == "program":
                                    require(type(child) is str and any(v["path"] == child for v in bundle.data["files"].values()),
                                            "native_config_program_not_frozen")
                                check_programs(child)
                        elif isinstance(value, list):
                            for child in value: check_programs(child)
                    check_programs(configured)
                if flag == "--file":
                    path = job["argv"][index + 1]
                    context = self.catalog.get("context_files", {}).get(path)
                    require(context is not None and any(v["path"] == path and v["sha256"] == context["sha256"]
                            for v in bundle.data["files"].values()), "native_context_full_source_not_frozen")
            planned = job.get("native_request_plan", {})
            require(planned.get("contract") == "reason-native-first-request-plan-v1"
                    and planned.get("native_bundle_sha256") == bundle.sha256 and planned.get("arm") == arm
                    and planned.get("argv_sha256") == digest(canonical(job["argv"]))
                    and planned.get("provider") == plan["provider"] and planned.get("model") == plan["model"]
                    and planned.get("question") == question, "native_preflight_plan_identity_mismatch")
            require(frozen_request_digest(job_id=jid, case_id=jid.rsplit("-", 1)[0], arm=arm, question=question,
                request=planned["request"], request_sha256=planned["request_sha256"], provider=plan["provider"],
                model=plan["model"], receipt_sha256=plan["receipt_sha256"]) == plan["frozen_request_bindings"][jid],
                "native_preflight_request_binding_mismatch")
        files = bundle.data["files"]
        require(any(k.startswith("config/") and v["sha256"] == digest(catalog_bytes) for k, v in files.items())
                and any(k.startswith("config/") and v["sha256"] == digest(policy_bytes) for k, v in files.items()),
                "native_policy_or_sources_not_in_bundle")
        self.control_url = control_url
        if control_url is not None:
            require(plan.get("purpose") == CONTROL and plan.get("model") == "fixture-model",
                    "loopback_control_not_a_provider_admission")
            self.environment = {"PATH": "/usr/bin:/bin", "MISTRAL_API_KEY": "fixture-only-key", "NO_PROXY": "127.0.0.1"}
        else:
            require(bundle.data.get("process_policy") == MAC_PROCESS_POLICY,
                    "provider_execution_requires_macos_no_descendant_profile")
            require(plan.get("purpose") != CONTROL, "control_requires_explicit_loopback")
            environment = provider_environment or {}
            require(set(environment) <= {"MISTRAL_API_KEY", "REASON_CA_BUNDLE", "HTTPS_PROXY", "HTTP_PROXY", "NO_PROXY"},
                    "unfrozen_native_environment")
            if "REASON_CA_BUNDLE" in environment:
                require(any(v["path"] == environment["REASON_CA_BUNDLE"] for v in files.values()), "native_unfrozen_ca_bundle")
            self.environment = {"PATH": "/usr/bin:/bin", **environment}
        bundle.verify()
        self.output = Path(output); private_new_directory(self.output)
        exclusive_json(self.output / "native-bundle.json", bundle.data)
        session.enable_native_admission(bundle_sha256=bundle.sha256, policy_sha256=digest(policy_bytes), sources_sha256=digest(catalog_bytes))
        self.next = 0
        self.pending = None
        self.stop_event = threading.Event()

    def cancel(self):
        """Revoke this evaluation, including a child running on another thread.

        Stop is durable and never clears or refunds a reservation. A killed
        controller cannot call this API; the child's lifeline handles that case.
        """
        self.stop_event.set()
        self.session.stop("native_controller_cancelled")

    def _halt(self, job_id, events, reason):
        if self.session._halted or self.session._inflight != job_id:
            self.pending = None
            return
        tokens = sum(e["total_tokens"] for e in events) if events else None
        attempts = len(events) if events else None
        self.session.observe(job_id, tokens=tokens, attempts=attempts, operational_failure=reason)
        self.pending = None

    def run_next(self):
        require(not self.stop_event.is_set() and not self.session._halted, "native_run_cancelled_or_halted")
        require(self.pending is None and self.next < 48, "native_review_pending_or_finished")
        self.bundle.verify()
        job = self.plan["jobs"][self.next]
        job_id, arm = job["id"], job["arm"]
        question = self.plan["questions"][job_id.rsplit("-", 1)[0]]
        validate_argv(job["argv"], self.bundle.binary(arm), question, self.bundle, job["budget"])
        self.session.reserve(job_id)
        capture = None
        events = None
        try:
            capture = NativeHttpCapture(self.session, job_id, question, self.output / job_id)
            isolation = ["--evaluation-isolated-config"] if "--config" in job["argv"] else []
            argv = job["argv"] + isolation + capture.cli_arguments()
            if self.control_url is not None: argv += ["--evaluation-http-loopback-url", self.control_url]
            # These exact argv/env identities are part of the packet; no
            # shell parsing, inherited loader settings, Python path or stdin.
            exclusive_json(capture.directory / "invocation.json", {"argv": argv, "environment_keys": sorted(self.environment),
                "mode": "loopback-control" if self.control_url is not None else "provider"})
            self.session.before_spawn(job_id)
            self.bundle.verify()
            exit_code, complete = capture_process(argv, directory=capture.directory, environment=self.environment,
                pass_fds=(capture.fd,), timeout=job["timeout_seconds"], cancel_grace=job.get("cancel_grace_seconds", 1),
                supervised=True, no_fork=self.bundle.data["process_policy"] == MAC_PROCESS_POLICY, stop_event=self.stop_event)
            capture.close()
            require(complete and exit_code in (0, 1), "native_process_incomplete")
            self.bundle.verify()
            events = capture.verify()
            stdout = private_snapshot(capture.directory / "cli.json")
            stderr = private_snapshot(capture.directory / "stderr.txt")
            cli = json.loads(stdout, object_pairs_hook=_unique)
            require((exit_code == 1) == (cli.get("result", {}).get("status") == "failed"), "native_exit_envelope_mismatch")
            display = capture_display(cli, self.catalog)
            exclusive_json(capture.directory / "capture.json", display)
            exclusive_json(capture.directory / "sources.json", self.catalog["sources"])
            files = {name: digest(private_snapshot(capture.directory / name)) for name in FILES}
            row = {"job_id": job_id, "case_id": job_id.rsplit("-", 1)[0], "arm": arm, "question": question,
                   "frozen_request_binding_sha256": self.plan["frozen_request_bindings"][job_id], "files": files}
            observation = collect_row(row, self.output, self.plan["provider"], self.plan["model"],
                                      self.plan["receipt_sha256"], self.session._receipt["executor_id"])
            require(observation["measurement_state"] != "operational_stop", "native_cli_operational_stop")
            packet = {"contract": "reason-cli-normal-v3-native-pre-advance-packet-v1", "job_id": job_id,
                      "native_bundle_sha256": self.bundle.sha256, "invocation_sha256": digest(private_snapshot(capture.directory / "invocation.json")),
                      "stdout": stdout.decode("utf-8"), "stderr": stderr.decode("utf-8"), "exit_code": exit_code,
                      "observation": observation, "full_sources": self.catalog["sources"], "capture_files": files}
            packet["execution_sha256"] = digest(private_snapshot(capture.directory / "execution.json"))
            packet["supervision_sha256"] = digest(private_snapshot(capture.directory / "supervision.json"))
            exclusive_json(capture.directory / "pre-advance-packet.json", packet)
            packet_sha = digest(private_snapshot(capture.directory / "pre-advance-packet.json"))
            expected = {"contract": ADMISSION, "evaluation_id": self.session._receipt["evaluation_id"], "job_id": job_id,
                        "executor_id": self.session._receipt["executor_id"], "evaluator_id": self.session._receipt["evaluator_id"],
                        "authority_receipt_sha256": self.session._sha, "native_bundle_sha256": self.bundle.sha256,
                        "http_binding_sha256": digest(canonical(capture.binding)), "packet_sha256": packet_sha,
                        "observation_sha256": digest(canonical(observation)), "sources_sha256": digest(self.catalog_bytes),
                        "stdout_sha256": digest(stdout), "stderr_sha256": digest(stderr)}
            exclusive_json(capture.directory / "pre-advance-request.json", expected)
            self.pending = (capture.directory, expected, stdout, stderr, events, observation)
            return json.loads(canonical(packet)), json.loads(canonical(expected))
        except BaseException:
            self._halt(job_id, events, "native_capture_or_runtime_rejected")
            raise
        finally:
            if capture is not None: capture.close()

    def accept_review(self, attestation):
        require(self.pending is not None, "native_no_pending_review")
        directory, expected, stdout, stderr, events, observation = self.pending
        try:
            self.bundle.verify()
            require(private_snapshot(directory / "cli.json") == stdout and private_snapshot(directory / "stderr.txt") == stderr
                    and digest(private_snapshot(directory / "pre-advance-packet.json")) == expected["packet_sha256"],
                    "native_display_changed_before_advance")
            for name, value in observation["files"].items():
                require(digest(private_snapshot(directory / name)) == value, "native_capture_changed_before_advance")
            packet = json.loads(private_snapshot(directory / "pre-advance-packet.json"))
            for name, field in (("execution.json", "execution_sha256"), ("supervision.json", "supervision_sha256")):
                require(digest(private_snapshot(directory / name)) == packet[field], "native_supervision_changed_before_advance")
            require(not self.stop_event.is_set() and not self.session._halted, "native_run_cancelled_or_halted")
            authentication = verify_pre_advance(policy_bytes=self.policy_bytes, attestation=attestation,
                expected=expected, stdout=stdout, stderr=stderr, sources=self.catalog["sources"])
            private_bytes(directory / "pre-advance-attestation.json", attestation)
            self.session.acknowledge_native_admission(expected["job_id"], packet_sha256=expected["packet_sha256"],
                observation_sha256=expected["observation_sha256"], authentication=authentication)
            require(self.session.observe(expected["job_id"], tokens=sum(e["total_tokens"] for e in events), attempts=len(events),
                    observation_sha256=expected["observation_sha256"]), "native_authority_rejected_advance")
            self.next += 1; self.pending = None
            if self.next == 48: self.session.complete()
            return observation
        except BaseException:
            self._halt(expected["job_id"], events, "native_pre_advance_review_rejected")
            raise
