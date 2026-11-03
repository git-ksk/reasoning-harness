#!/usr/bin/env python3
"""Read-only frozen-byte collector; never authorizes the scheduler or model API."""
import argparse
from pathlib import Path

from contracts import (
    FILES, OBS_ID, PHASES, PLAN_ID, canonical, digest, exclusive_json,
    frozen_request_digest, keyed_digest, positive, read_json, require, safe_id,
    sha_field, strict_keys, unsigned, private_new_directory,
)

OUTPUT_CONTRACTS = frozenset((
    "reason-natural-output-v4", "reason-natural-output-v5", "reason-natural-output-v6",
))
ALLOWED_ERRORS = frozenset((
    "credentials", "transport", "provider_error", "rate_limit", "quota",
    "provider_unavailable", "timeout", "protocol", "unsupported_capability",
    "cancelled", "usage_budget_exceeded", "usage_budget_unmeasurable",
))


def diagnostics(result):
    """Only known source paths; protocol text is NEVER an eligibility signal."""
    candidates = []
    failure = result.get("failure")
    if isinstance(failure, dict):
        require(result.get("status") == "failed", "failure_without_failed_status")
        candidates.append(("initial_candidate", failure))
    investigation = result.get("investigation")
    if isinstance(investigation, dict) and investigation.get("generation_failure") is not None:
        candidates.append((None, investigation["generation_failure"]))
    if result.get("rendering_failure") is not None:
        candidates.append(("final_render", result["rendering_failure"]))
    annotated = []
    for expected_phase, failure in candidates:
        require(isinstance(failure, dict), "malformed_failure")
        kind = failure.get("failure_class")
        require(type(kind) is str and kind, "malformed_failure_class")
        mark = failure.get("model_output_diagnostic")
        if mark is None:
            annotated.append({"failure_class": kind, "phase": expected_phase, "local_decode": False})
            continue
        require(
            type(mark) is dict
            and set(mark) == {"origin", "phase"}
            and mark["origin"] == "local_model_output_decode"
            and mark["phase"] in PHASES
            and kind == "protocol",
            "invalid_typed_model_output_diagnostic",
        )
        require(expected_phase is None or mark["phase"] == expected_phase, "wrong_diagnostic_phase")
        require(
            expected_phase is not None
            or mark["phase"] in {
                "candidate_regeneration", "investigation_plan",
                "investigation_intent", "investigation_action"
            },
            "wrong_investigation_diagnostic_phase",
        )
        annotated.append({"failure_class": kind, "phase": mark["phase"], "local_decode": True})
    return annotated


