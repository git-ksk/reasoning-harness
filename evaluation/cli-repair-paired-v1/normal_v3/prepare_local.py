"""Provider-free preparation of a consumed real24 paired Mac evaluation.

Consumes already materialized immutable source bytes. Does not register a
reviewer, issue an admitted receipt, start a run, reserve tokens or score.
"""
import argparse
import json
from pathlib import Path

from contracts import canonical, digest, exclusive_json, private_new_directory, require, safe_id
from local_store import initialize_local_store
from native_admission import CATALOG, source_catalog
from native_bundle import NativeBundle
from native_lane import plan_first_request, private_snapshot, private_bytes

INPUT_M1 = "597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3"
PURPOSE = "CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE"
MODEL = "ministral-8b-2512"


def checked_input(root):
    root = Path(root).resolve(strict=True)
    raw = private_snapshot(root / "manifest.json")
    require(digest(raw) == INPUT_M1, "consumed_manifest_changed")
    inventory = json.loads(raw)["files"]
    require(not any(p.is_symlink() for p in root.rglob("*")), "consumed_source_symlink")
    require({str(p.relative_to(root)) for p in root.rglob("*") if p.is_file() and p != root / "manifest.json"}
            == set(inventory), "consumed_source_inventory_changed")
    for name, expected in inventory.items():
        path = root / name
        require(path.resolve().is_relative_to(root) and digest(private_snapshot(path)) == expected,
                "consumed_source_bytes_changed")
    corpus = json.loads(private_snapshot(root / "corpus.json"))
    require(corpus["identity"] == "engine073-realworld-independent-v1"
            and len(corpus["cases"]) == 24, "consumed_real24_required")
    for sid, source in corpus["sources"].items():
        safe_id(sid, "invalid_consumed_source_id")
        text = private_snapshot(root / source["normalized_path"])
        require(digest(text) == source["normalized_sha256"]
                and text[source["start_byte"]:source["end_byte"]] == source["quote"].encode()
                and digest(source["quote"].encode()) == source["quote_sha256"]
                and not source["synthetic_mutation"], "consumed_full_source_quote_mismatch")
    return corpus, json.loads(private_snapshot(root / "gold.json"))


def scope(value):
    return {key: {"kind": "values", "values": [value[key]]} for key in sorted(value)}


def captured_case(corpus, case):
    """The old resolver's exact data, with a process-free transport only.

    No gold labels, answers or scoring groups are read here. Admission is still
    performed by the ordinary ExternalEvidenceAdmissionPolicy at acquisition.
    """
    require("e2e_host_facts" not in case, "new_host_fact_profile_requires_review")
    sources = {sid: {"authority_class": "primary", "max_age_seconds": 86400,
                    "scope": scope(corpus["sources"][sid]["scope"])}
               for sid in case["source_ids"] if not corpus["sources"][sid]["synthetic_mutation"]}
    admission = {"authority_ranks": {"primary": 20, "reference_host": 20, "untrusted": 0},
                 "minimum_authority_class": "primary", "required_scope": scope(case["required_scope"]),
                 "evaluation_time_unix_seconds": max(d["retrieved_at_unix_seconds"] for d in corpus["documents"].values()) + 1,
                 "sources": sources}
    capabilities, snapshots, registry = [], [], []
    for slot, sid in enumerate(case["source_ids"]):
        source = corpus["sources"][sid]
        visible = {key: source[key] for key in ("quote", "context_summary", "document_id", "url", "locator",
                                               "source_version", "origin_group", "scope", "synthetic_mutation")}
        record = {"id": f"snapshot:{case['id']}:{slot}", "source": sid,
                  "observation": json.dumps(visible, sort_keys=True), "facts": source["reference_facts"],
                  "acquisition_metadata": {"observed_at_unix_seconds": source["retrieved_at_unix_seconds"],
                      "retrieved_at_unix_seconds": source["retrieved_at_unix_seconds"],
                      "scope": scope(source["scope"]), "claimed_authority_class": "primary"}}
        capabilities.append({"id": "independent-v2-source-" + str(slot), "kind": "captured_evidence",
            "read_only": True, "supported_fact_keys": sorted(set(record["facts"]) | {case["lookup_key"]}),
            "evidence": [record], "admission": admission})
        owner = sources[sid]; at = record["acquisition_metadata"]["observed_at_unix_seconds"]
        evidence = {"id": record["id"], "source": sid, "observation": record["observation"],
                    "facts": dict(sorted(record["facts"].items())), "metadata": {
                        "temporal": {"effective_from_unix_seconds": at, "effective_until_unix_seconds": at + 86400},
                        "scope": owner["scope"], "provenance_class": owner["authority_class"]}}
        hash_ = digest(json.dumps(evidence, ensure_ascii=False, separators=(",", ":")).encode())
        snapshots.append((evidence, hash_))
        for key, value in sorted(record["facts"].items()):
            registry.append({"proposition": {"key": key, "value": value}, "evidence_id": evidence["id"],
                "source_id": sid, "evidence_sha256": hash_, "quote_pointer": "/quote", "url_pointer": "/url",
                "version_pointer": "/source_version", "url": source["url"], "source_version": source["source_version"]})
    config = {"schema_version": "reason-config-v1", "resolution": {"investigation": {
        "capabilities": capabilities, "max_actions": 4, "max_no_progress_rounds": 2, "max_rounds": 3,
        "max_targets": 3, "planner_max_tokens": 512}},
        "answer_context": {"schema_version": "reason-answer-context-config-v1", "sources": registry,
                           "unresolved_inputs": []}}
    return config, snapshots


