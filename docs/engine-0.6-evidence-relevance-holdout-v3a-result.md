# Engine 0.6 evidence relevance independent holdout v3a result

Status: immutable canonical FAIL.

- Freeze tag: engine-0.6-evidence-relevance-holdout-v3a-freeze
- Freeze commit: 7d8d0cb5bb665f7fd26e147f3140076e63c38409
- Canonical run: 36586770785, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Groq + Google
- Effective qualification: v5
- Materialization: v18
- Operational: all three arms 26/26, provider failures 0
- Final gate: FAIL

The earlier v3 freeze run 36585464494 failed in preflight before any live observation because of a stale suite-id assertion. v3a repaired only that operational assertion/tag identity. The manifest, labels, v5 qualification and v18 materialization semantics did not change.

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, or reinterpret this holdout identity as PASS.

## Provider results

Mistral ministral-8b-latest:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 45,907
- latency p50/p95/max 1,253 / 5,001 / 7,187 ms
- terminal miss: v3h02_positive_authorized_alias_availability, Relevant -> Ambiguous

Groq openai/gpt-oss-120b:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 65,015
- latency p50/p95/max 10,697 / 11,272 / 14,151 ms
- retry wait 0
- terminal miss: v3h25_ambiguous_single_signal_near_sibling, Ambiguous -> Irrelevant

Google gemini-3.5-flash-lite:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 47,277
- latency p50/p95/max 7,288 / 37,747 / 59,582 ms
- retry wait 0
- terminal miss: v3h18_negative_comparison_different_relation, Irrelevant -> Ambiguous

## Root cause

The remaining gap is Harness/model authority composition, not provider operation.

1. A repeatedly owned Harness-authorized canonical/alias identity can justify positive identity even when model identity votes disagree, if requested relation support is independently confirmed.
2. A single-signal near sibling is insufficient negative identity authority even if both model stages say different/distinct.
3. A comparison-only target mention plus a stable repeated sibling subject can establish negative ownership; an explicitly excluded requested relation can establish a different relation separately.
4. URL-only target identity with locally unnamed ownership must remain a context gap.

These rules must not create positive relevance from comparison-only text, reject on mere anchor absence, or collapse mapping/ownership/truncation uncertainty.

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- utility: FAIL
- materialization: FAIL
- qualification: FAIL

Issue #462 remains open. A separately versioned successor may replay these immutable observations for regression only. A fresh independent holdout may be authored only after successor semantics are frozen.
