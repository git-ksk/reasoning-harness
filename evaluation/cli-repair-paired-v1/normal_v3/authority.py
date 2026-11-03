"""#580: prospective v3 operator-owned run/first-score authority.

The normal single-host backend is LocalAtomicStore. A shared/cloud backend is
an explicit optional extension. This module never selects storage from receipt
data and never authorizes a provider call. Native-runtime, HTTP/accounting and
semantic-review acceptance remain separate from durable local claims.
"""
import json
import re
from dataclasses import dataclass

from contracts import canonical, digest, safe_id, sha_field

RECEIPT = "reason-cli-normal-v3-authority-receipt-v1"
NATIVE_RECEIPT = "reason-cli-normal-v3-native-authority-receipt-v1"
RUN_CLAIM = "reason-cli-normal-v3-authority-run-claim-v1"
RESERVATION = "reason-cli-normal-v3-authority-reservation-v1"
INFLIGHT = "reason-cli-normal-v3-authority-inflight-v1"
OBSERVED = "reason-cli-normal-v3-authority-observed-v1"
COMPLETE = "reason-cli-normal-v3-authority-complete-v1"
SCORE_CLAIM = "reason-cli-normal-v3-authority-first-score-v1"
COHORT_CLAIM = "reason-cli-normal-v3-authority-cohort-claim-v1"
PREFIX = "reason-cli-normal-v3/authority/v1"
RECEIPT_KEYS = {
    "contract", "evaluation_id", "plan_sha256", "legacy_receipt_sha256",
    "source_sha256", "image_sha256", "cli_build_sha256", "runtime_sha256",
    "provider", "model", "executor_id", "evaluator_id", "token_budget", "jobs",
}
JOB_KEYS = {"job_id", "case_id", "arm", "reserve_tokens",
            "max_provider_attempts", "question_sha256",
            "frozen_request_binding_sha256"}


class AuthorityFailure(ValueError):
    """An untrusted/missing binding, ambiguous state or unavailable authority."""


def check(ok, message):
    if not ok:
        raise AuthorityFailure(message)


def _json_bytes(obj):
    return canonical(obj)


def _key(evaluation_id, section):
    return f"{PREFIX}/claims/{evaluation_id}/{section}.json"


def _cohort_sha256(receipt):
    """Stable run cohort even when receipt IDs, local ledgers or budget change.

    Operator admission must independently attest the source/image/CLI/runtime.
    This fingerprint deliberately excludes local receipt bytes, plan/claim IDs,
    first-request hashes (which include receipt IDs), and mutable budgets.
    It cannot prevent maliciously changed external identities, changed bucket,
    or abuse by a privileged issuer: those are operator admission gates.
    """
    return digest(canonical({
        "source_sha256": receipt["source_sha256"],
        "image_sha256": receipt["image_sha256"],
        "cli_build_sha256": receipt["cli_build_sha256"],
        "runtime_sha256": receipt["runtime_sha256"],
        "provider": receipt["provider"], "model": receipt["model"],
        "jobs": sorted(({
            "job_id": j["job_id"], "case_id": j["case_id"],
            "arm": j["arm"], "question_sha256": j["question_sha256"],
        } for j in receipt["jobs"]), key=lambda j: j["job_id"]),
    }))


def _cohort_key(receipt):
    return f"{PREFIX}/cohorts/{_cohort_sha256(receipt)}.json"


def _cohort_record(receipt, authority_sha):
    return {
        "contract": COHORT_CLAIM,
        "evaluation_id": receipt["evaluation_id"],
        "plan_sha256": receipt["plan_sha256"],
        "cohort_sha256": _cohort_sha256(receipt),
        "authority_receipt_sha256": authority_sha,
        "reissue_allowed": False,
    }


def _job_reserve(budget):
    check(type(budget) is dict and set(budget) == {
        "max_generations", "max_input_tokens_per_generation",
        "max_output_tokens_per_generation", "max_attempts_per_generation",
    } and all(type(value) is int and 0 < value <= 1000000
              for value in budget.values()), "invalid_frozen_job_budget")
    return (budget["max_generations"] *
            (budget["max_input_tokens_per_generation"] +
             budget["max_output_tokens_per_generation"]) *
            budget["max_attempts_per_generation"])


