# Engine 0.6 evidence relevance calibration v23 result

Status: immutable canonical PASS。

- Freeze tag: `engine-0.6-evidence-relevance-calibration-v23-freeze`
- Candidate commit: `0871169303d20c464b0454dbe0a150d2fef0ec43`
- Canonical run: `36400085595`、attempt 1のみ
- Cases: 48
- Seed: `4626210`
- Required provider: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v23`
- Materialization: v16

## Required Mistral

`ministral-8b-latest` は48/48完走。

- successful provider cases: 48/48
- provider attempts: 96
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- total tokens: 84,234
- latency p50/p95/max: 1,268 / 2,014 / 2,581 ms

## Required Groq

`openai/gpt-oss-120b` はquota latch / retry waitなしで48/48完走。

- successful provider cases: 48/48
- provider attempts: 96
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- total tokens: 120,075
- latency p50/p95/max: 10,740 / 11,956 / 12,190 ms

v23のpaced Groq admission planはrequired full armでoperationally成功した。

## Required Google

`gemini-3.5-flash-lite` は48/48完走し、v22ではnon-gating 47/48だったauthority gateも48/48でPASSした。

- successful provider cases: 48/48
- provider attempts: 97
- authority-qualified effective qualification: 48/48
- materialized disposition: 48/48
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- latency p50/p95/max: 6,830 / 33,238 / 52,579 ms
- bounded retry 1回。armはoperationally completeを維持

authority gateを緩めず、以前露出したrelation-composition境界でeffective qualification v3を検証できた。

## Final gate

事前固定したrequired gateを全てPASS。

- required operational completeness: PASS
- required correctness: PASS
- required utility: PASS
- required materialization: PASS
- required qualification: PASS

v23はimmutable。rerun / rescore / relabel / freeze tagの移動・再作成は禁止。

## Next step

Issue #462の残りacceptanceは、wrong-target relevance admission 0を要求する別freezeのindependent holdout。holdoutはこのv23 PASS後にのみauthorし、frozen v23 semanticsを使い、holdout observationへ合わせたtuningは禁止する。
