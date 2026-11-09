"""Same-job native controller. Every advance uses the existing signed gate."""
import argparse
from dataclasses import asdict
import json
import os
from pathlib import Path
import signal
import time

from nacl.public import PrivateKey

from annotate import make_packets, read_observations, validate_gold, validate_review
from authority import PREFIX, ScoringContext, TrustedAuthority
from ci_exchange import APPROVAL, MAX_ARCHIVE, MAX_PLAIN, Mailbox, authenticate, seal
from ci_prepare import TAG, TAG_V5, context, load, pack_files
from collect import collect
from contracts import PLAN_ID, canonical, digest, exclusive_json, keyed_digest, private_new_directory, require
from local_store import LocalAtomicStore, LocalStoreProfile
from native_admission import ADMISSION
from native_bundle import NativeBundle
from native_lane import NativeLane, private_bytes, private_snapshot
from review_auth import ReviewAuthentication
from score import score


def drive(lane, publish, receive, deadline, *, stage_prefix=""):
    """Keep the actual NativeLane/session alive through all signed decisions.

    V3 remains 48 jobs in one run. A V4 pair is exactly two jobs under its
    own receipt, unique stage namespace and global 24-case manifest.
    """
    observations = []
    # Provider runs always supply the fully authenticated NativeLane plan.
    # Keep the historical synthetic drive fixture (no plan attribute) pinned
    # to its original 48/360 contract rather than altering its test surface.
    jobs = lane.plan["jobs"] if hasattr(lane, "plan") else None
    total = len(jobs) if jobs is not None else 48
    while lane.next < total:
        maximum = jobs[lane.next]["timeout_seconds"] if jobs is not None else 360
        require(time.monotonic() + maximum + 60 <= deadline, "ci_job_deadline_before_next")
        packet, expected = lane.run_next()
        stage = stage_prefix + "job-" + str(lane.next).zfill(2)
        publish(stage, canonical({"packet": packet, "expected": expected}))
        attestation = receive(stage, expected, deadline)
        require(time.monotonic() < deadline, "ci_review_arrived_after_deadline")
        observations.append(lane.accept_review(canonical(attestation)))
    return observations


def final_inputs(root, lane, observations, operator, bundle, plan, binding, policy):
    manifest = {"contract": PLAN_ID, "provider": plan["provider"], "model": plan["model"],
                "executor_id": operator["executor_id"], "receipt_sha256": plan["receipt_sha256"],
                "rows": [{k: row[k] for k in ("job_id", "case_id", "arm", "question", "frozen_request_binding_sha256", "files")}
                         for row in observations]}
    manifest_path = root / "capture-manifest.json"
    private_bytes(manifest_path, canonical(manifest))
    collect(manifest_path, lane.output, root / "observations")
    make_packets(root / "observations", root / "gold.json", root / "packets")
    index_raw = private_snapshot(root / "observations/index.json")
    packet_raw = private_snapshot(root / "packets/packet-index.json")
    rows = read_observations(root / "observations")[1]
    gold = validate_gold(load(root / "gold.json"))
    expected = {"receipt_sha256": digest(private_snapshot(root / "receipt.json")),
        "observation_index_sha256": digest(index_raw), "packet_index_sha256": digest(packet_raw),
        "gold_sha256": digest(private_snapshot(root / "gold.json")),
        "execution_binding_sha256": keyed_digest(asdict(binding)), "plan_sha256": keyed_digest(plan)}

    def validate(value):
        require(type(value) is dict and set(value) == {"reviews", "attestation"}
                and type(value["reviews"]) is dict and set(value["reviews"]) == set(rows), "ci_final_review_inventory")
        reviewers = []
        for jid, review in value["reviews"].items():
            observation, observation_sha = rows[jid]
            validate_review(review, observation, observation_sha, gold[observation["case_id"]], operator["executor_id"])
            reviewers.append(review["reviewer"])
        review_index = {"contract": "reason-cli-normal-v3-review-index-v1", "receipt_sha256": plan["receipt_sha256"],
            "observation_index_sha256": digest(index_raw), "packet_index_sha256": digest(packet_raw),
            "executor_id": operator["executor_id"], "reviews": {jid: digest(canonical(review)) for jid, review in value["reviews"].items()}}
        ReviewAuthentication(policy, canonical(value["attestation"])).verify(
            evaluation_id=lane.session._receipt["evaluation_id"], executor_id=operator["executor_id"],
            evaluator_id=operator["evaluator_id"], bindings={**expected, "review_manifest_sha256": digest(canonical(review_index))},
            reviewers=reviewers)

    packet_files = {"observations/" + p.name: p for p in (root / "observations").iterdir()}
    packet_files.update({"packets/" + p.name: p for p in (root / "packets").iterdir()})
    packet_files["gold.json"] = root / "gold.json"
    packet = {"contract": "reason-cli-normal-v3-ci-final-request-v1", "evaluation_id": lane.session._receipt["evaluation_id"],
              "bindings": expected, "files": json.loads(pack_files(packet_files))["files"]}
    return packet, validate


