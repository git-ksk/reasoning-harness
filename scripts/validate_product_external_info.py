#!/usr/bin/env python3
"""Fail-closed validator for the pre-observation product external-information v1 freeze."""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import pathlib
import sys
import urllib.parse
from typing import Any

EXPECTED_CORPUS_ID = "product-external-info-v1"
EXPECTED_CASE_SCHEMA = "product-external-info-case-v1"
EXPECTED_PROTOCOL = "2026-07-28"
EXPECTED_ADAPTER = "mcp_readonly_v1"
EXPECTED_RESOLVER_CLASS = "evidence_acquisition"
EXPECTED_TOOL = "fetch_json_fact"
EXPECTED_SERVER_ID = "product_external_info_fixture_v1"
EXPECTED_FAMILIES = {
    "current_fresh_snapshot",
    "entity_identity_ambiguity",
    "authoritative_source_requirement",
    "stale_fresh_or_source_conflict",
    "no_result_or_insufficient_evidence",
    "typed_operational_failure",
    "irrelevant_or_instruction_like_content",
}
ALLOWED_HOSTS = {
    "api.github.com",
    "raw.githubusercontent.com",
    "pypi.org",
    "registry.npmjs.org",
    "crates.io",
}
ALLOWED_NON_NETWORK_MODES = {"generic_content", "rpc_error", "tool_error", "timeout"}
EXPECTED_OPERATIONAL_CLASSES = {"protocol", "tool_execution", "timeout"}
FORBIDDEN_CORPUS_TOKENS = {
    "#193",
    "#195",
    "#196",
    "issue-193",
    "issue-195",
    "issue-196",
    "mcp-identity-context",
    "mcp_identity_gate",
    "wikidata",
    "wikipedia",
}
FORBIDDEN_WRITE_KEYS = {
    "body",
    "command",
    "delete",
    "headers",
    "method",
    "mutation",
    "patch",
    "payload",
    "post",
    "put",
    "update",
    "write",
}
REQUIRED_METRICS = {
    "external_acquisition_attempts",
    "external_acquisition_successes",
    "verification_successes",
    "expected_grounded_target_coverage",
    "expected_unknown_preservation",
    "false_target_abstention",
    "unsupported_grounded_claims",
    "missed_target_insufficiency",
    "identity_unsafe_admission",
    "stale_rejection",
    "authority_rejection",
    "scope_rejection",
    "conflict_rejection",
    "tool_protocol_timeout_operational_failures",
}
REQUIRED_ZERO_GATES = {
    "unsupported_grounded_claims",
    "missed_target_insufficiency",
    "identity_unsafe_admission",
    "mcp_output_authority_self_promotion",
}


def fail(message: str) -> None:
    raise ValueError(message)


def load_json(path: pathlib.Path) -> Any:
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot load JSON {path}: {error}")


def walk_keys(value: Any):
    if isinstance(value, dict):
        for key, child in value.items():
            yield str(key)
            yield from walk_keys(child)
    elif isinstance(value, list):
        for child in value:
            yield from walk_keys(child)


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


def collect_product_targets(repo_root: pathlib.Path) -> set[tuple[str, str]]:
    targets: set[tuple[str, str]] = set()
    for dirname in ("product-dogfood-v1", "product-dogfood-v2", "product-dogfood-holdout-v1"):
        for path in (repo_root / "fixtures" / dirname).glob("*.json"):
            fixture = load_json(path)
            for proposition in fixture.get("input", {}).get("hypotheses", []):
                key = proposition.get("key")
                value = proposition.get("value")
                if isinstance(key, str) and isinstance(value, str):
                    targets.add((key, value))
    return targets


def verify_legacy_dogfood_manifest(repo_root: pathlib.Path) -> None:
    root = repo_root / "fixtures" / "product-dogfood-v1"
    manifest = repo_root / "fixtures" / "product-dogfood-v1.sha256"
    verify_manifest(root, manifest)
    if len(list(root.glob("*.json"))) != 6:
        fail("product-dogfood-v1 must remain exactly six JSON fixtures")


