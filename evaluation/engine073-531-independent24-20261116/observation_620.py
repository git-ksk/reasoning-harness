"""#620 offline evidence joins, using native capture and pre-advance verification.

All expected context is supplied by the trusted coordinator, not by an observed
artifact. The return value is diagnostic data, not a formal capability.
"""
import json
import math
from auth620 import decode, encode, fields, sha, require, match, SHA
from native_http import normalize_journal
from collect import audit_http, OUTPUT_CONTRACTS
from native_admission import CATALOG, ADMISSION, source_catalog, capture_display, verify_pre_advance
from first_claim_620 import FIXED_CONDITIONS


def _unique(pairs):
    data = {}
    for k, v in pairs:
        require(k not in data, "duplicate_observation_key")
        data[k] = v
    return data


def _no_constant(_):
    raise ValueError("nonfinite_observation")


def _finite(raw):
    value = float(raw)
    require(math.isfinite(value), "nonfinite_observation")
    return value


def _legacy_json(raw):
    # Native CLI/trace wire predates RH-AUTH/1 and can contain finite floats.
    require(type(raw) is bytes and 0 < len(raw) <= 16 * 1024 * 1024, "native_json_size")
    try:
        return json.loads(raw.decode("utf-8", errors="strict"), object_pairs_hook=_unique,
                          parse_constant=_no_constant, parse_float=_finite)
    except (UnicodeError, RecursionError) as error:
        raise ValueError("invalid_native_json") from error


