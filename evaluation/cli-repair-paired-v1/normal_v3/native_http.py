"""Operator-owned native HTTP capture; no model execution or safety admission.

Only the measured native Mistral transport writes the passed private FD. The
parent independently binds its journal to the fixed local authority and trace.
The trusted owner/runtime can forge files; this is not a same-UID boundary.
"""
import json
import os
from pathlib import Path

from collect import audit_trace
from contracts import (canonical, digest, exclusive_json, frozen_request_digest,
                       private_new_directory, require, sha_field, strict_keys,
                       _open_trusted_directory, _reject_permissive_acl)

JOURNAL = "reason-cli-evaluation-http-journal-v1"


def _unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate_http_json_key")
        result[key] = value
    return result


def normalize_journal(raw, binding, trace):
    """Fail closed on any partial/operational evidence; never fill usage with 0."""
    require(type(raw) is bytes and len(raw) <= 16 * 1024 * 1024
            and raw.endswith(b"\n"), "partial_or_oversized_http_journal")
    require(binding.get("contract") == "reason-cli-normal-v3-native-http-binding-v1",
            "native_http_binding_contract_mismatch")
    require(trace.get("completion_status") in ("completed", "partial"), "native_trace_completion_missing")
    job = binding["job"]
    expected_binding = digest(canonical(binding))
    requests = [row for row in trace.get("events", []) if row.get("kind") == "model_request"]
    require(0 < len(requests) <= job["max_provider_attempts"], "invalid_native_request_inventory")
    require(trace.get("provider") == binding["provider"] and trace.get("model") == binding["model"],
            "native_trace_identity_mismatch")
    first = requests[0]
    case_id, arm = job["job_id"].rsplit("-", 1)
    require(frozen_request_digest(
        job_id=job["job_id"], case_id=case_id, arm=arm, question=binding["question"],
        request=first["request"], request_sha256=first["request_sha256"],
        provider=binding["provider"], model=binding["model"],
        receipt_sha256=binding["legacy_receipt_sha256"],
    ) == job["frozen_request_binding_sha256"], "native_first_request_not_frozen")
    previous = "0" * 64
    pending = None
    finalizing = None
    events, seen = [], set()
    rows = raw.splitlines()
    require(len(rows) <= 1 + 3 * job["max_provider_attempts"], "excess_native_http_records")
    for sequence, line in enumerate(rows):
        row = json.loads(line, object_pairs_hook=_unique)
        strict_keys(row, {"contract", "sequence", "previous_sha256", "binding_sha256", "event"},
                    {"contract", "sequence", "previous_sha256", "binding_sha256", "event"}, "native_http_row")
        require(canonical(row) == line and type(row["sequence"]) is int
                and row["sequence"] == sequence and row["previous_sha256"] == previous
                and row["contract"] == JOURNAL and row["binding_sha256"] == expected_binding,
                "native_http_chain_or_binding_mismatch")
        previous = digest(line)
        event = row["event"]
        require(type(event) is dict, "native_http_event_required")
        kind = event.get("kind")
        if sequence == 0:
            require(event == {"kind": "binding", "max_attempts": job["max_provider_attempts"],
                              "token_reserve": job["reserve_tokens"]}, "native_http_limits_mismatch")
        elif kind == "request":
            fields = {"kind", "request_id", "context", "provider", "model", "wire_request_sha256"}
            strict_keys(event, fields, fields, "native_http_request")
            require(pending is None and finalizing is None and len(seen) < len(requests), "native_http_request_order")
            context = event["context"]
            strict_keys(context, {"correlation_id", "logical_attempt", "request_sha256"},
                        {"correlation_id", "logical_attempt", "request_sha256"}, "native_http_context")
            expected = requests[len(seen)]
            require(type(context["logical_attempt"]) is int and context == {"correlation_id": expected["correlation_id"],
                                "logical_attempt": expected["attempt"],
                                "request_sha256": expected["request_sha256"]}
                    and event["provider"] == binding["provider"] and event["model"] == binding["model"]
                    and event["request_id"] == f"http-{context['correlation_id']}-{context['logical_attempt']}"
                    and event["request_id"] not in seen, "native_http_trace_request_mismatch")
            sha_field(event["wire_request_sha256"], "invalid_native_wire_hash")
            seen.add(event["request_id"])
            pending = event
        elif kind == "response":
            fields = {"kind", "request_id", "http_status", "response_sha256", "usage", "failure_class"}
            strict_keys(event, fields, fields, "native_http_response")
            require(pending is not None and event["request_id"] == pending["request_id"],
                    "native_http_unpaired_response")
            sha_field(event["response_sha256"], "invalid_native_response_hash")
            require(type(event["http_status"]) is int and 200 <= event["http_status"] < 300
                    and event["failure_class"] is None, "native_http_operational_failure")
            usage = event["usage"]
            require(type(usage) is dict and set(usage) == {"input_tokens", "output_tokens", "total_tokens"}
                    and all(type(v) is int and v >= 0 for v in usage.values())
                    and usage["input_tokens"] + usage["output_tokens"] == usage["total_tokens"],
                    "native_http_unknown_or_inconsistent_usage")
            events.append({"request_id": event["request_id"], **pending["context"],
                           "provider": binding["provider"], "model": binding["model"],
                           "http_status": event["http_status"], **usage})
            del events[-1]["request_sha256"]  # verified above; normalized collector schema is unchanged
            pending = None
            finalizing = event["request_id"]
        elif kind == "completed":
            require(event == {"kind": "completed", "request_id": finalizing} and finalizing is not None,
                    "native_http_completion_mismatch")
            finalizing = None
        else:
            raise ValueError("native_http_aborted_or_unknown_event")
    require(pending is None and finalizing is None and len(events) == len(requests), "native_http_incomplete")
    require(sum(e["total_tokens"] for e in events) <= job["reserve_tokens"], "native_http_reserve_exceeded")
    require(not audit_trace(trace, events, binding["provider"], binding["model"]),
            "native_trace_operational_failure")
    return events


