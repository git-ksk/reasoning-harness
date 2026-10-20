"""Integrity adapter for the saved, canonical ordinary-CLI development capture.

No Engine/provider/scorer execution or semantic support inference. This subset
adapter is not a complete real-document M0 or a generic natural-prose parser.
"""
from collections import defaultdict
import hashlib
from html.parser import HTMLParser
import json
from pathlib import Path
import re


def sha(data):
    return hashlib.sha256(data).hexdigest()


def strict_json(data):
    def unique(pairs):
        out = {}
        for key, value in pairs:
            if key in out:
                raise ValueError("duplicate_json_key")
            out[key] = value
        return out
    return json.loads(data, object_pairs_hook=unique,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError("nonfinite_json")))


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def indexed(items, name):
    require(isinstance(items, list), "invalid_" + name)
    out = {}
    for item in items:
        require(isinstance(item, dict) and isinstance(item.get("id"), str) and item["id"], "invalid_" + name + "_id")
        require(item["id"] not in out, "duplicate_" + name + "_id")
        out[item["id"]] = item
    return out


def proposition(value):
    require(isinstance(value, dict) and set(value) == {"key", "value"}
            and all(isinstance(v, str) and v for v in value.values()), "invalid_proposition")
    return value["key"], value["value"]


def verify_model_events(trace, configuration):
    require(trace.get("contract") == "reason-natural-diagnostic-trace-v1", "unsupported_trace_contract")
    for key in ("provider", "model"):
        require(trace.get(key) == configuration.get(key), "misbound_trace_" + key)
    events = trace.get("events")
    require(isinstance(events, list) and events, "missing_trace_events")
    requests, responses = {}, {}
    for number, event in enumerate(events):
        require(type(event.get("sequence")) is int and event["sequence"] == number, "invalid_trace_sequence")
        kind = event.get("kind")
        require(kind != "model_failure", "model_failure_not_semantic_abstention")
        if kind not in {"model_request", "model_response"}:
            continue
        require(type(event.get("attempt")) is int and event["attempt"] > 0, "invalid_model_attempt")
        cid = event.get("correlation_id")
        require(isinstance(cid, str) and cid, "missing_model_correlation")
        key = cid, event["attempt"]
        for field in ("provider", "model"):
            require(event.get(field) == trace.get(field), "misbound_model_" + field)
        target = requests if kind == "model_request" else responses
        require(key not in target, "duplicate_model_request_or_response")
        if kind == "model_request":
            request = event.get("request", {})
            require(isinstance(request.get("task"), str) and request["task"]
                    and type(request.get("max_tokens")) is int and request["max_tokens"] > 0,
                    "invalid_model_request")
        else:
            require(key in requests, "model_response_before_request")
            request = requests[key]
            require(all(event.get(k) == request.get(k) for k in ("phase", "round")), "misbound_model_phase")
            require(isinstance(event.get("response"), dict), "missing_model_response")
        target[key] = event
    require(requests and requests.keys() == responses.keys(), "incomplete_model_trace")
    return events, len(requests)


