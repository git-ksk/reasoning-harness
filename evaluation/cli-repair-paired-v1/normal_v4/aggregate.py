#!/usr/bin/env python3
"""Fail-closed aggregation of 24 independently signed REAL v4 paired shards.

Only consumes decrypted, reviewer-obtained artifact trees. Does NOT decrypt
material, issue a new authority receipt, call a provider, rewrite any shard,
or infer missing observations. Requires all 48 authentic signed observations
before computing global F1; no release/quality admission is granted.
"""
import argparse
from dataclasses import asdict
import json
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))
from annotate import read_observations, validate_gold, validate_review
from authority import AuthorityBinding, validate_frozen_receipt
from ci_prepare import gold_cases
from ci_exchange import APPROVAL, authenticate
from tag_gate import verify_proof
from contracts import canonical, digest, keyed_digest, require
from review_auth import ReviewAuthentication
from score import score_case, report
from prepare_local import checked_input
from shard_contract import EXPECTED_IDS, global_manifest, verify_all_shards, sha


def raw(path):
    return path.read_bytes()


def obj(path):
    return json.loads(raw(path))


def check_source_enrolled_operator(bootstrap, final, operator):
    """Prevent reviewer policy/key substitution from an attacker-controlled archive.

    The immutable native bundle's source files are independently SHA checked
    before this function is called, and include the operator config from the
    checked-out source commit. Review/approval must bind that actual source
    config, not a final archive's self-declared new verification key.
    """
    source = "evaluation/cli-repair-paired-v1/normal_v3/ci-operator.json"
    pinned = []
    for arm in ("baseline","candidate"):
        p = bootstrap / "bundle-files" / ("source/" + arm) / source
        require(p.is_file() and not p.is_symlink(),
                "v4_enrolled_operator_source_missing")
        pinned.append(raw(p))
    source_bytes = raw(final / "operator.json")
    require(pinned[0] == pinned[1] == source_bytes
            and canonical(operator) == source_bytes,
            "v4_enrolled_operator_changed_from_git_source")
    require(raw(final / "policy.json") ==
            canonical(operator["review_policy"]), "v4_review_policy_substituted")