def freeze_jobs(plan):
    """Recompute the 24 paired-case inventory, not a caller-provided copy."""
    check(type(plan) is dict and plan.get("schema") == "engine07-operational-plan-v1",
          "invalid_frozen_plan")
    cases, questions, jobs = (plan.get(name) for name in ("case_ids", "questions", "jobs"))
    bindings = plan.get("frozen_request_bindings")
    check(type(cases) is list and len(cases) == 24
          and all(type(c) is str for c in cases)
          and len(set(cases)) == 24
          and type(questions) is dict and set(questions) == set(cases)
          and type(bindings) is dict
          and type(jobs) is list and len(jobs) == 48, "invalid_paired_cohort")
    check(type(plan.get("token_budget")) is int and plan["token_budget"] > 0,
          "invalid_arm_budget")
    expected = set()
    for case_id in cases:
        safe_id(case_id, "invalid_authority_case_id")
        for arm in ("baseline", "candidate"):
            expected.add(f"{case_id}-{arm}")
        question = questions[case_id]
        check(type(question) is str and bool(question.strip()), "invalid_frozen_question")
    check(set(bindings) == expected, "unexpected_frozen_request_binding")
    inventory = []
    seen = set()
    for job in jobs:
        check(type(job) is dict, "invalid_authority_job")
        job_id = job.get("id")
        check(type(job_id) is str and job_id in expected and job_id not in seen,
              "invalid_job_pair")
        case_id, arm = job_id.rsplit("-", 1)
        check(job.get("arm") == arm and job.get("contract") == "normal",
              "invalid_job_arm")
        reserve = _job_reserve(job.get("budget"))
        sha_field(bindings.get(job_id), "invalid_request_binding")
        inventory.append({
            "job_id": job_id, "case_id": case_id, "arm": arm,
            "reserve_tokens": reserve,
            "max_provider_attempts": (
                job["budget"]["max_generations"] *
                job["budget"]["max_attempts_per_generation"]
            ),
            "question_sha256": digest(questions[case_id].encode("utf-8")),
            "frozen_request_binding_sha256": bindings[job_id],
        })
        seen.add(job_id)
    check(seen == expected, "unexpected_job_inventory")
    # The execution order remains frozen by the entire canonical plan hash.
    return inventory


def validate_frozen_receipt(receipt, plan, local_receipt, *,
                            source_sha256, image_sha256, cli_build_sha256,
                            runtime_sha256, executor_id, evaluator_id):
    native_required = type(receipt) is dict and receipt.get("contract") == NATIVE_RECEIPT
    check(type(receipt) is dict and set(receipt) == RECEIPT_KEYS | ({"native_admission"} if native_required else set()),
          "invalid_authority_receipt_schema")
    check(receipt["contract"] in (RECEIPT, NATIVE_RECEIPT), "invalid_authority_receipt_version")
    if native_required:
        profile = receipt["native_admission"]
        check(type(profile) is dict and set(profile) == {"bundle_sha256", "policy_sha256", "sources_sha256"}
              and profile["bundle_sha256"] == image_sha256, "invalid_native_authority_profile")
        for value in profile.values(): sha_field(value, "invalid_native_authority_profile_hash")
    safe_id(receipt["evaluation_id"], "invalid_evaluation_id")
    check(type(executor_id) is str and type(evaluator_id) is str
          and executor_id != evaluator_id, "authority_principal_separation_required")
    for name in ("executor_id", "evaluator_id"):
        safe_id(receipt[name], "invalid_authority_principal")
    check(receipt["executor_id"] == executor_id
          and receipt["evaluator_id"] == evaluator_id,
          "authority_principal_mismatch")
    for name, expected in (
        ("plan_sha256", digest(canonical(plan))),
        ("legacy_receipt_sha256", digest(local_receipt)),
        ("source_sha256", source_sha256), ("image_sha256", image_sha256),
        ("cli_build_sha256", cli_build_sha256),
        ("runtime_sha256", runtime_sha256),
    ):
        sha_field(expected, "invalid_expected_" + name)
        check(receipt[name] == expected, "authority_" + name + "_mismatch")
    check(receipt["provider"] == plan.get("provider")
          and receipt["model"] == plan.get("model")
          and type(receipt["model"]) is str and bool(receipt["model"]),
          "authority_provider_or_model_mismatch")
    check(receipt["token_budget"] == plan.get("token_budget"),
          "authority_arm_budget_mismatch")
    check(receipt["jobs"] == freeze_jobs(plan), "authority_job_inventory_mismatch")
    check(sum(j["reserve_tokens"] for j in receipt["jobs"]) <= receipt["token_budget"],
          "authority_total_reservations_exceed_budget")
    return receipt