def verify_observation(*, registry, now, evaluation_id, attempt_id, slot, binding_raw,
                       expected_binding_raw, trace_raw, journal_raw, stdout, stderr,
                       sources_raw, source_pin, final_bytes, bundle_pin, authority_pin,
                       observation_envelope, safety_envelope, legacy_policy_raw,
                       legacy_policy_pin, legacy_attestation, executor_id, evaluator_id,
                       expected_parents):
    """Verify one complete first-observation graph; no fallback digest on failure."""
    for v in (attempt_id, source_pin, bundle_pin, authority_pin, legacy_policy_pin):
        match(v, SHA, "observation_pin")
    require(binding_raw == expected_binding_raw and sha(sources_raw) == source_pin
            and sha(legacy_policy_raw) == legacy_policy_pin, "trusted_capture_context_mismatch")
    binding, trace = _legacy_json(binding_raw), _legacy_json(trace_raw)
    require(binding["provider"] == "mistral"
            and binding["model"] == FIXED_CONDITIONS["model"]
            and binding["job"]["job_id"] == slot[1] + "-" + slot[2]
            and type(binding["job"]["max_provider_attempts"]) is int
            and binding["job"]["max_provider_attempts"] == 12
            and type(binding["job"]["reserve_tokens"]) is int
            and binding["job"]["reserve_tokens"] == 28096
            and type(binding["job"].get("http_limits")) is dict
            and binding["job"]["http_limits"].get("max_generations") == 12
            and binding["legacy_receipt_sha256"] == authority_pin
            and trace.get("completion_status") == "completed", "frozen_conditions")
    normalized = normalize_journal(journal_raw, binding, trace)
    catalog = source_catalog(sources_raw)
    cli = _legacy_json(stdout)
    require(cli.get("schema_version") == "reason-cli-output-v1" and cli.get("command") == "ask",
            "native_cli_output_contract")
    result = cli.get("result")
    require(type(result) is dict and result.get("output_contract") in OUTPUT_CONTRACTS
            and not any(k in result for k in ("status", "failure", "partial_diagnostics"))
            and result.get("task") == binding["question"]
            and type(result.get("configuration")) is dict
            and result["configuration"].get("provider") == binding["provider"]
            and result["configuration"].get("model") == binding["model"], "native_cli_context")
    exposure = result.get("exposed_text")
    require(type(exposure) is dict and set(exposure) == {"policy_id", "renderer_text_exposed"}
            and exposure["policy_id"] in {"harness-canonical-exposed-text-v1",
                "harness-bound-source-context-v1", "harness-investigation-coverage-v1",
                "harness-source-and-investigation-coverage-v1"}
            and exposure["renderer_text_exposed"] is False, "native_cli_exposure")
    accounted = audit_http(normalized, trace, result, binding["provider"], binding["model"])
    require(not accounted["trace_operational_failure"], "native_trace_failure")
    finals = [x.get("result") for x in trace["events"] if x.get("kind") == "finalization_state"]
    final = result.get("finalization")
    require(type(final) is dict and bool(finals) and type(finals[-1]) is dict
            and final.get("status") in {"grounded_answer", "qualified_partial_answer",
                        "requires_verification", "unresolved", "abstain", "rejected"}
            and type(final.get("text")) is str
            and type(final.get("uncovered_propositions")) is list
            and all(finals[-1].get(k) == final[k] for k in
                    ("status", "text", "uncovered_propositions")), "native_terminal_trace")
    captured = capture_display(cli, catalog)
    require(type(final_bytes) is bytes and final["text"].encode("utf-8") == final_bytes
            and captured["display_artifacts"][0]["text"].encode("utf-8") == final_bytes,
            "final_display_mismatch")
    captured_raw = encode(captured)
    payload = {"attempt_id": attempt_id, "slot": list(slot), "status": "complete",
               "binding_sha256": sha(binding_raw), "trace_sha256": sha(trace_raw),
               "journal_sha256": sha(journal_raw), "stdout_sha256": sha(stdout),
               "stderr_sha256": sha(stderr), "sources_sha256": source_pin,
               "capture_sha256": sha(captured_raw), "final_length": len(final_bytes),
               "final_sha256": sha(final_bytes), "calls": len(normalized),
               "tokens": sum(x["total_tokens"] for x in normalized),
               "native_bundle_sha256": bundle_pin, "authority_receipt_sha256": authority_pin}
    payload_raw = encode(payload)
    obs_receipt = registry.verify(observation_envelope, payload_raw, artifact_type="terminal",
                                  artifact_id="terminal-" + attempt_id, role="capture",
                                  parents=expected_parents, now=now)
    expected = {"contract": ADMISSION, "evaluation_id": evaluation_id,
                "job_id": binding["job"]["job_id"], "executor_id": executor_id,
                "evaluator_id": evaluator_id, "authority_receipt_sha256": authority_pin,
                "native_bundle_sha256": bundle_pin, "http_binding_sha256": sha(binding_raw),
                "packet_sha256": sha(captured_raw), "observation_sha256": obs_receipt,
                "sources_sha256": source_pin, "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)}
    advance = verify_pre_advance(policy_bytes=legacy_policy_raw, attestation=legacy_attestation,
                                 expected=expected, stdout=stdout, stderr=stderr, sources=catalog["sources"])
    safety_raw = encode({"attempt_id": attempt_id, "terminal_receipt": obs_receipt,
                         "legacy_attestation": sha(legacy_attestation),
                         "legacy_authentication": sha(advance.authentication_bytes)})
    admission = registry.verify(safety_envelope, safety_raw, artifact_type="admission",
                                artifact_id="admission-" + attempt_id, role="safety_reviewer",
                                parents=[obs_receipt], now=now)
    return {**payload, "observation_sha256": obs_receipt, "admission_sha256": admission,
            "display": final_bytes.decode("utf-8"), "source_texts": [
                catalog["sources"][k]["text"] for k in sorted(catalog["sources"])],
            "citation_targets": sorted({x["id"].rsplit("-", 1)[0] for x in captured["display_citations"]})}


def verify_partial_capture(*, registry, now, attempt_id, slot, status, stdout, stderr,
                           journal_raw, trace_raw, sources_raw, source_pin, final_bytes,
                           envelope, parents):
    """Preserve signed incomplete capture as diagnostic evidence only.

    Never infer an output, an omission, metered usage, a confirmed semantic
    violation or score eligibility from a missing/partial terminal record.
    """
    match(attempt_id, SHA, "partial_attempt")
    match(source_pin, SHA, "partial_source")
    require(type(status) is str and status in ("timeout","cancelled","incomplete","error","unknown","ambiguous"),
            "partial_status")
    require(type(slot) is tuple and len(slot) == 3 and type(slot[0]) is int
            and 1 <= slot[0] <= 8 and type(slot[1]) is str and slot[2] in ("baseline","candidate"),
            "partial_slot")
    require(sha(sources_raw) == source_pin, "partial_sources")
    source_catalog(sources_raw)
    require(final_bytes is None or type(final_bytes) is bytes, "partial_final")
    # Partial bytes are evidence, not a verified final display. A missing
    # terminal stays null; no artificial digest is installed.
    payload = {"attempt_id":attempt_id,"slot":list(slot),"status":status,
               "stdout_sha256":sha(stdout),"stderr_sha256":sha(stderr),
               "journal_sha256":sha(journal_raw),"trace_sha256":sha(trace_raw),
               "sources_sha256":source_pin,
               "partial_display_sha256":None if final_bytes is None else sha(final_bytes),
               "final_sha256":None,"calls":None,"tokens":None,
               "formal_eligible":False}
    receipt=registry.verify(envelope,encode(payload),artifact_type="raw_evidence",
        artifact_id="partial-"+attempt_id,role="capture",parents=parents,now=now)
    return {**payload,"observation_sha256":receipt,"admission_sha256":None,
            "evidence_eligibility":"UNRATIFIED","diagnostic_only":True}
