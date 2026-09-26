# Engine 0.6 evidence-target relevance calibration v18 — immutable result

Status: FAIL. This first/only canonical v18 result is immutable. Do not rerun, rescore, relabel, or retag v18.

Freeze:
- commit: 8abb0f9b8d9b7e8a6859e6c791e7e600cdd54e9c
- tag: engine-0.6-evidence-relevance-calibration-v18-freeze
- canonical run: 36237860382
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v18

The fixed calibration core remains closed. No case is added, removed, or relabeled because of this result.

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider attempts: 96 started / 96 completed
- model calls: 96
- total tokens: 84,233
- active execution: 63,592 ms
- provider/retry wait: 0 / 0 ms
- provider-arm latch: none
- operational abort: none

Semantic/materialization:
- proposal exact: 37/48 (77.08%)
- local qualification exact: 26/48 (54.17%)
- materialized exact: 47/48 (97.92%)
- wrong-target / false Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 1
- deterministic guard overrides: 16

The v18 deterministic local-risk floor fixed the v17 safety regression. In particular, `59_fresh_shared_owner_positive_looking` materialized Ambiguous despite provider output that otherwise looked positive. No unsafe Relevant outcome remained.

The only disposition miss was `14_sibling_product_overlap`: expected Irrelevant -> Ambiguous. The observed primary proposal was `target_binding=unresolved, relation_binding=different`; the independent verifier reported `identity_scope=distinct_target, relation_scope=different_relation, scope_risk=none`. v13 did not permit a terminal target-negative result because the primary target axis was unresolved. This is a conservative utility miss, not a correctness failure.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operational:
- successful provider cases: 13
- failed provider cases: 35
- first 13 successful observations were 13/13 exact for proposal, qualification, and materialization
- typed daily-quota failure occurred during local qualification for `74_v13_exact_target_same_relation_no_cue`
- provider arm latched immediately after the confirmed quota failure
- remaining 34 cases were suppressed
- provider attempts: 28 started / 28 completed
- model calls: 28
- total tokens before latch: 32,572
- active execution: 12,971 ms
- provider wait: 231,724 ms, all pacing wait
- retry wait: 0 ms

The runtime quota circuit behaved as designed: no retry storm and no timeout conversion. The required Groq arm is operationally incomplete and non-scorable, so v18 cannot pass even though all 13 successful observations were exact.

## Google replication

Model: gemini-3.5-flash-lite. This arm is non-gating.

Operational:
- successful provider cases: 47
- failed provider cases: 1
- one local-qualification semantic execution timeout at `56_fresh_exact_exact_shared_row_abstain`
- no provider-arm latch
- provider attempts: 97 started / 96 completed

Semantic/materialization over successful observations:
- proposal exact: 38/47 (80.85%)
- local qualification exact: 28/48 expected cases recorded by the run
- materialized exact: 42/47 (89.36%)
- wrong-target / false Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 5

Disposition misses were 13, 15, 16, 36, and 44, all expected Irrelevant -> Ambiguous. The replication confirms that v18's safety floor is conservative across providers, but negative disagreement remains model-sensitive.

## Cross-provider conclusion

v18 materially improved the required Mistral result from v17 44/48 to 47/48 while restoring zero unsafe Relevant retention. Groq's first 13 successful observations were perfect before quota exhaustion. The remaining required semantic miss is one conservative negative disagreement.

The v19 successor should therefore be minimal:
1. keep primary proposal v5 and local verifier v8 unchanged;
2. keep the deterministic local-risk floor unchanged;
3. add one Harness-owned target-negative terminal rule: when deterministic risk is absent, verifier `scope_risk=none`, verifier `identity_scope=distinct_target`, and primary `target_binding != exact`, materialize Irrelevant even if the primary target axis is unresolved;
4. never apply that rule when primary target is exact, verifier risk is non-none, or deterministic local risk is present;
5. keep all 48 scored semantics unchanged and add only unscored property controls for the new disagreement boundary;
6. retain all v18 operational budgets, retry/latch logic, telemetry, and sanitization.

A fixed-core audit found no expected Ambiguous case with `primary target != exact + verifier distinct_target + risk none`; every expected instance of that terminal shape is Irrelevant. This supports the v19 rule without changing scored labels.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false (Groq quota latch)
- required correctness gate: false at aggregate final-gate level because required operational completeness was not satisfied
- required utility gate: false (Mistral one utility miss and Groq incomplete)
- required materialization gate: false
- required qualification gate: false

v18 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