def execute(root):
    root = Path(root).resolve(strict=True)
    operator, session = load(root / "operator.json"), load(root / "session.json")
    shard_id = session.get("shard_case_id")
    current = context(os.environ, "live", shard_case_id=shard_id)
    expected_tag = current["execution_tag"]
    require(all(current[k] == session[k] for k in current) and
            session["execution_tag"] == expected_tag, "ci_context_changed")
    bundle = NativeBundle(private_snapshot(root / "native-bundle.json"))
    bundle.verify()
    policy = private_snapshot(root / "policy.json")
    key = private_snapshot(root / "session-key.bin")
    require(bytes(PrivateKey(key).public_key).hex() == session["inbox_public_key_hex"], "ci_session_key_changed")
    deadline = time.monotonic() + session["job_deadline_unix"] - time.time()
    mailbox = Mailbox(run_id=session["run_id"], token=os.environ.get("GH_TOKEN"), since=session["since"],
                      sender_id=operator["review_sender_id"])
    recipient = bytes.fromhex(operator["recipient_public_key_hex"])
    request = load(root / "start-request.json")
    def verify_start(envelope):
        authenticate(envelope, policy, operator["evaluator_id"], operator["executor_id"], APPROVAL, request)
    stage_prefix = shard_id + "-" if shard_id else ""
    approval = mailbox.receive(stage_prefix + "start", key, verify_start, deadline)
    require(approval["payload"].get("decision") == "approve", "ci_start_denied")
    private_bytes(root / "start-approval.json", canonical(approval))
    readiness = load(root / "readiness.json")
    require(readiness["authentication_and_model_available"] is True
            and readiness["model_generations"] == 0 and readiness["metadata_requests_attempted"] == 1
            and readiness["checked_at_unix_seconds"] <= time.time() < readiness["valid_until_unix_seconds"], "ci_readiness_expired")
    credential = os.environ.get("MISTRAL_API_KEY")
    require(isinstance(credential, str) and bool(credential), "ci_missing_credential")
    profile = LocalStoreProfile.from_json(load(root / "preparation/operator-profile.json"))
    receipt = load(root / "authority-receipt.json")
    LocalAtomicStore(profile, role="issuer").create(f"{PREFIX}/receipts/{receipt['evaluation_id']}.json", canonical(receipt))
    binding = bundle.binding(operator["executor_id"], operator["evaluator_id"])
    authority = TrustedAuthority(LocalAtomicStore(profile, role="executor"))
    plan = load(root / "plan.json")
    run = authority.start(receipt["evaluation_id"], plan, private_snapshot(root / "receipt.json"), binding)
    lane = None
    old_handlers = {}
    def interrupt(signum, frame):
        if lane is not None:
            lane.cancel()
        else:
            run.stop("ci_controller_interrupted")
        raise KeyboardInterrupt("ci_controller_interrupted")
    try:
        for signum in (signal.SIGINT, signal.SIGTERM):
            old_handlers[signum] = signal.signal(signum, interrupt)
        lane = NativeLane(session=run, plan=plan, bundle=bundle, policy_bytes=policy,
                          catalog_bytes=private_snapshot(root / "preparation/source-catalog.json"), output=root / "lane",
                          provider_environment={"MISTRAL_API_KEY": credential})
        def receive(stage, expected, limit):
            return mailbox.receive(stage + "-review", key, lambda value: authenticate(value, policy,
                operator["evaluator_id"], operator["executor_id"], ADMISSION, expected), limit)
        def publish(stage, raw):
            payload = {**json.loads(raw), "ci_session_sha256": digest(private_snapshot(root / "session.json"))}
            mailbox.publish(stage, canonical(payload), recipient, limit=MAX_ARCHIVE if stage == "final" else MAX_PLAIN)
        observations = drive(lane, publish, receive, deadline, stage_prefix=stage_prefix)
        final_packet, validate = final_inputs(root, lane, observations, operator, bundle, plan, binding, policy)
        publish(stage_prefix + "final", canonical(final_packet))
        final = mailbox.receive(stage_prefix + "final-review", key, validate, deadline, limit=MAX_ARCHIVE)
        require(time.monotonic() < deadline, "ci_final_review_after_deadline")
        reviews = root / "reviews"; private_new_directory(reviews)
        for jid, review in final["reviews"].items():
            private_bytes(reviews / (jid + ".review.json"), canonical(review))
        index = {"contract": "reason-cli-normal-v3-review-index-v1", "receipt_sha256": plan["receipt_sha256"],
            "observation_index_sha256": digest(private_snapshot(root / "observations/index.json")),
            "packet_index_sha256": digest(private_snapshot(root / "packets/packet-index.json")),
            "executor_id": operator["executor_id"], "reviews": {jid: digest(canonical(review)) for jid, review in final["reviews"].items()}}
        private_bytes(reviews / "review-index.json", canonical(index))
        private_bytes(root / "semantic-attestation.json", canonical(final["attestation"]))
        evaluator = TrustedAuthority(LocalAtomicStore(profile, role="evaluator"))
        score(root / "observations", root / "packets", root / "gold.json", reviews, root / "receipt.json", root / "first-score",
              authoritative_score=ScoringContext(evaluator, receipt["evaluation_id"], plan, binding),
              review_authentication=ReviewAuthentication(policy, canonical(final["attestation"])))
    except BaseException:
        if not run._halted and not run._completed:
            run.stop("ci_controller_or_review_failed")
        raise
    finally:
        for signum, handler in old_handlers.items():
            signal.signal(signum, handler)


