#!/usr/bin/env python3
"""Offline signer for externally adjudicated v3 CI review. Never judges or publishes.

The reviewer MUST inspect the exact source/output and supply a decision file.
Signing possession is not semantic evidence, and this tool makes no such claim.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import tempfile

from nacl.signing import SigningKey

from annotate import build_packet, read_observations, validate_gold, validate_review
from ci_exchange import APPROVAL, authenticate
from contracts import canonical, digest, keyed_digest, read_json, require
from native_admission import ADMISSION, verify_pre_advance
from native_lane import private_bytes, private_snapshot
from review_auth import ATTESTATION, ReviewAuthentication, validate_policy


def signer(operator, seed):
    policy = canonical(operator["review_policy"])
    aliases, keys = validate_policy(policy)
    reviewer = aliases.get(operator["evaluator_id"])
    require(reviewer is not None, "unknown_independent_reviewer")
    eligible = [key for key in keys.values() if key["subject_id"] == reviewer["subject_id"]
                and key["revoked"] is False]
    require(len(eligible) == 1, "one_independent_signer_required")
    raw = private_snapshot(seed)
    require(len(raw) == 32, "review_signing_seed_size")
    key = SigningKey(raw)
    require(key.verify_key.encode().hex() == eligible[0]["public_key_hex"], "review_signer_mismatch")
    return key, eligible[0]["key_id"], policy


def envelope(kind, payload, key, key_id):
    require(kind in (APPROVAL, ADMISSION, ATTESTATION), "unknown_review_signing_domain")
    sig = key.sign((kind + "\0").encode("ascii") + canonical(payload)).signature
    return {"contract": kind, "algorithm": "Ed25519", "key_id": key_id,
            "payload": payload, "signature_hex": sig.hex()}


def start(bootstrap, decision, operator, key, key_id, policy):
    session = read_json(bootstrap / "session.json")
    request = read_json(bootstrap / "start-request.json")
    readiness = read_json(bootstrap / "readiness.json")
    plan = read_json(bootstrap / "plan.json")
    gold = read_json(bootstrap / "gold.json")
    shard_id = session.get("shard_case_id")
    expected_tag = (("cli-normal-real24-development-v5-" + shard_id) if shard_id
                    else "cli-normal-real24-development-v3")
    require(session["mode"] == "live" and session["execution_tag"] == expected_tag,
            "not_a_live_frozen_session")
    require(session.get("repository") == "git-ksk/reasoning-harness"
            and type(session.get("run_id")) is int and session["run_id"] > 0
            and session.get("run_attempt") == 1
            and isinstance(session.get("source_commit"), str)
            and len(session["source_commit"]) == 40, "live_ci_session_identity")
    if shard_id:
        require(plan["schema"] == "engine07-operational-paired-shard-v1"
                and len(plan["jobs"]) == 2 and len(validate_gold(gold)) == 1
                and list(validate_gold(gold)) == [shard_id]
                and plan["shard"] == read_json(bootstrap / "shard.json")
                and plan["shard"]["case_id"] == shard_id
                and plan["shard"]["global_manifest_sha256"] == digest(
                    private_snapshot(bootstrap / "global-manifest.json")),
                "not_frozen_v4_paired_shard")
        # Re-derive the 24-case global manifest from the actual raw corpus
        # and all 48 pinned first model requests. A matching single SHA from
        # unreviewed bytes is not a substitute for this full comparison.
        import sys
        from pathlib import Path as _Path
        sys.path.insert(0, str(_Path(__file__).resolve().parents[1] / "normal_v4"))
        from shard_contract import global_manifest, select_case
        input_corpus = read_json(bootstrap / "input/corpus.json")
        requests = read_json(bootstrap / "requests.json")["jobs"]
        derived = global_manifest(input_corpus, requests, session["source_commit"])
        require(derived == read_json(bootstrap / "global-manifest.json")
                and select_case(derived, requests, shard_id) == plan["shard"],
                "ci_v4_global_24_source_request_review_missing")
    else:
        require(len(plan["jobs"]) == 48 and len(validate_gold(gold)) == 24,
                "not_frozen_48_24_cohort")
    require(request["session_sha256"] == digest(private_snapshot(bootstrap / "session.json"))
            and request["run_id"] == session["run_id"]
            and request["policy_sha256"] == digest(policy)
            and request["plan_sha256"] == digest(private_snapshot(bootstrap / "plan.json"))
            and request["native_bundle_sha256"] == digest(private_snapshot(bootstrap / "native-bundle.json"))
            and request["authority_receipt_sha256"] == digest(private_snapshot(bootstrap / "authority-receipt.json"))
            and request["fee_mode"] == "free-only", "ci_start_frozen_binding_mismatch")
    receipt_sha = digest(private_snapshot(bootstrap / "receipt.json"))
    require(gold["receipt_sha256"] == receipt_sha and plan["receipt_sha256"] == receipt_sha,
            "ci_start_receipt_gold_mismatch")
    require(readiness["authentication_and_model_available"] is True and readiness["model_generations"] == 0
            and readiness["metadata_requests_attempted"] == 1, "provider_not_ready")
    require(operator["fee_mode"] == "free-only" and operator["additional_charges_enabled"] is False,
            "fee_consent_not_free_only")
    require(decision.get("contract") == "reason-cli-v3-independent-reviewed-start-v1"
            and decision.get("decision") == "approve"
            and decision.get("start_request_sha256") == digest(private_snapshot(bootstrap / "start-request.json"))
            and decision.get("full_sources_reviewed") is True
            and type(decision.get("notes")) is str and len(decision["notes"].strip()) >= 30,
            "explicit_independent_bootstrap_approval_required")
    payload = {**request, "decision": "approve"}
    signed = envelope(APPROVAL, payload, key, key_id)
    authenticate(signed, policy, operator["evaluator_id"], operator["executor_id"], APPROVAL, request)
    return signed


def job(bootstrap, packet_file, decision, operator, key, key_id, policy):
    wrapper = read_json(packet_file)
    require(type(wrapper) is dict and set(wrapper) == {"packet", "expected", "ci_session_sha256"},
            "ci_job_review_input_malformed")
    packet, expected = wrapper["packet"], wrapper["expected"]
    session = read_json(bootstrap / "session.json")
    require(session.get("mode") == "live"
            and wrapper["ci_session_sha256"] == digest(private_snapshot(bootstrap / "session.json"))
            and expected["evaluation_id"] == (
                ("ci-v4-" + str(session["run_id"]) + "-" + session["shard_case_id"])
                if session.get("shard_case_id") else ("ci-v3-" + str(session["run_id"])))
            and expected["authority_receipt_sha256"] == digest(private_snapshot(bootstrap / "authority-receipt.json"))
            and expected["native_bundle_sha256"] == digest(private_snapshot(bootstrap / "native-bundle.json")),
            "ci_job_foreign_session_or_authority")
    require(packet["job_id"] == expected["job_id"]
            and (not session.get("shard_case_id") or
                 packet["job_id"] in (session["shard_case_id"] + "-baseline",
                                      session["shard_case_id"] + "-candidate"))
            and expected["packet_sha256"] == keyed_digest(packet)
            and expected["stdout_sha256"] == digest(packet["stdout"].encode("utf-8"))
            and expected["stderr_sha256"] == digest(packet["stderr"].encode("utf-8"))
            and expected["observation_sha256"] == keyed_digest(packet["observation"]),
            "ci_job_review_identity")
    require(decision.get("contract") == "reason-cli-v3-independent-reviewed-job-v1"
            and decision.get("packet_sha256") == keyed_digest(packet)
            and decision.get("decision") in ("safe", "unsafe", "unknown")
            and type(decision.get("full_sources_reviewed")) is bool
            and isinstance(decision.get("notes"), str) and len(decision["notes"].strip()) >= 30
            and type(decision.get("segments")) is dict, "explicit_independent_safe_review_required")
    payload = {**expected, "policy_sha256": digest(policy), "decision": decision["decision"],
               "full_sources_reviewed": decision["full_sources_reviewed"],
               "segments": decision["segments"], "notes": decision["notes"]}
    signed = envelope(ADMISSION, payload, key, key_id)
    if decision["decision"] == "safe":
        require(decision["full_sources_reviewed"], "safe_review_requires_complete_sources")
        verify_pre_advance(policy_bytes=policy, attestation=canonical(signed), expected=expected,
            stdout=packet["stdout"].encode("utf-8"), stderr=packet["stderr"].encode("utf-8"),
            sources=packet["full_sources"])
    else:
        # An explicit signed unsafe/unknown verdict tells CI to halt immediately.
        # Never promote one to safe or invent source/stream acknowledgments.
        authenticate(signed, policy, operator["evaluator_id"], operator["executor_id"], ADMISSION, expected)
    return signed


def _write_private_review_packet(root, name, data):
    """Extract nested review bytes without 0755 intermediate dirs on macOS.

    Root must be a newly enrolled 0700 directory. Never follow a
    packet-controlled symlink or chmod an existing caller-owned path.
    """
    path = Path(name)
    require(str(path) == name and not path.is_absolute()
            and ".." not in path.parts and "." not in path.parts,
            "ci_final_unsafe_file_name")
    target = root / path
    directory = root
    for part in path.parts[:-1]:
        directory = directory / part
        require(not directory.is_symlink(), "ci_final_symlink_parent")
        directory.mkdir(mode=0o700, exist_ok=True)
        require(not directory.is_symlink(), "ci_final_symlink_parent")
    private_bytes(target, data)


def final(bootstrap, packet_file, decisions, operator, key, key_id, policy):
    packet = read_json(packet_file)
    require(packet.get("contract") == "reason-cli-normal-v3-ci-final-request-v1"
            and type(packet.get("files")) is dict and type(packet.get("bindings")) is dict,
            "ci_final_review_request_malformed")
    session = read_json(bootstrap / "session.json")
    require(session.get("mode") == "live"
            and packet.get("ci_session_sha256") == digest(private_snapshot(bootstrap / "session.json"))
            and packet.get("evaluation_id") == (
                ("ci-v4-" + str(session["run_id"]) + "-" + session["shard_case_id"])
                if session.get("shard_case_id") else ("ci-v3-" + str(session["run_id"]))),
            "ci_final_foreign_session")
    require(decisions.get("contract") == "reason-cli-v3-independent-reviewed-final-v1"
            and decisions.get("full_sources_reviewed") is True
            and decisions.get("complete_display_reviewed") is True
            and decisions.get("final_request_sha256") == keyed_digest(packet)
            and type(decisions.get("reviews")) is dict,
            "explicit_independent_final_review_required")
    with tempfile.TemporaryDirectory(prefix="reason-v3-final-read-") as folder:
        root = Path(folder)
        os.chmod(root, 0o700)
        require(1 <= len(packet["files"]) <= 128, "ci_final_file_count_bound")
        total = 0
        for name, encoded in packet["files"].items():
            path = Path(name)
            require(str(path) == name and not path.is_absolute() and ".." not in path.parts,
                    "ci_final_unsafe_file_name")
            require(type(encoded) is str and len(encoded) <= 46 * 1024 * 1024, "ci_final_bad_encoded_file")
            raw = base64.b64decode(encoded, validate=True)
            total += len(raw)
            require(total <= 256 * 1024 * 1024, "ci_final_archive_size_bound")
            _write_private_review_packet(root, name, raw)
        raw_gold = private_snapshot(root / "gold.json")
        raw_index = private_snapshot(root / "observations/index.json")
        raw_packets = private_snapshot(root / "packets/packet-index.json")
        require(digest(raw_gold) == packet["bindings"]["gold_sha256"]
                and digest(raw_index) == packet["bindings"]["observation_index_sha256"]
                and digest(raw_packets) == packet["bindings"]["packet_index_sha256"],
                "ci_final_frozen_index_mismatch")
        gold = validate_gold(json.loads(raw_gold))
        _, observations = read_observations(root / "observations")
        packet_index = json.loads(raw_packets)
        expected_ids = set(observations)
        require(set(row["job_id"] for row in packet_index["rows"]) == expected_ids
                and len(packet_index["rows"]) == len(expected_ids),
                "ci_final_packet_inventory_mismatch")
        reviews = decisions["reviews"]
        required = 2 if session.get("shard_case_id") else 48
        require(len(observations) == len(reviews) == required
                and set(observations) == set(reviews),
                "ci_final_complete_reviews_required")
        for jid, (observed, obs_sha) in observations.items():
            case = gold[observed["case_id"]]
            validate_review(reviews[jid], observed, obs_sha, case, operator["executor_id"])
            raw = private_snapshot(root / "packets" / (jid + ".review-packet.json"))
            require(json.loads(raw) == build_packet(observed, obs_sha, case),
                    "ci_final_review_packet_changed")
            matching = [row for row in packet_index["rows"] if row["job_id"] == jid]
            require(len(matching) == 1 and matching[0]["sha256"] == digest(raw),
                    "ci_final_packet_index_mismatch")
        review_index = {
            "contract": "reason-cli-normal-v3-review-index-v1",
            "receipt_sha256": json.loads(raw_gold)["receipt_sha256"],
            "observation_index_sha256": digest(raw_index),
            "packet_index_sha256": digest(raw_packets),
            "executor_id": operator["executor_id"],
            "reviews": {jid: digest(canonical(v)) for jid, v in reviews.items()},
        }
    payload = {
        "contract": ATTESTATION,
        "evaluation_id": packet["evaluation_id"],
        "executor_id": operator["executor_id"],
        "evaluator_id": operator["evaluator_id"],
        "policy_sha256": digest(policy),
        "complete_display_reviewed": True,
        "full_sources_reviewed": True,
        **packet["bindings"], "review_manifest_sha256": digest(canonical(review_index)),
    }
    signed = envelope(ATTESTATION, payload, key, key_id)
    ReviewAuthentication(policy, canonical(signed)).verify(
        evaluation_id=packet["evaluation_id"], executor_id=operator["executor_id"],
        evaluator_id=operator["evaluator_id"], bindings={
            **packet["bindings"], "review_manifest_sha256": payload["review_manifest_sha256"]
        }, reviewers=[review["reviewer"] for review in reviews.values()])
    return {"reviews": reviews, "attestation": signed}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=("start", "job", "final"))
    parser.add_argument("--operator", type=Path, required=True)
    parser.add_argument("--signing-key", type=Path, required=True)
    parser.add_argument("--decision", type=Path, required=True)
    parser.add_argument("--bootstrap", type=Path)
    parser.add_argument("--packet", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    op = read_json(args.operator)
    key, key_id, policy = signer(op, args.signing_key)
    decision = read_json(args.decision)
    require(args.bootstrap is not None, "missing_bootstrap")
    if args.stage == "start":
        result = start(args.bootstrap, decision, op, key, key_id, policy)
    elif args.stage == "job":
        require(args.packet is not None, "missing_job_packet")
        result = job(args.bootstrap, args.packet, decision, op, key, key_id, policy)
    else:
        require(args.packet is not None, "missing_final_request")
        result = final(args.bootstrap, args.packet, decision, op, key, key_id, policy)
    private_bytes(args.output, canonical(result))
    print(json.dumps({"stage": args.stage, "signature_written": True,
                      "signed_content_sha256": digest(canonical(result))}))


if __name__ == "__main__":
    main()