class GCSAtomicStore:
    """Optional distributed extension; NOT required for Core/CLI/local evaluation.

    The operator must separately attest IAM: publisher writes immutable receipts;
    executor can READ receipts and CREATE claim objects only, never update/delete
    receipts or claims. GCS bucket retention/immutability is a deployment gate.
    The SDK is an optional *runtime* dependency, not imported for offline tests.
    """
    def __init__(self, operator_bucket, *, client=None):
        check(type(operator_bucket) is str and
              re.fullmatch(r"[a-z0-9][a-z0-9._-]{2,221}", operator_bucket)
              is not None, "operator_bucket_required")
        if client is None:
            try:
                from google.cloud import storage
            except ImportError as error:
                raise AuthorityFailure("google_cloud_storage_sdk_required") from error
            client = storage.Client()
        self.bucket = client.bucket(operator_bucket)

    def read(self, key):
        try:
            from google.api_core.exceptions import NotFound
        except ImportError:
            NotFound = ()  # SDK unavailable: fake test client only.
        try:
            return self.bucket.blob(key).download_as_bytes()
        except NotFound:
            return None

    def create(self, key, data):
        check(type(data) is bytes, "authority_claim_bytes_required")
        try:
            from google.api_core.exceptions import PreconditionFailed
        except ImportError:
            PreconditionFailed = ()
        try:
            self.bucket.blob(key).upload_from_string(
                data, content_type="application/json", if_generation_match=0
            )
        except PreconditionFailed as error:
            raise FileExistsError("authoritative_claim_exists") from error


@dataclass(frozen=True)
class AuthorityBinding:
    """Frozen execution artifacts. A native profile pins a native bundle hash
    in the legacy image_sha256 field; it does not require a container image.
    Historical receipts retain their original artifact meaning.
    """
    source_sha256: str
    image_sha256: str
    cli_build_sha256: str
    runtime_sha256: str
    executor_id: str
    evaluator_id: str


@dataclass(frozen=True)
class ScoringContext:
    """Operator-injected context, normally a fixed local profile on one host.
    Never infer the authority from an output directory or receipt URI.
    """
    authority: object
    evaluation_id: str
    plan: dict
    binding: AuthorityBinding