class NativeHttpCapture:
    """Explicit capability setup. Caller reserves first; no automatic restart."""
    def __init__(self, session, job_id, question, directory):
        self.session, self.job_id = session, job_id
        self.binding = session.freeze_http_capture(job_id, question)
        self.directory = Path(directory)
        private_new_directory(self.directory)
        exclusive_json(self.directory / "binding.json", self.binding)
        parent = _open_trusted_directory(self.directory)
        try:
            self.fd = os.open("http-journal.jsonl", os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW,
                              0o600, dir_fd=parent)
            try:
                os.fchmod(self.fd, 0o600)
                _reject_permissive_acl(self.fd)
                os.fsync(self.fd)
                os.fsync(parent)
            except BaseException:
                os.close(self.fd)
                raise
        finally:
            os.close(parent)

    def cli_arguments(self):
        job = self.binding["job"]
        return ["--diagnostic-trace", str(self.directory / "trace.json"),
                "--evaluation-http-journal-fd", str(self.fd),
                "--evaluation-http-binding-sha256", digest(canonical(self.binding)),
                "--evaluation-http-max-attempts", str(job["max_provider_attempts"]),
                "--evaluation-http-token-reserve", str(job["reserve_tokens"])]

    def close(self):
        if self.fd is not None:
            os.close(self.fd)
            self.fd = None

    def subprocess_options(self):
        require(self.fd is not None, "native_http_capability_closed")
        return {"pass_fds": (self.fd,), "umask": 0o077}

    def verify(self):
        """Run only after the child exits and the parent's write capability closes.

        Does not authorize next-job execution: #582 display/source admission is
        still mandatory. A failed verification leaves the reservation burned.
        """
        require(self.fd is None, "native_http_writer_still_open")
        parent = _open_trusted_directory(self.directory)
        try:
            def snapshot(name):
                fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=parent)
                try:
                    from local_store import _read
                    return _read(fd)
                finally:
                    os.close(fd)
            raw = snapshot("http-journal.jsonl")
            trace_raw = snapshot("trace.json")
            require(json.loads(snapshot("binding.json"), object_pairs_hook=_unique) == self.binding,
                    "native_http_binding_file_changed")
        finally:
            os.close(parent)
        trace = json.loads(trace_raw, object_pairs_hook=_unique)
        events = normalize_journal(raw, self.binding, trace)
        exclusive_json(self.directory / "http.json", events)
        http_raw = (self.directory / "http.json").read_bytes()
        self.session.acknowledge_http_capture(
            self.job_id, binding_sha256=digest(canonical(self.binding)),
            journal_sha256=digest(raw), trace_sha256=digest(trace_raw), http_sha256=digest(http_raw),
            tokens=sum(e["total_tokens"] for e in events), attempts=len(events))
        return events
