# Engine 0.6 evidence-target relevance calibration v19 — immutable result

Status: FAIL. This first/only canonical v19 result is immutable. Do not rerun, rescore, relabel, or retag v19.

Freeze:
- commit: `471bc11a83ba36dbaf5cb69a5fa4550b0e35b578`
- tag: `engine-0.6-evidence-relevance-calibration-v19-freeze`
- canonical run: `36247789205`
- attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1` (48 cases)
- annotation protocol: `evidence-relevance-effective-qualification-v19`

No case is added, removed, or relabeled because of this result.

## Required Mistral

`ministral-8b-latest` completed 48/48 with 48 provider successes, 0 failures, 96/96 provider attempts, 96 model calls, and 84,234 total tokens.

Semantic results:
- proposal exact: 36/48 (75.00%)
- raw verifier exact: 26/48 (54.17%)
- raw scope-risk misses / spurious: 9 / 1
- raw identity-scope misses: 13
- raw relation-scope misses: 14
- effective qualification: 48/48 exact
- effective risk / identity / relation misses: 0
- materialized exact: 48/48
- wrong-target Relevant: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

This validates the v19 authority split: raw verifier disagreement remains observable, while Harness-owned effective qualification and final materialization both satisfy the frozen core exactly.

## Required Groq

`openai/gpt-oss-120b` produced 10 successful cases, then hit typed daily quota during local qualification for `63_fresh_stale_contradiction_relevant`. The provider arm latched immediately and suppressed the remaining 37 cases.

Operational evidence:
- successful provider cases: 10
- failed provider cases: 38
- first 10 successful observations: 10/10 effective qualification and 10/10 materialization exact
- provider attempts: 22/22 completed
- model calls: 22
- total tokens before latch: 25,505
- active execution: 12,460 ms
- pacing wait: 179,197 ms
- retry wait: 0 ms

The quota circuit behaved as designed: one confirmed daily-quota failure triggered the arm latch, with no retry storm and no semantic-failure conversion. The required Groq arm is operationally incomplete and non-scorable, so v19 cannot PASS.

## Google replication

`gemini-3.5-flash-lite` is non-gating. It completed 46 successful cases with two timeouts: `05_title_identity_body_relation` and `27_explicit_not_rename_distinct`.

Results:
- provider attempts: 107 started / 105 completed
- model calls: 95
- total tokens: 84,033
- proposal exact: 34/46
- effective qualification: 43/46 exact
- effective relation-scope misses: 3
- effective risk / identity misses: 0
- materialized exact: 45/46
- wrong-target Relevant: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility misses: 1

The single materialization miss was `13_same_service_different_feature`: expected Irrelevant, observed Ambiguous. Effective qualification established `exact_target + different_relation + no risk`, while the primary proposal returned `target=exact, relation=unresolved`. This is the exact-target / independently-established-different-relation shape that was explicitly left as a v19 candidate before live observation.

The three effective relation-scope mismatches were `25_insufficient_local_passage`, `26_url_only_identity`, and `80_v13_clipped_relation_context`. All retained non-none `context_gap` risk and still materialized the frozen Ambiguous disposition. They are provider-sensitive normalization disagreements, not unsafe terminal outcomes.

## Cross-provider conclusion

v19's main semantic design is supported: Mistral reached 48/48 effective qualification and materialization, Groq's first 10 successful observations were exact, and Google retained zero unsafe Relevant outcomes. The canonical failure is primarily operational because a required provider hit daily quota.

## v20 direction

v20 must not be a disguised v19 rerun and must not weaken gates to obtain PASS. The successor should:
1. keep the fixed 48 cases, labels, primary proposal v5, typed local-risk floor, raw/effective telemetry split, and positive-admission rules unchanged;
2. keep Mistral + Groq required and Google full non-gating replication;
3. retain typed quota fail-fast and immediate provider-arm latch; no v19 rerun and no manual quota probe as acceptance evidence;
4. consider exactly one predeclared semantic delta: exact target + independently established different relation + no safety risk may terminally materialize Irrelevant even when the primary relation is unresolved;
5. defer context-gap relation normalization until a generic property/replay rule can reproduce every frozen expectation without case-specific wording or weakened fail-closed behavior;
6. require immutable v19 Mistral/Groq-success/Google-success replay with zero correctness regressions before any v20 freeze;
7. keep independent holdout authoring prohibited until a first/only frozen v20 canonical PASS.

## Final decision

Required top-level acceptance is FAIL because required operational completeness is false. Aggregate correctness, utility, materialization, and qualification gates are therefore false at final-gate level. v19 is immutable and must not be rerun.
