"""Freeze a same-job macOS evaluation before any model generation."""
import argparse
import base64
from dataclasses import asdict
from datetime import datetime, timezone
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import stat
import time
import urllib.parse
import urllib.request

from nacl.public import PrivateKey

from annotate import validate_gold
from authority import NATIVE_RECEIPT, freeze_jobs, validate_frozen_receipt
from ci_exchange import MAX_ARCHIVE, NoRedirect, REPOSITORY, seal
from contracts import canonical, digest, exclusive_json, frozen_request_digest, private_new_directory, require
from http_limits import FROZEN_HTTP_LIMITS
from local_store import LocalStoreProfile
from native_bundle import NativeBundle
from native_lane import private_bytes, private_snapshot, plan_first_request
from prepare_local import INPUT_M1, MODEL, PURPOSE, checked_input, prepare, verify_request_pairs
from review_auth import validate_policy

TAG = "cli-normal-real24-development-v3"
WORKFLOW = ".github/workflows/cli-normal-real24-development-v3.yml"
TAG_V4 = "cli-normal-real24-development-v4"
WORKFLOW_V4 = ".github/workflows/cli-normal-real24-development-v4.yml"
INPUT_COMMIT = "6165de77c1544a7fe58fc1e675e7cb3a8ec7cc98"
INPUT_PATH = "evaluation/engine073-realworld-independent-v1-input"


def load(path):
    return json.loads(private_snapshot(path))


def context(environment, mode, *, shard_case_id=None):
    require(environment.get("GITHUB_REPOSITORY") == REPOSITORY
            and environment.get("GITHUB_RUN_ATTEMPT") == "1", "ci_first_attempt_required")
    source = environment.get("GITHUB_SHA", "")
    require(re.fullmatch(r"[0-9a-f]{40}", source) is not None, "ci_source_commit_required")
    run_id = environment.get("GITHUB_RUN_ID", "")
    require(run_id.isdecimal() and int(run_id) > 0, "ci_run_id_required")
    shard = shard_case_id is not None
    if shard:
        require(re.fullmatch(r"rw-(?:0[1-9]|1[0-9]|2[0-4])", shard_case_id) is not None
                and environment.get("V4_CASE_ID") == shard_case_id,
                "ci_v4_invalid_case_shard")
    tag = TAG_V4 if shard else TAG
    if mode == "live":
        require(environment.get("GITHUB_EVENT_NAME") == "push"
                and environment.get("GITHUB_REF") == "refs/tags/" + tag,
                "ci_live_one_shot_tag_required")
    else:
        require(mode == "prepare"
                and environment.get("GITHUB_EVENT_NAME") == "workflow_dispatch",
                "ci_prepare_dispatch_required")
    started = float(environment["V4_JOB_STARTED_UNIX" if shard else "V3_JOB_STARTED_UNIX"])
    allowed = 290 * 60 if shard else 330 * 60
    require(0 <= time.time() - started < allowed, "ci_job_deadline")
    result = {"repository": REPOSITORY, "source_commit": source, "run_id": int(run_id),
            "run_attempt": 1, "mode": mode,
            "execution_tag": tag if mode == "live" else None,
            "job_deadline_unix": started + allowed}
    if shard:
        result["shard_case_id"] = shard_case_id
    return result


def fetch(url, headers=None, maximum=4 * 1024 * 1024):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    with opener.open(urllib.request.Request(url, headers=headers or {}), timeout=30) as response:
        raw = response.read(maximum + 1)
    require(len(raw) <= maximum, "ci_download_limit")
    return raw


def materialize(repo, destination):
    private_new_directory(destination)
    listing = subprocess.check_output(["git", "ls-tree", "-r", "--name-only", INPUT_COMMIT, INPUT_PATH], cwd=repo).decode().splitlines()
    for name in listing:
        relative = Path(name).relative_to(INPUT_PATH)
        require(len(relative.parts) == 1 and relative.suffix == ".json", "ci_input_inventory")
        private_bytes(destination / relative, subprocess.check_output(["git", "show", INPUT_COMMIT + ":" + name], cwd=repo))
    require(digest(private_snapshot(destination / "manifest.json")) == INPUT_M1, "ci_input_manifest_changed")
    corpus = load(destination / "corpus.json")
    for document in corpus["documents"].values():
        url = urllib.parse.urlsplit(document["immutable_capture_url"])
        require(url.scheme == "https" and url.netloc == "raw.githubusercontent.com"
                and not url.query and not url.fragment and re.search(r"/[0-9a-f]{40}/", url.path), "ci_immutable_source_required")
        raw = fetch(url.geturl())
        require(digest(raw) == document["raw_sha256"] and document["normalization"] == "utf8-plain-whitespace-v1",
                "ci_source_capture_changed")
        normalized = " ".join(raw.decode("utf-8").split()).encode()
        require(digest(normalized) == document["normalized_sha256"], "ci_normalized_source_changed")
        for field, value in (("raw_path", raw), ("normalized_path", normalized)):
            path = destination / document[field]
            require(not Path(document[field]).is_absolute() and ".." not in Path(document[field]).parts,
                    "ci_source_path_escape")
            path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
            private_bytes(path, value)
    checked_input(destination)


