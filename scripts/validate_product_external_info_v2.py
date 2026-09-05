#!/usr/bin/env python3
"""Fail-closed validator for the pre-observation product external-information v2 successor freeze."""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import pathlib
import sys
import urllib.parse
from typing import Any

EXPECTED_CORPUS_ID = "product-external-info-v2"
EXPECTED_CASE_SCHEMA = "product-external-info-case-v2"
EXPECTED_CORPUS_SCHEMA = "product-external-info-corpus-v2"
EXPECTED_SCORING_ID = "product-external-info-scoring-v2"
EXPECTED_SEMANTIC_ID = "verified-target-finalization-successor-v2"
EXPECTED_FOUR_ARM_ID = "product-external-info-four-arm-v2"
EXPECTED_BASELINE = "a365a46d5fa948063e9ac745ad14646c23456ede"
EXPECTED_PROTOCOL = "2026-07-28"
EXPECTED_ADAPTER = "mcp_readonly_v1"
EXPECTED_RESOLVER_CLASS = "evidence_acquisition"
EXPECTED_TOOL = "fetch_json_fact"
EXPECTED_SERVER_ID = "product_external_info_fixture_v2"
EXPECTED_PROVIDER = "mistral"
EXPECTED_MODEL = "ministral-8b-latest"
EXPECTED_SEED = 26000
EXPECTED_MAX_TOKENS = 1024
EXPECTED_FAMILIES = {
    "current_fresh_snapshot",
    "entity_identity_ambiguity",
    "authoritative_source_requirement",
    "stale_fresh_or_source_conflict",
    "no_result_or_insufficient_evidence",
    "typed_operational_failure",
    "irrelevant_or_instruction_like_content",
}
EXPECTED_ARMS = [
    "raw_model_no_external",
    "harness_no_external",
    "raw_model_with_external",
    "harness_with_mcp_external",
]
PRIMARY_COMPARISON = ["raw_model_with_external", "harness_with_mcp_external"]
ALLOWED_HOSTS = {
    "api.github.com",
    "raw.githubusercontent.com",
    "pypi.org",
    "registry.npmjs.org",
    "crates.io",
}
ALLOWED_NON_NETWORK_MODES = {"generic_content", "rpc_error", "tool_error", "timeout"}
EXPECTED_OPERATIONAL_CLASSES = {"protocol", "tool_execution", "timeout"}
REQUIRED_METRICS = {
    "expected_grounded_target_coverage",
    "false_target_abstention",
    "expected_unknown_preservation",
    "unsupported_grounded_claims",
    "missed_target_insufficiency",
    "external_acquisition_attempts",
    "external_acquisition_successes",
    "verification_successes",
    "identity_unsafe_admission",
    "stale_rejection",
    "authority_rejection",
    "scope_rejection",
    "conflict_rejection",
    "typed_operational_failures",
    "model_latency_ms",
    "external_latency_ms",
    "input_tokens",
    "output_tokens",
    "total_tokens",
}
REQUIRED_ZERO_GATES = {
    "unsupported_grounded_claims",
    "missed_target_insufficiency",
    "identity_unsafe_admission",
    "mcp_output_authority_self_promotion",
}
FORBIDDEN_HISTORICAL_TOKENS = {
    "#193", "#195", "#196", "issue-193", "issue-195", "issue-196",
    "mcp-identity-context", "mcp_identity_gate", "wikidata", "wikipedia",
}
FORBIDDEN_WRITE_KEYS = {
    "body", "command", "delete", "headers", "method", "mutation", "patch",
    "payload", "post", "put", "update", "write",
}


def fail(message: str) -> None:
    raise ValueError(message)


def load_json(path: pathlib.Path) -> Any:
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot load JSON {path}: {error}")


def verify_manifest(root: pathlib.Path, manifest_path: pathlib.Path) -> None:
    declared: dict[str, str] = {}
    for line in manifest_path.read_text().splitlines():
        if not line.strip():
            continue
        parts = line.split()
        if len(parts) != 2 or len(parts[0]) != 64:
            fail(f"invalid manifest line: {line!r}")
        digest, name = parts
        if name in declared:
            fail(f"duplicate manifest entry: {name}")
        declared[name] = digest.lower()
    actual_names = {path.name for path in root.glob("*.json")}
    if set(declared) != actual_names:
        fail(f"manifest membership mismatch: declared={sorted(declared)} actual={sorted(actual_names)}")
    for name, expected in declared.items():
        actual = hashlib.sha256((root / name).read_bytes()).hexdigest()
        if actual != expected:
            fail(f"SHA-256 mismatch for {name}: expected {expected}, got {actual}")


