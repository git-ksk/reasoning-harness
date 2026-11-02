"""#580 adversarial tests. Entirely synthetic: never make provider/GCS calls."""
import copy
import json
import tempfile
import threading
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import authority
import offline_lane
from contracts import canonical, digest
from test_offline_lane import make_plan


class FakeRemoteObjectStore:
    """Atomic across separately constructed host clients to one remote namespace."""
    def __init__(self):
        self.objects = {}
        self.lock = threading.Lock()
        self.fail_before = None
        self.fail_after = None

    def read(self, key):
        with self.lock:
            return self.objects.get(key)

    def create(self, key, value):
        with self.lock:
            if self.fail_before and self.fail_before in key:
                raise OSError("simulated_remote_storage_unavailable")
            if key in self.objects:
                raise FileExistsError("remote_object_already_exists")
            self.objects[key] = value
            if self.fail_after and self.fail_after in key:
                raise OSError("simulated_ambiguous_ack")


class AuthorityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.path = Path(cls.temp.name)
        cls.plan, cls.receipt_file, cls.script, _ = make_plan(cls.path)
        cls.local_receipt = cls.receipt_file.read_bytes()
        cls.binding = authority.AuthorityBinding(
            source_sha256="a" * 64, image_sha256="b" * 64,
            cli_build_sha256="c" * 64, runtime_sha256="d" * 64,
            executor_id="trusted-executor", evaluator_id="trusted-evaluator",
        )

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def fixture(self, evaluation_id="synthetic-run-580", *,
                plan=None, binding=None, local_receipt=None):
        plan = self.plan if plan is None else plan
        binding = self.binding if binding is None else binding
        local_receipt = self.local_receipt if local_receipt is None else local_receipt
        jobs = authority.freeze_jobs(plan)
        receipt = {
            "contract": authority.RECEIPT,
            "evaluation_id": evaluation_id,
            "plan_sha256": digest(canonical(plan)),
            "legacy_receipt_sha256": digest(local_receipt),
            "source_sha256": binding.source_sha256,
            "image_sha256": binding.image_sha256,
            "cli_build_sha256": binding.cli_build_sha256,
            "runtime_sha256": binding.runtime_sha256,
            "provider": plan["provider"], "model": plan["model"],
            "executor_id": binding.executor_id,
            "evaluator_id": binding.evaluator_id,
            "token_budget": plan["token_budget"], "jobs": jobs,
        }
        store = FakeRemoteObjectStore()
        store.objects[f"{authority.PREFIX}/receipts/{evaluation_id}.json"] = canonical(receipt)
        return store, receipt

    def start(self, store, evaluation_id="synthetic-run-580", *,
              plan=None, local_receipt=None, binding=None):
        return authority.TrustedAuthority(store).start(
            evaluation_id,
            self.plan if plan is None else plan,
            self.local_receipt if local_receipt is None else local_receipt,
            self.binding if binding is None else binding,
        )

    def all_48(self, session):
        for job in self.plan["jobs"]:
            jid = job["id"]
            amount = session.reserve(jid)
            session.before_spawn(jid)
            self.assertTrue(session.observe(
                jid, tokens=1, attempts=1,
                observation_sha256=digest(canonical({"id": jid}))))
            self.assertGreater(amount, 0)
        session.complete()

    def test_remote_receipt_freezes_every_execution_identity(self):
        store, _ = self.fixture()
        changes = [
            {"source_sha256": "e" * 64}, {"image_sha256": "e" * 64},
            {"cli_build_sha256": "e" * 64}, {"runtime_sha256": "e" * 64},
            {"executor_id": "other-executor"}, {"evaluator_id": "other-evaluator"},
        ]
        for diff in changes:
            with self.subTest(diff=diff):
                binder = authority.AuthorityBinding(**{**vars(self.binding), **diff})
                with self.assertRaisesRegex(ValueError, "authority_.*_mismatch"):
                    self.start(store, binding=binder)
        for mutation in ("provider", "model", "token_budget", "jobs", "questions",
                         "frozen_request_bindings"):
            with self.subTest(plan_field=mutation):
                plan = copy.deepcopy(self.plan)
                if mutation == "provider":
                    plan["provider"] = "groq"
                elif mutation == "model":
                    plan["model"] = "other"
                elif mutation == "token_budget":
                    plan["token_budget"] += 1
                elif mutation == "jobs":
                    plan["jobs"][0]["budget"]["max_generations"] += 1
                elif mutation == "questions":
                    plan["questions"]["synthetic-00"] = "different question"
                else:
                    plan["frozen_request_bindings"][self.plan["jobs"][0]["id"]] = "f" * 64
                with self.assertRaises(ValueError):
                    self.start(store, plan=plan)
        with self.assertRaisesRegex(ValueError, "legacy_receipt_sha256_mismatch"):
            self.start(store, local_receipt=self.local_receipt + b" ")
        self.assertFalse(any("/claims/" in k for k in store.objects))

    def test_two_hosts_compete_and_replay_cannot_relabel(self):
        store, _ = self.fixture()
        def host(_):
            try:
                self.start(store)
                return "claimed"
            except FileExistsError:
                return "rejected"
        with ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(host, (0, 1)))
        self.assertCountEqual(results, ["claimed", "rejected"])
        self.assertEqual(sum("/claims/" in k for k in store.objects), 1)
        with self.assertRaises(FileExistsError):
            self.start(store)
        # A second store or receipt-chosen directory is not a fallback backend:
        with self.assertRaisesRegex(ValueError, "missing_authority_receipt"):
            self.start(FakeRemoteObjectStore())

    def test_sigkill_crash_burns_entire_run_and_unknown_charge(self):
        store, _ = self.fixture()
        session = self.start(store)
        first = self.plan["jobs"][0]["id"]
        charge = session.reserve(first)
        session.before_spawn(first)
        # Simulated abrupt SIGKILL: child may still exist and usage is unknown.
        self.assertEqual(json.loads(store.read(authority._key(
            "synthetic-run-580", "reserved/" + first)))["reserve_tokens"], charge)
        self.assertIsNone(store.read(authority._key("synthetic-run-580", "completed")))
        with self.assertRaises(FileExistsError):
            self.start(store)
        with self.assertRaisesRegex(ValueError, "inflight"):
            session.reserve(self.plan["jobs"][1]["id"])
        with self.assertRaisesRegex(ValueError, "incomplete"):
            session.complete()

    def test_run_refuses_inadequate_frozen_full48_token_budget(self):
        plan = copy.deepcopy(self.plan)
        plan["token_budget"] = 40
        store, receipt = self.fixture(plan=plan)
        with self.assertRaisesRegex(ValueError, "total_reservations_exceed_budget"):
            self.start(store, plan=plan)
        self.assertFalse(any("/claims/" in k for k in store.objects))

    def test_remote_storage_loss_fails_before_work_and_cannot_retry_ambiguous_ack(self):
        for stage in ("run", "reserved/", "inflight/", "observed/"):
            with self.subTest(stage=stage):
                store, _ = self.fixture()
                store.fail_after = "/" + stage if stage != "run" else "/run.json"
                if stage == "run":
                    with self.assertRaises(OSError):
                        self.start(store)
                else:
                    session = self.start(store)
                    first = self.plan["jobs"][0]["id"]
                    try:
                        session.reserve(first)
                        session.before_spawn(first)
                        session.observe(first, tokens=1, attempts=1)
                    except OSError:
                        pass
                    else:
                        self.fail("injected storage fault not encountered")
                    with self.assertRaisesRegex(ValueError, "halted"):
                        session.reserve(first)
                with self.assertRaises(FileExistsError):
                    self.start(store)

    def test_remote_write_failure_without_ack_suppresses_spawning(self):
        store, _ = self.fixture()
        session = self.start(store)
        store.fail_before = "/inflight/"
        first = self.plan["jobs"][0]["id"]
        session.reserve(first)
        with self.assertRaises(OSError):
            session.before_spawn(first)
        with self.assertRaisesRegex(ValueError, "halted"):
            session.reserve(first)
        self.assertIsNone(store.read(authority._key(
            "synthetic-run-580", "inflight/" + first)))

    def test_unknown_usage_overcap_and_failure_halt_and_charge_full_reservation(self):
        for tokens, attempts, reason in [(None, 1, None), (1, None, None),
                                         (999999, 1, None), (1, 999999, None),
                                         (1, 1, "provider_quota")]:
            with self.subTest(data=(tokens, attempts, reason)):
                store, _ = self.fixture()
                session = self.start(store)
                jid = self.plan["jobs"][0]["id"]
                reserve = session.reserve(jid)
                session.before_spawn(jid)
                self.assertFalse(session.observe(
                    jid, tokens=tokens, attempts=attempts,
                    operational_failure=reason))
                observation = json.loads(store.read(authority._key(
                    "synthetic-run-580", "observed/" + jid)))
                self.assertEqual(observation["accounting_charge_tokens"], reserve)
                with self.assertRaises(ValueError):
                    session.reserve(self.plan["jobs"][1]["id"])
                with self.assertRaises(ValueError):
                    session.complete()

    def test_pre_spawn_marker_required_and_duplicate_preflight_rejected(self):
        store, _ = self.fixture()
        session = self.start(store)
        first = self.plan["jobs"][0]["id"]
        session.reserve(first)
        with self.assertRaisesRegex(ValueError, "unmatched_observation"):
            session.observe(first, tokens=1, attempts=1)
        session.before_spawn(first)
        with self.assertRaisesRegex(ValueError, "duplicate_inflight"):
            session.before_spawn(first)
        self.assertTrue(session.observe(first, tokens=1, attempts=1))

    def test_tampered_remote_reservation_or_inflight_blocks_scoring(self):
        for component in ("reserved", "inflight", "observed", "completed"):
            with self.subTest(component=component):
                store, _ = self.fixture()
                self.all_48(self.start(store))
                suffix = (component + "/" + self.plan["jobs"][0]["id"]
                          if component != "completed" else component)
                key = authority._key("synthetic-run-580", suffix)
                saved = store.objects[key]
                corrupted = json.loads(saved)
                corrupted["authority_receipt_sha256"] = "0" * 64
                store.objects[key] = canonical(corrupted)
                with self.assertRaises(ValueError):
                    authority.TrustedAuthority(store).claim_score(
                        "synthetic-run-580", self.plan, self.local_receipt,
                        self.binding, observation_index_sha256="1" * 64,
                        review_manifest_sha256="2" * 64,
                        observation_hashes={j["id"]: digest(canonical({"id": j["id"]}))
                                            for j in self.plan["jobs"]},
                    )
                self.assertNotIn(authority._key("synthetic-run-580", "first-score"),
                                 store.objects)
                store.objects[key] = saved

    def test_gcs_adapter_uses_atomic_generation_zero(self):
        class Blob:
            def __init__(self):
                self.content = None
                self.uploads = []
            def download_as_bytes(self):
                return self.content
            def upload_from_string(self, value, **kwargs):
                self.uploads.append(kwargs)
                if self.content is not None:
                    raise FileExistsError()
                self.content = value
        class Bucket:
            def __init__(self):
                self.blobs = {}
            def blob(self, key):
                return self.blobs.setdefault(key, Blob())
        class Client:
            def __init__(self):
                self.b = Bucket()
            def bucket(self, _name):
                return self.b
        client = Client()
        store = authority.GCSAtomicStore("operator-controlled-bucket", client=client)
        store.create("claim", b"{}")
        self.assertEqual(store.read("claim"), b"{}")
        self.assertEqual(client.b.blob("claim").uploads,
                         [{"content_type": "application/json", "if_generation_match": 0}])
        with self.assertRaises(FileExistsError):
            store.create("claim", b"{}")
        with self.assertRaises(ValueError):
            authority.GCSAtomicStore("INVALID BUCKET", client=client)

    def test_full48_complete_and_first_score_irrevocably_once(self):
        store, _ = self.fixture()
        self.all_48(self.start(store))
        service = authority.TrustedAuthority(store)
        args = dict(observation_index_sha256="1"*64, review_manifest_sha256="2"*64,
                    observation_hashes={j["id"]: digest(canonical({"id": j["id"]}))
                                        for j in self.plan["jobs"]})
        service.claim_score("synthetic-run-580", self.plan, self.local_receipt,
                            self.binding, **args)
        with self.assertRaises(FileExistsError):
            service.claim_score("synthetic-run-580", self.plan, self.local_receipt,
                                self.binding,
                                observation_index_sha256="3"*64,
                                review_manifest_sha256="4"*64,
                                observation_hashes=args["observation_hashes"])
        claim = json.loads(store.read(authority._key(
            "synthetic-run-580", "first-score")))
        self.assertFalse(claim["rescore_allowed"])

    def test_first_score_requires_all_48_authoritative_observations(self):
        store, _ = self.fixture()
        service = authority.TrustedAuthority(store)
        args = dict(observation_index_sha256="1"*64, review_manifest_sha256="2"*64,
                    observation_hashes={j["id"]: digest(canonical({"id": j["id"]}))
                                        for j in self.plan["jobs"]})
        with self.assertRaisesRegex(ValueError, "run_claim_missing"):
            service.claim_score("synthetic-run-580", self.plan, self.local_receipt,
                                self.binding, **args)
        self.all_48(self.start(store))
        first = self.plan["jobs"][0]["id"]
        del store.objects[authority._key("synthetic-run-580", "observed/" + first)]
        with self.assertRaisesRegex(ValueError, "incomplete_authority_observed"):
            service.claim_score("synthetic-run-580", self.plan, self.local_receipt,
                                self.binding, **args)
        self.assertIsNone(store.read(authority._key("synthetic-run-580", "first-score")))

    def test_forged_noncanonical_or_replaced_receipt_is_rejected(self):
        for variant in ("noncanonical", "changed_sha", "forged_id", "bad_jobs",
                        "same_principals"):
            with self.subTest(variant=variant):
                store, receipt = self.fixture()
                key = f"{authority.PREFIX}/receipts/synthetic-run-580.json"
                if variant == "noncanonical":
                    store.objects[key] += b"\n"
                else:
                    receipt = copy.deepcopy(receipt)
                    if variant == "changed_sha":
                        receipt["runtime_sha256"] = "0"*64
                    elif variant == "forged_id":
                        receipt["evaluation_id"] = "other"
                    elif variant == "bad_jobs":
                        receipt["jobs"][0]["case_id"] = "other"
                    else:
                        receipt["evaluator_id"] = receipt["executor_id"]
                    store.objects[key] = canonical(receipt)
                with self.assertRaises(ValueError):
                    self.start(store)

    def test_full48_actual_inert_subprocesses_with_remote_ledger(self):
        # This exercises actual inherited durable child scheduling, not just
        # Python-method calls. No network, no credentials, no real provider.
        with tempfile.TemporaryDirectory() as t:
            plan, receipt_file, script, _ = make_plan(t)
            local = receipt_file.read_bytes()
            store, _ = self.fixture("inert-real-scheduler", plan=plan, local_receipt=local)
            session = self.start(store, "inert-real-scheduler",
                                 plan=plan, local_receipt=local)
            out = Path(t) / "authoritative-inert-lane"
            result = offline_lane.OfflineV3Runner(
                plan, out, receipt_file, str(script),
                authority_session=session,
            ).run()
            self.assertTrue(result["complete"])
            self.assertEqual(len(list(out.glob("*/spawn.json"))), 48)
            self.assertIsNotNone(store.read(authority._key(
                "inert-real-scheduler", "completed")))
            self.assertEqual(len([k for k in store.objects if "/observed/" in k]), 48)
            with self.assertRaises(FileExistsError):
                self.start(store, "inert-real-scheduler", plan=plan, local_receipt=local)

    def test_actual_inert_subprocess_failure_never_spawns_next(self):
        with tempfile.TemporaryDirectory() as t:
            plan, receipt_file, script, _ = make_plan(
                t, {"synthetic-00-baseline": "missing_usage"})
            local = receipt_file.read_bytes()
            store, _ = self.fixture("inert-unknown-usage", plan=plan,
                                    local_receipt=local)
            session = self.start(store, "inert-unknown-usage",
                                 plan=plan, local_receipt=local)
            out = Path(t) / "failed-authoritative-inert-lane"
            result = offline_lane.OfflineV3Runner(
                plan, out, receipt_file, str(script),
                authority_session=session,
            ).run()
            self.assertFalse(result["complete"])
            self.assertEqual(len(list(out.glob("*/spawn.json"))), 1)
            self.assertIsNone(store.read(authority._key(
                "inert-unknown-usage", "completed")))
            self.assertEqual(len([k for k in store.objects if "/reserved/" in k]), 1)


if __name__ == "__main__":
    unittest.main()