class TrustedAuthority:
    """Explicit fixed storage, no receipt-selected location or implicit retry."""
    def __init__(self, store):
        check(callable(getattr(store, "read", None))
              and callable(getattr(store, "create", None)),
              "operator_owned_atomic_store_required")
        self._store = store

    def _receipt(self, evaluation_id):
        safe_id(evaluation_id, "invalid_evaluation_id")
        blob = self._store.read(f"{PREFIX}/receipts/{evaluation_id}.json")
        check(type(blob) is bytes, "missing_authority_receipt")
        try:
            parsed = json.loads(blob)
        except (ValueError, UnicodeError) as error:
            raise AuthorityFailure("malformed_authority_receipt") from error
        check(canonical(parsed) == blob, "noncanonical_authority_receipt")
        check(type(parsed) is dict and parsed.get("evaluation_id") == evaluation_id,
              "authority_receipt_identity_mismatch")
        return parsed, digest(blob)

    def _create(self, key, value):
        # Creating the exact named key is the one atomic multi-host operation:
        # conditional create, no overwrite, no best-effort local lock.
        self._store.create(key, canonical(value))

    def start(self, evaluation_id, plan, local_receipt, binding):
        receipt, authority_sha = self._receipt(evaluation_id)
        validate_frozen_receipt(
            receipt, plan, local_receipt,
            source_sha256=binding.source_sha256,
            image_sha256=binding.image_sha256,
            cli_build_sha256=binding.cli_build_sha256,
            runtime_sha256=binding.runtime_sha256,
            executor_id=binding.executor_id, evaluator_id=binding.evaluator_id,
        )
        # A new evaluation ID must not create a fresh spend/score authority
        # for the exact same immutable plan. Ambiguous ACK burns this cohort.
        self._create(_cohort_key(receipt),
                     _cohort_record(receipt, authority_sha))
        self._create(_key(evaluation_id, "run"), {
            "contract": RUN_CLAIM, "evaluation_id": evaluation_id,
            "authority_receipt_sha256": authority_sha,
            "plan_sha256": receipt["plan_sha256"],
            "executor_id": binding.executor_id,
            "retry_allowed": False,
        })
        return RunSession(self._store, receipt, authority_sha)

    def claim_score(self, evaluation_id, plan, local_receipt, binding, *,
                    observation_index_sha256, review_manifest_sha256,
                    observation_hashes, review_authentication_sha256=None):
        receipt, authority_sha = self._receipt(evaluation_id)
        validate_frozen_receipt(
            receipt, plan, local_receipt,
            source_sha256=binding.source_sha256,
            image_sha256=binding.image_sha256,
            cli_build_sha256=binding.cli_build_sha256,
            runtime_sha256=binding.runtime_sha256,
            executor_id=binding.executor_id, evaluator_id=binding.evaluator_id,
        )
        check(type(observation_hashes) is dict
              and set(observation_hashes) == {
                  j["job_id"] for j in receipt["jobs"]
              }, "authority_scored_observation_inventory_mismatch")
        # Preserve the index's original order through dict insertion order.
        # A valid set of 48 hashes is insufficient if the scorer consumes a
        # reordered index relative to the externally frozen plan.
        check(list(observation_hashes) == [j["job_id"] for j in receipt["jobs"]],
              "authority_scored_observation_order_mismatch")
        for value in observation_hashes.values():
            sha_field(value, "invalid_observation_sha256")
        for label, value in (("observation_index", observation_index_sha256),
                             ("review_manifest", review_manifest_sha256)):
            sha_field(value, "invalid_" + label + "_digest")
        run_claim = self._store.read(_key(evaluation_id, "run"))
        check(type(run_claim) is bytes, "authority_run_claim_missing")
        try:
            run = json.loads(run_claim)
        except (ValueError, UnicodeError) as error:
            raise AuthorityFailure("malformed_authority_run_claim") from error
        check(canonical(run) == run_claim, "noncanonical_authority_run_claim")
        check(run == {
            "contract": RUN_CLAIM, "evaluation_id": evaluation_id,
            "authority_receipt_sha256": authority_sha,
            "plan_sha256": receipt["plan_sha256"],
            "executor_id": receipt["executor_id"],
            "retry_allowed": False,
        }, "untrusted_authority_run_claim")
        cohort_claim = self._store.read(_cohort_key(receipt))
        check(type(cohort_claim) is bytes, "authority_cohort_claim_missing")
        check(cohort_claim == canonical(_cohort_record(receipt, authority_sha)),
              "untrusted_authority_cohort_claim")
        completed = self._store.read(_key(evaluation_id, "completed"))
        check(self._store.read(_key(evaluation_id, "halted")) is None, "authority_run_explicitly_stopped")
        check(type(completed) is bytes, "authority_run_not_completed")
        expected = {
            "contract": COMPLETE, "evaluation_id": evaluation_id,
            "authority_receipt_sha256": authority_sha,
            "job_count": 48, "unknown_usage": 0,
        }
        try:
            result = json.loads(completed)
        except (ValueError, UnicodeError) as error:
            raise AuthorityFailure("malformed_authority_completion") from error
        check(canonical(result) == completed, "noncanonical_authority_completion")
        check(type(result) is dict and set(result) ==
              set(expected) | {"charged_reservation_tokens"}
              and all(result.get(k) == v for k, v in expected.items()),
              "untrusted_authority_completion")
        # Every reservation, pre-spawn marker and fully known observation
        # must exist and be bound to the EXACT receipt. The completion marker
        # by itself does not establish a credible 48-job execution.
        native_raw = self._store.read(_key(evaluation_id, "native-admission-profile"))
        check(receipt["contract"] != NATIVE_RECEIPT or native_raw is not None,
              "native_pre_advance_profile_required_for_score")
        native = None
        admission_hashes = []
        if native_raw is not None:
            native = json.loads(native_raw)
            check(canonical(native) == native_raw and native.get("contract") == "reason-cli-normal-v3-native-admission-profile-v1"
                  and native.get("authority_receipt_sha256") == authority_sha
                  and native.get("bundle_sha256") == receipt["image_sha256"], "untrusted_native_admission_profile")
            for name in ("policy_sha256", "sources_sha256"):
                sha_field(native.get(name), "invalid_native_admission_profile")
            if receipt["contract"] == NATIVE_RECEIPT:
                check(all(native.get(k) == v for k, v in receipt["native_admission"].items()),
                      "native_admission_receipt_profile_mismatch")
        charge = 0
        for ordinal, item in enumerate(receipt["jobs"]):
            jid = item["job_id"]
            records = {}
            for part in ("reserved", "inflight", "observed"):
                blob = self._store.read(_key(evaluation_id, part + "/" + jid))
                check(type(blob) is bytes, "incomplete_authority_" + part)
                try:
                    record = json.loads(blob)
                except (ValueError, UnicodeError) as error:
                    raise AuthorityFailure("malformed_authority_" + part) from error
                check(type(record) is dict and canonical(record) == blob,
                      "noncanonical_authority_" + part)
                records[part] = record
            base = {
                "evaluation_id": evaluation_id,
                "authority_receipt_sha256": authority_sha,
                "job_id": jid,
            }
            check(records["reserved"] == {
                **base, "contract": RESERVATION, "ordinal": ordinal,
                "reserve_tokens": item["reserve_tokens"],
            }, "untrusted_authority_reservation")
            check(records["inflight"] == {
                **base, "contract": INFLIGHT, "reserve_charged": True,
            }, "untrusted_authority_inflight")
            obs = records["observed"]
            check(obs.get("contract") == OBSERVED
                  and all(obs.get(k) == v for k, v in base.items())
                  and obs.get("known_usage") is True
                  and obs.get("operational_failure") is None
                  and type(obs.get("tokens")) is int
                  and 0 <= obs["tokens"] <= item["reserve_tokens"]
                  and type(obs.get("attempts")) is int
                  and 1 <= obs["attempts"] <= item["max_provider_attempts"]
                  and obs.get("reserve_tokens") == item["reserve_tokens"]
                  and obs.get("accounting_charge_tokens") == item["reserve_tokens"]
                  and obs.get("observation_sha256") == observation_hashes[jid],
                  "untrusted_authority_observation")
            charge += item["reserve_tokens"]
            if native is not None:
                admitted_raw = self._store.read(_key(evaluation_id, "admission/" + jid))
                check(type(admitted_raw) is bytes, "native_pre_advance_record_missing")
                admitted = json.loads(admitted_raw)
                check(canonical(admitted) == admitted_raw
                      and admitted.get("contract") == "reason-cli-normal-v3-native-admission-v1"
                      and admitted.get("job_id") == jid
                      and admitted.get("native_profile_sha256") == digest(native_raw)
                      and admitted.get("observation_sha256") == observation_hashes[jid]
                      and admitted.get("authentication", {}).get("policy_sha256") == native["policy_sha256"],
                      "untrusted_native_pre_advance_record")
                sha_field(admitted.get("packet_sha256"), "invalid_native_packet_hash")
                sha_field(admitted["authentication"].get("attestation_sha256"), "invalid_native_review_hash")
                admission_hashes.append(digest(admitted_raw))
        check(type(result["charged_reservation_tokens"]) is int
              and result["charged_reservation_tokens"] == charge
              and charge <= receipt["token_budget"],
              "untrusted_authority_charged_budget")
        review_binding = {}
        if native is not None:
            review_binding["native_admission_profile_sha256"] = digest(native_raw)
            review_binding["ordered_pre_advance_records_sha256"] = digest(canonical(admission_hashes))
        if review_authentication_sha256 is not None:
            sha_field(review_authentication_sha256, "invalid_review_authentication_digest")
            review_binding["review_authentication_sha256"] = review_authentication_sha256
        self._create(_key(evaluation_id, "first-score"), {
            "contract": SCORE_CLAIM, "evaluation_id": evaluation_id,
            "authority_receipt_sha256": authority_sha,
            "completion_sha256": digest(completed),
            "observation_index_sha256": observation_index_sha256,
            "review_manifest_sha256": review_manifest_sha256,
            "evaluator_id": binding.evaluator_id,
            "rescore_allowed": False,
            **review_binding,
        })


