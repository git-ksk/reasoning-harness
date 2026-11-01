#!/usr/bin/env python3
"""Prepare source-complete blinded review packets; NEVER infer semantic judgments."""
import argparse
from pathlib import Path

from contracts import (
    OBS_ID, PACKET_ID, REVIEW_ID, canonical, digest, exclusive_json,
    keyed_digest, read_json, require, safe_id, sha_field, strict_keys, unsigned,
)


def validate_gold(gold):
    strict_keys(gold, {"contract", "receipt_sha256", "cases"},
                {"contract", "receipt_sha256", "cases"}, "gold")
    require(gold["contract"] == "reason-cli-normal-v3-gold-v1", "unknown_gold_contract")
    sha_field(gold["receipt_sha256"], "gold_receipt_hash_missing")
    cases = gold["cases"]
    require(type(cases) is list and cases, "empty_gold")
    result = {}
    for case in cases:
        strict_keys(case, {"case_id", "question", "label", "required_groups"},
                    {"case_id", "question", "label", "required_groups"}, "gold_case")
        require(type(case["question"]) is str and bool(case["question"].strip()),
                "gold_case_question_missing")
        cid = safe_id(case["case_id"], "gold_case_id")
        require(cid not in result, "duplicate_gold_case")
        label = case["label"]
        require(label in ("positive", "negative"), "unknown_gold_label")
        groups = case["required_groups"]
        require(type(groups) is list and all(type(g) is str and g for g in groups)
                and len(groups) == len(set(groups)), "invalid_required_groups")
        require((label == "positive" and bool(groups)) or
                (label == "negative" and not groups), "gold_groups_label_mismatch")
        result[cid] = case
    return result


def read_observations(root):
    index = read_json(root / "index.json")
    require(index["contract"] == OBS_ID and index.get("continuation_authorized") is False,
            "unrecognized_observation_index")
    sha_field(index["receipt_sha256"], "unbound_observation_receipt")
    safe_id(index["executor_id"], "unbound_executor")
    rows = {}
    for entry in index["rows"]:
        safe_id(entry["job_id"], "invalid_index_job_id")
        sha_field(entry["sha256"], "invalid_observation_hash")
        path = root / (entry["job_id"] + ".json")
        require(path.is_file() and not path.is_symlink(), "missing_observation")
        raw = path.read_bytes()
        require(digest(raw) == entry["sha256"], "modified_observation")
        observation = __import__("json").loads(raw)
        require(
            observation.get("contract") == OBS_ID
            and observation.get("receipt_sha256") == index["receipt_sha256"]
            and observation.get("executor_id") == index["executor_id"]
            and observation.get("job_id") == entry["job_id"]
            and observation.get("case_id") == entry["case_id"]
            and observation.get("arm") == entry["arm"],
            "observation_index_identity_mismatch",
        )
        require(entry["job_id"] not in rows, "duplicate_observation")
        rows[entry["job_id"]] = (observation, entry["sha256"])
    require(len(rows) == len(index["rows"]), "duplicate_observation")
    return index, rows


def build_packet(observation, obs_hash, case):
    """Deterministic packet; scorer reconstitutes it to prevent review laundering."""
    return {
        "contract": PACKET_ID, "job_id": observation["job_id"],
        "case_id": observation["case_id"], "question": observation["question"],
        "arm": observation["arm"],
        "observation_sha256": obs_hash, "gold_sha256": keyed_digest(case),
        "gold": case, "receipt_sha256": observation["receipt_sha256"],
        "display_artifacts": observation["display_artifacts"],
        "display_citations": observation["display_citations"],
        "admitted_source_ids": observation["admitted_source_ids"],
        "full_sources": observation["sources"],
        "terminal_status": observation["terminal_status"],
        "diagnostics": observation["diagnostics"],
        "measurement_state": observation["measurement_state"],
        "review_state": "PENDING_INDEPENDENT_SEMANTIC_ADJUDICATION",
        "auto_graded": False,
    }


def make_packets(observations_root, gold_file, output):
    observations_root = Path(observations_root)
    gold_file = Path(gold_file)
    gold = read_json(gold_file)
    by_case = validate_gold(gold)
    index, rows = read_observations(observations_root)
    require(gold["receipt_sha256"] == index["receipt_sha256"],
            "gold_observation_receipt_mismatch")
    packets = []
    for job_id, (observation, obs_hash) in rows.items():
        cid = observation["case_id"]
        require(cid in by_case, "case_without_gold")
        case = by_case[cid]
        require(observation["question"] == case["question"], "gold_case_question_mismatch")
        packets.append(build_packet(observation, obs_hash, case))
    output = Path(output)
    output.mkdir()
    for item in packets:
        exclusive_json(output / (item["job_id"] + ".review-packet.json"), item)
    exclusive_json(output / "packet-index.json", {
        "contract": PACKET_ID, "gold_sha256": digest(gold_file.read_bytes()),
        "observation_index_sha256": digest((observations_root / "index.json").read_bytes()),
        "receipt_sha256": index["receipt_sha256"],
        "executor_id": index["executor_id"],
        "rows": [
            {"job_id": packet["job_id"], "sha256": digest(
                (output / (packet["job_id"] + ".review-packet.json")).read_bytes()
            )} for packet in packets
        ],
        "semantic_review": "PENDING",
        "score": None,
    })
    return packets


