# Engine 0.6 evidence-target relevance calibration v16 — immutable result

Status: FAIL. This first/only canonical v16 result is immutable. Do not rerun, rescore, relabel, or retag v16.

Freeze:
- commit: 98cc85d495d7e9dba49389f5cf15505088609fa4
- tag: engine-0.6-evidence-relevance-calibration-v16-freeze
- canonical run: 36221674316
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v16

The fixed calibration core remains closed. Do not append cases because of this result.

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 48/48
- successful provider cases: 48
- failed provider cases: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 71,385
- operational abort: none
- provider-arm latch: none

Semantic / materialization:
- proposal exact: 36/48 (75.00%)
- local qualification exact: 25/48 (52.08%)
- scope-risk misses: 8
- spurious scope risks: 4
- identity-scope misses: 13
- relation-scope misses: 9
- materialized exact: 42/48 (87.50%)
- wrong-target / false Relevant retention: 0
- false relevance rejections: 0
- expected Relevant left Ambiguous: 2
- utility misses: 6

The six disposition misses are:
- Relevant -> Ambiguous: 04_semantic_paraphrase, 55_fresh_positive_injection_ignored
- Irrelevant -> Ambiguous: 15_navigation_only_match, 16_broad_landing_no_support, 29_explicit_no_target_information, 44_prompt_injection_local_absence

v16 removed the v15 correctness failure: no wrong-target Relevant survived. The remaining failures are conservative utility misses.

Two generic residuals remain:
1. `allow_semantic_equivalent` has no bounded positive fallback when the primary target axis is unresolved even though the independent verifier establishes exact target/relation with no risk (case 04 shape).
2. The local verifier still overuses `context_gap` and over-credits navigation/footer identity in bounded negative units. Generic pages, explicit local absence, clearly different substantive ownership, and ignored prompt-injection text are sometimes treated as missing context rather than usable negative scope evidence.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operational:
- successful provider cases: 4
- failed provider cases: 44
- first typed daily-quota failure occurred at 07_url_omits_target_terms
- provider arm latched immediately after the quota failure
- final 43 cases were suppressed
- successful observations before the latch were semantically exact: 4/4 proposal, verifier, and materialization
- wrong-target Relevant retention: 0
- utility misses before latch: 0

The runtime quota policy behaved as designed: typed daily quota stopped the arm without retry storm or timeout conversion. This arm is operationally incomplete and non-scorable. The quota event occurred in the same daily window as the earlier v15/v16 work, so a successor canonical should not knowingly be consumed again in that already-exhausted quota window.

## Google replication

Model: gemini-3.5-flash-lite. This arm is non-gating.

- successful provider cases: 0
- two early typed rate-limit failures triggered the correlated-capacity latch
- remaining 46 cases were suppressed
- replication is non-scorable
- the circuit behaved as designed

No semantic conclusion is drawn from this replication arm.

## Cross-provider conclusion

v16 materially improved safety and utility versus v15:
- Mistral materialized exact: 33/48 -> 42/48
- wrong-target Relevant retention: 1 -> 0
- utility misses: 14 -> 6

The next successor should preserve v16's agreement-based terminal policy and make only bounded generic changes:
1. allow a positive fallback only under the explicit `allow_semantic_equivalent` identity policy, only when primary relation is exact, verifier identity/relation are exact, and scope risk is none; strict-identity unresolved primary must remain Ambiguous.
2. tighten verifier semantics so candidate instructions are ignored rather than treated as context loss; navigation/footer target mentions never establish exact target ownership; complete generic/broad units and explicit local absence become `target_absent` / `relation_absent` rather than `context_gap`; clearly different substantive ownership remains `distinct_target` without a context gap merely because the Harness target appears only in navigation.
3. keep the fixed 48 cases unchanged and add only unscored structural/property controls for the new policy boundary.
4. retain v16 operational budgets, retry ownership, telemetry, sanitization, quota latch, and correlated-capacity latch.

v16 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
