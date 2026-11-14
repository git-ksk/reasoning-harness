#!/usr/bin/env python3
"""One-shot, public fictional four-case, two-arm Mistral development experiment.

Not the private 24-case evaluation, independently reviewed model quality, or a release gate.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time

IDS = ("p04", "p15", "n01", "n13")
ARMS = ("baseline", "candidate")
BASELINE = "4abee90f0501f0d8fc09b453a0b45c942fbed69e"
CANDIDATE = "7466c6f90d420a98c99713b96d6a0623e2aeef52"
CLI = "988b69f086ac4a209fe93eb10cf1fa6786977567"
MODEL = "ministral-8b-2512"
TOTAL_CAP = 12000
MAX_CALLS = 6
MAX_GEN = 512
MAX_OUTPUT = 1536
BUDGET_CASES = len(IDS) * len(ARMS)
MAX_USD_ESTIMATE = BUDGET_CASES * TOTAL_CAP * .15 / 1_000_000


def checked(value, why):
    if not value:
        raise RuntimeError(why)


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def scope(raw):
    return {k: {"kind": "values", "values": [raw[k]]} for k in sorted(raw)}


def make_config(case_id, source_text, positive):
    sid = "synthetic-608:" + case_id
    timestamp = 1800000000
    raw_scope = {"project": "quartz-lantern"}
    facts = {"quartz-lantern.launch_month": "2026-06"} if positive else {
        "quartz-lantern.event_status": "not confirmed"
    }
    source = {
        "quote": source_text,
        "context_summary": "One standalone fictional development document.",
        "document_id": "synthetic-doc-" + case_id,
        "url": "https://example.invalid/synthetic/" + case_id,
        "locator": "example:" + case_id,
        "source_version": "dev-608-v1",
        "origin_group": case_id,
        "scope": raw_scope,
        "synthetic_mutation": False,
    }
    metadata = {
        "observed_at_unix_seconds": timestamp,
        "retrieved_at_unix_seconds": timestamp,
        "scope": scope(raw_scope),
        "claimed_authority_class": "primary",
    }
    record = {
        "id": "snapshot:" + case_id + ":0",
        "source": sid,
        "observation": json.dumps(source, sort_keys=True),
        "facts": facts,
        "acquisition_metadata": metadata,
    }
    admission = {
        "authority_ranks": {"primary": 20, "reference_host": 20, "untrusted": 0},
        "minimum_authority_class": "primary",
        "required_scope": scope(raw_scope),
        "evaluation_time_unix_seconds": timestamp + 1,
        "sources": {
            sid: {
                "authority_class": "primary",
                "max_age_seconds": 86400,
                "scope": scope(raw_scope),
            }
        },
    }
    evidence = {
        "id": record["id"],
        "source": sid,
        "observation": record["observation"],
        "facts": dict(sorted(facts.items())),
        "metadata": {
            "temporal": {
                "effective_from_unix_seconds": timestamp,
                "effective_until_unix_seconds": timestamp + 86400,
            },
            "scope": scope(raw_scope),
            "provenance_class": "primary",
        },
    }
    evidence_hash = hashlib.sha256(
        json.dumps(evidence, ensure_ascii=False, separators=(",", ":")).encode()
    ).hexdigest()
    registry = [
        {
            "proposition": {"key": key, "value": value},
            "evidence_id": evidence["id"],
            "source_id": sid,
            "evidence_sha256": evidence_hash,
            "quote_pointer": "/quote",
            "url_pointer": "/url",
            "version_pointer": "/source_version",
            "url": source["url"],
            "source_version": source["source_version"],
        }
        for key, value in sorted(facts.items())
    ]
    return {
        "schema_version": "reason-config-v1",
        "resolution": {
            "investigation": {
                "capabilities": [
                    {
                        "id": "synthetic-source",
                        "kind": "captured_evidence",
                        "read_only": True,
                        "supported_fact_keys": sorted(
                            set(facts) | {"quartz-lantern.launch_month"}
                        ),
                        "evidence": [record],
                        "admission": admission,
                    }
                ],
                "max_actions": 4,
                "max_no_progress_rounds": 2,
                "max_rounds": 3,
                "max_targets": 3,
                "planner_max_tokens": 512,
            }
        },
        "answer_context": {
            "schema_version": "reason-answer-context-config-v1",
            "sources": registry,
            "unresolved_inputs": [],
        },
    }


def cases_from_public(repo, folder):
    cases = read_json(
        repo / "evaluation/engine073-development-quality-probe-v1/cases.json"
    )
    indexed = {row["id"]: row for row in cases}
    checked(set(IDS).issubset(indexed), "missing_public_cases")
    folder.mkdir(mode=0o700, parents=True, exist_ok=False)
    rows = []
    for case_id in IDS:
        row = indexed[case_id]
        checked(row["entity"] == "Quartz Lantern", "case_entity_changed")
        checked(len(row["signals"]) >= 1, "source_signal_missing")
        quote = row["signals"][-1]["text"]
        positive = row["expected_relevant"]
        checked(type(positive) is bool and type(quote) is str, "invalid_case")
        cfg = folder / (case_id + ".json")
        cfg.write_text(json.dumps(make_config(case_id, quote, positive), indent=2) + "\n")
        rows.append(
            {
                "id": case_id,
                "config": str(cfg),
                "question": "When was Quartz Lantern launched? Give the month and year if supported; otherwise say the source does not establish a launch date.",
                "source_excerpt": quote,
                "expected_relevant": positive,
            }
        )
    (folder / "cases.json").write_text(json.dumps(rows, indent=2) + "\n")
    return rows


def build_checked(path):
    d = read_json(path)
    checked(d["cli_commit"] == CLI, "wrong_cli")
    checked(d["arms"]["baseline"]["engine_commit"] == BASELINE, "wrong_baseline")
    checked(d["arms"]["candidate"]["engine_commit"] == CANDIDATE, "wrong_candidate")
    checked(d["release_or_adoption"] is False, "wrong_release_state")
    for arm in ARMS:
        checked(sha(d["arms"][arm]["binary"]) == d["arms"][arm]["binary_sha256"],
                "changed_binary_" + arm)
    return d


def argv(row, arm, build, index, *, plan=None):
    args = [
        build["arms"][arm]["binary"],
        "--plain", "--provider", "mistral", "--model", MODEL,
        "--seed", str(49200 + index),
        "--max-tokens", str(MAX_GEN), "--safety-profile", "current",
        "--max-model-calls", str(MAX_CALLS),
        "--max-output-tokens", str(MAX_OUTPUT),
        "--max-total-tokens", str(TOTAL_CAP),
        "--config", row["config"], "--format", "json",
        "--evaluation-isolated-config",
        row["question"],
    ]
    if plan is not None:
        args += ["--evaluation-request-plan", str(plan)]
    return args


def preflight(repo, root, rows, build):
    digests = {}
    for index, row in enumerate(rows):
        arm_plans = []
        for arm in ARMS:
            destination = root / (row["id"] + "-" + arm + "-plan.json")
            output = subprocess.run(
                argv(row, arm, build, index, plan=destination),
                cwd=repo,
                env={"PATH": os.environ.get("PATH", ""), "HOME": os.environ.get("HOME", "")},
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=35,
            )
            checked(output.returncode == 0 and output.stdout == b"" and output.stderr == b"",
                    "first_request_planner_failed_" + row["id"] + "_" + arm)
            data = read_json(destination)
            checked(data["provider"] == "mistral" and data["model"] == MODEL, "provider_changed")
            checked(data["question"] == row["question"], "question_changed")
            arm_plans.append(data)
        checked(arm_plans[0]["request"] == arm_plans[1]["request"],
                "paired_request_changed_" + row["id"])
        checked(arm_plans[0]["request_sha256"] == arm_plans[1]["request_sha256"],
                "paired_sha_changed_" + row["id"])
        digests[row["id"]] = arm_plans[0]["request_sha256"]
    return digests


def live(repo, root, rows, build):
    checked(os.environ.get("REASON_DEV_LITE_FREE_CONFIRMED") == "1",
            "explicit_free_allowance_confirmation_missing")
    checked(os.environ.get("MISTRAL_API_KEY", "") != "", "secret_not_present")
    checked(MAX_USD_ESTIMATE < .05 and MAX_USD_ESTIMATE < 9.98,
            "budget_not_safe_against_user_reported_allowance")
    sys.path.insert(0, str(repo / "evaluation/dev-lite"))
    import run as lite
    summary = {
        "contract": "reason-608-synthetic-mistral-development-v1",
        "classification": "DEVELOPMENT_ONLY_NOT_FORMAL_ACCEPTANCE",
        "model": MODEL,
        "run_status": "incomplete",
        "jobs_planned": BUDGET_CASES,
        "jobs_completed": 0,
        "source_cli_commit": CLI,
        "baseline": BASELINE,
        "candidate": CANDIDATE,
        "price_usd_per_million_assumed": .15,
        "max_tokens_all_jobs": BUDGET_CASES * TOTAL_CAP,
        "max_price_usd_estimate": MAX_USD_ESTIMATE,
        "user_reported_remaining_free_allowance_usd": 9.98,
        "total_observed_tokens": 0,
        "cases": [],
        "formal_f1": None,
        "quality_go": False,
        "adoption_go": False,
    }
    status = "complete"
    try:
        for i, row in enumerate(rows):
            for arm in (ARMS if i % 2 == 0 else ARMS[::-1]):
                checked(summary["total_observed_tokens"] + TOTAL_CAP <=
                        BUDGET_CASES * TOTAL_CAP, "global_cap_exceeded")
                started = time.monotonic()
                completed = subprocess.run(
                    argv(row, arm, build, i),
                    cwd=repo,
                    env={
                        "PATH": os.environ.get("PATH", ""),
                        "HOME": os.environ.get("HOME", ""),
                        "MISTRAL_API_KEY": os.environ["MISTRAL_API_KEY"],
                        "NO_COLOR": "1",
                        "NO_PROXY": "127.0.0.1",
                    },
                    capture_output=True,
                    timeout=270,
                )
                raw = completed.stdout
                # Keep raw model answers private on the ephemeral runner only.
                private = root / (row["id"] + "-" + arm + ".stdout.json")
                private.write_bytes(raw)
                os.chmod(private, 0o600)
                result = {
                    "case_id": row["id"],
                    "arm": arm,
                    "expected_relevant": row["expected_relevant"],
                    "exit_code": completed.returncode,
                    "elapsed_seconds": round(time.monotonic() - started, 2),
                    "stdout_sha256": hashlib.sha256(raw).hexdigest(),
                    "parseable": False,
                    "answer": None,
                    "citation_sources": [],
                    "usage": None,
                }
                summary["cases"].append(result)
                summary["jobs_completed"] += 1
                try:
                    usage, answer_status, cli_status = lite.usage_from_cli(raw)
                    data = json.loads(raw)
                    result["usage"] = usage
                    result["answer_status"] = answer_status
                    result["cli_status"] = cli_status
                    summary["total_observed_tokens"] += usage["total_tokens"]
                    checked(usage["model_calls"] <= MAX_CALLS and
                            usage["total_tokens"] <= TOTAL_CAP, "usage_over_cap")
                    final = data["result"].get("finalization")
                    answer = final.get("text") if isinstance(final, dict) else None
                    context = data["result"].get("answer_context")
                    citations = context.get("citations") if isinstance(context, dict) else []
                    result["parseable"] = True
                    # Synthetic case inputs are already public; this is an
                    # unreviewed truncated display, not a quality label.
                    result["answer"] = answer[:800] if isinstance(answer, str) else None
                    result["citation_sources"] = [
                        c.get("source_id") for c in citations if isinstance(c, dict)
                        and isinstance(c.get("source_id"), str)
                    ][:8]
                    if completed.returncode or cli_status == "failed":
                        status = "stopped_cli_failure"
                        break
                except (ValueError, KeyError, TypeError, lite.GateError, RuntimeError) as exc:
                    result["failure_class"] = type(exc).__name__
                    status = "stopped_unknown_usage_or_output"
                    break
                print("observed_case=" + row["id"] + " arm=" + arm +
                      " model_calls=" + str(result["usage"]["model_calls"]) +
                      " total_tokens=" + str(result["usage"]["total_tokens"]),
                      flush=True)
            if status != "complete":
                break
    except subprocess.TimeoutExpired:
        status = "stopped_timeout_no_retry"
    finally:
        summary["run_status"] = status
        output = root / "sanitized-summary.json"
        output.write_text(json.dumps(summary, indent=2) + "\n")
        os.chmod(output, 0o600)
        print("model_quality_formal_f1=null")
        print("run_status=" + status)
        print("completed_jobs=" + str(summary["jobs_completed"]))
        print("total_known_tokens=" + str(summary["total_observed_tokens"]))
        print("max_price_usd_estimate=" + str(MAX_USD_ESTIMATE))
    checked(status == "complete", "live_run_incomplete_" + status)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--mode", choices=("preflight", "live"), required=True)
    args = parser.parse_args()
    repo = args.repo.resolve(strict=True)
    build = build_checked(args.build)
    root = args.output.resolve()
    checked(not root.exists(), "new_private_output_required")
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    os.chmod(root, 0o700)
    rows = cases_from_public(repo, root / "configs")
    hashes = preflight(repo, root, rows, build)
    print("native_case_pairs_verified=" + str(len(hashes)), flush=True)
    print("native_request_plans_verified=" + str(len(hashes) * 2), flush=True)
    if args.mode == "live":
        live(repo, root, rows, build)
    else:
        print("provider_calls=0")
        print("formal_f1=null")
        print("max_price_usd_estimate=" + str(MAX_USD_ESTIMATE))


if __name__ == "__main__":
    main()