def validate_profile(profile: dict[str, Any], case_id: str) -> None:
    required = {
        "id",
        "adapter",
        "server_id",
        "program",
        "args",
        "protocol_version",
        "transport",
        "stateless_tools_call",
        "tool",
        "allowed_tools",
        "read_only",
        "resolver_class",
        "http_method",
        "source_identity",
        "timeout_ms",
        "fixed_arguments",
        "argument_ownership",
    }
    if set(profile) != required:
        fail(f"{case_id}: acquisition profile keys must be exact; got {sorted(profile)}")
    if profile["adapter"] != EXPECTED_ADAPTER:
        fail(f"{case_id}: adapter changed")
    if profile["server_id"] != EXPECTED_SERVER_ID:
        fail(f"{case_id}: server identity changed")
    if profile["program"] != "python3" or profile["args"] != ["scripts/product_external_info_mcp.py"]:
        fail(f"{case_id}: fixture server command changed")
    if profile["protocol_version"] != EXPECTED_PROTOCOL or profile["transport"] != "stdio":
        fail(f"{case_id}: MCP protocol/transport mismatch")
    if profile["stateless_tools_call"] is not True:
        fail(f"{case_id}: stateful/session MCP is forbidden")
    if profile["tool"] != EXPECTED_TOOL or profile["allowed_tools"] != [EXPECTED_TOOL]:
        fail(f"{case_id}: tool must be explicitly allowlisted and fixed")
    if profile["read_only"] is not True or profile["resolver_class"] != EXPECTED_RESOLVER_CLASS:
        fail(f"{case_id}: resolver must remain read-only evidence acquisition")
    if not isinstance(profile["source_identity"], str) or not profile["source_identity"].strip():
        fail(f"{case_id}: source identity must be Harness-configured")
    if not isinstance(profile["timeout_ms"], int) or profile["timeout_ms"] <= 0:
        fail(f"{case_id}: timeout must be positive and bounded")

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
            fail(f"{case_id}: writable/arbitrary request key is forbidden: {key}")
    extra_ownership = set(ownership) - set(fixed) - {"source_identity"}
    if extra_ownership:
        fail(f"{case_id}: ownership contains non-fixed fields: {sorted(extra_ownership)}")

    mode = fixed.get("mode", "live_json")
    if mode == "live_json":
        if profile["http_method"] != "GET":
            fail(f"{case_id}: network acquisition must be GET only")
        for field in ("url", "value_pointer", "fact_key", "authority_class", "identity_assertions"):
            if field not in fixed:
                fail(f"{case_id}: live_json missing fixed argument {field}")
        validate_url(str(fixed["url"]))
        if not isinstance(fixed["value_pointer"], str) or not fixed["value_pointer"].startswith("/"):
            fail(f"{case_id}: selected field must be a fixed JSON Pointer")
        if not isinstance(fixed["fact_key"], str) or not fixed["fact_key"].strip():
            fail(f"{case_id}: fact key must be fixed")
        assertions = fixed["identity_assertions"]
        if not isinstance(assertions, list):
            fail(f"{case_id}: identity assertions must be fixed array")
        for assertion in assertions:
            if set(assertion) != {"pointer", "equals"} or not str(assertion["pointer"]).startswith("/"):
                fail(f"{case_id}: invalid fixed identity assertion")
    else:
        if mode not in ALLOWED_NON_NETWORK_MODES:
            fail(f"{case_id}: unsupported non-network deterministic mode {mode!r}")
        if profile["http_method"] is not None:
            fail(f"{case_id}: deterministic failure/content mode must not claim a network method")


