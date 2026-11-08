"""Prospective new real24 paired shard protocol; no model or authority calls.

A shard contains one fixed case with BOTH arms, pinned to the same immutable
24-case global manifest. A complete global cohort always requires 24 distinct
shards and 48 independently reviewed observations. This module cannot issue
runtime permission, evaluate answers, or authorize any retry.
"""
import hashlib
import json

MANIFEST_CONTRACT = "reason-cli-real24-v4-global-manifest-v1"
SHARD_CONTRACT = "reason-cli-real24-v4-paired-shard-v1"
PLAN_SCHEMA = "engine07-operational-paired-shard-v1"
EXPECTED_IDS = tuple(f"rw-{i:02d}" for i in range(1, 25))
ARMS = ("baseline", "candidate")


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode("utf-8")


def sha(value):
    return hashlib.sha256(value).hexdigest()


def validate_requests(cases, requests):
    if (type(cases) is not list or len(cases) != 24
            or tuple(row["id"] for row in cases) != EXPECTED_IDS):
        raise ValueError("v4_frozen_24_case_order")
    if type(requests) is not list or len(requests) != 48:
        raise ValueError("v4_frozen_48_request_inventory")
    by_id = {}
    for row in requests:
        if type(row) is not dict:
            raise ValueError("v4_request_schema")
        job_id = row.get("job_id")
        case_id = row.get("case_id")
        arm = row.get("arm")
        if (case_id not in EXPECTED_IDS or arm not in ARMS
                or job_id != f"{case_id}-{arm}" or job_id in by_id):
            raise ValueError("v4_duplicate_or_mismatched_job")
        planned = row.get("request", {})
        if not isinstance(planned, dict) or planned.get("question") != next(x["question"] for x in cases if x["id"] == case_id):
            raise ValueError("v4_question_mismatch")
        request = planned.get("request")
        if not isinstance(request, dict) or not isinstance(planned.get("request_sha256"), str):
            raise ValueError("v4_missing_frozen_model_request")
        by_id[job_id] = row
    if set(by_id) != {f"{cid}-{arm}" for cid in EXPECTED_IDS for arm in ARMS}:
        raise ValueError("v4_incomplete_pair_inventory")
    return by_id


def global_manifest(corpus, requests, source_commit):
    """Frozen identities only, never content-bearing private model requests."""
    cases = corpus["cases"]
    by_id = validate_requests(cases, requests)
    if not (type(source_commit) is str and len(source_commit) == 40
            and all(c in "0123456789abcdef" for c in source_commit)):
        raise ValueError("v4_source_commit_required")
    source_ids = sorted(corpus["sources"])
    docs = corpus["documents"]
    if not docs or not source_ids:
        raise ValueError("v4_missing_source_context")
    manifest = {
        "contract": MANIFEST_CONTRACT,
        "source_commit": source_commit,
        "frozen_input_sha256": sha(canonical(corpus)),
        "case_count": 24,
        "job_count": 48,
        "case_order": list(EXPECTED_IDS),
        "documents": {doc_id: {"raw_sha256": doc["raw_sha256"],
                                "normalized_sha256": doc["normalized_sha256"]}
                      for doc_id, doc in sorted(docs.items())},
        "cases": [],
    }
    for idx, case in enumerate(cases):
        cid = case["id"]
        manifest["cases"].append({
            "case_id": cid,
            "index": idx,
            "question_sha256": sha(case["question"].encode("utf-8")),
            "source_ids": list(case["source_ids"]),
            "first_request_digests": {
                arm: by_id[f"{cid}-{arm}"]["request"]["request_sha256"]
                for arm in ARMS
            },
        })
    return manifest


def select_case(manifest, requests, case_id):
    if (type(manifest) is not dict or manifest.get("contract") != MANIFEST_CONTRACT
            or manifest.get("case_count") != 24 or manifest.get("job_count") != 48
            or manifest.get("case_order") != list(EXPECTED_IDS)):
        raise ValueError("v4_global_manifest_invalid")
    if case_id not in EXPECTED_IDS:
        raise ValueError("v4_nonfrozen_case_requested")
    case = manifest["cases"][EXPECTED_IDS.index(case_id)]
    if case.get("case_id") != case_id or case.get("index") != EXPECTED_IDS.index(case_id):
        raise ValueError("v4_shard_case_index_mismatch")
    chosen = [r for r in requests if r.get("case_id") == case_id]
    if len(chosen) != 2 or {r["arm"] for r in chosen} != set(ARMS):
        raise ValueError("v4_incomplete_shard_pair")
    for row in chosen:
        if row["request"]["request_sha256"] != case["first_request_digests"][row["arm"]]:
            raise ValueError("v4_shard_model_request_changed")
    return {
        "contract": SHARD_CONTRACT,
        "global_manifest_sha256": sha(canonical(manifest)),
        "case_id": case_id,
        "case_index": EXPECTED_IDS.index(case_id),
        "total_cases": 24,
        "case_question_sha256": case["question_sha256"],
        "arm_order": [row["arm"] for row in chosen],
        "job_ids": [row["job_id"] for row in chosen],
        "first_request_digests": case["first_request_digests"],
    }