def validate_capture(output, trace, journal, exit_record, config, corpus, corpus_sha256):
    """Bind a legacy canonical grounded display to captured source/fact receipts.

    Supported capture mode is host_curated_facts with one unambiguous journal
    request per admission identity. Unknown formats/ambiguity fail explicitly;
    no output is assigned a true-negative label or a semantic score here.
    """
    require(type(exit_record.get("returncode")) is int and exit_record["returncode"] == 0, "process_not_successful")
    require(output.get("schema_version") == "reason-cli-output-v1" and output.get("command") == "ask", "unsupported_output_contract")
    result = output.get("result", {})
    require(result.get("output_contract") == "reason-natural-output-v4", "unsupported_natural_contract")
    events, model_calls = verify_model_events(trace, result.get("configuration", {}))
    final = result.get("finalization", {})
    require(final.get("status") == "grounded_answer" and isinstance(final.get("text"), str), "unsupported_terminal_display")
    require(result.get("final_outcome", {}).get("verdict") == "accept", "grounded_display_without_accept")
    terminal = [e for e in events if e.get("kind") == "finalization_state"]
    grounded = [e for e in events if e.get("kind") == "grounding_state"]
    require(terminal and terminal[-1].get("result") == final, "terminal_display_mismatch")
    require(grounded and grounded[-1].get("outcome") == result.get("final_outcome"), "terminal_artifact_mismatch")
    require(grounded[-1]["sequence"] < terminal[-1]["sequence"], "invalid_terminal_order")
    artifact = result["final_outcome"]["artifact"]
    evidence = indexed(artifact.get("evidence"), "evidence")
    claims = indexed(artifact.get("claims"), "claim")
    receipts = indexed(artifact.get("verification_receipts"), "receipt")
    for receipt in receipts.values():
        cid = receipt.get("claim_id")
        require(cid in claims and receipt.get("proposition") == claims[cid].get("proposition"), "orphan_or_misbound_receipt")
    require(isinstance(journal, list) and journal, "acquisition_journal_missing")
    case_ids = {entry.get("case") for entry in journal}
    require(len(case_ids) == 1, "mixed_acquisition_cases")
    cases = [case for case in corpus["cases"] if case["id"] in case_ids]
    require(len(cases) == 1 and result.get("task") == cases[0]["question"], "acquisition_question_mismatch")
    case = cases[0]
    task_scope = {k: {"kind": "values", "values": [v]} for k, v in case["required_scope"].items()}
    acquired_by_request = {}
    for entry in journal:
        require(entry.get("corpus_sha256") == corpus_sha256, "acquisition_corpus_mismatch")
        request = entry.get("request", {})
        require(request.get("schema_version") == "reason-investigation-external-resolver-request-v1", "unsupported_resolver_request")
        request = request.get("request", {})
        key = request.get("id")
        require(isinstance(key, str) and key and key not in acquired_by_request, "ambiguous_resolver_request_identity")
        response = entry.get("response", {})
        require(response.get("schema_version") == "reason-external-resolver-response-v1", "unsupported_resolver_response")
        contribution = response.get("contribution", {})
        require(contribution.get("kind") == "acquired_evidence", "unsupported_resolver_contribution")
        acquired_by_request[key] = (request, indexed(contribution.get("evidence"), "acquired_evidence"))
    admissions, joined_sources = {}, {}
    capabilities = {c["id"]: c for c in config["resolution"]["investigation"]["capabilities"]}
    require(len(capabilities) == len(config["resolution"]["investigation"]["capabilities"]), "duplicate_capability_id")
    for event in events:
        if event.get("kind") != "evidence_admission":
            continue
        require(event["sequence"] < grounded[-1]["sequence"], "admission_after_final_artifact")
        target, capability = event.get("target_id"), event.get("capability_id")
        key = f"investigation:{target}:{capability}"
        require(key in acquired_by_request and capability in capabilities, "admission_without_acquisition")
        request, acquired = acquired_by_request[key]
        require(request.get("target", {}).get("target_id") == target, "acquisition_wrong_target")
        require(request.get("target", {}).get("expected_fact_key") == case["lookup_key"], "acquisition_wrong_fact_key")
        policy = capabilities[capability]["admission"]
        require(policy.get("required_scope") == task_scope, "task_admission_scope_mismatch")
        ids = event.get("admitted_evidence_ids")
        require(isinstance(ids, list) and len(set(ids)) == len(ids), "invalid_admitted_ids")
        keys = set()
        for eid in ids:
            require(eid not in admissions and eid in acquired and eid in evidence, "duplicate_or_unobserved_admission")
            raw, admitted = acquired[eid], evidence[eid]
            require(all(admitted.get(k) == raw.get(k) for k in ("id", "source", "observation", "facts")), "admitted_evidence_changed")
            source = corpus["sources"].get(raw["source"])
            require(source and raw["source"] in case["source_ids"]
                    and not source["synthetic_mutation"], "unsupported_or_mutated_source")
            observed = strict_json(raw["observation"])
            for field in ("document_id", "url", "locator", "source_version", "origin_group", "scope", "quote", "context_summary", "synthetic_mutation"):
                require(observed.get(field) == source[field], "source_binding_mismatch:" + field)
            require(sha(source["quote"].encode()) == source["quote_sha256"], "source_quote_hash_mismatch")
            require(raw["facts"] == source["reference_facts"], "curated_facts_changed")
            owner = policy["sources"].get(raw["source"])
            require(owner is not None, "source_not_in_host_admission_policy")
            ranks = policy["authority_ranks"]
            rank, floor = ranks.get(owner["authority_class"]), ranks.get(policy["minimum_authority_class"])
            require(type(rank) is int and type(floor) is int and rank >= floor, "host_authority_below_minimum")
            acq = raw["acquisition_metadata"]
            require(type(acq.get("observed_at_unix_seconds")) is int
                    and type(acq.get("retrieved_at_unix_seconds")) is int
                    and acq["retrieved_at_unix_seconds"] == source["retrieved_at_unix_seconds"], "invalid_acquisition_timestamp")
            expected_scope = {k: {"kind": "values", "values": [v]} for k, v in source["scope"].items()}
            require(acq["scope"] == expected_scope and acq["observed_at_unix_seconds"] == source["retrieved_at_unix_seconds"], "acquired_scope_or_time_changed")
            require(owner.get("scope") == expected_scope, "host_source_scope_mismatch")
            require(type(owner.get("max_age_seconds")) is int and owner["max_age_seconds"] > 0, "unsupported_temporal_policy")
            # Authority comes from host configuration, never the source's claim.
            expected_metadata = {"scope": expected_scope, "provenance_class": owner["authority_class"],
                                 "temporal": {"effective_from_unix_seconds": acq["observed_at_unix_seconds"],
                                              "effective_until_unix_seconds": acq["observed_at_unix_seconds"] + owner["max_age_seconds"]}}
            require(admitted.get("metadata") == expected_metadata, "admission_metadata_mismatch")
            require(all(type(v) is int for v in admitted["metadata"]["temporal"].values()), "invalid_admitted_timestamp")
            now = policy["evaluation_time_unix_seconds"]
            require(type(now) is int and acq["observed_at_unix_seconds"] <= now
                    < acq["observed_at_unix_seconds"] + owner["max_age_seconds"], "evidence_not_effective_at_evaluation")
            require(all(expected_scope.get(k) == v for k, v in policy["required_scope"].items()), "admitted_scope_not_applicable")
            admissions[eid] = event["sequence"]
            joined_sources[eid] = {"evidence_id": eid, "source_id": raw["source"], **source}
            keys.update(raw["facts"])
        require(set(event.get("fact_keys", [])) == keys, "admission_fact_keys_mismatch")
    require(set(evidence) == set(admissions), "artifact_evidence_not_acquired_and_admitted")
    by_proposition = defaultdict(list)
    for cid, claim in claims.items():
        if claim.get("state") not in {"known", "supported"}:
            continue
        prop = proposition(claim.get("proposition"))
        ids = claim.get("evidence_ids")
        require(isinstance(ids, list) and ids and len(set(ids)) == len(ids)
                and all(eid in admissions for eid in ids), "claim_has_unadmitted_evidence")
        matching = []
        for receipt in receipts.values():
            if receipt.get("claim_id") != cid:
                continue
            require(receipt.get("proposition") == claim["proposition"], "receipt_proposition_mismatch")
            if receipt.get("conclusion") != "supported":
                continue
            require(receipt.get("verifier") in {"structured_fact_equality", "qualified_structured_fact_equality"}, "unsupported_receipt_verifier")
            eids = receipt.get("evidence_ids")
            require(isinstance(eids, list) and eids and len(set(eids)) == len(eids), "invalid_receipt_evidence")
            require(all(eid in admissions and eid in claim.get("evidence_ids", [])
                        and evidence[eid]["facts"].get(prop[0]) == prop[1] for eid in eids), "receipt_without_matching_admitted_fact")
            matching.append(receipt["id"])
        require(matching, "supported_claim_without_receipt")
        by_proposition[prop].append({"claim_id": cid, "receipt_ids": matching, "evidence_ids": claim["evidence_ids"]})
    display_claims, cursor = [], 0
    text = final["text"]
    for number, segment in enumerate(text.split("; ")):
        match = re.fullmatch(r"([^;=()]+?) = (.+)", segment)
        require(match is not None, "unsupported_noncanonical_display")
        prop = match[1].strip(), match[2].strip()
        require(prop in by_proposition, "display_without_verified_proposition")
        size = len(segment.encode("utf-8"))
        display_claims.append({"id": f"display-{number + 1}", "proposition": {"key": prop[0], "value": prop[1]},
                               "start_byte": cursor, "end_byte": cursor + size,
                               "text": segment, "internal_bindings": by_proposition[prop]})
        cursor += size + 2
    require(type(final.get("factual_claims")) is int and final["factual_claims"] == len(display_claims), "display_claim_count_mismatch")
    require(type(final.get("covered_claims")) is int and final["covered_claims"] == len(display_claims), "display_uncovered_claims")
    require(type(final.get("factual_claim_coverage")) in {int, float}
            and final["factual_claim_coverage"] == 1 and final.get("uncovered_propositions") == [], "invalid_final_coverage")
    return {"schema": "ordinary-cli-capture-binding-development-v1", "binding_status": "STRUCTURAL_MATCH",
            "capture_mode": "host_curated_facts", "provider": trace["provider"], "model": trace["model"],
            "model_calls_in_saved_trace": model_calls, "display_artifacts": [{"id": "answer", "kind": "final_answer", "text": text}],
            "display_claims": display_claims, "sources": list(joined_sources.values()),
            "displayed_citations": {"status": "NOT_PRESENT_IN_SUPPORTED_CANONICAL_SURFACE", "count": 0},
            "semantic_support_review": "EXTERNAL_REVIEW_REQUIRED", "actual_answer_f1": None,
            "acceptance": "NOT_EVALUATED", "source_full_snapshot_integrity": "NOT_CHECKED_BY_TRACE_JOIN"}