def archive(root, operator_path):
    root = Path(root).resolve(strict=True)
    operator = json.loads(Path(operator_path).read_bytes())
    files = {}
    excluded = {"session-key.bin", "bootstrap.sealed", "final.sealed"}
    for name in ("session.json", "operator.json", "policy.json", "readiness.json", "metadata-attempt.json", "native-bundle.json",
                 "requests.json", "receipt.json", "plan.json", "gold.json", "global-manifest.json", "shard.json",
                 "authority-receipt.json", "start-request.json",
                 "start-approval.json", "tag-reservation.json", "capture-manifest.json", "semantic-attestation.json", "controller.log"):
        if (root / name).is_file(): files[name] = root / name
    for name in ("lane", "observations", "packets", "reviews", "first-score", "score-ledger", "preparation/authority"):
        folder = root / name
        if folder.is_dir():
            files.update({str(path.relative_to(root)): path for path in folder.rglob("*") if path.is_file() and path.name not in excluded})
    public = root / "public"
    if not public.exists(): private_new_directory(public)
    private_bytes(public / "final.sealed", seal(pack_files(files), bytes.fromhex(operator["recipient_public_key_hex"]), limit=MAX_ARCHIVE))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("run", "archive"))
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--operator", type=Path)
    args = parser.parse_args()
    if args.action == "run":
        execute(args.root)
    else:
        require(args.operator is not None, "ci_operator_required")
        archive(args.root, args.operator)
