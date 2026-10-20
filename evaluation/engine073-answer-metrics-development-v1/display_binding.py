"""Byte/span checks for externally captured displays and external annotations.

This proves declared coverage/binding only, not semantic segmentation correctness,
completeness of the runtime display inventory, citation support or independence.
"""
import hashlib
import json

from claim_metrics import digest, nonempty, score_case, unique_strings


def bundle_sha256(artifacts):
    """Versioned canonical envelope; preserves each text's exact Unicode bytes."""
    canonical = json.dumps({"schema": "display-bundle-v1", "artifacts": artifacts},
                           ensure_ascii=False, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def validate_display_binding(artifacts, expected_artifact_ids, captured_sha256,
                             segments, claims):
    """Validate UTF-8 byte spans before sending claims to the arithmetic kernel.

    expected_artifact_ids and captured_sha256 must come from the independent
    runtime recorder, not the annotation being checked. Each display byte must
    belong to a declared factual occurrence or a justified nonfact span.
    """
    expected = unique_strings(expected_artifact_ids, "expected_artifact_ids")
    digest(captured_sha256)
    if not isinstance(artifacts, list) or not isinstance(segments, list) or not isinstance(claims, list):
        raise ValueError("invalid_display_binding_inputs")
    byid = {}
    for artifact in artifacts:
        if not isinstance(artifact, dict) or set(artifact) != {"id", "kind", "text"}:
            raise ValueError("invalid_artifact_schema")
        aid = nonempty(artifact["id"], "artifact_id")
        if aid in byid:
            raise ValueError("duplicate_artifact_id")
        if artifact["kind"] not in {"final_answer", "claim_card", "citation_caption"}:
            raise ValueError("unknown_artifact_kind")
        if not isinstance(artifact["text"], str):
            raise ValueError("invalid_display_text")
        byid[aid] = artifact["text"].encode("utf-8")
    if set(byid) != expected:
        raise ValueError("missing_or_extra_display_artifact")
    if bundle_sha256(artifacts) != captured_sha256:
        raise ValueError("captured_display_bundle_mismatch")
    claim_index = {}
    for claim in claims:
        if not isinstance(claim, dict):
            raise ValueError("invalid_claim")
        cid = nonempty(claim.get("id"), "claim_id")
        if cid in claim_index:
            raise ValueError("duplicate_claim_id")
        claim_index[cid] = nonempty(claim.get("proposition_id"), "proposition_id")
    spans = {aid: [] for aid in byid}
    seen_segments, covered_claims = set(), set()
    for segment in segments:
        if not isinstance(segment, dict):
            raise ValueError("invalid_segment")
        sid = nonempty(segment.get("id"), "segment_id")
        if sid in seen_segments:
            raise ValueError("duplicate_segment_id")
        seen_segments.add(sid)
        aid = segment.get("artifact_id")
        if aid not in byid:
            raise ValueError("foreign_segment_artifact")
        start, end = segment.get("start_byte"), segment.get("end_byte")
        if type(start) is not int or type(end) is not int or not 0 <= start < end <= len(byid[aid]):
            raise ValueError("invalid_byte_span")
        try:
            text = byid[aid][start:end].decode("utf-8")
        except UnicodeDecodeError as exc:
            raise ValueError("span_splits_utf8_character") from exc
        if segment.get("text") != text:
            raise ValueError("span_text_mismatch")
        kind = segment.get("kind")
        if kind == "fact":
            cid = segment.get("claim_id")
            if cid not in claim_index or segment.get("proposition_id") != claim_index[cid]:
                raise ValueError("misbound_fact_claim")
            covered_claims.add(cid)
        elif kind == "nonfact":
            nonempty(segment.get("reason"), "nonfact_reason")
            if "claim_id" in segment or "proposition_id" in segment:
                raise ValueError("nonfact_cannot_bind_claim")
        else:
            raise ValueError("invalid_segment_kind")
        spans[aid].append((start, end))
    for aid, ranges in spans.items():
        cursor = 0
        for start, end in sorted(ranges):
            if start != cursor:
                raise ValueError("display_gap_or_overlap:" + aid)
            cursor = end
        if cursor != len(byid[aid]):
            raise ValueError("display_gap_or_overlap:" + aid)
    if covered_claims != set(claim_index):
        raise ValueError("claim_without_display_span")
    return {"captured_bundle_sha256": captured_sha256, "artifacts": len(artifacts),
            "segments": len(segments), "claim_occurrences": len(covered_claims),
            "declared_byte_coverage": "COMPLETE", "semantic_segmentation": "EXTERNAL_REVIEW_REQUIRED",
            "trace_and_support": "NOT_VERIFIED"}


def score_bound_case(plan, review, artifacts, expected_artifact_ids, segments):
    """Join display coverage checks and arithmetic, retaining external review limits."""
    binding = validate_display_binding(artifacts, expected_artifact_ids,
                                       plan.get("observation_sha256"), segments,
                                       review.get("claims"))
    return {"display_binding": binding, "claim_metrics": score_case(plan, review),
            "acceptance": "NOT_EVALUATED"}