def audit_trace(trace, events, provider, model):
    """Bind logical CLI calls to individually metered HTTP attempts.

    Provider response usage can aggregate several successful attempts. No
    inference from an attempt count without matching raw provider usage is
    permitted. An independent provider-journal attestor must authenticate
    these normalized HTTP records before any live use.
    """
    require(
        trace.get("contract") == "reason-natural-diagnostic-trace-v1"
        and trace.get("provider") == provider and trace.get("model") == model,
        "trace_provider_or_contract_mismatch",
    )
    ordered = trace["events"]
    require(bool(ordered), "empty_diagnostic_trace")
    pending = {}
    completed = []
    failures = []
    last_attempt = {}
    allowed_event_kinds = frozenset((
        "model_request", "model_response", "model_failure",
        "candidate_state", "grounding_state", "evidence_admission",
        "final_render_state", "finalization_state", "acquisition_coverage",
        "acquisition_coverage_progress", "answer_safety_state",
    ))
    for index, item in enumerate(ordered):
        require(type(item) is dict and item.get("sequence") == index,
                "noncanonical_trace_sequence")
        kind = item.get("kind")
        require(kind in allowed_event_kinds, "unknown_trace_event_kind")
        if kind not in ("model_request", "model_response", "model_failure"):
            continue
        require(
            type(item.get("correlation_id")) is str and item["correlation_id"]
            and type(item.get("attempt")) is int and item["attempt"] > 0
            and item.get("provider") == provider and item.get("model") == model,
            "invalid_trace_model_identity",
        )
        key = (item["correlation_id"], item["attempt"])
        if kind == "model_request":
            # v3 runs one CLI generation at a time; no later request may be
            # physically sent ahead of the first frozen request.
            require(not any(opened is False for opened in pending.values()),
                    "overlapping_trace_model_requests")
            require(
                item["attempt"] == last_attempt.get(item["correlation_id"], 0) + 1,
                "noncanonical_adapter_attempt",
            )
            last_attempt[item["correlation_id"]] = item["attempt"]
            require(key not in pending, "duplicate_trace_request")
            pending[key] = False
        else:
            require(key in pending and pending[key] is False,
                    "unpaired_or_repeated_trace_response")
            pending[key] = True
            completed.append((key, item))
            if kind == "model_failure":
                failures.append(item.get("failure_class"))
    require(pending and all(pending.values()), "pending_model_call")
    grouped = {}
    group_order = []
    prior_key = None
    for http in events:
        key = (http["correlation_id"], http["logical_attempt"])
        require(key == prior_key or key not in grouped,
                "noncontiguous_http_attempt_group")
        if key not in grouped:
            grouped[key] = []
            group_order.append(key)
        grouped[key].append(http)
        prior_key = key
    require([key for key, _ in completed] == group_order,
            "trace_http_logical_order_mismatch")
    for key, item in completed:
        batch = grouped[key]
        if item["kind"] == "model_failure":
            # A provider failure is never eligible to continue even if an
            # independently captured HTTP response reported 2xx earlier.
            continue
        response = item.get("response")
        require(type(response) is dict, "missing_trace_response")
        usage = response.get("usage")
        require(type(usage) is dict, "missing_trace_usage")
        require(
            type(response.get("provider_attempts")) is int
            and response["provider_attempts"] == len(batch),
            "trace_http_attempt_count_mismatch",
        )
        for field in ("input_tokens", "output_tokens", "total_tokens"):
            require(
                type(usage.get(field)) is int
                and usage[field] == sum(http[field] for http in batch),
                "trace_http_token_mismatch",
            )
    return failures

def audit_http(events, trace, result, provider, model):
    """Bind normalized immutable HTTP attempts to the CLI's actually reported usage.

    The normalized HTTP file is an input to independent runtime admission,
    NOT proof of an actual HTTP response on its own.
    """
    require(type(events) is list and events, "missing_http_attempts")
    require(type(trace) is dict and type(trace.get("events")) is list, "missing_trace_events")
    seen = set()
    in_sum = out_sum = total_sum = 0
    for attempt in events:
        strict_keys(
            attempt,
            {"request_id", "correlation_id", "logical_attempt", "provider", "model",
             "http_status", "input_tokens", "output_tokens", "total_tokens"},
            {"request_id", "correlation_id", "logical_attempt", "provider", "model",
             "http_status", "input_tokens", "output_tokens", "total_tokens"},
            "http_attempt",
        )
        safe_id(attempt["correlation_id"], "http_correlation_id")
        positive(attempt["logical_attempt"], "http_logical_attempt")
        request_id = safe_id(attempt["request_id"], "http_request_id")
        require(request_id not in seen, "duplicate_http_request")
        seen.add(request_id)
        require(attempt["provider"] == provider and attempt["model"] == model, "http_identity_mismatch")
        unsigned(attempt["http_status"], "invalid_http_status")
        for key in ("input_tokens", "output_tokens", "total_tokens"):
            unsigned(attempt[key], "unknown_http_usage")
        require(
            attempt["input_tokens"] + attempt["output_tokens"] == attempt["total_tokens"],
            "inconsistent_http_usage",
        )
        in_sum += attempt["input_tokens"]
        out_sum += attempt["output_tokens"]
        total_sum += attempt["total_tokens"]
    bound = {
        "provider_attempts": len(events),
        "input_tokens": in_sum,
        "output_tokens": out_sum,
        "total_tokens": total_sum,
    }
    usage = result.get("usage", {}).get("run") if isinstance(result.get("usage"), dict) else None
    if usage is None:
        partial = result.get("partial_diagnostics")
        usage = partial.get("usage", {}).get("run") if isinstance(partial, dict) and isinstance(partial.get("usage"), dict) else None
    require(type(usage) is dict and usage.get("tracked_from_session_start") is True, "missing_cli_cumulative_usage")
    for field, actual in bound.items():
        require(type(usage.get(field)) is int and usage[field] == actual, "cli_http_usage_mismatch")
    require(type(usage.get("model_calls")) is int and usage["model_calls"] > 0, "missing_logical_calls")
    require(
        # The CLI counts a generation once. Its structured-output fallback
        # invokes generate() again under the SAME diagnostic correlation.
        # Physical adapter/HTTP attempts are accounted separately above.
        usage["model_calls"] == len({
            item["correlation_id"] for item in trace["events"]
            if item.get("kind") == "model_request"
        }),
        "cli_trace_model_calls_mismatch",
    )
    # Traces are retained byte-for-byte. Phase 3 must additionally establish
    # correlation with the provider's audit journal before ANY continuation.
    require(trace.get("completion_status") in ("completed", "partial"), "invalid_trace_completion")
    failures = audit_trace(trace, events, provider, model)
    # Any adapter-reported model failure is operational. In particular a
    # provider capability refusal (even if a later fallback succeeded) cannot
    # be rebranded as a locally failed JSON/DTO decode for continuation.
    bound["trace_operational_failure"] = bool(failures)
    return bound


