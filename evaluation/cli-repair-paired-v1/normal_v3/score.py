#!/usr/bin/env python3
"""Exclusive prospective v3 score; preserves historical v1/v2 first scores."""
import argparse
from collections import Counter, defaultdict
from pathlib import Path

from annotate import build_packet, read_observations, validate_gold, validate_review
from contracts import (
    PACKET_ID, REVIEW_ID, SCORE_ID, digest, exclusive_json, keyed_digest,
    read_json, require, sha_field, strict_keys, exclusive_durable_claim,
)


def metric(tp, fp, fn):
    precision = tp / (tp + fp) if tp + fp else 0.0
    recall = tp / (tp + fn) if tp + fn else 0.0
    f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
    return {"precision": precision, "recall": recall, "f1": f1}


def score_case(observation, review, gold):
    """Only independent semantic annotation grants correctness; typed errors never do."""
    label = gold["label"]
    decision = review["decision"]
    unsafe = bool(review["hard_violations"])
    operational = observation["measurement_state"] == "operational_stop"
    invalid = decision == "invalid" and not operational and not unsafe
    correct_groups = set(review["correct_groups"]) if decision == "answer" and not unsafe else set()
    missing_groups = set(gold["required_groups"]) - correct_groups
    incorrect = review["incorrect_assertions"] if decision == "answer" else 0
    tp = fp = fn = tn = 0
    if label == "positive":
        # Unsafe display, terminal invalid and unusable answers remain FN.
        if not unsafe and decision == "answer" and not missing_groups and incorrect == 0:
            tp = 1
        else:
            fn = 1
    else:
        if unsafe or (decision == "answer" and not operational):
            fp = 1
        elif decision == "withhold" and not operational:
            tn = 1
        # Negative invalid/error is neither TN nor fabricated FP.
    citation_checks = review["citation_checks"]
    citation_ok = all(c["supported"] and c["literal_exact"] for c in citation_checks)
    checks = {c["id"]: c for c in citation_checks}
    # Count every factual assertion actually displayed, including those in
    # unsafe/invalid/withheld output. Excluding non-answers can conceal
    # uncited safety-critical facts behind a favorable coverage metric.
    fact_segments = [seg for seg in review["segments"] if seg["kind"] == "fact"]
    supported_cited_claims = sum(
        bool(seg["citation_ids"])
        and all(checks[cid]["supported"] and checks[cid]["literal_exact"]
                for cid in seg["citation_ids"])
        for seg in fact_segments
    )
    groups = len(gold["required_groups"])
    return {
        "job_id": observation["job_id"],
        "case_id": observation["case_id"],
        "arm": observation["arm"],
        "label": label,
        "decision": "unsafe" if unsafe else "operational_error" if operational else decision,
        "source_failure_class": [
            d["failure_class"] + (":" + d["phase"] if d["phase"] else "")
            for d in observation["diagnostics"]
        ],
        "tp": tp, "fp": fp, "fn": fn, "tn": tn,
        "claim_tp": len(correct_groups), "claim_fp": incorrect,
        "claim_fn": len(missing_groups), "claim_denominator": groups,
        "quality_invalid": invalid,
        "operational_error": operational,
        "hard_violations": len(review["hard_violations"]),
        "citation_count": len(citation_checks),
        "citation_passed_count": sum(c["supported"] and c["literal_exact"] for c in citation_checks),
        "citation_ok": citation_ok,
        "citation_evaluable": bool(fact_segments),
        "factual_claims": len(fact_segments),
        "supported_cited_claims": supported_cited_claims,
        "valid_semantic_outcome": not (invalid or operational or unsafe),
        "valid_answer": decision == "answer" and not (invalid or operational or unsafe),
        "recovered_local_decode": (
            decision in ("answer", "withhold") and not (operational or unsafe)
            and observation["measurement_state"] == "local_decode_observed"
        ),
        "task_success": bool(tp or tn) and not unsafe and not operational,
    }


