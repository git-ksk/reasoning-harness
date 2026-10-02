# Engine 0.6 evidence relevance independent holdout v2 result

Status: immutable canonical **FAIL**.

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- Freeze commit: `c38b5f7dbfcf8c01dcaa6f6cd7341e43d3b87325`
- Canonical run: `36533340582`, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v4`
- Materialization: v17
- Operational status: all three required arms completed 26/26; provider jobs succeeded
- Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, change expected labels, recreate or move the freeze tag, or reinterpret this holdout identity as PASS.

## Required Mistral

`ministral-8b-latest` completed 26/26.

- authority-qualified effective qualification: 25/26
- materialized exact: 25/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 1
- total tokens: 45,977
- latency p50/p95/max: 1,338 / 2,472 / 2,677 ms

The only terminal miss was `v2h13_negative_unrelated_launch`: expected Irrelevant -> observed Ambiguous. The proposal and raw local qualification both over-bound the candidate to the requested target, but the Harness-owned strict identity floor prevented a wrong-target Relevant result because no canonical/alias target anchor was present.

## Required Groq

`openai/gpt-oss-120b` completed 26/26 and satisfied the full frozen semantic and operational contract.

- authority-qualified effective qualification: 26/26
- materialized exact: 26/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- total tokens: 65,555
- latency p50/p95/max: 10,708 / 11,026 / 11,607 ms
- provider failures: 0
- retry wait: 0
- provider arm latch: none

## Required Google

`gemini-3.5-flash-lite` completed 26/26.

- authority-qualified effective qualification: 25/26
- materialized exact: 25/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 1
- total tokens: 47,427
- latency p50/p95/max: 6,877 / 7,655 / 7,818 ms

The only terminal miss was the same `v2h13_negative_unrelated_launch`: expected Irrelevant -> observed Ambiguous. Google proposed target=unresolved while the raw local verifier over-bound identity to exact_target. Effective qualification therefore could not establish Harness-owned negative identity authority and the materializer abstained.

## Shared failed case

Frozen case `v2h13_negative_unrelated_launch`:

- target/relation: `Obsidian Monitor` / change_or_launch
- candidate source title: `Obsidian Trace release`
- candidate excerpt: `Obsidian Trace launches a new span waterfall view for distributed tracing.`
- expected proposal: different / exact
- expected local qualification: distinct_target / requested_relation / none
- expected disposition: Irrelevant

The candidate contains no canonical or alias anchor for `Obsidian Monitor`; it consistently names a sibling-looking subject, `Obsidian Trace`, while carrying the requested launch relation. Groq bound this correctly enough for final 26/26 acceptance. Mistral and Google did not.

The v4/v17 safety boundary behaved correctly: lack of the required Harness target anchor prevented wrong-target material from becoming Relevant. The remaining defect is utility/negative-identity authority, not positive relevance safety.

## Root cause

v4 repaired the holdout-v1 comparison/context-only defect but did not add a generic Harness-owned way to distinguish these two cases when the strict target anchor is absent:

1. genuinely unresolved identity, where the candidate may still refer to the requested target through an unconfirmed alias/rename/successor or omitted referent; and
2. locally substantive evidence whose visible subject is a different sibling/entity and whose requested relation belongs to that visible subject.

The current deterministic negative-identity signals cover explicit separation (`separate` / `distinct` product or service), explicit non-mapping, and comparison markers. `v2h13` has none of those explicit cues, so Mistral/Google model over-binding cannot be corrected into `distinct_target` by Harness authority.

A successor must therefore improve **negative identity corroboration without turning mere target-anchor absence into Irrelevant**. URL-only identity, identity-mapping uncertainty, truncated/omitted ownership, and pronoun-only evidence must remain Ambiguous.

## Final gate

Precommitted required gates:

- required operational completeness: PASS
- required correctness hard gate: PASS
- required utility: **FAIL**
- required materialization: **FAIL**
- required qualification: **FAIL**

The zero wrong-target Relevant hard gate passed for all three providers. Issue #462 remains open because the independent holdout did not satisfy every required semantic gate.

## Successor direction

Holdout v2 is not a tuning surface and will not be rerun or relabeled. A separately versioned successor may replay its immutable observations for postmortem/regression only.

The successor must remain provider/fixture/entity neutral and preserve:

- v23 immutable calibration regression 0;
- holdout-v1 wrong-target safety repair;
- holdout-v2 zero wrong-target Relevant admissions;
- Ambiguous for URL-only identity, identity-mapping uncertainty, ownership/context gaps, and missing/truncated referents;
- no branch on `v2h13`, Obsidian names, provider names, or fixture text.

Only after successor semantics are frozen may another fresh independent holdout be authored.