def collect_row(job, root, provider, model, receipt_sha256, executor_id):
    job_id = safe_id(job["job_id"], "invalid_job_id")
    question = job["question"]
    require(type(question) is str and bool(question.strip()), "missing_frozen_question")
    folder = root / job_id
    require(folder.resolve().parent == root.resolve(), "job_path_escape")
    files = job["files"]
    require(type(files) is dict and set(files) == set(FILES), "incorrect_capture_file_inventory")
    payloads = {}
    for filename in FILES:
        sha_field(files[filename], "invalid_capture_hash")
        path = folder / filename
        require(path.is_file() and not path.is_symlink(), "missing_or_symlinked_capture_file")
        raw = path.read_bytes()
        require(digest(raw) == files[filename], "changed_capture_file")
        payloads[filename] = __import__("json").loads(raw)
    cli = payloads["cli.json"]
    trace = payloads["trace.json"]
    http = payloads["http.json"]
    capture = payloads["capture.json"]
    sources = payloads["sources.json"]
    require(
        cli.get("schema_version") == "reason-cli-output-v1" and cli.get("command") == "ask",
        "unknown_cli_output_contract",
    )
    result = cli.get("result")
    require(type(result) is dict, "missing_cli_result")

    failed = result.get("status") == "failed"
    if failed:
        require(type(result.get("failure")) is dict and "output_contract" not in result, "ambiguous_cli_failure")
        require(result["failure"].get("failure_class") in ALLOWED_ERRORS, "unknown_top_level_failure")
    else:
        require(
            result.get("output_contract") in OUTPUT_CONTRACTS
            and not any(k in result for k in ("status", "failure", "partial_diagnostics")),
            "unknown_or_ambiguous_success_contract",
        )
        config = result.get("configuration")
        require(
            type(config) is dict and config.get("provider") == provider and config.get("model") == model,
            "cli_model_identity_mismatch",
        )
        # The Rust NaturalOutput always reports its canonical exposure policy.
        # A caller must not silently replace or omit the renderer exposure state.
        exposure = result.get("exposed_text")
        require(
            type(exposure) is dict
            and set(exposure) == {"policy_id", "renderer_text_exposed"}
            and exposure["policy_id"] in {
                "harness-canonical-exposed-text-v1",
                "harness-bound-source-context-v1",
                "harness-investigation-coverage-v1",
                "harness-source-and-investigation-coverage-v1",
            },
            "missing_cli_exposure_policy",
        )
        require(
            exposure["renderer_text_exposed"] is False,
            "unexpected_renderer_text_exposure",
        )
    # The externally frozen plan binds *both* the exact case question and
    # complete first model request: substring matching cannot distinguish
    # overlapping prompts or a failed invocation for another case.
    # A self-authored plan is not a signature; real-run trust is owned by
    # #580/#581 and MUST be verified before execution/continuation.
    requests = [
        item for item in trace.get("events", [])
        if type(item) is dict and item.get("kind") == "model_request"
    ]
    require(
        bool(requests) and type(requests[0].get("request")) is dict
        and type(requests[0]["request"].get("task")) is str,
        "invalid_first_model_request",
    )
    sha_field(job["frozen_request_binding_sha256"], "missing_frozen_request_binding")
    # Unlike the redacted diagnostic request, this hash was recorded by the
    # Rust adapter *before* sanitization. Both proofs must be frozen together.
    sha_field(requests[0].get("request_sha256"), "missing_raw_request_digest")
    binding = frozen_request_digest(
        job_id=job_id, case_id=job["case_id"], arm=job["arm"],
        question=question, request=requests[0]["request"],
        request_sha256=requests[0]["request_sha256"],
        provider=provider, model=model, receipt_sha256=receipt_sha256,
    )
    require(binding == job["frozen_request_binding_sha256"],
            "unbound_model_request_hash")
    if not failed:
        require(result.get("task") == question, "cli_case_question_mismatch")
    marks = diagnostics(result)
    accounted = audit_http(http, trace, result, provider, model)
    # The physical CLI's finalization trace is the authoritative output
    # transition. A standalone CLI JSON/capture pair cannot invent a clean
    # or different displayed final result.
    if not failed:
        require(trace.get("completion_status") == "completed",
                "successful_cli_trace_not_completed")
        final_states = [
            item.get("result") for item in trace["events"]
            if item.get("kind") == "finalization_state"
        ]
        require(bool(final_states) and type(final_states[-1]) is dict,
                "missing_finalization_trace")
        trace_final = final_states[-1]
        cli_final = result["finalization"]
        require(
            trace_final.get("status") == cli_final.get("status")
            and (trace_final.get("text") or "") == (cli_final.get("text") or "")
            and trace_final.get("uncovered_propositions")
                == cli_final.get("uncovered_propositions"),
            "trace_finalization_mismatch",
        )
    else:
        require(trace.get("completion_status") == "partial",
                "failed_cli_trace_not_partial")
    require(type(capture) is dict and type(sources) is dict, "invalid_capture_or_sources")
    require(
        set(capture) == {"display_artifacts", "display_citations", "admitted_source_ids"},
        "invalid_display_capture_contract",
    )
    artifacts = capture.get("display_artifacts")
    require(
        type(artifacts) is list and len(artifacts) == 1
        and type(artifacts[0]) is dict and artifacts[0].get("id") == "answer"
        and set(artifacts[0]) == {"id", "kind", "text"}
        and artifacts[0].get("kind") == "final_answer"
        and type(artifacts[0].get("text")) is str,
        "invalid_display_capture_contract",
    )
    display = artifacts[0]["text"]
    if failed:
        require(display == "" and not capture.get("display_citations"), "error_promoted_to_answer")
    else:
        final = result.get("finalization")
        require(
            type(final) is dict and (final.get("text") or "") == display
            and final.get("status") in {
                "grounded_answer", "qualified_partial_answer", "requires_verification",
                "unresolved", "rejected",
            },
            "display_or_terminal_mismatch",
        )
    citations = capture.get("display_citations")
    admitted = capture.get("admitted_source_ids")
    require(type(citations) is list and type(admitted) is list, "missing_citation_or_admission_inventory")
    require(all(type(s) is str and s in sources for s in admitted) and len(set(admitted)) == len(admitted), "unbound_source_identity")
    for source_id, source_record in sources.items():
        require(
            type(source_id) is str and type(source_record) is dict
            and type(source_record.get("text")) is str and bool(source_record["text"])
            and type(source_record.get("source_version")) is str
            and bool(source_record["source_version"]),
            "invalid_full_source_record",
        )
    citation_ids = set()
    display_bytes = display.encode("utf-8")
    for cite in citations:
        require(type(cite) is dict, "invalid_citation")
        require(
            set(cite) == {"id", "source_id", "source_version", "start_byte", "end_byte"},
            "invalid_citation_contract",
        )
        source = cite.get("source_id")
        require(source in admitted and source in sources, "citation_outside_admitted_sources")
        require(
            type(cite.get("id")) is str and cite["id"] and cite["id"] not in citation_ids
            and cite.get("source_version") == sources[source]["source_version"],
            "citation_identity_or_version_mismatch",
        )
        citation_ids.add(cite["id"])
        require(
            type(cite.get("start_byte")) is int and type(cite.get("end_byte")) is int
            and 0 <= cite["start_byte"] < cite["end_byte"] <= len(display_bytes),
            "citation_outside_actual_display",
        )
        try:
            display_bytes[:cite["start_byte"]].decode("utf-8")
            display_bytes[:cite["end_byte"]].decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("citation_invalid_utf8_boundary") from exc
    unsafe = any(not 200 <= item["http_status"] < 300 for item in http)
    uncertain = accounted.pop("trace_operational_failure") or any(
        x["failure_class"] != "protocol" or not x["local_decode"] for x in marks
    )
    status = (
        "operational_stop" if unsafe or uncertain
        else "local_decode_observed" if any(x["local_decode"] for x in marks)
        else "normal_observed"
    )
    # A recovered/canonical answer must still be semantically annotated.
    # A local typed marker is NOT an automatic invalid or a continuation permit.
    return {
        "contract": OBS_ID,
        "job_id": job_id,
        "case_id": job["case_id"],
        "question": question,
        "frozen_request_binding_sha256": binding,
        "arm": job["arm"],
        "provider": provider,
        "model": model,
        "receipt_sha256": receipt_sha256,
        "executor_id": executor_id,
        "files": files,
        "observation_binding_sha256": keyed_digest(files),
        "display_artifacts": artifacts,
        "display_citations": citations,
        "admitted_source_ids": admitted,
        "sources": sources,
        "terminal_status": "failed" if failed else result["finalization"]["status"],
        "diagnostics": marks,
        "accounted_usage": accounted,
        "measurement_state": status,
        "runtime_admission": "NOT_EVALUATED_NO_CONTINUATION_AUTHORITY",
    }


