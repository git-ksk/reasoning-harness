# Engine 0.6 evidence relevance calibration v23 result

Status: immutable canonical PASS.

- Freeze tag: `engine-0.6-evidence-relevance-calibration-v23-freeze`
- Candidate commit: `0871169303d20c464b0454dbe0a150d2fef0ec43`
- Canonical run: `36400085595`, attempt 1 only
- Cases: 48
- Seed: `4626210`
- Required providers: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v23`
- Materialization: v16

## Required Mistral

`ministral-8b-latest` completed 48/48.

- successful provider cases: 48/48
- provider attempts: 96
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- total tokens: 84,234
- latency p50/p95/max: 1,268 / 2,014 / 2,581 ms

## Required Groq

`openai/gpt-oss-120b` completed 48/48 without quota latch or retry wait.

- successful provider cases: 48/48
- provider attempts: 96
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- total tokens: 120,075
- latency p50/p95/max: 10,740 / 11,956 / 12,190 ms

The v23 paced Groq admission plan therefore succeeded for the full required arm.

## Required Google

`gemini-3.5-flash-lite` completed 48/48 and passed the authority gate that was non-gating and 47/48 in v22.

- successful provider cases: 48/48
- provider attempts: 97
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility misses: 0
- latency p50/p95/max: 6,830 / 33,238 / 52,579 ms
- one bounded retry occurred; the arm remained operationally complete

This validates effective qualification v3 on the previously exposed relation-composition boundary without weakening the authority gate.

## Final gate

All precommitted required gates passed:

- required operational completeness: PASS
- required correctness: PASS
- required utility: PASS
- required materialization: PASS
- required qualification: PASS

v23 is immutable. Do not rerun, rescore, relabel, or move/recreate the freeze tag.

## Next step

Issue #462 requires a separately frozen independent holdout with zero wrong-target relevance admissions. The holdout may be authored only after this v23 PASS and must use the frozen v23 semantics without tuning to holdout observations.