def validate_case(case: dict[str, Any], product_targets: set[tuple[str, str]]) -> None:
    required = {
        "schema_version",
        "id",
        "capability_family",
        "task",
        "expected_outcome",
        "target_proposition",
        "acquisition_profiles",
        "source_identities",
        "expected_admission_behavior",
        "freshness_requirement",
        "authority_requirement",
        "scope_requirement",
        "expected_operational_class",
        "admission_policy",
    }
    if set(case) != required:
        fail(f"{case.get('id','<unknown>')}: case keys must be exact")
    case_id = case["id"]
    if case["schema_version"] != EXPECTED_CASE_SCHEMA:
        fail(f"{case_id}: schema version mismatch")
    if case["capability_family"] not in EXPECTED_FAMILIES:
        fail(f"{case_id}: unknown family")
    if not isinstance(case["task"], str) or not case["task"].strip():
        fail(f"{case_id}: task is empty")
    if "resolver_facts" in set(walk_keys(case)):
        fail(f"{case_id}: resolver_facts are forbidden in the live MCP workload")

    target = case["target_proposition"]
    if set(target) != {"identity", "key", "value"}:
        fail(f"{case_id}: target proposition shape changed")
    if target["identity"] != f"{EXPECTED_CORPUS_ID}:{case_id}:target":
        fail(f"{case_id}: target identity is not corpus-scoped and frozen")
    if (target["key"], target["value"]) in product_targets:
        fail(f"{case_id}: target proposition duplicates prior product dogfood target identity")

    profiles = case["acquisition_profiles"]
    if not isinstance(profiles, list) or not profiles:
        fail(f"{case_id}: at least one acquisition profile is required")
    profile_ids = [profile.get("id") for profile in profiles]
    if len(profile_ids) != len(set(profile_ids)):
        fail(f"{case_id}: duplicate acquisition profile IDs")
    for profile in profiles:
        validate_profile(profile, case_id)
    sources = [profile["source_identity"] for profile in profiles]
    if case["source_identities"] != sources:
        fail(f"{case_id}: source identity list must exactly follow acquisition profiles")

    admission = case["admission_policy"]
    if admission.get("evaluation_time_owner") != "harness":
        fail(f"{case_id}: evaluation time must be Harness-owned")
    if case["freshness_requirement"].get("evaluation_time_owner") != "harness":
        fail(f"{case_id}: freshness requirement must be Harness-owned")
    if case["authority_requirement"].get("policy_owner") != "harness":
        fail(f"{case_id}: authority requirement must be Harness-owned")
    if case["scope_requirement"].get("policy_owner") != "harness":
        fail(f"{case_id}: scope requirement must be Harness-owned")
    if admission.get("minimum_authority_class") != case["authority_requirement"].get("minimum_class"):
        fail(f"{case_id}: authority requirement and admission config diverged")
    if admission.get("required_scope") != case["scope_requirement"].get("required_scope"):
        fail(f"{case_id}: scope requirement and admission config diverged")
    if set(admission.get("sources", {})) != set(sources):
        fail(f"{case_id}: admission source policy must exactly cover configured sources")
    for source, policy in admission["sources"].items():
        if policy.get("max_age_seconds") != case["freshness_requirement"].get("max_age_seconds"):
            fail(f"{case_id}: source freshness policy diverged for {source}")
        if policy.get("scope") != admission.get("required_scope"):
            fail(f"{case_id}: source scope exceeds or diverges from frozen required scope")

    operational = case["capability_family"] == "typed_operational_failure"
    if operational:
        if case["expected_outcome"] != "operational_failure":
            fail(f"{case_id}: operational case must remain outside semantic outcome classes")
        if case["expected_operational_class"] not in EXPECTED_OPERATIONAL_CLASSES:
            fail(f"{case_id}: invalid expected operational class")
    else:
        if case["expected_outcome"] not in {"grounded", "unknown"}:
            fail(f"{case_id}: semantic case has invalid expected outcome")
        if case["expected_operational_class"] is not None:
            fail(f"{case_id}: semantic case unexpectedly declares operational class")


