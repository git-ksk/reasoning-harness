# Engine 0.6 evidence relevance independent holdout v1 result

Status: immutable canonical **FAIL**.

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v1-freeze`
- Freeze commit: `5fda8675e642ee98de57e2c4d69e34a5622fa485`
- Canonical run: `36495389012`, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v23`
- Materialization: v16
- Operational status: all three required arms completed 26/26; provider jobs succeeded
- Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, change expected labels, recreate or move the freeze tag, or reinterpret this holdout identity as PASS.

## Required Mistral

`ministral-8b-latest` completed 26/26.

- authority-qualified effective qualification: 24/26
- materialized exact: 24/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 2
- total tokens: 45,544
- latency p50/p95/max: 1,265 / 2,077 / 4,142 ms

The two terminal misses were conservative:

- `h10_negative_sibling_same_relation`: expected Irrelevant -> observed Ambiguous
- `h13_negative_unrelated_announcement`: expected Irrelevant -> observed Ambiguous

Neither admitted wrong-target material as Relevant.

## Required Groq

`openai/gpt-oss-120b` completed 26/26.

- authority-qualified effective qualification: 26/26
- materialized exact: 26/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- total tokens: 65,244
- latency p50/p95/max: 10,720 / 11,348 / 11,586 ms
- no provider-arm latch or retry wait

This arm satisfied the full frozen semantic and operational contract.

## Required Google

`gemini-3.5-flash-lite` completed 26/26.

- authority-qualified effective qualification: 25/26
- materialized exact: 24/26
- wrong-target relevance retention: **1**
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 1
- total tokens: 47,067
- latency p50/p95/max: 6,760 / 8,028 / 11,862 ms

The correctness hard-gate failure is `h14_negative_comparison_only_mention`.

Frozen case shape:

- requested target/relation: `Umber DB` / pricing
- candidate title: `Violet DB pricing`
- candidate excerpt: `Unlike Umber DB, Violet DB charges by provisioned shard-hour. This page lists only Violet DB prices.`
- expected proposal: target=different / relation=exact
- expected local qualification: distinct_target / requested_relation / none
- expected disposition: Irrelevant

Observed Google result:

- proposal: exact / exact
- raw local qualification: distinct_target / requested_relation / none
- effective local qualification: exact_target / requested_relation / none
- final disposition: **Relevant**
- reasons included `harness_canonical_name_anchor`, `positive_target_local_binding_confirmed`, and `model_relevant`

The raw verifier correctly identified a distinct target. The failure was created by Harness-owned composition: a literal canonical-name occurrence in comparison/context was treated as sufficient positive target identity and overrode the raw `distinct_target` result.

`h13_negative_unrelated_announcement` remained conservative Ambiguous rather than becoming wrong-target Relevant.

## Root cause

In effective qualification v1, `distinct_target` received priority over a Harness name anchor only when both the deterministic distinctness signal and the advisory proposal agreed on target=different. When the proposal said target=exact, the corroborated-distinct branch was skipped and the next `has_harness_anchor` branch promoted identity to `exact_target`.

Effective qualification v2/v3 repaired relation-axis conflicts but did not repair this identity-axis precedence defect. Materialization v16 then saw exact target + requested relation and followed the positive path.

The generic safety gap is therefore:

> a canonical target name occurring in a candidate is identity metadata, not proof that the candidate's substantive proposition is owned by that target.

Requested-relation language is likewise not target ownership evidence.

## Final gate

Precommitted required gates:

- required operational completeness: PASS
- required correctness: **FAIL**
- required utility: **FAIL**
- required materialization: **FAIL**
- required qualification: **FAIL**

Issue #462 remains open because acceptance requires an independently frozen holdout with zero wrong-target relevance admissions.

## Successor postmortem replay

The immutable v1 observation is not rescored or rewritten. A separately versioned successor applies effective qualification v4 + materialization v17 to recorded observations only for regression/postmortem analysis.

Replay fixtures are derived from the immutable canonical artifacts:

- `fixtures/evidence-relevance-holdout-successor-v2/v23-observation-replay.json`
- `fixtures/evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json`

Current successor replay:

- calibration v23: all 144 provider observations (48 x 3) preserve expected authority/materialization; regression 0
- holdout v1 Groq: 26/26 remains exact
- holdout v1 Mistral: 24/26 remains exact; wrong-target Relevant remains 0
- holdout v1 Google: improves to 25/26 exact; the frozen h14 observation changes only under successor semantics from Relevant to Irrelevant; wrong-target Relevant becomes 0 in replay
- Relevant utility misses remain 0 across all three successor replays

This replay is evidence about successor semantics only. It does **not** convert holdout v1 into PASS.

See [successor v2 design](engine-0.6-evidence-relevance-holdout-successor-v2.md).
