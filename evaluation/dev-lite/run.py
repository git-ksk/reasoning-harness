#!/usr/bin/env python3
"""Small, repeatable *development-only* comparison of two pinned native CLIs.

No reviewer keys, Ed25519 signatures, single-use Git tags or formal F1. This
does NOT relax the legacy/v6 formal acceptance workflow. No provider access in
plan mode; live mode requires explicit free-only confirmation and credentials.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from uuid import uuid4

DEFAULT_CASES = ("rw-01", "rw-06", "rw-07", "rw-23")
CASE_ID = re.compile(r"rw-(?:0[1-9]|1[0-9]|2[0-4])\Z")
ARMS = ("baseline", "candidate")
REPO = Path(__file__).resolve().parents[2]
CONTRACT = "reason-development-lite-v1"


class GateError(Exception):
    pass


def need(condition, reason):
    if not condition:
        raise GateError(reason)


def sha(path):
    h = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def read_json(path):
    with Path(path).open("r", encoding="utf-8") as stream:
        return json.load(stream)


def flag(argv, name):
    need(argv.count(name) == 1 and argv.index(name) + 1 < len(argv) - 1,
         "missing_or_duplicate_" + name)
    return argv[argv.index(name) + 1]


def replace_flag(argv, name, value):
    argv[argv.index(name) + 1] = str(value)


def cases_from_string(raw):
    cases = raw.split(",")
    need(4 <= len(cases) <= 6, "four_to_six_cases_required")
    need(len(set(cases)) == len(cases) and all(CASE_ID.fullmatch(c) for c in cases),
         "invalid_or_duplicate_cases")
    return cases


def prepare(requests_path, build_path, cases, caps):
    """Verify local developer inputs and their distinct pinned binaries."""
    requests_path = Path(requests_path).resolve(strict=True)
    build_path = Path(build_path).resolve(strict=True)
    requests, build = read_json(requests_path), read_json(build_path)
    need(requests.get("contract") == "reason-cli-normal-v3-prospective-requests-v1",
         "wrong_preparation_contract")
    need(build.get("contract") == "reason-cli-normal-v3-native-paired-build-v1",
         "wrong_build_contract")
    need(re.fullmatch(r"[0-9a-f]{40}", str(build.get("cli_commit", ""))) is not None,
         "invalid_cli_source_commit")
    need(build.get("release_or_adoption") is False and set(build["arms"]) == set(ARMS),
         "wrong_dev_build_arms")
    jobs = requests.get("jobs")
    need(isinstance(jobs, list) and len(jobs) == 48, "expected_original_48_jobs")
    by_case = {}
    for job in jobs:
        cid, arm = job.get("case_id"), job.get("arm")
        need(isinstance(cid, str) and CASE_ID.fullmatch(cid) and arm in ARMS,
             "invalid_prepared_job")
        need(job.get("job_id") == cid + "-" + arm, "job_id_mismatch")
        pair = by_case.setdefault(cid, {})
        need(arm not in pair, "duplicate_prepared_arm")
        pair[arm] = job
    need(len(by_case) == 24 and all(set(pair) == set(ARMS) for pair in by_case.values()),
         "incomplete_prepared_pairs")
    selected = []
    frozen = {}
    for case_index, cid in enumerate(cases):
        need(cid in by_case, "unavailable_case")
        pair = by_case[cid]
        first, second = (pair[a]["request"] for a in ARMS)
        need(first["request_sha256"] == second["request_sha256"] and
             first["request"] == second["request"], "first_requests_not_paired")
        for arm in (ARMS if case_index % 2 == 0 else ARMS[::-1]):
            job = pair[arm]
            argv = list(job["argv"])
            bin_path = Path(argv[0]).resolve(strict=True)
            proof = build["arms"][arm]
            need(bin_path == Path(proof["binary"]).resolve(strict=True),
                 "binary_build_path_mismatch")
            need(sha(bin_path) == proof["binary_sha256"], "binary_hash_mismatch")
            need(flag(argv, "--provider") == "mistral" and
                 flag(argv, "--model") == "ministral-8b-2512" and
                 flag(argv, "--format") == "json" and "--plain" in argv,
                 "provider_or_json_profile_changed")
            config = Path(flag(argv, "--config")).resolve(strict=True)
            need(config.name == cid + ".json", "case_config_mismatch")
            need(argv[-1] and argv[-1] == pair[ARMS[0]]["argv"][-1],
                 "pair_question_mismatch")
            for f, cap in (("--max-model-calls", caps["model_calls"]),
                           ("--max-tokens", caps["generation_tokens"]),
                           ("--max-output-tokens", caps["output_tokens"]),
                           ("--max-total-tokens", caps["total_tokens"])):
                value = flag(argv, f)
                need(value.isdigit() and 0 < cap <= int(value), "budget_exceeds_frozen_" + f)
                replace_flag(argv, f, cap)
            need("--evaluation-isolated-config" not in argv, "duplicated_isolation_flag")
            argv.insert(-1, "--evaluation-isolated-config")
            selected.append({"case_id": cid, "arm": arm, "job_id": job["job_id"],
                             "argv": argv, "binary_sha256": proof["binary_sha256"],
                             "config_sha256": sha(config), "config_path": str(config),
                             "first_request_sha256": job["request"]["request_sha256"]})
            frozen[str(bin_path)] = proof["binary_sha256"]
    return {"contract": CONTRACT, "classification": "DEVELOPMENT_ONLY_NOT_ACCEPTANCE",
            "quality_go": False, "source_commit": build["cli_commit"],
            "requests_sha256": sha(requests_path), "build_sha256": sha(build_path),
            "cases": cases, "caps": caps, "jobs": selected,
            "frozen_binaries": frozen, "formal_f1": None,
            "reviewer_signatures_required": False, "one_shot_git_tag_required": False}



def native_preflight(plan, timeout=30):
    """Actually invoke each native first-request planner, with NO model key.

    Recomputing after development caps change is mandatory: the historic
    frozen first-request digest alone does not prove today's request identity.
    """
    pairs = {}
    with tempfile.TemporaryDirectory(prefix="reason-dev-lite-preflight-") as dirname:
        root = Path(dirname)
        for job in plan["jobs"]:
            destination = root / (job["job_id"] + ".json")
            need(sha(job["argv"][0]) == job["binary_sha256"] and
                 sha(job["config_path"]) == job["config_sha256"],
                 "native_source_changed_before_preflight")
            try:
                proc = subprocess.run(job["argv"] + ["--evaluation-request-plan", str(destination)],
                    cwd=REPO, env={"PATH": "/usr/bin:/bin", "HOME": os.environ.get("HOME", "")},
                    timeout=timeout, capture_output=True, check=False)
            except subprocess.TimeoutExpired as exc:
                raise GateError("native_preflight_timeout") from exc
            need(proc.returncode == 0 and not proc.stdout and not proc.stderr and destination.is_file(),
                 "native_preflight_failed")
            data = read_json(destination)
            need(data.get("contract") == "reason-native-first-request-plan-v1" and
                 data.get("provider") == "mistral" and data.get("model") == "ministral-8b-2512" and
                 data.get("question") == job["argv"][-1], "native_preflight_identity_changed")
            need(re.fullmatch(r"[0-9a-f]{64}", str(data.get("request_sha256", ""))) is not None,
                 "native_request_hash_missing")
            need(sha(job["argv"][0]) == job["binary_sha256"] and
                 sha(job["config_path"]) == job["config_sha256"],
                 "native_source_changed_after_preflight")
            pairs.setdefault(job["case_id"], {})[job["arm"]] = (
                data["request_sha256"], data["request"])
    need(all(set(pairs[c]) == set(ARMS) and pairs[c]["baseline"] == pairs[c]["candidate"]
             for c in plan["cases"]), "development_first_requests_not_identical")
    return {"mode": "native_provider_free_preflight", "cases_verified": len(pairs),
            "native_plans_verified": len(plan["jobs"]),
            "request_sha256_by_case": {c: pairs[c]["baseline"][0] for c in plan["cases"]},
            "model_api_calls": 0, "formal_f1": None, "quality_go": False}


def private_file(path, data):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
    except BaseException:
        raise


def json_bytes(data):
    return (json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def usage_from_cli(raw):
    data = json.loads(raw)
    need(data.get("schema_version") == "reason-cli-output-v1" and
         data.get("command") == "ask", "wrong_cli_output_envelope")
    result = data.get("result", {})
    used = result.get("usage", {}).get("run")
    if used is None:
        used = result.get("partial_diagnostics", {}).get("usage", {}).get("run")
    need(isinstance(used, dict) and used.get("tracked_from_session_start") is True,
         "unknown_provider_usage")
    fields = ("model_calls", "provider_attempts", "input_tokens", "output_tokens", "total_tokens")
    need(all(type(used.get(f)) is int and used[f] >= 0 for f in fields),
         "unaccounted_provider_tokens")
    need(used["provider_attempts"] > 0 and used["model_calls"] > 0 and
         used["input_tokens"] + used["output_tokens"] == used["total_tokens"],
         "inconsistent_provider_usage")
    status = result.get("finalization", {}).get("status")
    return {f: used[f] for f in fields}, status, result.get("status")


def run_live(plan, output, *, confirmed, token, timeout, total_budget):
    need(confirmed and os.environ.get("REASON_DEV_LITE_FREE_CONFIRMED") == "1",
         "explicit_current_free_only_confirmation_required")
    need(isinstance(token, str) and bool(token), "MISTRAL_API_KEY_required")
    need(total_budget >= len(plan["jobs"]) * plan["caps"]["total_tokens"],
         "global_budget_does_not_cover_all_reserved_jobs")
    output = Path(output).expanduser().resolve()
    need(output != REPO and REPO not in output.parents, "private_output_must_be_outside_repository")
    output.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    need(not output.exists(), "run_output_must_be_new")
    output.mkdir(mode=0o700)
    output.chmod(0o700)
    manifest = {k: v for k, v in plan.items() if k != "jobs"}
    manifest.update({"created_at_utc": datetime.now(timezone.utc).isoformat(),
                     "job_count": len(plan["jobs"]), "run_token_budget": total_budget,
                     "status": "incomplete", "result": [], "total_known_tokens": 0})
    private_file(output / "plan.json", json_bytes(plan))
    private_file(output / "status-initial.json", json_bytes(manifest))
    known = 0
    status = "complete"
    try:
        for i, job in enumerate(plan["jobs"]):
            if known + plan["caps"]["total_tokens"] > total_budget:
                status = "stopped_global_token_reserve"
                break
            if sha(job["argv"][0]) != job["binary_sha256"] or (
                sha(job["config_path"]) != job["config_sha256"]):
                status = "stopped_input_changed"
                break
            name = job["job_id"]
            started = time.monotonic()
            env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"),
                   "MISTRAL_API_KEY": token, "NO_COLOR": "1",
                   "HOME": os.environ.get("HOME", ""),
                   "NO_PROXY": "127.0.0.1"}
            # Each CLI child has its own limited model calls and token ceiling.
            # Do not retry a timeout, ambiguous response, or budget violation.
            try:
                completed = subprocess.run(job["argv"], cwd=REPO, env=env,
                                           capture_output=True, timeout=timeout, check=False)
                raw_out, raw_err = completed.stdout, completed.stderr
                rc = completed.returncode
            except subprocess.TimeoutExpired as exc:
                raw_out, raw_err, rc = exc.stdout or b"", exc.stderr or b"", None
            private_file(output / (name + ".stdout"), raw_out)
            private_file(output / (name + ".stderr"), raw_err)
            record = {"case_id": job["case_id"], "arm": job["arm"],
                      "exit_code": rc, "elapsed_seconds": round(time.monotonic() - started, 3),
                      "stdout_sha256": hashlib.sha256(raw_out).hexdigest(),
                      "stderr_sha256": hashlib.sha256(raw_err).hexdigest(),
                      "usage": None, "review_status": "unreviewed",
                      "automatic_f1": None}
            try:
                used, answer_status, cli_status = usage_from_cli(raw_out)
                record["usage"] = used
                record["answer_status"] = answer_status
                known += used["total_tokens"]
                if used["model_calls"] > plan["caps"]["model_calls"] or used["total_tokens"] > plan["caps"]["total_tokens"]:
                    status = "stopped_reported_budget_exceeded"
                elif rc != 0 or cli_status == "failed":
                    status = "stopped_cli_or_model_failure"
            except (GateError, ValueError, TypeError, KeyError) as exc:
                record["parse_or_usage_failure"] = type(exc).__name__
                status = "stopped_unknown_usage_or_output"
            if rc is None:
                status = "stopped_timeout"
            record["status"] = status if status != "complete" else "observed_unreviewed"
            manifest["result"].append(record)
            manifest["total_known_tokens"] = known
            # Each checkpoint is create-only, even after a crash. No blind resume.
            private_file(output / f"checkpoint-{i+1:02d}.json", json_bytes({**manifest,"status":status}))
            if status != "complete":
                break
    except BaseException:
        status = "stopped_unhandled_or_interrupted"
        raise
    finally:
        manifest["status"] = status
        manifest["total_known_tokens"] = known
        manifest["completed_jobs"] = len(manifest["result"])
        private_file(output / "final-summary.json", json_bytes(manifest))
    return manifest


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--requests", required=True, type=Path, help="Existing 48-job developer preparation requests.json")
    parser.add_argument("--build", required=True, type=Path, help="Matching two-arm pinned native build.json")
    parser.add_argument("--cases", default=",".join(DEFAULT_CASES))
    parser.add_argument("--mode", choices=("plan", "preflight", "live"), default="plan")
    parser.add_argument("--max-calls", type=int, default=6)
    parser.add_argument("--generation-tokens", type=int, default=512)
    parser.add_argument("--output-tokens", type=int, default=1536)
    parser.add_argument("--total-tokens", type=int, default=12000)
    parser.add_argument("--timeout-seconds", type=int, default=360)
    parser.add_argument("--output", type=Path, help="Fresh, private output directory (outside repo)")
    parser.add_argument("--confirm-no-extra-charges", action="store_true")
    parser.add_argument("--allow-older-build", action="store_true",
                        help="Explicitly compare a pinned CLI that is older than checkout HEAD")
    args = parser.parse_args(argv)
    try:
        need(1 <= args.max_calls <= 12 and 1 <= args.generation_tokens <= 1024 and
             1 <= args.output_tokens <= 4096 and 1 <= args.total_tokens <= 28096 and
             1 <= args.timeout_seconds <= 360, "invalid_development_caps")
        caps = {"model_calls": args.max_calls, "generation_tokens": args.generation_tokens,
                "output_tokens": args.output_tokens, "total_tokens": args.total_tokens}
        cases = cases_from_string(args.cases)
        plan = prepare(args.requests, args.build, cases, caps)
        if args.mode == "plan":
            print(json.dumps({"mode": "provider_free", "classification":plan["classification"],
                "source_commit":plan["source_commit"], "cases":cases,
                "jobs":len(plan["jobs"]), "caps_per_arm":caps,
                "worst_case_reserved_tokens":len(plan["jobs"])*caps["total_tokens"],
                "model_api_calls":0, "formal_f1":None, "quality_go":False},indent=2))
            return 0
        verified = native_preflight(plan)
        if args.mode == "preflight":
            print(json.dumps(verified, indent=2))
            return 0
        current_head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO).decode().strip()
        need(plan["source_commit"] == current_head or args.allow_older_build,
             "pinned_cli_older_than_checkout_use_--allow-older-build")
        need(args.output is not None, "explicit_private_output_required")
        result = run_live(plan, args.output, confirmed=args.confirm_no_extra_charges,
                          token=os.environ.get("MISTRAL_API_KEY"),
                          timeout=args.timeout_seconds,
                          total_budget=len(plan["jobs"])*caps["total_tokens"])
        print(json.dumps({"status":result["status"], "completed_jobs":result["completed_jobs"],
                          "total_known_tokens":result["total_known_tokens"],
                          "quality_go":False, "output":str(args.output)}, indent=2))
        return 0 if result["status"] == "complete" else 2
    except GateError as exc:
        print("development gate refused: "+str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
