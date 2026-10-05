# Engine 0.6 candidate: evidence-target relevance calibration v9

Status: pre-freeze successor to immutable v8 FAIL. v1-v8 observations, tags, fixtures, and scores remain historical evidence and must not be rerun or rescored.

## Why v9 exists

Frozen v8 run `36090688415` exposed three independent boundaries that v9 addresses:

- non-exact primary target binding can be `different` **or** `unresolved`, so negative confirmation must cover both;
- `exact/exact` primary output is still advisory and can falsely promote mixed/shared material to `relevant`, so the Relevant path needs an independent positive target-local confirmation;
- a tiny confirmation contract should not depend on JSON Schema transport. v9 confirmations use strict enum-only text with no fuzzy extraction, substring recovery, semantic repair, or runner-level retry.

Google `gemini-3.5-flash-lite` was operationally requalified by frozen run `36078211994` (26/26 operational, all HTTP 200 first attempt), but historical v4-v6 serving-tail instability remains part of the record. Google is therefore full non-gating replication in v9, not a required arm.

## Materialization policy v5

The primary binding proposal remains advisory:

- `target=different` or `target=unresolved` invokes negative target-local confirmation;
- `confirmed_distinct_entity` -> `irrelevant`;
- `confirmed_local_target_absent` -> `irrelevant`;
- `not_confirmed` or missing confirmation -> `ambiguous`;
- `target=exact, relation=different` -> `irrelevant` without confirmation;
- `target=exact, relation=unresolved` -> `ambiguous`;
- `target=exact, relation=exact` can reach `relevant` only after positive confirmation `confirmed_target_local_binding`;
- positive `not_confirmed` or missing confirmation -> `ambiguous`.

Positive confirmation is deliberately local. A page-level target name, shared heading, or target-looking URL cannot establish ownership of an ambiguous row/section. Uncertain rename, alias, successor, lineage, cross-language mapping, truncated identity, or mixed-product ownership must abstain.

The Harness continues to own target identity, aliases, provenance, policy, and final disposition. Relevance remains distinct from truth, authority, freshness, verification, and sufficiency. Candidate instructions remain untrusted data.

## Confirmation transport and budgets

Primary binding keeps its existing bounded transport:

- maximum model calls: 2 (`JsonSchema`, then provider-neutral JSON-object fallback only when required);
- no runner-level retry loop.

Each case may then invoke at most one separate confirmation call:

- output format: `Text`;
- negative token set: `confirmed_distinct_entity | confirmed_local_target_absent | not_confirmed`;
- positive token set: `confirmed_target_local_binding | not_confirmed`;
- exact trimmed token parsing only;
- malformed text, provider failure, and timeout are typed operational failures;
- confirmation has no JSON fallback and no semantic repair.

Primary and confirmation stages share the existing 60,000 ms case deadline. The runner still opens its run circuit after two consecutive operational provider failures. Provider-adapter internal bounded retry behavior is unchanged.

## Calibration corpus

v9 contains 47 synthetic calibration cases:

- the 32 v8 synthetic cases are retained as regression/comparability cases with v9 confirmation expectations;
- 15 independently authored fresh identities/texts cover exact-binding conflict, positive exact local binding, shared-table ownership ambiguity, explicit distinct product, sibling overlap, generic local target absence, explicit local target absence, unknown rename, unknown successor, unknown cross-language alias, partial/truncated mapping, prompt injection, URL-only identity, mixed multi-product ownership, and same-target relation mismatch.

The fresh cases use new synthetic product identities and wording rather than rewriting only the v8 misses. Production motivating product content is excluded. Independent holdout authoring remains forbidden until this canonical calibration passes.

## Provider roles

Required arms:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:

- Google `gemini-3.5-flash-lite`, with attempt telemetry enabled.

All three use the same frozen 47-case corpus. Google results are preserved and reported but cannot rescue or fail the required gate.

## First/only canonical acceptance

The frozen tag is `engine-0.6-evidence-relevance-calibration-v9-freeze`. Workflow reruns are rejected; the tag is never overwritten.

Each required arm must independently satisfy:

- planned/completed: 47/47;
- operational abort: none;
- failed provider cases: 0;
- unsafe/wrong-target relevance retention: 0;
- false relevance rejection: 0;
- expected Relevant left ambiguous: 0;
- utility miss: 0;
- materialized disposition exact: 47/47;
- false safe-negative confirmation: 0;
- false positive target-local confirmation: 0.

Primary proposal accuracy and exact confirmation subtype accuracy are diagnostic. A safe negative subtype disagreement (`confirmed_distinct_entity` vs `confirmed_local_target_absent`) does not fail the gate when the final safety/disposition boundary is correct.

If canonical v9 fails, it becomes immutable FAIL and is not rerun/rescored. Only a canonical v9 PASS permits authoring a fresh independent holdout, followed by runtime acceptance. PR #466 remains Draft until those stages are complete.
