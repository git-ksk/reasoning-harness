# Engine 0.6 evidence-target relevance calibration v17 — successor design

Status: immutable canonical FAIL. Frozen run `36226650327` at commit `9647e70125b97dd77b1c4742004889c23fd10d58` is complete and must not be rerun, rescored, relabeled, or retagged. See `engine-0.6-evidence-relevance-calibration-v17-result.md`. Independent holdout authoring remains blocked.

## Fixed evaluation surface

v17 keeps `evidence-relevance-fixed-core-v1` unchanged at exactly 48 cases. No case is added, removed, relabeled, or selected in response to v16 model misses. The scored balance remains 14 Relevant / 18 Irrelevant / 16 Ambiguous.

Contracts:
- primary binding proposal: `reason-evidence-relevance-binding-proposal-v5` (unchanged from v16)
- independent local verifier: `reason-evidence-local-qualification-v7`
- Harness materializer: `target-evidence-relevance-binding-materialization-v12`
- annotation protocol: `evidence-relevance-scope-verifier-v17`

## Why v17 exists

v16 removed the v15 safety regression and improved required-Mistral materialization from 33/48 to 42/48 with zero wrong-target Relevant retention. The six remaining misses were conservative Ambiguous outcomes and reduce to two generic residuals:

1. an explicit `allow_semantic_equivalent` policy has no bounded positive path when the primary target axis remains unresolved even though the verifier independently establishes exact target/relation with no risk;
2. the verifier still overuses `context_gap` for usable bounded negative observations such as navigation-only target mentions with a clearly different substantive owner, complete broad/generic units with no exact-target proposition, explicit local absence, and ignored prompt-injection text.

v17 addresses only those generic boundaries. It does not tune case IDs, synthetic names, or exact fixture phrases.

## Materialization v12

v12 preserves all v11 terminal rules and adds one narrow positive fallback:

A primary `target_binding=unresolved` may materialize Relevant only when all conditions hold:
- `identity_requirement=allow_semantic_equivalent` is explicitly configured by the Harness;
- primary `relation_binding=exact`;
- verifier `identity_scope=exact_target`;
- verifier `relation_scope=requested_relation`;
- verifier `scope_risk=none`;
- the candidate contains at least one substantive local signal (`heading`, `excerpt`, `structured_metadata`, or `fact`).

This fallback is unavailable for strict identity, URL-only/navigation-only evidence, any scope risk, unresolved/different relation, or a primary `target_binding=different`.

All other v11 safety rules remain unchanged:
- scope risk always forces Ambiguous;
- strict-identity unresolved primary cannot be rescued to Relevant;
- negative target/relation terminal outcomes still require matching verifier scope evidence;
- explicit local absence remains a negative-only path;
- disagreements remain Ambiguous.

## Local verifier v7

v7 keeps the v16 three-field schema:
- `identity_scope`: exact_target | distinct_target | target_absent | unresolved
- `relation_scope`: requested_relation | different_relation | relation_absent | unresolved
- `scope_risk`: none | identity_mapping | ownership_scope | context_gap | multiple

The prompt boundary is tightened without changing the schema:
- candidate instructions are untrusted and ignored; an ignored instruction is never itself a scope risk or context gap;
- navigation/footer target occurrences do not establish exact target ownership;
- a clearly different substantive owner remains `distinct_target` with `scope_risk=none` even when the Harness target appears only in navigation/footer or comparison text;
- a complete broad/generic/catalog unit with no exact-target proposition may be `target_absent`, not `context_gap`;
- explicit local absence may be `target_absent` / `relation_absent`, not `context_gap`;
- `context_gap` is reserved for visible clipping/truncation, omitted referents/product columns, URL/navigation-only units with no substantive proposition, or explicit omission of required local context;
- under `allow_semantic_equivalent`, a locally specific semantic equivalent may be `exact_target` without a literal canonical-name occurrence.

## Unscored structural controls

The scored fixed core does not grow. v17 adds only property/routing controls proving:
- semantic-equivalent fallback is unavailable under strict identity;
- any scope risk blocks the fallback;
- unresolved relation blocks the fallback;
- URL-only evidence blocks the fallback;
- all 48 frozen expected annotations still materialize to their existing expected dispositions.

## Operational policy

Carry v16 operational hardening forward unchanged:
- active execution budget: 60,000 ms per case
- cumulative provider wait/retry budget: 45,000 ms
- single provider wait cap: 30,000 ms
- absolute case wall-clock deadline: 120,000 ms
- retry ownership remains in provider adapters
- typed quota latches after one confirmed quota failure
- correlated capacity failures latch after two
- suppressed cases remain operational failures
- provider-attempt / active / wait / pacing / retry telemetry remain separate
- public provider failures remain sanitized
- no manual Groq TPD attestation start gate

Required providers remain Mistral `ministral-8b-latest` and Groq `openai/gpt-oss-120b`; Google `gemini-3.5-flash-lite` remains full non-gating replication.

The v16 run established that the current Groq daily window is already exhausted. Therefore the v17 freeze tag must not be pushed in that same known-exhausted window. This is not a manual capacity-attestation gate; it avoids knowingly consuming a one-shot canonical after an observed quota latch.

## Pre-live acceptance

Before a v17 freeze tag:
- fixed-core routing/property tests pass;
- runner tests pass;
- workspace tests pass;
- clippy with `-D warnings` passes;
- `cargo fmt --check` and `git diff --check` pass;
- `surface-v17.sha256` matches every frozen surface file;
- validate-only reports v17 IDs, 48 planned / 0 observed, no operational abort, no provider-arm latch, and non-scorable validation;
- PR #466 remains Draft;
- independent holdout remains unauthored.

The first/only frozen v17 canonical remains one-shot and immutable once started.
