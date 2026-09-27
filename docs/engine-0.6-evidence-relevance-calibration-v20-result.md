# Engine 0.6 evidence-target relevance calibration v20 — immutable result

Status: FAIL. The first/only frozen v20 canonical is immutable. Do not rerun, rescore, relabel, or retag v20.

Freeze:
- commit: `e9ca7cce6a60c9b69129375aeea62a60cc3b127f`
- tag: `engine-0.6-evidence-relevance-calibration-v20-freeze`
- canonical run: `36283988719`
- run attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1` (48 cases)
- annotation protocol: `evidence-relevance-effective-qualification-v20`

No frozen case, label, or expected disposition is changed because of this result.

## Required Mistral arm

Model: `ministral-8b-latest`.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider attempts: 96/96 completed
- model calls: 96
- total tokens: 84,234
- active execution: 56,910 ms
- provider/retry wait: 0 / 0 ms
- provider-arm latch: none

Semantic/materialization:
- proposal exact: 36/48 (75.00%)
- raw verifier exact: 26/48 (54.17%)
- raw scope-risk misses / spurious: 10 / 1
- raw identity-scope misses: 14
- raw relation-scope misses: 14
- effective qualification: 48/48 exact
- effective risk / identity / relation misses: 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

Mistral is a complete semantic PASS for the frozen v20 surface.

## Required Groq arm

Model: `openai/gpt-oss-120b`.

Operational:
- successful provider cases: 39
- failed provider cases: 9
- typed daily quota occurred at `31_possible_successor_unconfirmed`
- provider arm latched immediately after the confirmed quota failure
- remaining 8 cases were suppressed
- provider attempts: 79/79 completed
- model calls: 79
- total tokens before latch: 97,259
- active execution: 53,336 ms
- provider/pacing wait: 661,643 ms
- retry wait: 0 ms

The quota circuit again behaved as intended: fail-fast, immediate arm latch, no retry storm, and no assessment-timeout conversion.

However v20 also exposed a semantic safety miss before quota exhaustion:
- case: `13_same_service_different_feature`
- expected proposal: `target=exact, relation=different`
- observed primary proposal: `target=exact, relation=exact`
- raw verifier: `identity=exact_target, relation=different_relation, risk=none`
- effective qualification: `identity=exact_target, relation=requested_relation, risk=none`
- expected disposition: Irrelevant
- materialized disposition: Relevant

The Harness-owned effective qualifier promoted the primary proposal's `relation=exact` over the independent verifier's `different_relation` because no deterministic requested-relation cue was present. That authority conflict created one wrong-target/false Relevant outcome.

Groq successful-observation semantics:
- effective qualification derivations: 39
- effective exact: 37
- effective relation-scope misses: 2
- materialized exact: 38/39
- wrong-target Relevant retention: 1
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

The second effective relation-scope miss was `25_insufficient_local_passage`; deterministic `context_gap` kept the final disposition Ambiguous, so it was not a terminal safety failure.

## Google replication

Model: `gemini-3.5-flash-lite`. Non-gating.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider attempts: 96/96 completed
- model calls: 96
- total tokens: 86,842
- active execution: 141,930 ms
- provider/pacing wait: 228,023 ms
- retry wait: 0 ms

Semantic/materialization:
- proposal exact: 36/48 (75.00%)
- raw verifier exact: 28/48 (58.33%)
- effective qualification exact: 45/48
- effective relation-scope misses: 3
- effective risk / identity misses: 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejections: 0
- Relevant -> Ambiguous: 0
- utility misses: 0

The three effective relation-scope mismatches remained the same context-gap shapes (`25`, `26`, `80`). All retained Ambiguous terminal disposition. v20 successfully fixed Google case 13 from the v19 Ambiguous miss to Irrelevant.

## Cross-provider conclusion

v20 validates materialization v15 on Mistral and Google, but fails required acceptance for two independent reasons:
1. Groq operational incompleteness from typed daily quota;
2. a Groq semantic safety miss on case 13 caused by the effective qualifier trusting an exact primary relation over an independently different relation.

The semantic miss is more important than the quota event for successor design. v21 must fix the authority-conflict rule generically before any new canonical run. Merely waiting for quota reset would be insufficient.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false at aggregate final-gate level because the required set is incomplete
- required materialization gate: false
- required qualification gate: false

v20 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
