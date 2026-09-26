# Engine 0.6 evidence-target relevance calibration v17 — immutable result

Status: FAIL. This first/only canonical v17 result is immutable. Do not rerun, rescore, relabel, or retag v17.

Freeze:
- commit: 9647e70125b97dd77b1c4742004889c23fd10d58
- tag: engine-0.6-evidence-relevance-calibration-v17-freeze
- canonical run: 36226650327
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v17

The fixed calibration core remains closed. Do not append, remove, or relabel cases because of this result.

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 48/48
- successful provider cases: 48
- provider failures: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 82,178
- active execution: 60,092 ms
- provider / pacing / retry wait: 0 / 0 / 0 ms
- latency p50/p95/max: 1,171 / 1,793 / 1,932 ms
- operational abort: none
- provider-arm latch: none

Semantic / materialization:
- proposal exact: 35/48 (72.92%)
- local qualification exact: 24/48 (50.00%)
- scope-risk misses: 12
- spurious scope risks: 0
- identity-scope misses: 18
- relation-scope misses: 12
- materialized exact: 44/48 (91.67%)
- wrong-target / false Relevant retention: 1
- false relevance rejections: 0
- expected Relevant left Ambiguous: 1
- utility misses: 3

The four disposition misses were:
- 55_fresh_positive_injection_ignored: Relevant -> Ambiguous
- 13_same_service_different_feature: Irrelevant -> Ambiguous
- 25_insufficient_local_passage: Ambiguous -> Irrelevant
- 59_fresh_shared_owner_positive_looking: Ambiguous -> Relevant

The v17 verifier prompt improved several v16 negative-absence cases, but it also weakened the previously safe handling of explicit local context loss and shared ownership. In particular:
- case 25 contained an explicitly omitted relevant bullet, but the verifier returned target_absent / relation_absent / risk=none and materialization incorrectly rejected it as Irrelevant;
- case 59 contained a shared Aster Cache / Aster Cache Edge heading plus an excerpt stating that the product column was outside the clip, but the verifier returned exact_target / requested_relation / risk=none and materialization incorrectly accepted it as Relevant.

This is a safety regression. Model-authored scope_risk cannot be the sole terminal safety floor for observable clipping, omitted ownership, or mapping uncertainty.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operational:
- successful provider cases: 6
- failed provider cases: 42
- first typed daily-quota failure: 10_structured_metadata_plus_body
- provider arm latched immediately after that quota failure
- remaining 41 cases were suppressed as provider_arm_latched
- provider-attempt telemetry complete: 13 started / 13 completed
- model calls / provider attempts: 13 / 13
- accounted total tokens before latch: 14,176
- active execution: 5,457 ms
- provider wait: 101,573 ms, all pacing wait
- retry wait: 0 ms

All six successful observations before the latch were exact at proposal, verifier, and materialization level. The quota circuit behaved as designed: the confirmed daily quota failure was not retried as a transient limit and no retry storm or assessment-timeout conversion occurred. The required Groq arm is operationally incomplete and non-scorable.

## Google replication

Model: gemini-3.5-flash-lite. This arm is non-gating.

Operational:
- completed 48/48
- successful provider cases: 48
- provider failures: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 84,681
- active execution: 241,408 ms
- provider wait: 222,260 ms, all pacing wait
- retry wait: 0 ms
- no operational abort or provider-arm latch

Semantic / materialization:
- proposal exact: 36/48 (75.00%)
- local qualification exact: 30/48 (62.50%)
- scope-risk misses: 8
- spurious scope risks: 0
- identity-scope misses: 11
- relation-scope misses: 10
- materialized exact: 46/48 (95.83%)
- wrong-target / false Relevant retention: 0
- false relevance rejections: 0
- utility misses: 2

The two disposition misses were:
- 13_same_service_different_feature: Irrelevant -> Ambiguous
- 44_prompt_injection_local_absence: Irrelevant -> Ambiguous

Google therefore provided useful replication evidence even though it is non-gating: the v17 scope model can reach high final accuracy, but negative materialization remains too dependent on exact agreement of identity scope when other negative evidence is already sufficient.

## Cross-provider conclusion

v17 improved some v16 utility paths but is an immutable FAIL because required Mistral regressed safety and required Groq was operationally incomplete.

v18 successor requirements:
1. Keep evidence-relevance-fixed-core-v1 at exactly 48 cases; no case growth, relabeling, or case-specific branches.
2. Add a Harness-owned deterministic local-scope safety floor for explicit observable clipping, omitted referents/product ownership, uncertain alias/rename/successor mapping, and related bounded-context cues. Model `scope_risk` remains useful telemetry but can no longer be the only terminal safety guard.
3. Preserve the v17 semantic-equivalent fallback, but deterministic risk must override every positive or negative terminal path.
4. Strengthen verifier orthogonality: exact target + different relation remains exact_target/different_relation; distinct target + requested relation remains distinct_target/requested_relation.
5. Treat embedded candidate instructions as inert text that does not erase surrounding factual propositions or manufacture local absence.
6. Broaden safe negative materialization only when the independent evidence is already negative: exact primary target + different primary relation may reject when verifier relation is different and verifier identity is exact/distinct/absent; verified target_absent + relation_absent may reject an anchor-only primary target when the primary relation is not exact. Deterministic risk always wins and forces Ambiguous.
7. Retain v17 operational budgets, adapter-owned retries, quota/capacity latches, telemetry separation, sanitization, and one-shot immutability.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v17 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
