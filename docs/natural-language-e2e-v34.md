# Natural-language E2E v34

## Purpose

v34 is a fresh held-out successor to immutable v33. The v33 freeze (`natural-language-e2e-v33-freeze`, `1a03f1a700766cf0c31672601b45bd15eec2cdc7`, seed `323518`) is historical evidence and is never rerun, rescored, or reclassified. The v33 canonical release disposition is **FAIL**: Mistral paired and Groq candidate-only passed; Gemini was INCONCLUSIVE because the control had one 180-second request timeout while the candidate completed 13/13 with zero operational failures; Gemma failed at 10/13 candidate completion because Google returned HTTP 500 / `provider_unavailable`, including an explicit high-demand response. Observed correctness-boundary violations were zero on these rows, so provider operational failure is not treated as a reasoning regression.

The v34 candidate is `0741e2ba06618b5be37fd90b85495dbd36c0b166`, main after #343/#344. Relative to the v33 candidate, the **only product-runtime resilience delta** is: Google request timeout increases from 180s to 300s, and retryable HTTP 500/502/503/504 transient backoff changes from `0.5s -> 1s -> 2s` to `2s -> 5s -> 10s`. Maximum provider attempts remain 4 (initial plus up to three retries). Rate-limit/quota classification, 429 retry behavior, Google request-start pacing, worker counts, and model ordering are unchanged. Metric v13, released control `29a9e4be6273dbffeda324e15517dc64930ad315`, prompt/case semantics, and correctness/authority/admission/verification/finalization boundaries are unchanged.

## Fresh surface

- predecessor: `natural-language-e2e-v33-freeze` -> `1a03f1a700766cf0c31672601b45bd15eec2cdc7`
- corpus: `natural-language-e2e-v34`
- seed: `940105`
- 13 synthetic cases with fresh identities, source refs, fact keys, and markers collision-checked against all observed predecessors including v33
- scoring: `natural-language-e2e-scoring-v34-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `0741e2ba06618b5be37fd90b85495dbd36c0b166`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

All v33 execution controls are preserved unchanged. Gemini and Gemma canonical jobs remain serialized at the GitHub Actions job layer (`max-parallel: 1`). Each Google job keeps `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=3000`. Only Gemma may use up to two workers for eligible stateless investigation cases, while all request starts still pass through one absolute shared pacer. Adaptive follow-up, MCP non-promotion, and session cases remain serial, and paired execution remains control then candidate.

The only new operational resilience allowed by v34 is the adapter-level 300-second request timeout and longer transient-5xx backoff. No semantic retries, malformed-output retries, whole-run retries, favorable case replacement, result-dependent rescheduling, or scoring changes are introduced. `paired-canonical-observation-v2` is carried unchanged from immutable v33 as evaluator-only orchestration.

## Release discipline

Do not create the v34 freeze tag or attach live labels until deterministic pre-live validation and normal PR CI are green. After freeze, launch the Mistral paired canonical exactly once; only a PASS permits the cross-model gate. Each Google row remains independently required, candidate operational failure remains a hard FAIL, `INCONCLUSIVE` is non-releasable, cross-model averaging is forbidden, and no observed v34 outcome may be followed by changing v34 settings and rerunning the same identity.