def validate_review(review, observation, obs_hash, gold, executor_id):
    """Strict independent semantics; a review cannot rewrite captured bytes or gold."""
    strict_keys(
        review,
        {
            "contract", "job_id", "observation_sha256", "gold_sha256",
            "reviewer", "decision", "correct_groups", "incorrect_assertions",
            "hard_violations", "citation_checks", "readable_and_actionable",
            "safety_audit_complete", "notes",
        },
        {
            "contract", "job_id", "observation_sha256", "gold_sha256",
            "reviewer", "decision", "correct_groups", "incorrect_assertions",
            "hard_violations", "citation_checks", "readable_and_actionable",
            "safety_audit_complete", "notes",
        },
        "review",
    )
    require(review["contract"] == REVIEW_ID, "unrecognized_review_contract")
    require(
        review["job_id"] == observation["job_id"]
        and review["observation_sha256"] == obs_hash
        and review["gold_sha256"] == keyed_digest(gold),
        "stale_or_foreign_semantic_review",
    )
    reviewer = review["reviewer"]
    strict_keys(reviewer,
                {"role", "id", "independent_of_execution", "full_source_context_reviewed"},
                {"role", "id", "independent_of_execution", "full_source_context_reviewed"},
                "reviewer")
    require(
        reviewer["role"] in ("human", "agent")
        and type(reviewer["id"]) is str and bool(reviewer["id"])
        and reviewer["id"] != executor_id
        and reviewer["independent_of_execution"] is True
        and reviewer["full_source_context_reviewed"] is True,
        "nonindependent_or_unreviewed_sources",
    )
    require(review["decision"] in ("answer", "withhold", "invalid"), "invalid_review_decision")
    require(type(review["safety_audit_complete"]) is bool and review["safety_audit_complete"],
            "incomplete_safety_audit")
    require(type(review["readable_and_actionable"]) is bool, "invalid_readability")
    require(type(review["notes"]) is str and bool(review["notes"].strip()), "missing_review_rationale")
    groups = review["correct_groups"]
    require(type(groups) is list and all(type(group) is str for group in groups)
            and len(set(groups)) == len(groups)
            and set(groups) <= set(gold["required_groups"]), "invented_group_credit")
    unsigned(review["incorrect_assertions"], "invalid_incorrect_assertion_count")
    violations = review["hard_violations"]
    require(type(violations) is list and all(type(v) is str and v.strip() for v in violations),
            "invalid_safety_violations")
    cites = review["citation_checks"]
    require(type(cites) is list, "missing_citation_checks")
    ids = [c.get("id") for c in observation["display_citations"]]
    checks = []
    for c in cites:
        strict_keys(c, {"id", "supported", "literal_exact"},
                    {"id", "supported", "literal_exact"}, "citation_check")
        require(type(c["supported"]) is bool and type(c["literal_exact"]) is bool, "invalid_citation_verdict")
        checks.append(c["id"])
    require(sorted(checks) == sorted(ids) and len(checks) == len(set(checks)),
            "incomplete_or_duplicate_citation_audit")
    display = observation["display_artifacts"][0]["text"]
    state = observation["measurement_state"]
    if state == "operational_stop":
        require(review["decision"] == "invalid", "operational_stop_cannot_be_graded")
    if not display.strip():
        require(not violations, "safety_violation_without_display")
    if observation["terminal_status"] == "failed" or not display.strip():
        require(review["decision"] == "invalid", "empty_or_failed_output_cannot_be_withhold")
    if review["decision"] == "withhold":
        require(
            observation["terminal_status"] in ("unresolved", "requires_verification", "rejected")
            and state == "normal_observed",
            "error_cannot_be_correct_withhold",
        )
    if review["decision"] == "answer":
        require(bool(display.strip()) and review["readable_and_actionable"]
                and state != "operational_stop", "unusable_result_cannot_be_answer")
    if review["decision"] != "answer":
        require(not groups, "nonanswer_cannot_receive_group_credit")
    return review


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observations", type=Path, required=True)
    parser.add_argument("--gold", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    packets = make_packets(args.observations, args.gold, args.output)
    print(__import__("json").dumps({"review_packets": len(packets), "automated_semantic_decisions": 0}))


if __name__ == "__main__":
    main()
