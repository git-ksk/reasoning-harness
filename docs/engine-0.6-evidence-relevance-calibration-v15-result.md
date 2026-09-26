# Engine 0.6 evidence-target relevance calibration v15 — immutable result

Status: FAIL. This first/only canonical v15 result is immutable. Do not rerun, rescore, relabel, or retag v15.

Freeze:
- commit: 80aba233aa375fa88d4515d68c586f9f0c5f02bb
- tag: engine-0.6-evidence-relevance-calibration-v15-freeze
- canonical run: 36217988625
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-binding-verifier-v15

The fixed calibration core remains closed. Do not append cases because of this result.

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 48/48
- successful provider cases 48
- provider failures 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 67,667
- active execution: 53,152 ms
- provider wait / pacing wait / retry wait: 0 / 0 / 0 ms
- latency p50/p95/max: 1,037 / 1,589 / 1,926 ms

Semantic / materialization:
- proposal exact: 33/48 (68.75%)
- local qualification exact: 23/48 (47.92%)
- blocking-reason misses: 1
- spurious blocking reasons: 13
- binding-confirmation misses: 14
- spurious binding confirmations: 1
- materialized exact: 33/48 (68.75%)
- wrong-target / false Relevant retention: 1
- false relevance rejections: 0
- expected Relevant left Ambiguous: 3
- utility misses: 14

Disposition failures are structurally concentrated:
- expected Relevant -> Ambiguous: 04_semantic_paraphrase, 55_fresh_positive_injection_ignored, 63_fresh_stale_contradiction_relevant
- expected Irrelevant -> Ambiguous: 13_same_service_different_feature, 14_sibling_product_overlap, 16_broad_landing_no_support, 17_unrelated_announcement, 18_comparison_only_mention, 19_relation_mismatch_same_target, 29_explicit_no_target_information, 50_fresh_sibling_local_scope, 60_fresh_comparison_sibling, 76_v13_sibling_different_relation_no_cue, 81_v13_exact_target_other_relation
- correctness failure: 59_fresh_shared_owner_positive_looking materialized Relevant instead of Ambiguous

The critical correctness failure is case 59. The primary proposal was unresolved/exact, while the independent verifier emitted blocking_reason=none plus confirmed_target_relation. Materialization v10 treated that verifier output plus the Harness identity floor as sufficient positive rescue. The candidate was locally ownership-ambiguous, so the rescue admitted a wrong target/relation ownership binding. This demonstrates that model-authored positive confirmation cannot independently rescue an unresolved primary under the current contract.

The dominant utility failure is verifier over-abstention. Mistral emitted context_gap for ten cases whose expected blocker was none, and also collapsed several identity_mapping/multiple blockers into context_gap. Negative primary bindings were therefore frequently prevented from reaching Irrelevant even when the primary target/relation axes were correct.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operational:
- planned/completed observations: 48/48
- successful provider cases: 43
- failed provider cases: 5
- one typed daily-quota failure at 45_url_only_identity_fresh
- provider arm latched immediately after that quota failure
- four remaining cases suppressed as provider_arm_latched
- provider attempts complete: 88 started / 88 completed
- model calls / provider attempts: 88 / 88
- accounted total tokens before latch: 90,148
- active execution: 49,606 ms
- provider wait: 725,682 ms, all pacing wait
- retry wait: 0 ms
- successful latency p50/p95/max: 17,740 / 18,364 / 18,532 ms

The runtime quota policy behaved as intended. A confirmed daily quota failure was not retried as a transient short-window rate limit; the arm latched immediately and suppressed guaranteed-failure calls. There was no retry storm and no assessment-timeout conversion. This operational behavior is retained for v16.

Semantic evidence before the quota latch was already insufficient for acceptance:
- proposal exact accuracy: 83.72% over successful observations
- local qualification exact: 26/48 expected cases recorded by the run
- spurious blocking reasons: 5
- binding-confirmation misses: 13
- materialized exact: 36 successful observations
- wrong-target / false Relevant retention: 0
- false relevance rejections: 0
- expected Relevant left Ambiguous: 1
- utility misses: 7

Successful-observation disposition misses were 04, 13, 14, 15, 16, 50, and 81. Therefore v15 would still require a semantic successor even if Groq had not exhausted daily capacity.

## Google replication

Model: gemini-3.5-flash-lite. This arm is non-gating.

Operational:
- successful provider cases: 0
- case 01 reached local qualification and failed with typed rate_limit
- case 02 failed with a second typed rate_limit after 30,000 ms retry wait
- the two-consecutive capacity-failure circuit then latched the arm
- remaining 46 cases were suppressed
- provider attempts: 4 started / 4 completed
- provider attempts incomplete observations: 0
- active execution: 1,335 ms
- provider wait: 35,287 ms
- pacing wait: 5,287 ms
- retry wait: 30,000 ms

The replication is non-scorable. No semantic conclusion is drawn from this arm. The operational circuit behaved as designed by stopping correlated capacity failures early.

## Cross-provider conclusion

v15 is not a quota-only failure. The required Mistral arm was fully operational and independently failed correctness, utility, materialization, and qualification-safety gates. Groq also showed semantic misses before its quota latch.

The evidence supports a v16 successor with these constraints:
1. Keep evidence-relevance-fixed-core-v1 at exactly 48 cases. No growth, relabeling of v15, or case-specific branches.
2. Remove positive rescue authority from an unresolved primary. A verifier positive alone must never produce Relevant.
3. Replace the free-form binding_confirmation surface with orthogonal local identity scope, relation scope, and concrete scope-risk fields so impossible or conflated confirmations are derived away rather than model-authored.
4. Strengthen primary binding instructions for explicit Harness canonical/alias ownership, cross-language identity, stale-but-relevant material, distributed same-target sections, semantic-equivalent policy, and untrusted candidate instructions without weakening shared-ownership / clipped-context abstention.
5. Derive final Relevant/Irrelevant/Ambiguous from agreement between the primary axes and verifier axes. Any concrete identity/ownership/context risk remains fail-closed.
6. Keep the v15 operational budgets, retry ownership, telemetry, sanitization, quota latch, correlated-capacity latch, and no-manual-TPD-start-gate policy unchanged unless independent operational evidence requires otherwise.

## Final decision

Required top-level acceptance is FAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v15 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.
