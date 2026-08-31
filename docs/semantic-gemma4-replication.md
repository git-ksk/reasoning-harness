# Gemma 4 semantic replication

`gemma4-31b-replication-v1` is a cross-family replication arm for the existing semantic-judge research. It does not replace, relabel, or tune any calibration or holdout fixture, and it does not retroactively add Gemma 4 to the predeclared D3 adoption providers.

## Frozen model

- provider: `google`
- model: `gemma-4-31b-it`
- rationale: the repository's earlier five-trial live benchmark completed 5/5 trials for the 31B model, while the 26B model had provider-side copyright/recitation blocks and remains experimental.

## Frozen stages

| stage | corpus | seed range | trials | max tokens | provider calls |
| --- | --- | --- | ---: | ---: | ---: |
| R2 materialization | `fixtures/semantic-judges` | 2000-2004 | 5 | 512 | 180 |
| D2 decidability | `fixtures/semantic-decidability-d2` | 6000-6004 | 5 | 512 | 75 |
| v5 pilot | `fixtures/semantic-decidability-holdout-v5` | 7000-7004 | 5 | 512 | 120 |

Total frozen provider calls: 375.

R2 is diagnostic replication of the existing calibration-only materialization study. D2 reuses the frozen deterministic decidability contract. v5 reuses the immutable holdout-v5 payloads and their SHA-256 manifest. No stage may modify labels, thresholds, seeds, model identity, corpus membership, or semantic contracts after provider observation.

## Interpretation

The replication asks whether the same qualitative pattern observed with Ministral 8B transfers to a different model family:

1. measure the model-backed semantic judge on the existing R2 calibration surface;
2. measure how many unsafe assertive decisions the frozen decidability gate removes on D2;
3. run the unchanged v5 independent pilot and compare clear-case coverage/precision/recall, typed-insufficiency abstention, composed unsafe assertions, seed stability, and ambiguous abstention diagnostics.

A Gemma 4 v5 pass is supporting evidence for cross-family generalization. It is not, by itself, a retroactive D3 adoption result because the original D3 provider set was frozen before this replication arm existed.

Operational failures such as rate limits, daily quota exhaustion, provider unavailability, and transport timeouts remain separate from semantic failure. Study CLIs log the bounded `failure_class` as each case completes so an operational limit can be diagnosed even if a workflow-level timeout prevents final JSON assembly.

## Observed result

GitHub Actions run `33384957101` executed the frozen replication from merged main commit
`14871a8375881f07a3813a4d584209859c30ac93` without changing the model, seeds, corpora, token
budget, or semantic contracts.

### R2 materialization

The R2 stage attempted 90 observations per arm over the 18-case calibration corpus and five seeds.
The harness-materialized arm completed 90/90 calls and all five trials. The v3 full-JSON arm
completed 87/90 calls: three cases failed with `representation_protocol`, leaving two operationally
incomplete v3 trials. On every operationally complete trial in both arms, labelled precision and
recall were 1.000. The materialized arm's ambiguous-abstention rate was 0.429, 0.429, 0.571,
0.429, and 0.429 across the five trials. Among 87 matched successful pairs, two decisions changed,
for a decision-flip rate of 0.023.

This is protocol evidence in favor of harness-owned materialization for Gemma 4 31B, not evidence
that ambiguity is solved: the materialized arm remained assertive on several intentionally ambiguous
fixtures.

### D2 decidability

The D2 stage completed 75/75 provider calls and 5/5 trials with no operational failure. Aggregate
eligible clear coverage, precision, and recall were all 1.000. All 35 typed-insufficiency variants
were assertive before composition and were forced to abstain after composition: base unsafe
assertions 35 -> composed unsafe assertions 0, typed-insufficiency abstention 35/35 = 1.000, and
clear-case seed disagreement 0. Eligible ambiguous abstention was 15/20 = 0.750.

These D2 aggregate metrics match the earlier Ministral 8B D2 observation, including the 0.750
ambiguous-abstention diagnostic.

### Holdout-v5 pilot

The v5 stage completed 120/120 provider calls and 5/5 trials with no operational failure. Aggregate
eligible clear coverage, precision, and recall were all 1.000. All 50 typed-insufficiency variants
were assertive before composition and safely abstained after composition: base unsafe assertions
50 -> composed unsafe assertions 0, typed-insufficiency abstention 50/50 = 1.000, and clear-case seed
disagreement 0. Eligible ambiguous abstention was 20/40 = 0.500.

The Gemma 4 31B v5 aggregate is identical to the earlier Ministral 8B v5 aggregate. A direct
case/seed comparison also found zero base-decision differences across all 120 matched semantic
observations. The two model families abstained on the same four ambiguous fixtures
(`v5h05`, `v5h06`, `v5h23`, `v5h24`) and returned `finding` on the same four
(`v5h11`, `v5h12`, `v5h17`, `v5h18`) for every seed.

This is strong cross-family replication evidence that the frozen typed decidability composition is
not merely compensating for one Ministral-specific decision pattern. It is still not a claim of
universal model portability, and it does not retroactively modify the original D3 adoption provider
set. The shared assertiveness on four ambiguous v5 fixtures remains a visible model-facing semantic
boundary rather than a reason to tune the frozen corpus after observation.
