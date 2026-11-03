"""Actual stream/source packet and enrolled pre-advance safety decision (#582).

The signature authenticates the registered reviewer's decision. It does not
automatically discover facts or certify semantic truth. All displayed bytes,
including JSON side fields and stderr, remain in the reviewed packet.
"""
import json
from dataclasses import dataclass

from contracts import canonical, digest, require, safe_id, sha_field, strict_keys
from review_auth import validate_policy, ReviewAuthentication, _hex

CATALOG = "reason-cli-normal-v3-source-catalog-v1"
ADMISSION = "reason-cli-normal-v3-pre-advance-review-v1"
DOMAIN = (ADMISSION + "\0").encode()
EXPECTED = {"contract", "evaluation_id", "job_id", "executor_id", "evaluator_id",
            "authority_receipt_sha256", "native_bundle_sha256", "http_binding_sha256", "packet_sha256",
            "observation_sha256", "sources_sha256", "stdout_sha256", "stderr_sha256"}


@dataclass(frozen=True)
class VerifiedAdvance:
    """Trusted-code capability, never constructed from CLI/receipt JSON.

    Like the local store roles, this prevents integration mistakes; it is not
    protection against malicious code executing as the trusted OS owner.
    """
    expected_bytes: bytes
    authentication_bytes: bytes

    @property
    def authentication(self): return json.loads(self.authentication_bytes)


def source_catalog(raw):
    data = json.loads(raw)
    fields = {"contract", "sources", "evidence_bindings", "context_files"}
    strict_keys(data, fields, fields - {"context_files"}, "native_source_catalog")
    require(data["contract"] == CATALOG and canonical(data) == raw, "invalid_native_source_catalog")
    require(type(data["sources"]) is dict and type(data["evidence_bindings"]) is dict,
            "invalid_native_source_inventory")
    for source_id, source in data["sources"].items():
        safe_id(source_id, "invalid_native_source_id")
        strict_keys(source, {"text", "source_version"}, {"text", "source_version"}, "native_full_source")
        require(type(source["text"]) is str and bool(source["text"])
                and type(source["source_version"]) is str and bool(source["source_version"]),
                "missing_native_full_source")
    for evidence_id, binding in data["evidence_bindings"].items():
        safe_id(evidence_id, "invalid_native_evidence_id")
        fields = {"evidence_sha256", "source_id", "source_version"}
        strict_keys(binding, fields, fields, "native_evidence_binding")
        sha_field(binding["evidence_sha256"], "invalid_native_evidence_hash")
        require(binding["source_id"] in data["sources"] and binding["source_version"] ==
                data["sources"][binding["source_id"]]["source_version"], "native_source_version_mismatch")
    for path, binding in data.get("context_files", {}).items():
        require(type(path) is str and path.startswith("/"), "invalid_native_context_path")
        strict_keys(binding, {"source_id", "sha256"}, {"source_id", "sha256"}, "native_context_file")
        require(binding["source_id"] in data["sources"] and binding["sha256"] ==
                digest(data["sources"][binding["source_id"]]["text"].encode()), "native_context_full_text_missing")
    return data