def target_pairs(root: pathlib.Path) -> set[tuple[str, str]]:
    result: set[tuple[str, str]] = set()
    for path in root.glob("[0-9][0-9]_*.json"):
        case = load_json(path)
        target = case.get("target_proposition", {})
        key, value = target.get("key"), target.get("value")
        if isinstance(key, str) and isinstance(value, str):
            result.add((key, value))
    return result


def dogfood_targets(repo_root: pathlib.Path) -> set[tuple[str, str]]:
    result: set[tuple[str, str]] = set()
    for dirname in ("product-dogfood-v1", "product-dogfood-v2", "product-dogfood-holdout-v1"):
        for path in (repo_root / "fixtures" / dirname).glob("*.json"):
            fixture = load_json(path)
            for proposition in fixture.get("input", {}).get("hypotheses", []):
                key, value = proposition.get("key"), proposition.get("value")
                if isinstance(key, str) and isinstance(value, str):
                    result.add((key, value))
    return result


def validate_url(url: str) -> None:
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https":
        fail(f"non-HTTPS acquisition URL: {url}")
    if parsed.username or parsed.password or parsed.port not in (None, 443):
        fail(f"credential or non-default port in acquisition URL: {url}")
    host = (parsed.hostname or "").lower()
    if host not in ALLOWED_HOSTS:
        fail(f"non-allowlisted acquisition host {host!r}: {url}")
    if not parsed.path.startswith("/"):
        fail(f"acquisition URL must have an absolute path: {url}")


def validate_profile(profile: dict[str, Any], case_id: str) -> None:
    required = {
        "id", "adapter", "server_id", "program", "args", "protocol_version", "transport",
        "stateless_tools_call", "tool", "allowed_tools", "read_only", "resolver_class",
        "http_method", "source_identity", "timeout_ms", "fixed_arguments", "argument_ownership",
    }
    if set(profile) != required:
        fail(f"{case_id}: acquisition profile keys must be exact")
    if profile["adapter"] != EXPECTED_ADAPTER or profile["server_id"] != EXPECTED_SERVER_ID:
        fail(f"{case_id}: adapter/server identity changed")
    if profile["program"] != "python3" or profile["args"] != ["scripts/product_external_info_mcp.py"]:
        fail(f"{case_id}: fixture server command changed")
    if profile["protocol_version"] != EXPECTED_PROTOCOL or profile["transport"] != "stdio":
        fail(f"{case_id}: MCP protocol/transport mismatch")
    if profile["stateless_tools_call"] is not True:
        fail(f"{case_id}: stateful/session MCP is forbidden in #206")
    if profile["tool"] != EXPECTED_TOOL or profile["allowed_tools"] != [EXPECTED_TOOL]:
        fail(f"{case_id}: tool allowlist changed")
    if profile["read_only"] is not True or profile["resolver_class"] != EXPECTED_RESOLVER_CLASS:
        fail(f"{case_id}: resolver must remain read-only acquisition-only")
    if not isinstance(profile["timeout_ms"], int) or profile["timeout_ms"] <= 0:
        fail(f"{case_id}: timeout must be positive")
    fixed = profile["fixed_arguments"]
    ownership = profile["argument_ownership"]
    if not isinstance(fixed, dict) or not isinstance(ownership, dict):
        fail(f"{case_id}: fixed arguments/ownership must be objects")
    if ownership.get("source_identity") != "harness_config":
        fail(f"{case_id}: source identity must be Harness-owned")
    for key in fixed:
        if ownership.get(key) != "harness_fixed":
            fail(f"{case_id}: fixed argument {key!r} is not Harness-owned")
        if key.lower() in FORBIDDEN_WRITE_KEYS:
            fail(f"{case_id}: writable/arbitrary request key forbidden: {key}")
    if set(ownership) - set(fixed) - {"source_identity"}:
        fail(f"{case_id}: ownership includes non-fixed fields")
    mode = fixed.get("mode", "live_json")
    if mode == "live_json":
        if profile["http_method"] != "GET":
            fail(f"{case_id}: network acquisition must be GET only")
        for field in ("url", "value_pointer", "fact_key", "authority_class", "identity_assertions"):
            if field not in fixed:
                fail(f"{case_id}: live_json missing {field}")
        validate_url(str(fixed["url"]))
        if not isinstance(fixed["value_pointer"], str) or not fixed["value_pointer"].startswith("/"):
            fail(f"{case_id}: selected field must be a fixed JSON Pointer")
        assertions = fixed["identity_assertions"]
        if not isinstance(assertions, list):
            fail(f"{case_id}: identity assertions must be an array")
        for assertion in assertions:
            if set(assertion) != {"pointer", "equals"} or not str(assertion["pointer"]).startswith("/"):
                fail(f"{case_id}: invalid identity assertion")
    else:
        if mode not in ALLOWED_NON_NETWORK_MODES:
            fail(f"{case_id}: unsupported deterministic mode {mode!r}")
        if profile["http_method"] is not None:
            fail(f"{case_id}: non-network mode must not claim HTTP method")