def verify_all_shards(manifest, shards):
    """Verify the full paired cohort before any cross-shard score or claim."""
    if type(shards) is not list or len(shards) != 24:
        raise ValueError("v4_all_24_shards_required")
    found = {}
    digest = sha(canonical(manifest))
    for shard in shards:
        if (type(shard) is not dict or shard.get("contract") != SHARD_CONTRACT
                or shard.get("global_manifest_sha256") != digest
                or shard.get("total_cases") != 24):
            raise ValueError("v4_shard_global_manifest_conflict")
        cid = shard.get("case_id")
        if cid not in EXPECTED_IDS or cid in found:
            raise ValueError("v4_duplicate_or_out_of_scope_shard")
        row = manifest["cases"][EXPECTED_IDS.index(cid)]
        if (shard.get("case_index") != row["index"]
                or shard.get("case_question_sha256") != row["question_sha256"]
                or shard.get("first_request_digests") != row["first_request_digests"]
                or set(shard.get("job_ids", [])) != {f"{cid}-{arm}" for arm in ARMS}
                or len(shard["job_ids"]) != 2):
            raise ValueError("v4_shard_tamper_or_incomplete_pair")
        found[cid] = shard
    if set(found) != set(EXPECTED_IDS):
        raise ValueError("v4_missing_case_shard")
    return [found[cid] for cid in EXPECTED_IDS]


def make_shard_plan(*, manifest, selected, requests, cases, receipt_sha256,
                    provider="mistral", model="ministral-8b-2512", purpose=None,
                    timeout_seconds=1200):
    """Build an independent paired run from the same full frozen 48 requests."""
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))
    from contracts import frozen_request_digest

    if not isinstance(receipt_sha256, str) or len(receipt_sha256) != 64:
        raise ValueError("v4_receipt_required_before_plan")
    if not isinstance(timeout_seconds, int) or not 360 <= timeout_seconds <= 1200:
        raise ValueError("v4_bounded_shard_timeout")
    record = select_case(manifest, requests, selected)
    question = next((case.get("question") for case in cases if case.get("case_id") == selected), None)
    if not isinstance(question, str) or sha(question.encode("utf-8")) != record["case_question_sha256"]:
        raise ValueError("v4_gold_question_not_frozen")
    gold = next(case for case in cases if case["case_id"] == selected)
    if gold["label"] not in ("positive", "negative") or not isinstance(gold["required_groups"], list):
        raise ValueError("v4_invalid_gold_case")
    native = [row for row in requests if row["case_id"] == selected]
    budget = {"max_generations": 24, "max_input_tokens_per_generation": 12512,
              "max_output_tokens_per_generation": 1024, "max_attempts_per_generation": 1}
    jobs = []
    bindings = {}
    for row in native:
        jid,arm = row["job_id"],row["arm"]
        request = row["request"]
        argv = row["argv"]
        if not argv or argv[-1] != question:
            raise ValueError("v4_native_question_not_frozen")
        jobs.append({"id": jid, "arm": arm, "contract": "normal",
                     "argv": argv, "native_request_plan": request,
                     "budget": dict(budget), "http_limits": dict(row["http_limits"]),
                     "timeout_seconds": timeout_seconds, "cancel_grace_seconds": 1})
        bindings[jid] = frozen_request_digest(
            job_id=jid, case_id=selected, arm=arm, question=question,
            request=request["request"], request_sha256=request["request_sha256"],
            provider=provider, model=model, receipt_sha256=receipt_sha256)
    plan = {"schema": PLAN_SCHEMA,
            "purpose": purpose or "REAL24_PAIRED_SHARD_V4_DEVELOPMENT_DIAGNOSTIC",
            "provider": provider, "model": model,
            "case_ids": [selected], "questions": {selected: question},
            "jobs": jobs, "receipt_sha256": receipt_sha256,
            "frozen_request_bindings": bindings,
            "shard": record,
            "token_budget": 2*budget["max_generations"]*
            (budget["max_input_tokens_per_generation"]+budget["max_output_tokens_per_generation"]),
            }
    return plan