def capture_display(cli, catalog):
    result = cli.get("result")
    require(type(result) is dict, "native_cli_result_missing")
    display = "" if result.get("status") == "failed" else result.get("finalization", {}).get("text") or ""
    require(type(display) is str, "invalid_native_display")
    # Read actual admitted artifact evidence, never candidate-authored evidence.
    evidence = {}
    def walk(value):
        if isinstance(value, dict):
            if type(value.get("artifact")) is dict:
                for item in value["artifact"].get("evidence", []):
                    require(type(item) is dict and type(item.get("id")) is str, "invalid_native_evidence")
                    # Preserve the native struct's field order, matching
                    # serde_json::to_vec(Evidence), not a reordered object.
                    raw = json.dumps(item, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode()
                    observed = digest(raw)
                    require(item["id"] not in evidence or evidence[item["id"]] == observed,
                            "native_evidence_identity_changed")
                    evidence[item["id"]] = observed
            for child in value.values(): walk(child)
        elif isinstance(value, list):
            for child in value: walk(child)
    walk(result)
    admitted = set()
    for evidence_id, observed in evidence.items():
        binding = catalog["evidence_bindings"].get(evidence_id)
        require(binding is not None and binding["evidence_sha256"] == observed,
                "native_unbound_or_changed_evidence")
        admitted.add(binding["source_id"])
    citations = []
    context = result.get("answer_context")
    if context is not None:
        require(type(context) is dict and type(context.get("citations")) is list,
                "invalid_native_answer_context")
        for ordinal, cite in enumerate(context["citations"]):
            binding = catalog["evidence_bindings"].get(cite.get("evidence_id"))
            require(binding is not None and evidence.get(cite["evidence_id"]) == cite.get("evidence_sha256")
                    and all(cite.get(k) == binding[k] for k in ("source_id", "source_version", "evidence_sha256")),
                    "native_citation_source_mismatch")
            source = catalog["sources"][binding["source_id"]]
            require(type(cite.get("quote")) is str and cite["quote"] in source["text"],
                    "native_quote_outside_full_source")
            for name in ("fact", "quote"):
                start, end = cite[name + "_start_byte"], cite[name + "_end_byte"]
                raw = display.encode()
                require(type(start) is int and type(end) is int and 0 <= start < end <= len(raw),
                        "native_citation_outside_display")
                raw[:start].decode("utf-8"); raw[:end].decode("utf-8")
                if name == "quote":
                    require(cite.get("quote_display_encoding") == "json_string"
                            and json.loads(raw[start:end]) == cite["quote"]
                            and cite.get("quote_sha256") == digest(cite["quote"].encode()),
                            "native_actual_quote_bytes_mismatch")
                citations.append({"id": f"native-{ordinal}-{name}", "source_id": binding["source_id"],
                                  "source_version": binding["source_version"],
                                  "start_byte": cite[name + "_start_byte"], "end_byte": cite[name + "_end_byte"]})
    final = result.get("finalization", {})
    if final.get("status") in ("grounded_answer", "qualified_partial_answer") and display:
        require(context is not None and citations, "native_source_bound_display_required")
    return {"display_artifacts": [{"id": "answer", "kind": "final_answer", "text": display}],
            "display_citations": citations, "admitted_source_ids": sorted(admitted)}


def verify_pre_advance(*, policy_bytes, attestation, expected, stdout, stderr, sources):
    require(type(expected) is dict and set(expected) == EXPECTED and expected["contract"] == ADMISSION,
            "missing_pre_advance_execution_bindings")
    for name in EXPECTED - {"contract", "evaluation_id", "job_id", "executor_id", "evaluator_id"}:
        sha_field(expected[name], "invalid_pre_advance_execution_hash")
    require(type(stdout) is bytes and type(stderr) is bytes and digest(stdout) == expected["stdout_sha256"]
            and digest(stderr) == expected["stderr_sha256"], "pre_advance_actual_stream_mismatch")
    aliases, keys = validate_policy(policy_bytes)
    executor, evaluator = aliases.get(expected["executor_id"]), aliases.get(expected["evaluator_id"])
    require(executor is not None and evaluator is not None
            and executor["subject_id"] != evaluator["subject_id"]
            and not set(executor["conflict_groups"]) & set(evaluator["conflict_groups"]),
            "pre_advance_reviewer_conflict")
    require(type(attestation) is bytes and len(attestation) <= 4 * 1024 * 1024,
            "invalid_pre_advance_attestation")
    envelope = json.loads(attestation)
    require(canonical(envelope) == attestation, "noncanonical_pre_advance_attestation")
    strict_keys(envelope, {"contract", "algorithm", "key_id", "payload", "signature_hex"},
                {"contract", "algorithm", "key_id", "payload", "signature_hex"}, "pre_advance_signature")
    key = keys.get(envelope["key_id"])
    require(envelope["contract"] == ADMISSION and envelope["algorithm"] == "Ed25519"
            and key is not None and not key["revoked"] and key["subject_id"] == evaluator["subject_id"],
            "unknown_or_revoked_pre_advance_key")
    payload = envelope["payload"]
    fields = set(expected) | {"policy_sha256", "decision", "full_sources_reviewed", "segments", "notes"}
    strict_keys(payload, fields, fields, "pre_advance_payload")
    require(all(payload[k] == v for k, v in expected.items())
            and payload["policy_sha256"] == digest(policy_bytes), "stale_or_foreign_pre_advance_review")
    ReviewAuthentication(policy_bytes, b"")._verify_signature(
        _hex(key["public_key_hex"], 32, "invalid_ed25519_public_key"),
        _hex(envelope["signature_hex"], 64, "invalid_ed25519_signature"), DOMAIN + canonical(payload))
    require(payload["decision"] == "safe" and payload["full_sources_reviewed"] is True,
            "pre_advance_unsafe_or_unknown")
    require(type(payload["notes"]) is str and bool(payload["notes"].strip()), "pre_advance_notes_required")
    segments = payload["segments"]
    require(type(segments) is dict and set(segments) == {"stdout", "stderr"}, "pre_advance_stream_inventory")
    require(type(stdout) is bytes and bool(stdout), "empty_native_stdout")
    for name, raw in (("stdout", stdout), ("stderr", stderr)):
        raw.decode("utf-8")
        require(type(segments[name]) is list, "pre_advance_segments_required")
        cursor = 0
        for segment in segments[name]:
            fields = {"start_byte", "end_byte", "label", "source_ids"}
            strict_keys(segment, fields, fields, "pre_advance_segment")
            start, end = segment["start_byte"], segment["end_byte"]
            require(type(start) is int and type(end) is int and start == cursor and start < end <= len(raw),
                    "pre_advance_display_gap_or_overlap")
            raw[:start].decode("utf-8"); raw[:end].decode("utf-8")
            require(segment["label"] in ("fact", "nonfact"), "pre_advance_unsafe_segment")
            ids = segment["source_ids"]
            require(type(ids) is list and all(type(s) is str for s in ids) and len(set(ids)) == len(ids) and all(s in sources for s in ids)
                    and (bool(ids) if segment["label"] == "fact" else not ids), "pre_advance_unbound_fact")
            cursor = end
        require(cursor == len(raw), "pre_advance_display_incomplete")
    authentication = {"contract": ADMISSION, "attestation_sha256": digest(attestation),
            "policy_sha256": digest(policy_bytes), "key_id": key["key_id"],
            "subject_id": evaluator["subject_id"], "semantic_truth_certified": False}
    return VerifiedAdvance(canonical(expected), canonical(authentication))
