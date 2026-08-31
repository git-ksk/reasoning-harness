# Nemotron D3 probe

This is a bounded cross-family probe for Issue #73. It does not modify the frozen D3 adoption plan and is not an additional adoption arm.

The provider/model is fixed before observation to NVIDIA Hosted NIM `nvidia/nemotron-3.5-lightning-30b-a3b`. This model is intentionally informative because earlier semantic-judge studies were protocol-incomplete and strongly finding-biased.

The probe reuses the unchanged D2 and holdout-v5 corpora and semantic contracts:

| stage | corpus | seed | trials | max output tokens | calls |
| --- | --- | ---: | ---: | ---: | ---: |
| D2 probe | `fixtures/semantic-decidability-d2` | 6000 | 1 | 512 | 15 |
| v5 probe | `fixtures/semantic-decidability-holdout-v5` | 7000 | 1 | 512 | 24 |

The D2 probe is inspected first. v5 runs only if D2 completes as a GitHub Actions job. Holdout-v5 SHA-256 payload verification remains mandatory before provider initialization.

This probe asks only whether the existing R2 materialized decision protocol plus deterministic decidability composition is operationally usable on Nemotron and whether the one-trial clear-case, typed-insufficiency, unsafe-assertion, ambiguity, and stability metrics are directionally consistent with the already recorded Mistral/Gemma pilot. One trial cannot establish cross-seed stability or qualify Nemotron as a D3 adoption model.

No fixture, label, threshold, semantic prompt, decidability rule, or holdout payload may be changed in response to this result. An operational failure remains operational evidence rather than a semantic score.

## Observed result

GitHub Actions run `33392183569` executed the frozen probe from merged main. The D2 job completed, but only 7/15 provider observations were successful (`0.4667` protocol completion); the other eight were `materialization_protocol` failures. Each repeated failure returned a model-owned `finding` field even though `materialization-r2-v1` permits only `decision` plus optional `advisory_note`. The single D2 trial is therefore operationally incomplete and has no semantic aggregate score.

The dependent holdout-v5 job passed payload hash and credential checks but hit the 40-minute job timeout. It reached fixture 18/24: eight logged observations succeeded and ten failed with the same materialization-protocol class before cancellation. The output artifact was zero-length because the process was terminated before final JSON serialization; the per-fixture log still preserves the operational failure classes.

This result is a negative control for protocol compatibility. It does not reject D3 semantics: Nemotron frequently fails before a valid R2 base decision exists, so deterministic decidability composition cannot be evaluated on a complete denominator. No provider-specific prompt/schema relaxation is introduced to rescue this model.