def validate_case(case: dict[str, Any], forbidden_targets: set[tuple[str, str]], v1_ids: set[str]) -> None:
    required = {
        "schema_version", "id", "capability_family", "task", "expected_outcome",
        "target_proposition", "acquisition_profiles", "source_identities",
        "expected_admission_behavior", "freshness_requirement", "authority_requirement",
        "scope_requirement", "expected_operational_class", "admission_policy",
    }
    if set(case) != required:
        fail(f"{case.get('id','<unknown>')}: case keys must be exact")
    case_id = case["id"]
    if case["schema_version"] != EXPECTED_CASE_SCHEMA:
        fail(f"{case_id}: case schema mismatch")
    if case_id in v1_ids:
        fail(f"{case_id}: v1 case identity reused")
    if case["capability_family"] not in EXPECTED_FAMILIES:
        fail(f"{case_id}: unknown family")
    target = case["target_proposition"]
    if set(target) != {"identity", "key", "value"}:
        fail(f"{case_id}: target shape changed")
    if target["identity"] != f"{EXPECTED_CORPUS_ID}:{case_id}:target":
        fail(f"{case_id}: target identity is not successor-scoped")
    pair = (target["key"], target["value"])
    if pair in forbidden_targets:
        fail(f"{case_id}: target proposition reuses prior frozen target {pair}")
    profiles = case["acquisition_profiles"]
    if not isinstance(profiles, list) or not profiles:
        fail(f"{case_id}: at least one acquisition profile required")
    if len({p.get('id') for p in profiles}) != len(profiles):
        fail(f"{case_id}: duplicate acquisition profile ID")
    for profile in profiles:
        validate_profile(profile, case_id)
    sources = [profile["source_identity"] for profile in profiles]
    if case["source_identities"] != sources:
        fail(f"{case_id}: source list must exactly follow profiles")
    admission = case["admission_policy"]
    if admission.get("evaluation_time_owner") != "harness":
        fail(f"{case_id}: evaluation time must be Harness-owned")
    if case["freshness_requirement"].get("evaluation_time_owner") != "harness":
        fail(f"{case_id}: freshness must be Harness-owned")
    if case["authority_requirement"].get("policy_owner") != "harness":
        fail(f"{case_id}: authority must be Harness-owned")
    if case["scope_requirement"].get("policy_owner") != "harness":
        fail(f"{case_id}: scope must be Harness-owned")
    if admission.get("minimum_authority_class") != case["authority_requirement"].get("minimum_class"):
        fail(f"{case_id}: authority requirement diverged")
    if admission.get("required_scope") != case["scope_requirement"].get("required_scope"):
        fail(f"{case_id}: scope requirement diverged")
    if set(admission.get("sources", {})) != set(sources):
        fail(f"{case_id}: admission source policy mismatch")
    for source, policy in admission["sources"].items():
        if policy.get("max_age_seconds") != case["freshness_requirement"].get("max_age_seconds"):
            fail(f"{case_id}: freshness source policy mismatch for {source}")
        if policy.get("scope") != admission.get("required_scope"):
            fail(f"{case_id}: source scope mismatch for {source}")
    operational = case["capability_family"] == "typed_operational_failure"
    if operational:
        if case["expected_outcome"] != "operational_failure":
            fail(f"{case_id}: operational case mixed into semantic outcomes")
        if case["expected_operational_class"] not in EXPECTED_OPERATIONAL_CLASSES:
            fail(f"{case_id}: invalid operational class")
    else:
        if case["expected_outcome"] not in {"grounded", "unknown"}:
            fail(f"{case_id}: invalid semantic outcome")
        if case["expected_operational_class"] is not None:
            fail(f"{case_id}: semantic case declares operational class")