def metadata(key):
    require(isinstance(key, str) and key and "\n" not in key, "ci_missing_provider_credential")
    started = int(time.time())
    data = json.loads(fetch("https://api.mistral.ai/v1/models", {"Authorization": "Bearer " + key}))
    require(any(row.get("id") == MODEL and row.get("capabilities", {}).get("completion_chat") is True
                for row in data.get("data", [])), "ci_model_unavailable")
    return {"provider": "mistral", "model": MODEL, "http_status": 200,
            "authentication_and_model_available": True, "metadata_requests_attempted": 1,
            "model_generations": 0, "checked_at_unix_seconds": started,
            "valid_until_unix_seconds": started + 86400, "quota_headroom": "UNKNOWN"}


def gold_cases(corpus, obligations):
    questions = {row["id"]: row["question"] for row in corpus["cases"]}
    records = [row for row in obligations["records"] if row["track"] == "e2e_normal"]
    require(len(records) == len(questions) == 24 and {row["case_id"] for row in records} == set(questions),
            "ci_normal_gold_inventory")
    cases = [{"case_id": row["case_id"], "question": questions[row["case_id"]], "label": row["label"],
              "required_groups": [group["id"] for group in row["groups"]]} for row in records]
    return cases


def make_plan(requests, bundle, receipt_raw, executor, evaluator, cases):
    verify_request_pairs(requests)
    budget = {"max_generations": 24, "max_input_tokens_per_generation": 12512,
              "max_output_tokens_per_generation": 1024, "max_attempts_per_generation": 1}
    jobs = [{"id": row["job_id"], "arm": row["arm"], "contract": "normal", "argv": row["argv"],
             "native_request_plan": row["request"], "budget": dict(budget), "http_limits": dict(FROZEN_HTTP_LIMITS),
             "timeout_seconds": 360, "cancel_grace_seconds": 1} for row in requests]
    receipt_sha = digest(receipt_raw)
    questions = {row["case_id"]: row["question"] for row in cases}
    bindings = {}
    for row in requests:
        require(row["request"]["model"] == MODEL and row["request"]["provider"] == "mistral", "ci_model_request_changed")
        bindings[row["job_id"]] = frozen_request_digest(job_id=row["job_id"], case_id=row["case_id"], arm=row["arm"],
            question=questions[row["case_id"]], request=row["request"]["request"],
            request_sha256=row["request"]["request_sha256"], provider="mistral", model=MODEL, receipt_sha256=receipt_sha)
    plan = {"schema": "engine07-operational-plan-v1", "purpose": PURPOSE, "provider": "mistral", "model": MODEL,
            "case_ids": list(questions), "questions": questions, "jobs": jobs, "receipt_sha256": receipt_sha,
            "frozen_request_bindings": bindings, "token_budget": 48 * 24 * (12512 + 1024)}
    binding = bundle.binding(executor, evaluator)
    return plan, binding


def pack_files(files):
    require(all(not Path(name).is_absolute() and ".." not in Path(name).parts for name in files), "ci_archive_paths")
    encoded = {}
    total = 0
    for name, path in files.items():
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as stream:
            before = os.fstat(stream.fileno())
            require(stat.S_ISREG(before.st_mode) and before.st_uid in (0, os.getuid())
                    and not before.st_mode & 0o022 and before.st_size <= MAX_ARCHIVE, "ci_archive_file_unsafe")
            raw = stream.read(MAX_ARCHIVE + 1)
            after = os.fstat(stream.fileno())
        total += len(raw)
        require(total <= MAX_ARCHIVE * 3 // 4 and
                (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) ==
                (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns), "ci_archive_file_changed_or_limit")
        encoded[name] = base64.b64encode(raw).decode()
    result = canonical({"contract": "reason-cli-normal-v3-ci-private-files-v1", "files": encoded})
    require(len(result) <= MAX_ARCHIVE, "ci_archive_limit")
    return result