def read_pair(base, case_id):
    """Verify complete original binary/source, execution and semantic bindings."""
    bootstrap = base / case_id / "bootstrap"
    final = base / case_id / "final"
    require(bootstrap.is_dir() and final.is_dir(),
            "v4_missing_decrypted_pair_artifact")
    session_bytes = raw(bootstrap / "session.json")
    require(session_bytes == raw(final / "session.json"), "v4_session_changed")
    session = json.loads(session_bytes)
    require(session["mode"] == "live"
            and session["execution_tag"] == ("cli-normal-real24-development-v5-" + case_id)
            and session["shard_case_id"] == case_id and session["run_attempt"] == 1
            and session["repository"] == "git-ksk/reasoning-harness",
            "v4_wrong_live_shard_session")
    bplan = raw(bootstrap / "plan.json")
    require(bplan == raw(final / "plan.json"), "v4_plan_changed")
    plan = json.loads(bplan)
    require(plan["schema"] == "engine07-operational-paired-shard-v1"
            and plan["case_ids"] == [case_id]
            and len(plan["jobs"]) == 2
            and {x["id"] for x in plan["jobs"]} ==
                {case_id+"-baseline", case_id+"-candidate"},
            "v4_foreign_shard_plan")
    for filename in ("operator.json", "policy.json", "receipt.json", "gold.json",
                     "native-bundle.json", "authority-receipt.json",
                     "global-manifest.json", "shard.json"):
        require(raw(bootstrap / filename) == raw(final / filename),
                "v4_bootstrap_final_data_changed:"+filename)

    raw_corpus = obj(bootstrap / "input/corpus.json")
    checked_input(bootstrap / "input")
    generated_manifest = global_manifest(raw_corpus, obj(bootstrap / "requests.json")["jobs"],
                                        session["source_commit"])
    manifest = obj(bootstrap / "global-manifest.json")
    require(manifest == generated_manifest, "v4_global_manifest_not_actual_sources")
    shard = obj(bootstrap / "shard.json")
    require(shard == plan["shard"] and
            shard["global_manifest_sha256"] == sha(canonical(manifest)),
            "v4_unbound_global_shard")
    original_gold = gold_cases(raw_corpus, obj(bootstrap / "input/gold.json"))
    source_gold = next((x for x in original_gold if x["case_id"] == case_id), None)
    gold = obj(final / "gold.json")
    require(source_gold is not None
            and gold["cases"] == [source_gold]
            and gold["receipt_sha256"] == digest(raw(final / "receipt.json")),
            "v4_gold_not_frozen_source")

    # The binary and every enrolled configuration in the bootstrap must be
    # the exact originally enrolled bytes. We do not execute them here.
    bundle = obj(bootstrap / "native-bundle.json")
    require(bundle == obj(final / "native-bundle.json"), "v4_native_bundle_changed")
    for name, record in bundle["files"].items():
        path = bootstrap / "bundle-files" / name
        require(path.is_file() and not path.is_symlink()
                and digest(raw(path)) == record["sha256"],
                "v4_native_bundle_source_mismatch")
    receipt_raw = raw(final / "receipt.json")
    receipt = obj(final / "authority-receipt.json")
    operator = obj(final / "operator.json")
    check_source_enrolled_operator(bootstrap,final,operator)
    binding = AuthorityBinding(**{k: receipt[k] for k in (
        "source_sha256", "image_sha256", "cli_build_sha256",
        "runtime_sha256", "executor_id", "evaluator_id")})
    validate_frozen_receipt(receipt, plan, receipt_raw, **asdict(binding))
    require(receipt["evaluation_id"] == "ci-v4-" + str(session["run_id"]) + "-" + case_id
            and receipt["image_sha256"] == digest(raw(final / "native-bundle.json")),
            "v4_receipt_or_bundle_identity_mismatch")
    require(operator["fee_mode"] == "free-only"
            and operator["additional_charges_enabled"] is False
            and operator["executor_id"] == receipt["executor_id"]
            and operator["evaluator_id"] == receipt["evaluator_id"],
            "v4_unenrolled_fee_or_execution_identity")

    tag_proof = obj(final / "tag-reservation.json")
    verify_proof(tag_proof,{
        "GITHUB_RUN_ID":str(session["run_id"]),
        "GITHUB_SHA":session["source_commit"],
    })
    start_request = obj(final / "start-request.json")
    require(start_request == obj(bootstrap / "start-request.json")
            and start_request["session_sha256"] == digest(session_bytes)
            and start_request["plan_sha256"] == digest(bplan),
            "v4_start_request_not_exact_session")
    signed_start = obj(final / "start-approval.json")
    signed_payload = authenticate(
        signed_start,raw(final / "policy.json"),
        operator["evaluator_id"],operator["executor_id"],APPROVAL,start_request)
    require(signed_payload["decision"] == "approve",
            "v4_independent_start_not_approved")
    # NativeAuthority first-score is an irreversible authority claim, not
    # a self-reported property of a JSON report.
    store_claim_root = final / "preparation/authority/reason-cli-normal-v3/authority/v1/claims" / receipt["evaluation_id"]
    for arm in ("baseline","candidate"):
        jid = case_id + "-" + arm
        admission_file = store_claim_root / "admission" / (jid + ".json")
        require(admission_file.is_file() and not admission_file.is_symlink(),
                "v4_registered_pre_advance_admission_missing")
        admitted = obj(admission_file)
        require(admitted["job_id"] == jid
                and admitted["observation_sha256"],
                "v4_pre_advance_admission_identity_mismatch")

    index_bytes = raw(final / "observations/index.json")
    index, observations = read_observations(final / "observations", index_bytes=index_bytes)
    require(len(observations) == 2
            and {r["job_id"] for r, _ in observations.values()} ==
              {case_id+"-baseline", case_id+"-candidate"},
            "v4_partial_shard_observations")
    require(index["receipt_sha256"] == digest(receipt_raw)
            and index["executor_id"] == receipt["executor_id"], "v4_observation_receipt_changed")
    gold_by_case = validate_gold(gold)
    grades, reviewer_identities = [], []
    manifest_file = obj(final / "reviews/review-index.json")
    require(manifest_file["receipt_sha256"] == digest(receipt_raw)
            and manifest_file["observation_index_sha256"] == digest(index_bytes)
            and manifest_file["packet_index_sha256"] == digest(raw(final / "packets/packet-index.json")),
            "v4_review_manifest_not_bound")
    require(set(manifest_file["reviews"]) == set(observations),
            "v4_semantic_review_missing_or_extra")

    for jid, (observation, observation_sha) in observations.items():
        require(observation["case_id"] == case_id and
                observation["measurement_state"] != "operational_stop",
                "v4_operational_stop_not_a_quality_result")
        review_bytes = raw(final / "reviews" / (jid + ".review.json"))
        require(digest(review_bytes) == manifest_file["reviews"][jid],
                "v4_modified_semantic_review")
        review = json.loads(review_bytes)
        validate_review(review, observation, observation_sha,
                        gold_by_case[case_id], receipt["executor_id"])
        reviewer_identities.append(review["reviewer"])
        grades.append(score_case(observation, review, gold_by_case[case_id]))

    signed = raw(final / "semantic-attestation.json")
    signed_auth = ReviewAuthentication(raw(final / "policy.json"), signed)
    binding_values = {
        "receipt_sha256": digest(receipt_raw),
        "observation_index_sha256": digest(index_bytes),
        "packet_index_sha256": digest(raw(final / "packets/packet-index.json")),
        "gold_sha256": digest(raw(final / "gold.json")),
        "review_manifest_sha256": digest(canonical(manifest_file)),
        "execution_binding_sha256": keyed_digest(asdict(binding)),
        "plan_sha256": keyed_digest(plan),
    }
    verified = signed_auth.verify(
        evaluation_id=receipt["evaluation_id"],
        executor_id=receipt["executor_id"],
        evaluator_id=receipt["evaluator_id"],
        bindings=binding_values,
        reviewers=reviewer_identities)
    shard_score = obj(final / "first-score/report.json")
    require(shard_score.get("first_score_only") is True
            and shard_score.get("quality_go") is False
            and shard_score.get("review_authentication") == verified,
            "v4_shard_first_score_not_signed")
    prior_rows = obj(final / "first-score/rows.json")
    require(prior_rows == grades, "v4_shard_score_rows_changed")
    # Enrolled authority's create-only first-score claim must exist and match
    # this run; no report.json alone can establish completed execution.
    claims = list((final / "preparation/authority").rglob("first-score.json"))
    require(len(claims) == 1 and
            obj(claims[0])["evaluation_id"] == receipt["evaluation_id"],
            "v4_authority_first_score_claim_absent")
    return {
        "case_id": case_id,
        "session": session,
        "shard": shard,
        "global_manifest": manifest,
        "gold": source_gold,
        "rows": grades,
        "review_attestation_sha256": digest(signed),
        "source_receipt_sha256": digest(raw(final / "authority-receipt.json")),
        "first_score_sha256": digest(raw(final / "first-score/report.json")),
    }