def _session_serialized(method):
    from functools import wraps
    @wraps(method)
    def locked(self, *args, **kwargs):
        with self._state_lock: return method(self, *args, **kwargs)
    return locked


class RunSession:
    """A burned-on-start evaluation ID; never supports resume or unlock."""
    def __init__(self, store, receipt, receipt_sha):
        import threading
        self._state_lock = threading.RLock()
        self._completed = False
        self._store, self._receipt, self._sha = store, receipt, receipt_sha
        self._next = 0
        self._reserved = 0
        self._inflight = None
        self._inflight_ack = False
        self._halted = False
        self._http_binding = None
        self._http_captured = False
        self._http_usage = None
        self._native_profile = None
        self._admitted_observation = None

    @_session_serialized
    def enable_native_admission(self, *, bundle_sha256, policy_sha256, sources_sha256):
        check(not self._halted and self._next == 0 and self._inflight is None
              and self._native_profile is None, "native_admission_must_precede_reservation")
        for value in (bundle_sha256, policy_sha256, sources_sha256):
            sha_field(value, "invalid_native_admission_profile_hash")
        check(bundle_sha256 == self._receipt["image_sha256"], "native_bundle_receipt_mismatch")
        if self._receipt["contract"] == NATIVE_RECEIPT:
            check(self._receipt["native_admission"] == dict(bundle_sha256=bundle_sha256,
                policy_sha256=policy_sha256, sources_sha256=sources_sha256), "native_admission_receipt_profile_mismatch")
        self._native_profile = {"contract": "reason-cli-normal-v3-native-admission-profile-v1",
            "authority_receipt_sha256": self._sha, "bundle_sha256": bundle_sha256,
            "policy_sha256": policy_sha256, "sources_sha256": sources_sha256}
        self._write("native-admission-profile", self._native_profile)

    @_session_serialized
    def acknowledge_native_admission(self, job_id, *, packet_sha256, observation_sha256, authentication):
        from native_admission import VerifiedAdvance
        check(type(authentication) is VerifiedAdvance, "verified_native_admission_capability_required")
        expected = json.loads(authentication.expected_bytes)
        check(all(expected.get(k) == v for k, v in {
            "evaluation_id": self._receipt["evaluation_id"], "job_id": job_id,
            "authority_receipt_sha256": self._sha, "http_binding_sha256": self._http_binding,
            "packet_sha256": packet_sha256, "observation_sha256": observation_sha256,
        }.items()), "native_admission_capability_binding_mismatch")
        authentication = authentication.authentication
        check(not self._halted and self._native_profile is not None and self._http_captured
              and self._inflight == job_id and self._inflight_ack
              and self._admitted_observation is None, "native_admission_state_mismatch")
        for value in (packet_sha256, observation_sha256, authentication.get("attestation_sha256")):
            sha_field(value, "invalid_native_admission_hash")
        check(authentication.get("policy_sha256") == self._native_profile["policy_sha256"]
              and authentication.get("contract") == "reason-cli-normal-v3-pre-advance-review-v1",
              "native_admission_policy_mismatch")
        self._write("admission/" + job_id, {"contract": "reason-cli-normal-v3-native-admission-v1",
            "job_id": job_id, "native_profile_sha256": digest(canonical(self._native_profile)),
            "packet_sha256": packet_sha256, "observation_sha256": observation_sha256,
            "authentication": authentication})
        self._admitted_observation = observation_sha256

    def _write(self, suffix, data):
        try:
            self._store.create(_key(self._receipt["evaluation_id"], suffix), canonical(data))
        except BaseException:
            # Network timeout after an ACK is ambiguous: never retry a
            # partially committed claim or spawn the child.
            self._halted = True
            raise

    @_session_serialized
    def reserve(self, job_id):
        check(not self._halted and self._inflight is None, "authority_run_halted_or_inflight")
        check(self._receipt["contract"] != NATIVE_RECEIPT or self._native_profile is not None,
              "native_pre_advance_profile_required_before_reservation")
        check(self._next < len(self._receipt["jobs"]), "authority_no_next_job")
        job = self._receipt["jobs"][self._next]
        check(job_id == job["job_id"], "authority_job_order_mismatch")
        amount = job["reserve_tokens"]
        check(self._reserved + amount <= self._receipt["token_budget"],
              "authority_budget_reservation_exceeded")
        self._write("reserved/" + job_id, {
            "contract": RESERVATION, "evaluation_id": self._receipt["evaluation_id"],
            "authority_receipt_sha256": self._sha, "job_id": job_id,
            "ordinal": self._next, "reserve_tokens": amount,
        })
        # Persisting can fail. Do not advance/spawn without remote ACK. Any
        # partial reservation burns the whole run; no automatic recovery.
        self._reserved += amount
        self._inflight = job_id
        self._inflight_ack = False
        self._http_binding = None
        self._http_captured = False
        self._http_usage = None
        self._admitted_observation = None
        return amount

    @_session_serialized
    def stop(self, reason):
        """Durable operator cancellation, with all reservations retained."""
        check(type(reason) is str and bool(reason), "authority_stop_reason_required")
        check(not self._completed, "authority_run_already_completed")
        if self._halted: return
        # Latch before attempting I/O. A failed/ambiguous write must never
        # restore permission to run or advance within this controller.
        self._halted = True
        self._write("halted", {"contract": "reason-cli-normal-v3-stop-v1",
            "evaluation_id": self._receipt["evaluation_id"], "authority_receipt_sha256": self._sha,
            "reason": reason, "inflight_job_id": self._inflight, "retry_allowed": False,
            "charged_reservation_tokens": self._reserved})

    @_session_serialized
    def freeze_http_capture(self, job_id, question):
        """Opt-in native transport binding, persisted before the child starts."""
        check(not self._halted and self._inflight == job_id and not self._inflight_ack,
              "authority_http_binding_before_spawn_required")
        job = self._receipt["jobs"][self._next]
        check(type(question) is str and digest(question.encode()) == job["question_sha256"],
              "authority_http_question_mismatch")
        binding = {
            "contract": "reason-cli-normal-v3-native-http-binding-v1",
            "authority_receipt_sha256": self._sha,
            **{k: self._receipt[k] for k in (
                "evaluation_id", "plan_sha256", "legacy_receipt_sha256", "source_sha256",
                "image_sha256", "cli_build_sha256", "runtime_sha256", "executor_id", "provider", "model")},
            "job": job, "question": question,
        }
        self._write("http/" + job_id + "/binding", binding)
        self._http_binding = digest(canonical(binding))
        return binding

    @_session_serialized
    def acknowledge_http_capture(self, job_id, *, binding_sha256, journal_sha256,
                                 trace_sha256, http_sha256, tokens, attempts):
        check(not self._halted and self._inflight == job_id and self._inflight_ack
              and self._http_binding is not None and binding_sha256 == self._http_binding
              and not self._http_captured, "authority_http_capture_mismatch")
        hashes = dict(binding_sha256=binding_sha256, journal_sha256=journal_sha256,
                      trace_sha256=trace_sha256, http_sha256=http_sha256)
        for value in hashes.values():
            sha_field(value, "invalid_native_http_capture_hash")
        job = self._receipt["jobs"][self._next]
        check(type(tokens) is int and 0 <= tokens <= job["reserve_tokens"]
              and type(attempts) is int and 1 <= attempts <= job["max_provider_attempts"],
              "invalid_native_http_capture_usage")
        self._write("http/" + job_id + "/capture", {
            "contract": "reason-cli-normal-v3-native-http-capture-v1",
            "job_id": job_id, "authority_receipt_sha256": self._sha,
            "tokens": tokens, "attempts": attempts, **hashes,
        })
        self._http_captured = True
        self._http_usage = (tokens, attempts)

    @_session_serialized
    def before_spawn(self, job_id):
        check(not self._halted and self._inflight == job_id,
              "authority_unreserved_spawn")
        check(not self._inflight_ack, "authority_duplicate_inflight")
        self._write("inflight/" + job_id, {
            "contract": INFLIGHT, "evaluation_id": self._receipt["evaluation_id"],
            "authority_receipt_sha256": self._sha,
            "job_id": job_id, "reserve_charged": True,
        })
        self._inflight_ack = True

    @_session_serialized
    def observe(self, job_id, *, tokens, attempts, operational_failure=None,
                observation_sha256=None):
        check(self._inflight == job_id and self._inflight_ack and not self._halted,
              "authority_unmatched_observation")
        if self._http_binding is not None and not self._http_captured:
            tokens = attempts = None
            operational_failure = operational_failure or "native_http_capture_missing"
        elif self._http_binding is not None and self._http_usage != (tokens, attempts):
            tokens = attempts = None
            operational_failure = operational_failure or "native_http_usage_mismatch"
        if self._native_profile is not None and (self._admitted_observation is None
                or self._admitted_observation != observation_sha256):
            operational_failure = operational_failure or "native_pre_advance_admission_missing"
        job = self._receipt["jobs"][self._next]
        if observation_sha256 is not None:
            sha_field(observation_sha256, "invalid_authority_observation_sha256")
        known = (type(tokens) is int and tokens >= 0
                 and type(attempts) is int and attempts >= 0)
        valid = (known and tokens <= job["reserve_tokens"]
                 and 1 <= attempts <= job["max_provider_attempts"])
        # Unknown or over-budget usage is charged at the FULL reservation,
        # remains recorded as unknown, and permanently latches the run.
        self._write("observed/" + job_id, {
            "contract": OBSERVED, "evaluation_id": self._receipt["evaluation_id"],
            "authority_receipt_sha256": self._sha, "job_id": job_id,
            "reserve_tokens": job["reserve_tokens"],
            "known_usage": valid,
            "tokens": tokens if valid else None,
            "attempts": attempts if valid else None,
            "accounting_charge_tokens": job["reserve_tokens"],
            "operational_failure": operational_failure,
            "observation_sha256": observation_sha256,
        })
        self._inflight = None
        self._inflight_ack = False
        self._next += 1
        if not valid or operational_failure is not None:
            self._halted = True
        return valid and operational_failure is None

    @_session_serialized
    def complete(self):
        check(not self._halted and self._inflight is None
              and self._next == 48, "authority_incomplete_or_uncertain_run")
        # No implicit rescore or restart; the remote record is immutable.
        self._write("completed", {
            "contract": COMPLETE, "evaluation_id": self._receipt["evaluation_id"],
            "authority_receipt_sha256": self._sha,
            "job_count": 48, "unknown_usage": 0,
            "charged_reservation_tokens": self._reserved,
        })
        self._completed = True