def validate_corpus(repo_root: pathlib.Path, fixture_root: pathlib.Path, manifest: pathlib.Path) -> None:
    verify_manifest(fixture_root, manifest)
    v1_root = repo_root / "fixtures" / "product-external-info-v1"
    v1_manifest = repo_root / "fixtures" / "product-external-info-v1.sha256"
    verify_manifest(v1_root, v1_manifest)
    legacy_root = repo_root / "fixtures" / "product-dogfood-v1"
    verify_manifest(legacy_root, repo_root / "fixtures" / "product-dogfood-v1.sha256")
    if len(list(legacy_root.glob("*.json"))) != 6:
        fail("product-dogfood-v1 must remain exactly six immutable fixtures")

    corpus = load_json(fixture_root / "corpus.json")
    if corpus.get("schema_version") != EXPECTED_CORPUS_SCHEMA or corpus.get("corpus_identity") != EXPECTED_CORPUS_ID:
        fail("successor corpus identity/schema mismatch")
    if corpus.get("freeze_state") != "pre_observation" or corpus.get("baseline_main") != EXPECTED_BASELINE:
        fail("successor freeze/baseline mismatch")
    historical = corpus.get("historical_observation_policy", {})
    if historical.get("v1_run") != "33974104359" or historical.get("v1_result_is_immutable") is not True:
        fail("v1 historical evidence is not pinned immutable")
    for key in ("v1_cases_reused", "v1_target_pairs_reused", "historical_identity_holdouts_reused"):
        if historical.get(key) is not False:
            fail(f"historical reuse forbidden: {key}")
    if historical.get("post_observation_tuning_forbidden") is not True:
        fail("post-observation tuning must be forbidden")
    semantic = corpus.get("semantic_contract", {})
    if semantic.get("identity") != EXPECTED_SEMANTIC_ID:
        fail("successor semantic identity mismatch")
    if semantic.get("vendor_or_entity_specific_finalization") is not False:
        fail("entity/vendor-specific finalization forbidden")
    if semantic.get("global_verdict_promotion") is not False or semantic.get("reject_to_accept_promotion") is not False:
        fail("global verdict promotion forbidden")
    contract = corpus.get("mcp_contract", {})
    if contract.get("adapter") != EXPECTED_ADAPTER or contract.get("protocol_version") != EXPECTED_PROTOCOL:
        fail("mcp_readonly_v1 contract changed")
    if contract.get("stateless_single_tools_call") is not True or contract.get("session_negotiation_changes_in_scope") is not False:
        fail("#204 session/negotiation scope leaked into #206")
    if contract.get("generic_content_fact_promotion") is not False or contract.get("authority_owner") != "harness":
        fail("MCP authority boundary weakened")
    server = corpus.get("fixture_server", {})
    if server.get("server_id") != EXPECTED_SERVER_ID or server.get("tool") != EXPECTED_TOOL:
        fail("fixture server identity/tool mismatch")
    if server.get("read_only") is not True or server.get("resolver_class") != EXPECTED_RESOLVER_CLASS:
        fail("fixture server not read-only acquisition-only")
    if server.get("network_method") != "GET" or set(server.get("allowed_hosts", [])) != ALLOWED_HOSTS:
        fail("fixture server network boundary changed")
    four = corpus.get("four_arm_contract", {})
    if four.get("identity") != EXPECTED_FOUR_ARM_ID or four.get("arms") != EXPECTED_ARMS:
        fail("four-arm identity/order mismatch")
    if four.get("primary_comparison") != PRIMARY_COMPARISON:
        fail("primary comparison must be arm 3 vs arm 4")
    if four.get("shared_external_snapshot_between_arms_3_and_4") is not True or four.get("single_live_acquisition_per_case") is not True:
        fail("external snapshot fairness contract weakened")
    if four.get("arm_3_harness_admission_or_verification") is not False or four.get("arm_3_mcp_output_is_authority") is not False:
        fail("raw+external arm improperly gains Harness authority")
    scoring = corpus.get("scoring_contract", {})
    if scoring.get("identity") != EXPECTED_SCORING_ID or scoring.get("comparison_arms") != EXPECTED_ARMS:
        fail("scoring identity/arms mismatch")
    if scoring.get("primary_comparison") != PRIMARY_COMPARISON or set(scoring.get("metrics", [])) != REQUIRED_METRICS:
        fail("scoring metrics or primary comparison changed")
    if scoring.get("semantic_denominator_excludes_operational_failures") is not True:
        fail("operational failures must stay outside semantic denominator")
    if scoring.get("expected_unknown_preservation_must_equal") != 1.0:
        fail("expected-unknown preservation gate weakened")
    gates = scoring.get("safety_gates", {})
    if set(gates) != REQUIRED_ZERO_GATES or any(gates[k] != 0 for k in REQUIRED_ZERO_GATES):
        fail("zero safety gates changed")
    live = corpus.get("live_observation_contract", {})
    if (live.get("provider"), live.get("model"), live.get("seed"), live.get("max_tokens")) != (EXPECTED_PROVIDER, EXPECTED_MODEL, EXPECTED_SEED, EXPECTED_MAX_TOKENS):
        fail("live observation conditions changed after declaration")
    if live.get("first_valid_run_is_canonical") is not True or live.get("case_or_scoring_change_after_observation_requires_new_identity") is not True:
        fail("live observation freeze discipline weakened")

    case_paths = sorted(fixture_root.glob("[0-9][0-9]_*.json"))
    if len(case_paths) != 21:
        fail(f"expected 21 successor cases, found {len(case_paths)}")
    cases = [load_json(path) for path in case_paths]
    ids = [case.get("id") for case in cases]
    if len(ids) != len(set(ids)) or corpus.get("cases") != ids:
        fail("case order/membership/identity mismatch")
    counts = collections.Counter(case.get("capability_family") for case in cases)
    if set(counts) != EXPECTED_FAMILIES or any(count != 3 for count in counts.values()):
        fail(f"expected 7 families x 3, got {dict(counts)}")

    v1_cases = [load_json(path) for path in sorted(v1_root.glob("[0-9][0-9]_*.json"))]
    v1_ids = {case["id"] for case in v1_cases}
    forbidden_targets = target_pairs(v1_root) | dogfood_targets(repo_root)
    seen_pairs: set[tuple[str, str]] = set()
    for case in cases:
        validate_case(case, forbidden_targets, v1_ids)
        pair = (case["target_proposition"]["key"], case["target_proposition"]["value"])
        if pair in seen_pairs:
            fail(f"duplicate successor target pair: {pair}")
        seen_pairs.add(pair)

    text = "\n".join(path.read_text().lower() for path in [fixture_root / "corpus.json", *case_paths])
    for token in FORBIDDEN_HISTORICAL_TOKENS:
        if token.lower() in text:
            fail(f"historical identity holdout token forbidden in successor: {token}")
    if "resolver_facts" in text:
        fail("resolver_facts forbidden in live MCP successor")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[1])
    parser.add_argument("--fixtures", type=pathlib.Path)
    parser.add_argument("--manifest", type=pathlib.Path)
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()
    fixture_root = (args.fixtures or repo_root / "fixtures" / EXPECTED_CORPUS_ID).resolve()
    manifest = (args.manifest or repo_root / "fixtures" / f"{EXPECTED_CORPUS_ID}.sha256").resolve()
    try:
        validate_corpus(repo_root, fixture_root, manifest)
    except (ValueError, OSError) as error:
        print(f"product external-info successor freeze validation failed: {error}", file=sys.stderr)
        return 2
    print(json.dumps({
        "schema_version": "product-external-info-successor-freeze-validation-v2",
        "corpus_identity": EXPECTED_CORPUS_ID,
        "semantic_identity": EXPECTED_SEMANTIC_ID,
        "scoring_identity": EXPECTED_SCORING_ID,
        "four_arm_identity": EXPECTED_FOUR_ARM_ID,
        "cases": 21,
        "capability_families": 7,
        "historical_v1_reused": False,
        "legacy_product_dogfood_v1_cases": 6,
        "provider": EXPECTED_PROVIDER,
        "model": EXPECTED_MODEL,
        "seed": EXPECTED_SEED,
        "max_tokens": EXPECTED_MAX_TOKENS,
        "live_observation_performed": False,
        "valid": True,
    }, sort_keys=True))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