def verify_request_pairs(jobs):
    pairs = {}
    for job in jobs:
        pair = pairs.setdefault(job["case_id"], {})
        require(job["arm"] in ("baseline", "candidate") and job["arm"] not in pair,
                "duplicate_or_unknown_preparation_arm")
        pair[job["arm"]] = job["request"]
    require(len(jobs) == 48 and len(pairs) == 24, "paired_preparation_inventory_incomplete")
    for pair in pairs.values():
        require(set(pair) == {"baseline", "candidate"}, "paired_preparation_arm_missing")
        first, second = pair["baseline"], pair["candidate"]
        require(first["request_sha256"] == second["request_sha256"] and first["request"] == second["request"],
                "paired_first_request_prompt_or_schema_changed")


def prepare(*, inputs, build, output):
    corpus, gold = checked_input(inputs)
    build = Path(build).resolve(strict=True); output = Path(output).resolve()
    proof = json.loads(private_snapshot(build / "build.json"))
    require(proof["contract"] == "reason-cli-normal-v3-native-paired-build-v1", "native_pair_build_required")
    private_new_directory(output)
    private_new_directory(output / "config")
    private_new_directory(output / "requests")
    configs = {"build-proof": build / "build.json", "input-manifest": Path(inputs).resolve() / "manifest.json"}
    catalog = {"contract": CATALOG, "sources": {}, "evidence_bindings": {}}
    for sid, source in corpus["sources"].items():
        catalog["sources"][sid] = {"text": private_snapshot(Path(inputs).resolve() / source["normalized_path"]).decode(),
                                   "source_version": source["source_version"]}
    for case in corpus["cases"]:
        safe_id(case["id"], "invalid_preparation_case")
        config, snapshots = captured_case(corpus, case)
        path = output / "config" / (case["id"] + ".json"); exclusive_json(path, config)
        configs[case["id"]] = path
        for item, hash_ in snapshots:
            source = corpus["sources"][item["source"]]
            catalog["evidence_bindings"][item["id"]] = {"evidence_sha256": hash_, "source_id": item["source"],
                                                         "source_version": source["source_version"]}
    source_catalog(canonical(catalog)); exclusive_json(output / "source-catalog.json", catalog)
    configs["sources"] = output / "source-catalog.json"
    exclusive_json(output / "gold-obligations.json", gold)
    configs["gold-obligations"] = output / "gold-obligations.json"
    bundle = NativeBundle.enroll(source_roots={a: proof["arms"][a]["source_root"] for a in proof["arms"]},
        binaries={a: proof["arms"][a]["binary"] for a in proof["arms"]}, configurations=configs)
    for arm, record in proof["arms"].items():
        require(digest(Path(bundle.binary(arm)).read_bytes()) == record["binary_sha256"], "paired_build_binary_changed")
        require({n: digest((Path(record["source_root"]) / n).read_bytes())
                 for n in bundle.data["source_inventory"][arm]}
                == record["source_files"], "paired_build_source_changed")
    # NativeBundle accepts exact canonical bytes; a JSON-evidence newline is
    # inappropriate for this re-openable manifest and its byte digest.
    private_bytes(output / "native-bundle.json", bundle.manifest_bytes)
    bundle = NativeBundle(private_snapshot(output / "native-bundle.json"))
    bundle.verify()
    jobs = []
    for index, case in enumerate(corpus["cases"]):
        for arm in (("baseline", "candidate") if index % 2 == 0 else ("candidate", "baseline")):
            jid = case["id"] + "-" + arm
            argv = [bundle.binary(arm), "--plain", "--provider", "mistral", "--model", MODEL,
                    "--seed", str(49200 + index), "--max-tokens", "1024", "--safety-profile", "current",
                    "--max-model-calls", "12", "--max-output-tokens", "4096", "--max-total-tokens", "28096",
                    "--config", str(configs[case["id"]]), "--format", "json", case["question"]]
            request = plan_first_request(bundle, arm, argv, output / "requests" / (jid + ".json"))
            jobs.append({"job_id": jid, "case_id": case["id"], "arm": arm, "argv": argv, "request": request})
            print("planned " + jid, flush=True)
    verify_request_pairs(jobs)
    exclusive_json(output / "requests.json", {"contract": "reason-cli-normal-v3-prospective-requests-v1", "jobs": jobs})
    profile = initialize_local_store(output / "authority")
    exclusive_json(output / "operator-profile.json", profile.to_json())
    status = {"contract": "reason-cli-normal-v3-local-preparation-v1", "purpose": PURPOSE,
        "input_manifest_sha256": INPUT_M1, "native_bundle_sha256": bundle.verify(),
        "cases": 24, "jobs_planned": len(jobs), "paired_first_requests_identical": True,
        "full_documents": len(corpus["documents"]),
        "source_bindings": len(catalog["sources"]), "model_provider_calls": 0, "cloud_operations": 0,
        "run_claimed": False, "tokens_reserved": 0, "reviewer_enrolled": False,
        "provider_execution_authorized": False, "new_real_f1": None, "quality_go": False,
        "remaining": ["operator reviewer enrollment and independent source/runtime review",
                      "freeze actual provider readiness, pacing, per-generation and aggregate accounting/budget",
                      "new genuine receipt, fixed plan, authority admission and one-shot tag",
                      "48 actual observations with pre-advance reviews and first semantic score"]}
    exclusive_json(output / "status.json", status)
    return status


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--inputs", type=Path, required=True)
    parser.add_argument("--build", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    prepare(inputs=args.inputs, build=args.build, output=args.output)