def aggregate(root):
    root = Path(root)
    results = [read_pair(root, case) for case in EXPECTED_IDS]
    run_ids = {row["session"]["run_id"] for row in results}
    source_commits = {row["session"]["source_commit"] for row in results}
    require(len(run_ids) == 24 and len(source_commits) == 1,
            "v4_duplicate_run_or_different_source_commit")
    global_manifest_record = results[0]["global_manifest"]
    require(all(x["global_manifest"] == global_manifest_record for x in results),
            "v4_global_manifest_disagreement")
    verify_all_shards(global_manifest_record, [x["shard"] for x in results])
    all_gold = {x["case_id"]: x["gold"] for x in results}
    all_rows = [y for x in results for y in x["rows"]]
    require(len(all_gold) == 24 and len(all_rows) == 48
            and {x["job_id"] for x in all_rows} ==
            {f"{c}-{a}" for c in EXPECTED_IDS for a in ("baseline", "candidate")},
            "v4_48_unique_semantic_rows_required")
    metrics = report(all_rows, all_gold)
    require(metrics["cohort_measured"] is True,
            "v4_incomplete_global_cohort_no_f1")
    return {
        "contract": "reason-cli-real24-v4-global-first-score-v1",
        "case_run_ids": {row["case_id"]: row["session"]["run_id"] for row in results},
        "source_commit": next(iter(source_commits)),
        "global_manifest_sha256": sha(canonical(global_manifest_record)),
        "ordered_source_receipts_sha256": digest(canonical(
            [x["source_receipt_sha256"] for x in results])),
        "ordered_signed_semantic_reviews_sha256": digest(canonical(
            [x["review_attestation_sha256"] for x in results])),
        "ordered_shard_scores_sha256": digest(canonical(
            [x["first_score_sha256"] for x in results])),
        "semantic_reviewed_jobs": 48,
        "case_count": 24,
        "metrics": metrics,
        "first_score_only": True,
        "reissue_allowed": False,
        "quality_go": False,
        "previous_v3_run_reused": False,
    }


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--shards",required=True,type=Path)
    parser.add_argument("--output",required=True,type=Path)
    args=parser.parse_args()
    result=aggregate(args.shards)
    # Output parent must already be private; create-only, never overwrite a
    # score to hide an earlier, unfavorable grade.
    output = args.output.resolve()
    from contracts import _open_trusted_directory
    fd = _open_trusted_directory(output.parent)
    os.close(fd)
    descriptor=os.open(output,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(descriptor,"wb") as stream:
        stream.write(canonical(result))
        stream.flush()
        os.fsync(stream.fileno())
    print(json.dumps({"cohort_measured": True,"jobs":48,
                      "quality_go":False,"report_sha256":digest(canonical(result))}))


if __name__=="__main__":
    main()
