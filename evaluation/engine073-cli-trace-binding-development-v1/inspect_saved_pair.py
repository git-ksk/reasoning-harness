#!/usr/bin/env python3
"""Inspect retained ordinary-CLI development bytes without executing the CLI."""
import argparse
import json
import os
from pathlib import Path
import re
import sys

from trace_binding import load_saved_capture, require, sha, strict_json, verify_full_snapshot


def dump_new(path, value):
    with path.open("x") as stream:
        json.dump(value, stream, indent=2, ensure_ascii=False)
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())


def inspect_pair(repo, capture, corpus_path, snapshots, expected_index, output):
    # Reuse the existing arithmetic preparation's display schema without changing
    # its bytes. Only byte coverage is exercised; no semantic annotation/score.
    shared = repo / "evaluation/engine073-answer-metrics-development-v1"
    sys.path.insert(0, str(shared))
    from display_binding import bundle_sha256, validate_display_binding
    corpus_bytes = corpus_path.read_bytes()
    corpus = strict_json(corpus_bytes)
    result = {"identity": "engine073-cli-trace-binding-development-v1", "capture_scope": "saved controlled offline ordinary-CLI pair only",
              "fresh_case_authorship": 0, "Engine_provider_model_Oracle_calls": 0, "old_scorer_executions": 0,
              "real24_completed": 0, "actual_answer_f1": None, "acceptance": "NOT_EVALUATED",
              "arms": {}, "full_source_snapshot_checks": {}}
    current_source = repo / "crates/reasoning-harness-core/src/evidence_relevance.rs"
    result["current_source_sha256"] = sha(current_source.read_bytes())
    adapter_code = Path(__file__).with_name("trace_binding.py")
    result["inspection_code_sha256"] = sha(Path(__file__).read_bytes())
    result["trace_adapter_code_sha256"] = sha(adapter_code.read_bytes())
    output.mkdir(exist_ok=False)
    original_hashes, snapshot_hashes = {}, {}
    for arm in ("baseline", "candidate"):
        bound = load_saved_capture(capture, arm, corpus_path, expected_index)
        original_hashes[arm] = bound["input_hashes"]
        claims, segments, cursor = [], [], 0
        text = bound["display_artifacts"][0]["text"]
        for n, claim in enumerate(bound["display_claims"]):
            if claim["start_byte"] != cursor:
                part = text.encode()[cursor:claim["start_byte"]].decode()
                segments.append({"id": f"separator-{n}", "artifact_id": "answer", "start_byte": cursor,
                                 "end_byte": claim["start_byte"], "text": part, "kind": "nonfact", "reason": "canonical formatting separator"})
            pid = json.dumps(claim["proposition"], sort_keys=True, separators=(",", ":"))
            claims.append({"id": claim["id"], "proposition_id": pid})
            segments.append({"id": "span-" + claim["id"], "artifact_id": "answer", "start_byte": claim["start_byte"],
                             "end_byte": claim["end_byte"], "text": claim["text"], "kind": "fact",
                             "claim_id": claim["id"], "proposition_id": pid})
            cursor = claim["end_byte"]
        bundle_hash = bundle_sha256(bound["display_artifacts"])
        bound["declared_display_binding"] = validate_display_binding(bound["display_artifacts"], ["answer"], bundle_hash, segments, claims)
        bound["display_segments"] = segments
        for source in bound["sources"]:
            doc_id, source_id = source["document_id"], source["source_id"]
            require(re.fullmatch(r"[A-Za-z0-9_-]+", doc_id) is not None, "unsafe_snapshot_filename")
            raw_path, normalized_path = snapshots / (doc_id + ".html"), snapshots / (doc_id + ".txt")
            raw, normalized = raw_path.read_bytes(), normalized_path.read_bytes()
            proof = verify_full_snapshot(source, corpus["documents"][doc_id], raw, normalized)
            snapshot_hashes[str(raw_path)] = sha(raw)
            snapshot_hashes[str(normalized_path)] = sha(normalized)
            result["full_source_snapshot_checks"][source_id] = proof
        bound["source_full_snapshot_integrity"] = "EXACT_CAPTURED_BYTES"
        result["arms"][arm] = bound
    require(sha(corpus_path.read_bytes()) == sha(corpus_bytes), "corpus_changed_during_inspection")
    for arm, files in original_hashes.items():
        for name, digest in files.items():
            require(sha((capture / f"run-{arm}" / name).read_bytes()) == digest, "capture_changed_during_inspection")
    require(sha((capture / "artifact-sha256.json").read_bytes()) == expected_index, "capture_index_changed")
    for path, digest in snapshot_hashes.items():
        require(sha(Path(path).read_bytes()) == digest, "snapshot_changed_during_inspection")
    require(sha(current_source.read_bytes()) == result["current_source_sha256"], "current_source_changed")
    require(sha(Path(__file__).read_bytes()) == result["inspection_code_sha256"]
            and sha(adapter_code.read_bytes()) == result["trace_adapter_code_sha256"], "inspector_changed_during_run")
    result["display_texts_equal"] = result["arms"]["baseline"]["display_artifacts"] == result["arms"]["candidate"]["display_artifacts"]
    result["readiness_blockers"] = ["semantic_claim_and_citation_adjudication_not_frozen",
                                    "displayed_citations_absent_in_this_canonical_surface",
                                    "current_candidate_active_path_and_complete_real_M0_pending"]
    if all(arm["feature_path"]["reported_normal_api_calls"] == 0 for arm in result["arms"].values()):
        result["readiness_blockers"].append("targeted_relevance_APIs_inactive_in_saved_normal_path")
    result["readiness_blockers"].append("saved_capture_is_not_a_current_candidate_runtime_measurement")
    result["snapshot_input_hashes"] = snapshot_hashes
    result["corpus_sha256"] = sha(corpus_bytes)
    result["capture_index_sha256"] = expected_index
    result["shared_display_module_sha256"] = sha((shared / "display_binding.py").read_bytes())
    result["private_full_sources_republished"] = False
    dump_new(output / "inspection.json", result)
    dump_new(output / "artifact-sha256.json", {"inspection.json": sha((output / "inspection.json").read_bytes())})
    print(json.dumps({"binding": "STRUCTURAL_MATCH", "full_source": "EXACT_CAPTURED_BYTES", "actual_answer_f1": None,
                      "acceptance": "NOT_EVALUATED", "blockers": result["readiness_blockers"]}))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--repo", type=Path, required=True)
    p.add_argument("--capture", type=Path, required=True)
    p.add_argument("--corpus", type=Path, required=True)
    p.add_argument("--snapshots", type=Path, required=True)
    p.add_argument("--expected-index-sha256", required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    inspect_pair(a.repo.resolve(), a.capture.resolve(), a.corpus.resolve(), a.snapshots.resolve(), a.expected_index_sha256, a.output.resolve())


if __name__ == "__main__":
    main()
