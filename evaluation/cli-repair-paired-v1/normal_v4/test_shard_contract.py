"""Public, provider-free negative controls for v4 24-pair sharding."""
import copy
import unittest

from shard_contract import (
    EXPECTED_IDS, ARMS, MANIFEST_CONTRACT, global_manifest, select_case,
    verify_all_shards, make_shard_plan, sha, canonical,
)


def fixture():
    cases = [{"id": cid, "question": "Frozen question " + cid,
              "source_ids": ["doc-source"]} for cid in EXPECTED_IDS]
    corpus = {
        "cases": cases,
        "documents": {"document": {
            "raw_sha256": "a"*64, "normalized_sha256": "b"*64}},
        "sources": {"doc-source": {"source_version": "fixed"}},
    }
    requests = []
    for cid in EXPECTED_IDS:
        for arm in ARMS:
            requests.append({"job_id": cid+"-"+arm, "case_id": cid, "arm": arm,
                  "request": {"question": "Frozen question " + cid,
                      "request": {"task": "Frozen question " + cid},
                      "request_sha256": sha((cid+"-"+arm).encode())}})
    return corpus, requests


class V4ShardContractTests(unittest.TestCase):
    def setUp(self):
        self.corpus, self.requests = fixture()
        self.manifest = global_manifest(self.corpus, self.requests, "f"*40)
        self.shards = [select_case(self.manifest, self.requests, cid) for cid in EXPECTED_IDS]

    def test_shards_have_all_24_pairs_and_one_global_identity(self):
        self.assertEqual(self.manifest["contract"], MANIFEST_CONTRACT)
        self.assertEqual(len(self.manifest["cases"]), 24)
        self.assertEqual(len(verify_all_shards(self.manifest, self.shards)), 24)
        self.assertEqual(sum(len(s["job_ids"]) for s in self.shards), 48)
        self.assertEqual(len({s["global_manifest_sha256"] for s in self.shards}), 1)

    def test_missing_duplicate_or_foreign_pair_never_forms_global_score(self):
        for mutated in (self.shards[:-1], self.shards[:-1]+[self.shards[0]],
                        self.shards+ [self.shards[0]]):
            with self.assertRaises(ValueError):
                verify_all_shards(self.manifest, mutated)

    def test_source_mutation_changes_global_identity(self):
        tampered = copy.deepcopy(self.corpus)
        tampered["documents"]["document"]["raw_sha256"] = "c"*64
        manifest = global_manifest(tampered, self.requests, "f"*40)
        self.assertNotEqual(sha(canonical(self.manifest)), sha(canonical(manifest)))
        with self.assertRaisesRegex(ValueError, "global_manifest_conflict"):
            verify_all_shards(manifest, self.shards)

    def test_request_question_or_arm_swap_fails_before_execution(self):
        bad = copy.deepcopy(self.requests)
        bad[0]["request"]["request_sha256"] = "0"*64
        manifest = global_manifest(self.corpus, bad, "f"*40)
        with self.assertRaises(ValueError):
            select_case(manifest, self.requests, EXPECTED_IDS[0])
        bad = copy.deepcopy(self.requests)
        bad[1]["arm"] = "baseline"
        with self.assertRaisesRegex(ValueError, "duplicate_or_mismatched_job"):
            global_manifest(self.corpus, bad, "f"*40)
        bad = copy.deepcopy(self.requests)
        bad[0]["request"]["question"] = "foreign question"
        with self.assertRaisesRegex(ValueError, "question_mismatch"):
            global_manifest(self.corpus, bad, "f"*40)

    def test_actual_paired_shard_plan_budget_and_receipt_binding(self):
        cases = [{"case_id":c["id"],"question":c["question"],
                  "label":"positive","required_groups":["g1"]}
                 for c in self.corpus["cases"]]
        native=copy.deepcopy(self.requests)
        for row in native:
            row["argv"]=["/fixed/reason", "--format", "json", "Frozen question "+row["case_id"]]
            row["http_limits"]={"contract":"test-limits"}
            row["request"]["provider"]="mistral"
            row["request"]["model"]="ministral-8b-2512"
        plan=make_shard_plan(manifest=self.manifest,
            selected="rw-01",requests=native,cases=cases,receipt_sha256="c"*64)
        self.assertEqual(plan["schema"],"engine07-operational-paired-shard-v1")
        self.assertEqual(len(plan["jobs"]),2)
        self.assertEqual(plan["jobs"][0]["timeout_seconds"],1200)
        self.assertEqual(plan["receipt_sha256"],"c"*64)
        self.assertEqual(set(plan["frozen_request_bindings"]),
                         {"rw-01-baseline","rw-01-candidate"})
        with self.assertRaisesRegex(ValueError,"bounded_shard_timeout"):
            make_shard_plan(manifest=self.manifest,selected="rw-01",
                            requests=native,cases=cases,
                            receipt_sha256="c"*64,timeout_seconds=1500)

    def test_case_order_and_frozen_source_commit_required(self):
        wrong = copy.deepcopy(self.corpus)
        wrong["cases"].reverse()
        with self.assertRaisesRegex(ValueError, "frozen_24_case_order"):
            global_manifest(wrong, self.requests, "f"*40)
        with self.assertRaisesRegex(ValueError, "source_commit_required"):
            global_manifest(self.corpus, self.requests, "not-a-source-commit")

    def test_shard_identity_cannot_be_laundered_by_reorder(self):
        wrong = copy.deepcopy(self.shards)
        wrong[0]["case_index"] = 1
        with self.assertRaisesRegex(ValueError, "shard_tamper"):
            verify_all_shards(self.manifest, wrong)


if __name__ == "__main__":
    unittest.main()