def prepare_ci(repo, root, operator_path, mode, *, shard_case_id=None, tag_proof_path=None):
    repo, root = Path(repo).resolve(strict=True), Path(root).resolve(strict=True)
    session = context(os.environ, mode, shard_case_id=shard_case_id)
    shard = shard_case_id is not None
    tag = TAG_V4 if shard else TAG
    workflow = WORKFLOW_V4 if shard else WORKFLOW
    require(load(root / "build/build.json")["cli_commit"] == session["source_commit"], "ci_build_source_mismatch")
    operator_raw = Path(operator_path).read_bytes()
    expected_operator = subprocess.check_output(["git", "show", session["source_commit"] +
        ":evaluation/cli-repair-paired-v1/normal_v3/ci-operator.json"], cwd=repo)
    require(operator_raw == expected_operator, "ci_operator_not_in_source_commit")
    operator = json.loads(operator_raw)
    require(canonical(operator) == operator_raw, "ci_operator_config_noncanonical")
    policy_raw = canonical(operator["review_policy"])
    validate_policy(policy_raw)
    require(operator["model"] == MODEL and operator["fee_mode"] == "free-only"
            and operator["additional_charges_enabled"] is False, "ci_fee_conditions_required")
    private_bytes(root / "operator.json", operator_raw)
    for name, source in (("workflow.yml", workflow), ("tag-guard.py", "evaluation/cli-repair-paired-v1/tag_admission.py")):
        raw = subprocess.check_output(["git", "show", session["source_commit"] + ":" + source], cwd=repo)
        require(raw == (repo / source).read_bytes(), "ci_checkout_control_changed")
        private_bytes(root / name, raw)
    if mode == "live":
        require(operator["owner_free_mode_confirmed"] is True, "ci_owner_free_mode_required")
        guard_path = root / "tag-guard.py"
        spec = importlib.util.spec_from_file_location("v3_reviewed_tag_guard", guard_path)
        guard = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(guard)
        if shard:
            require(tag_proof_path is not None, "v4_global_tag_proof_required")
            import sys
            sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v4"))
            from tag_gate import verify_proof
            raw = Path(tag_proof_path).read_bytes()
            require(len(raw) <= 2 * 1024 * 1024 and
                    digest(raw) == os.environ.get("V4_TAG_GUARD_SHA256"),
                    "v4_tag_proof_artifact_digest_mismatch")
            proof = json.loads(raw)
            require(canonical(proof) == raw, "v4_tag_proof_noncanonical")
            verify_proof(proof, os.environ)
            private_bytes(root / "tag-reservation.json", raw)
        else:
            history = guard.collect_history(REPOSITORY)
            proof = guard.single_first_push(history, tag, session["source_commit"], session["run_id"],
                                            expected_tag=tag, workflow_path=workflow)
            exclusive_json(root / "tag-reservation.json", proof)
        # Record the attempted read before accessing the credential or network.
        exclusive_json(root / "metadata-attempt.json", {"maximum_requests": 1, "model_generations": 0})
        readiness = metadata(os.environ.get("MISTRAL_API_KEY"))
    else:
        readiness = {"purpose": "CI_PROVIDER_FREE_PREPARATION", "model_generations": 0, "metadata_requests_attempted": 0}
    exclusive_json(root / "readiness.json", readiness)
    key = PrivateKey.generate()
    private_bytes(root / "session-key.bin", bytes(key))
    session["inbox_public_key_hex"] = bytes(key.public_key).hex()
    session["github_bot_id"] = operator["github_bot_id"]
    session["since"] = datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")
    exclusive_json(root / "session.json", session)
    input_root, previous = root / "input", root / "preparation"
    materialize(repo, input_root)
    prepare(inputs=input_root, build=root / "build", output=previous)
    old = NativeBundle(private_snapshot(previous / "native-bundle.json"))
    old.verify()
    configs = {name.removeprefix("config/"): record["path"] for name, record in old.data["files"].items()
               if name.startswith("config/")}
    private_bytes(root / "policy.json", policy_raw)
    configs.update({"policy": root / "policy.json", "ci-operator": root / "operator.json", "ci-session": root / "session.json",
                    "ci-readiness": root / "readiness.json", "ci-authority-profile": previous / "operator-profile.json",
                    "ci-workflow": root / "workflow.yml", "ci-tag-guard": root / "tag-guard.py"})
    bundle = NativeBundle.enroll(source_roots=old.data["source_roots"], binaries={arm: old.binary(arm) for arm in ("baseline", "candidate")},
                                 configurations=configs)
    private_bytes(root / "native-bundle.json", bundle.manifest_bytes)
    requests = []
    private_new_directory(root / "requests")
    for row in load(previous / "requests.json")["jobs"]:
        planned = plan_first_request(bundle, row["arm"], row["argv"], root / "requests" / (row["job_id"] + ".json"))
        require(planned["request"] == row["request"]["request"] and planned["request_sha256"] == row["request"]["request_sha256"],
                "ci_enrollment_changed_request")
        requests.append({**row, "request": planned})
    verify_request_pairs(requests)
    exclusive_json(root / "requests.json", {"jobs": requests})
    private_new_directory(root / "score-ledger")
    receipt = {"contract": "reason-cli-normal-v3-ci-development-receipt-v1", "purpose": PURPOSE,
               "session_sha256": digest(private_snapshot(root / "session.json")), "native_bundle_sha256": bundle.sha256,
               "readiness_sha256": digest(private_snapshot(root / "readiness.json")), "operator_sha256": digest(operator_raw),
               "score_ledger_dir": str(root / "score-ledger"), "fee_mode": "free-only", "model": MODEL,
               "replay_allowed": False}
    receipt_raw = canonical(receipt)
    private_bytes(root / "receipt.json", receipt_raw)
    corpus, obligations = checked_input(input_root)
    cases = gold_cases(corpus, obligations)
    if shard:
        import sys
        from pathlib import Path as _Path
        sys.path.insert(0, str(_Path(__file__).resolve().parents[1] / "normal_v4"))
        from shard_contract import global_manifest, select_case, make_shard_plan
        manifest = global_manifest(corpus, requests, session["source_commit"])
        selection = select_case(manifest, requests, shard_case_id)
        private_bytes(root / "global-manifest.json", canonical(manifest))
        private_bytes(root / "shard.json", canonical(selection))
        plan = make_shard_plan(manifest=manifest, selected=shard_case_id,
             requests=requests, cases=cases, receipt_sha256=digest(receipt_raw),
             provider="mistral", model=MODEL, purpose=PURPOSE, timeout_seconds=1200)
        require(plan["shard"] == selection, "ci_v4_manifest_mismatch")
        binding = bundle.binding(operator["executor_id"], operator["evaluator_id"])
        cases = [case for case in cases if case["case_id"] == shard_case_id]
        require(len(cases) == 1 and len(plan["jobs"]) == 2, "ci_v4_exact_paired_shard")
    else:
        plan, binding = make_plan(requests, bundle, receipt_raw, operator["executor_id"], operator["evaluator_id"], cases)
    private_bytes(root / "plan.json", canonical(plan))
    gold = {"contract": "reason-cli-normal-v3-gold-v1", "receipt_sha256": digest(receipt_raw), "cases": cases}
    validate_gold(gold)
    private_bytes(root / "gold.json", canonical(gold))
    evaluation_id = ("ci-v4-" + str(session["run_id"]) + "-" + shard_case_id) if shard else ("ci-v3-" + str(session["run_id"]))
    authority_receipt = {"contract": NATIVE_RECEIPT, "evaluation_id": evaluation_id,
        "plan_sha256": digest(canonical(plan)), "legacy_receipt_sha256": digest(receipt_raw), **asdict(binding),
        "provider": "mistral", "model": MODEL, "token_budget": plan["token_budget"], "jobs": freeze_jobs(plan),
        "native_admission": {"bundle_sha256": bundle.sha256, "policy_sha256": digest(policy_raw),
                             "sources_sha256": digest(private_snapshot(previous / "source-catalog.json"))}}
    validate_frozen_receipt(authority_receipt, plan, receipt_raw, **asdict(binding))
    private_bytes(root / "authority-receipt.json", canonical(authority_receipt))
    approval = {"contract": "reason-cli-normal-v3-ci-approval-v1", "session_sha256": digest(private_snapshot(root / "session.json")),
                "native_bundle_sha256": bundle.sha256, "plan_sha256": digest(canonical(plan)),
                "authority_receipt_sha256": digest(canonical(authority_receipt)), "policy_sha256": digest(policy_raw),
                "run_id": session["run_id"], "stage": "start", "fee_mode": "free-only"}
    private_bytes(root / "start-request.json", canonical(approval))
    files = {name: root / name for name in ("session.json", "operator.json", "policy.json", "readiness.json", "native-bundle.json",
        "requests.json", "receipt.json", "plan.json", "gold.json", "authority-receipt.json", "start-request.json")}
    if shard:
        files.update({name: root / name for name in ("global-manifest.json", "shard.json")})
    for name, record in bundle.data["files"].items():
        files["bundle-files/" + name] = Path(record["path"])
    files.update({"input/" + str(p.relative_to(input_root)): p for p in input_root.rglob("*") if p.is_file()})
    public = root / "public"; private_new_directory(public)
    private_bytes(public / "bootstrap.sealed", seal(pack_files(files), bytes.fromhex(operator["recipient_public_key_hex"]), limit=MAX_ARCHIVE))
    require(bundle.verify() == bundle.sha256, "ci_bundle_changed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--operator", required=True, type=Path)
    parser.add_argument("--mode", required=True, choices=("prepare", "live"))
    parser.add_argument("--shard-case-id", default=None)
    parser.add_argument("--tag-proof",type=Path,default=None)
    args = parser.parse_args()
    prepare_ci(args.repo,args.root,args.operator,args.mode,
               shard_case_id=args.shard_case_id,tag_proof_path=args.tag_proof)