def report(rows, gold):
    """No full-cohort metrics without exactly 24 independent per-arm cases."""
    results = {}
    for arm in ("baseline", "candidate"):
        arm_rows = [r for r in rows if r["arm"] == arm]
        counts = Counter(r["case_id"] for r in arm_rows)
        measured = len(gold) == 24 and set(counts) == set(gold) and all(n == 1 for n in counts.values())
        complete = measured and len(arm_rows) == 24 and not any(r["operational_error"] for r in arm_rows)
        totals = {name: sum(r[name] for r in arm_rows)
                  for name in ("tp", "fp", "fn", "tn", "claim_tp", "claim_fp", "claim_fn",
                               "claim_denominator", "hard_violations")}
        invalid = sum(r["quality_invalid"] for r in arm_rows)
        by_class = Counter(
            cause for r in arm_rows if r["quality_invalid"]
            for cause in (r["source_failure_class"] or ["unusable_answer"])
        )
        recovered_by_class = Counter(
            cause for r in arm_rows if r["recovered_local_decode"]
            for cause in r["source_failure_class"]
        )
        cited = [r for r in arm_rows if r["citation_evaluable"]]
        citations_all = sum(r["citation_count"] for r in cited)
        citations_ok = sum(r["citation_passed_count"] for r in cited)
        # Coverage is per annotated displayed fact, not one citation per
        # answer. A single cited fact cannot certify an uncited second fact.
        total_facts = sum(r["factual_claims"] for r in cited)
        covered_facts = sum(r["supported_cited_claims"] for r in cited)
        citation_coverage = covered_facts / total_facts if total_facts else None
        citation_fidelity = (citations_ok / citations_all) if citations_all else None
        thresholds = (
            complete and invalid == 0 and totals["hard_violations"] == 0
            and citation_coverage == citation_fidelity == 1.0
        )
        case_metric = metric(totals["tp"], totals["fp"], totals["fn"]) if complete else None
        claim_metric = metric(totals["claim_tp"], totals["claim_fp"], totals["claim_fn"]) if complete else None
        if case_metric is not None and claim_metric is not None:
            thresholds = thresholds and all(
                m["f1"] >= 0.90 and m["precision"] >= 0.95 and m["recall"] >= 0.85
                for m in (case_metric, claim_metric)
            )
        results[arm] = {
            "expected_cases": 24,
            "observed_cases": len(arm_rows),
            "observation_complete": complete,
            "case_confusion": {k.upper(): v for k, v in totals.items() if k in ("tp", "fp", "fn", "tn")},
            "case_metrics": case_metric,
            "claim_confusion": {k: totals[k] for k in ("claim_tp", "claim_fp", "claim_fn", "claim_denominator")},
            "claim_metrics": claim_metric,
            "quality_invalid": invalid,
            "invalid_rate": invalid / 24 if complete else None,
            "invalid_by_class": dict(sorted(by_class.items())),
            "recovered_local_decode_rows": sum(r["recovered_local_decode"] for r in arm_rows),
            "recovered_by_class": dict(sorted(recovered_by_class.items())),
            "operational_error_rows": sum(r["operational_error"] for r in arm_rows),
            "hard_violations": totals["hard_violations"],
            "citation_coverage": citation_coverage,
            "citation_fidelity": citation_fidelity,
            "valid_answer_coverage": (
                sum(r["valid_answer"] for r in arm_rows) / 24 if complete else None
            ),
            "valid_finalization_coverage": (
                sum(r["valid_semantic_outcome"] for r in arm_rows) / 24 if complete else None
            ),
            "task_success_rate": sum(r["task_success"] for r in arm_rows) / 24 if complete else None,
            "semantic_thresholds_met": bool(thresholds),
        }
    return {
        "contract": SCORE_ID,
        "scope": "CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE",
        "arms": results,
        "cohort_measured": all(results[a]["observation_complete"] for a in results),
        "all_semantic_thresholds_met": all(results[a]["semantic_thresholds_met"] for a in results),
        "release_or_runtime_admission": "NOT_APPROVED",
        "live_run_authorized": False,
    }