def validate_corpus(repo_root: pathlib.Path, fixture_root: pathlib.Path, manifest_path: pathlib.Path) -> None:
    verify_manifest(fixture_root, manifest_path)
    verify_legacy_dogfood_manifest(repo_root)

    corpus_path = fixture_root / "corpus.json"
    corpus = load_json(corpus_path)
    if corpus.get("corpus_identity") != EXPECTED_CORPUS_ID or corpus.get("freeze_state") != "pre_observation":
        fail("corpus identity/freeze state mismatch")
    if corpus.get("baseline_main") != "aa0a8325ea4c3b53b38c8fe83cf3aae691a38599":
        fail("baseline main identity changed")
    contract = corpus.get("mcp_contract", {})
    if contract.get("adapter") != EXPECTED_ADAPTER or contract.get("protocol_version") != EXPECTED_PROTOCOL:
        fail("corpus MCP contract changed")
    if contract.get("generic_content_fact_promotion") is not False or contract.get("authority_owner") != "harness":
        fail("fail-closed generic MCP/authority boundary changed")
    server = corpus.get("fixture_server", {})
    if server.get("server_id") != EXPECTED_SERVER_ID or server.get("tool") != EXPECTED_TOOL:
        fail("fixture server identity/tool changed")
    if server.get("read_only") is not True or server.get("resolver_class") != EXPECTED_RESOLVER_CLASS:
        fail("fixture server is not read-only acquisition-only")
    if server.get("network_method") != "GET" or set(server.get("allowed_hosts", [])) != ALLOWED_HOSTS:
        fail("fixture server network allowlist/method changed")
    for field in (
        "model_generated_url",
        "model_generated_field",
        "model_generated_source_identity",
        "model_generated_authority",
        "model_generated_scope",
    ):
        if server.get(field) is not False:
            fail(f"fixture server permits forbidden model-generated control: {field}")

    direct = corpus.get("direct_generic_mcp_boundary", {})
    if direct.get("server") != "github/github-mcp-server" or direct.get("read_only_required") is not True:
        fail("direct GitHub MCP boundary profile changed")
    if direct.get("expected_fact_promotion_from_generic_output") is not False:
        fail("generic GitHub MCP output must remain non-promoting")

    families = corpus.get("capability_families")
    if set(families or []) != EXPECTED_FAMILIES or len(families or []) != 7:
        fail("corpus must freeze exactly seven capability families")

    scoring = corpus.get("scoring_contract", {})
    if scoring.get("identity") != "product-external-info-scoring-v1":
        fail("scoring identity changed")
    if scoring.get("comparison_arms") != [
        "raw_model",
        "harness_without_external_acquisition",
        "harness_with_mcp_external_acquisition",
    ]:
        fail("comparison arms changed")
    if set(scoring.get("metrics", [])) != REQUIRED_METRICS:
        fail("scoring metrics changed")
    if scoring.get("semantic_denominator_excludes_operational_failures") is not True:
        fail("operational failures must remain outside semantic denominator")
    gates = scoring.get("safety_gates", {})
    if set(gates) != REQUIRED_ZERO_GATES or any(gates[key] != 0 for key in REQUIRED_ZERO_GATES):
        fail("safety gates must remain the frozen zero-error contract")

    case_paths = sorted(fixture_root.glob("[0-9][0-9]_*.json"))
    if len(case_paths) != 21:
        fail(f"expected 21 cases, found {len(case_paths)}")
    cases = [load_json(path) for path in case_paths]
    ids = [case.get("id") for case in cases]
    if len(ids) != len(set(ids)):
        fail("case IDs must be unique")
    if corpus.get("cases") != ids:
        fail("corpus case order/membership differs from frozen fixture order")
    counts = collections.Counter(case.get("capability_family") for case in cases)
    if set(counts) != EXPECTED_FAMILIES or any(count != 3 for count in counts.values()):
        fail(f"expected exactly 7 families x 3 cases, got {dict(counts)}")

    product_targets = collect_product_targets(repo_root)
    target_ids: set[str] = set()
    target_pairs: set[tuple[str, str]] = set()
    for case in cases:
        validate_case(case, product_targets)
        target = case["target_proposition"]
        if target["identity"] in target_ids:
            fail(f"duplicate target identity: {target['identity']}")
        target_ids.add(target["identity"])
        pair = (target["key"], target["value"])
        if pair in target_pairs:
            fail(f"duplicate target proposition: {pair}")
        target_pairs.add(pair)

    corpus_text = "\n".join(path.read_text().lower() for path in [corpus_path, *case_paths])
    for token in FORBIDDEN_CORPUS_TOKENS:
        if token.lower() in corpus_text:
            fail(f"historical identity-research reference is forbidden in new corpus: {token}")
    if "resolver_facts" in corpus_text:
        fail("resolver_facts are forbidden anywhere in the new corpus")


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
        print(f"product external-info freeze validation failed: {error}", file=sys.stderr)
        return 2
    print(json.dumps({
        "schema_version": "product-external-info-freeze-validation-v1",
        "corpus_identity": EXPECTED_CORPUS_ID,
        "cases": 21,
        "capability_families": 7,
        "manifest": str(manifest.relative_to(repo_root)),
        "legacy_product_dogfood_v1_cases": 6,
        "live_observation_performed": False,
        "valid": True,
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