def load_saved_capture(root, arm, corpus_path, expected_index_sha256):
    """Read/hash saved development files; never launch the Engine or old scorer."""
    root, corpus_path = Path(root), Path(corpus_path)
    require(arm in {"baseline", "candidate"}, "invalid_arm")
    index_bytes = (root / "artifact-sha256.json").read_bytes()
    require(sha(index_bytes) == expected_index_sha256, "capture_index_anchor_mismatch")
    index = strict_json(index_bytes)
    def read(name):
        rel = f"run-{arm}/{name}"
        data = (root / rel).read_bytes()
        require(rel in index and sha(data) == index[rel], "capture_file_hash_mismatch:" + name)
        return data
    blobs = {n: read(n) for n in ("stdout.json", "trace.json", "resolver.jsonl", "exit.json", "config.json", "stderr.txt", "invocation.json")}
    require(not blobs["stderr.txt"], "legacy_capture_stderr_not_empty")
    corpus_bytes = corpus_path.read_bytes()
    result = validate_capture(strict_json(blobs["stdout.json"]), strict_json(blobs["trace.json"]),
                              [strict_json(line) for line in blobs["resolver.jsonl"].splitlines()],
                              strict_json(blobs["exit.json"]), strict_json(blobs["config.json"]),
                              strict_json(corpus_bytes), sha(corpus_bytes))
    result["input_hashes"] = {n: sha(data) for n, data in blobs.items()}
    result["capture_index_sha256"] = expected_index_sha256
    result["corpus_sha256"] = sha(corpus_bytes)
    invocation = strict_json(blobs["invocation.json"])
    argv = invocation.get("argv")
    require(invocation.get("offline_only") is True and isinstance(argv, list)
            and len(argv) > 1 and argv[1] == strict_json(blobs["stdout.json"])["result"]["task"], "unsupported_or_misbound_invocation")
    for flag, key in [("--provider", "provider"), ("--model", "model")]:
        require(argv.count(flag) == 1 and argv.index(flag) + 1 < len(argv)
                and argv[argv.index(flag) + 1] == result[key], "invocation_model_identity_mismatch")
    def root_json(name):
        data = (root / name).read_bytes()
        require(name in index and sha(data) == index[name], "capture_metadata_hash_mismatch")
        return strict_json(data)
    build, recorded = root_json("build-proof.json"), root_json("verified-result.json")
    require(build["host_commit"] == recorded["host_commit"] and build["engine_commits"] == recorded["engine_commits"], "capture_build_identity_mismatch")
    prior = recorded["arms"][arm]
    result["saved_build_identity"] = {"host_commit": build["host_commit"], "engine_commit": build["engine_commits"][arm],
                                      "binary_sha256": build["arms"][arm]["binary_sha256"],
                                      "binary_reexecuted": False, "original_engine_acceptance": False}
    result["feature_path"] = {"reported_normal_api_calls": prior["normal_path_public_relevance_api_calls"],
                              "reported_positive_control_api_calls": prior["positive_control_public_api_calls"],
                              "positive_control_v17_v30_seen": prior["positive_control_v17_v30_seen"],
                              "provenance": "Hash-bound prior controlled instrumentation report, not a new current-candidate run.",
                              "current_candidate_effect": "UNMEASURED"}
    return result


