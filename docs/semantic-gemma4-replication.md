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