def score(observations, packets, gold_file, reviews, receipt_file, output):
    observations = Path(observations)
    packets = Path(packets)
    reviews = Path(reviews)
    gold_file = Path(gold_file)
    receipt_file = Path(receipt_file)
    gold = read_json(gold_file)
    by_case = validate_gold(gold)
    index, rows = read_observations(observations)
    require(digest(receipt_file.read_bytes()) == index["receipt_sha256"] == gold["receipt_sha256"],
            "receipt_byte_mismatch")
    packet_index = read_json(packets / "packet-index.json")
    require(
        packet_index["contract"] == PACKET_ID
        and packet_index["observation_index_sha256"] == digest((observations / "index.json").read_bytes())
        and packet_index["gold_sha256"] == digest(gold_file.read_bytes())
        and packet_index["receipt_sha256"] == index["receipt_sha256"]
        and packet_index["executor_id"] == index["executor_id"]
        and packet_index.get("semantic_review") == "PENDING",
        "unbound_review_packets",
    )
    require(
        set(row["job_id"] for row in packet_index["rows"]) == set(rows)
        and len(packet_index["rows"]) == len(rows),
        "review_packet_inventory_mismatch",
    )
    for packet in packet_index["rows"]:
        sha_field(packet["sha256"], "invalid_packet_digest")
        path = packets / (packet["job_id"] + ".review-packet.json")
        require(digest(path.read_bytes()) == packet["sha256"], "modified_review_packet")
        p = read_json(path)
        observation, obs_sha = rows[packet["job_id"]]
        require(observation["question"] == by_case[observation["case_id"]]["question"],
                "scoring_question_mismatch")
        require(
            p == build_packet(observation, obs_sha, by_case[observation["case_id"]]),
            "packet_observation_or_gold_mismatch",
        )
    manifest = read_json(reviews / "review-index.json")
    strict_keys(
        manifest,
        {"contract", "receipt_sha256", "observation_index_sha256", "packet_index_sha256",
         "executor_id", "reviews"},
        {"contract", "receipt_sha256", "observation_index_sha256", "packet_index_sha256",
         "executor_id", "reviews"},
        "review_manifest",
    )
    require(
        manifest["contract"] == "reason-cli-normal-v3-review-index-v1"
        and manifest["receipt_sha256"] == index["receipt_sha256"]
        and manifest["executor_id"] == index["executor_id"]
        and manifest["observation_index_sha256"] == digest((observations / "index.json").read_bytes())
        and manifest["packet_index_sha256"] == digest((packets / "packet-index.json").read_bytes()),
        "review_freeze_binding_mismatch",
    )
    freeze = manifest["reviews"]
    require(type(freeze) is dict and set(freeze) == set(rows), "missing_or_extra_review_records")
    graded = []
    for job_id, (observation, obs_sha) in rows.items():
        filename = reviews / (job_id + ".review.json")
        require(filename.is_file() and not filename.is_symlink(), "missing_review_file")
        sha_field(freeze[job_id], "invalid_review_sha")
        require(digest(filename.read_bytes()) == freeze[job_id], "modified_review_file")
        review = read_json(filename)
        gold_case = by_case[observation["case_id"]]
        validate_review(review, observation, obs_sha, gold_case, manifest["executor_id"])
        graded.append(score_case(observation, review, gold_case))
    result = report(graded, by_case)
    result.update({
        "receipt_sha256": index["receipt_sha256"],
        "capture_manifest_sha256": index["capture_manifest_sha256"],
        "observation_index_sha256": digest((observations / "index.json").read_bytes()),
        "review_manifest_sha256": digest((reviews / "review-index.json").read_bytes()),
        "human_attestation": False,
        "first_score_only": True,
        "quality_go": False,  # Independent runtime, non-regression, efficiency, fresh accuracy separate.
        "remaining_gates": [
            "v3_scheduler_http_zero_and_operational_stop",
            "independent_runtime_source_image_admission",
            "new_readiness_receipt_tag",
            "one_new_Mistral_48_job_run",
            "independent_accuracy_acceptance",
            "efficiency_and_non_regression",
        ],
    })
    # An output-folder-only lock can be bypassed by changing --output. A
    # genuine receipt must irrevocably freeze its one durable score-ledger
    # directory. This gate is also used for synthetic offline evidence.
    receipt = read_json(receipt_file)
    ledger_value = receipt.get("score_ledger_dir") if type(receipt) is dict else None
    require(type(ledger_value) is str, "receipt_score_ledger_missing")
    ledger = Path(ledger_value)
    require(ledger.is_absolute() and ledger.is_dir()
            and ledger.resolve() == ledger and not ledger.is_symlink(),
            "receipt_score_ledger_not_canonical")
    output = Path(output)
    exclusive_durable_claim(ledger / (index["receipt_sha256"] + ".first-score.claim.json"), {
        "contract": SCORE_ID, "receipt_sha256": index["receipt_sha256"],
        "observation_index_sha256": result["observation_index_sha256"],
        "review_manifest_sha256": result["review_manifest_sha256"],
        "output": str(output.resolve()), "replay_allowed": False,
    })
    output.mkdir()  # Create-only: no overwrites or relabeling.
    exclusive_json(output / "first-score-started.json", {
        "receipt_sha256": index["receipt_sha256"],
        "review_manifest_sha256": result["review_manifest_sha256"],
        "rescore_or_relabel": False,
    })
    exclusive_json(output / "rows.json", graded)
    exclusive_json(output / "report.json", result)
    return result


def main():
    parser = argparse.ArgumentParser()
    for option in ("observations", "packets", "gold", "reviews", "receipt", "output"):
        parser.add_argument("--" + option, type=Path, required=True)
    a = parser.parse_args()
    result = score(a.observations, a.packets, a.gold, a.reviews, a.receipt, a.output)
    print(__import__("json").dumps({
        "cohort_measured": result["cohort_measured"],
        "semantic_thresholds_met": result["all_semantic_thresholds_met"],
        "quality_go": False,
    }))


if __name__ == "__main__":
    main()