def verify_full_snapshot(source, document, raw, normalized_bytes):
    """Verify the captured HTML→normalization→exact quote chain, not entailment."""
    require(document.get("normalization") == "htmlparser-skip-script-style-join-data-whitespace-v1", "unsupported_normalization")
    require(sha(raw) == document["raw_sha256"], "raw_snapshot_hash_mismatch")
    require(sha(normalized_bytes) == document["normalized_sha256"], "normalized_snapshot_hash_mismatch")
    class Text(HTMLParser):
        def __init__(self):
            super().__init__(convert_charrefs=True)
            self.skip, self.parts = 0, []
        def handle_starttag(self, tag, attrs):
            if tag in {"script", "style"}:
                self.skip += 1
        def handle_endtag(self, tag):
            if tag in {"script", "style"} and self.skip:
                self.skip -= 1
        def handle_data(self, data):
            if not self.skip:
                self.parts.append(data)
    parser = Text()
    parser.feed(raw.decode("utf-8"))
    parser.close()
    normalized = " ".join(" ".join(parser.parts).split()).encode("utf-8")
    require(normalized == normalized_bytes, "normalization_reproduction_mismatch")
    require(source["document_id"] == document["document_id"]
            and source["url"] == document["url"]
            and source["source_version"] == document["source_version"] == "sha256:" + sha(raw), "snapshot_source_identity_mismatch")
    start, end = source["normalized_byte_start"], source["normalized_byte_end"]
    require(type(start) is int and type(end) is int and 0 <= start < end <= len(normalized), "invalid_quote_locator")
    quote = normalized[start:end]
    require(quote == source["quote"].encode("utf-8") and sha(quote) == source["quote_sha256"], "quote_locator_mismatch")
    return {"status": "EXACT_CAPTURED_BYTES", "raw_sha256": sha(raw), "normalized_sha256": sha(normalized),
            "quote_sha256": sha(quote), "normalized_byte_span": [start, end],
            "semantic_entailment": "EXTERNAL_REVIEW_REQUIRED", "publisher_authenticity": "NOT_ATTESTED"}