def collect(plan_path, lane_path, destination):
    manifest = read_json(plan_path)
    strict_keys(
        manifest, {"contract", "provider", "model", "executor_id", "receipt_sha256", "rows"},
        {"contract", "provider", "model", "executor_id", "receipt_sha256", "rows"}, "capture_manifest",
    )
    require(manifest["contract"] == PLAN_ID, "unknown_capture_manifest")
    sha_field(manifest["receipt_sha256"], "missing_receipt_hash")
    require(type(manifest["provider"]) is str and type(manifest["model"]) is str and manifest["provider"] and manifest["model"], "missing_provider_identity")
    safe_id(manifest["executor_id"], "missing_executor_identity")
    rows = manifest["rows"]
    require(type(rows) is list and rows, "empty_capture_plan")
    seen = set()
    seen_jobs = set()
    # Validate every manifest identity before reading captures or creating
    # private output files: two cases must never alias the same job directory.
    for job in rows:
        strict_keys(job, {"job_id", "case_id", "arm", "question", "frozen_request_binding_sha256", "files"},
                    {"job_id", "case_id", "arm", "question", "frozen_request_binding_sha256", "files"}, "plan_job")
        job_id = safe_id(job["job_id"], "invalid_job_id")
        require(job_id not in seen_jobs, "duplicate_job_id")
        seen_jobs.add(job_id)
        safe_id(job["case_id"], "bad_case")
        require(job["arm"] in ("baseline", "candidate"), "bad_arm")
        identity = (job["case_id"], job["arm"])
        require(identity not in seen, "duplicate_case_arm")
        seen.add(identity)
        sha_field(job["frozen_request_binding_sha256"], "missing_frozen_request_binding")
    captured = []
    for job in rows:
        captured.append(collect_row(job, Path(lane_path), manifest["provider"], manifest["model"], manifest["receipt_sha256"], manifest["executor_id"]))
    destination = Path(destination)
    private_new_directory(destination)  # Exclusive 0700, no reused measurements
    for row in captured:
        exclusive_json(destination / (row["job_id"] + ".json"), row)
    exclusive_json(destination / "index.json", {
        "contract": OBS_ID, "capture_manifest_sha256": digest(Path(plan_path).read_bytes()),
        "receipt_sha256": manifest["receipt_sha256"],
        "executor_id": manifest["executor_id"],
        "rows": [
            {"job_id": row["job_id"], "case_id": row["case_id"], "arm": row["arm"],
             "sha256": digest((destination / (row["job_id"] + ".json")).read_bytes())}
            for row in captured
        ],
        "continuation_authorized": False,
        "first_score": None,
    })
    return captured


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--capture-manifest", type=Path, required=True)
    parser.add_argument("--lane", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    rows = collect(args.capture_manifest, args.lane, args.output)
    print(__import__("json").dumps({"captured_rows": len(rows), "scheduler_authorized": False}))


if __name__ == "__main__":
    main()
