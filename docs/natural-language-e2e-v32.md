# Natural-language E2E v32

## Purpose

v32 is the fresh held-out successor after immutable v31 r2. v31 r2 (`natural-language-e2e-v31-freeze-r2`, `22a035ae7e3c88de679a950ab4429b391f9ebafa`, seed `98473`) produced Mistral paired PASS, Gemma paired PASS, Groq candidate-only PASS, and Gemini paired FAIL. The Gemini candidate became operationally incomplete at 6/13 because of Google free-tier quota HTTP 429; observed correctness-boundary violations remained zero. v31 r2 is not rerun or rescored.

The v32 candidate is `eafa6015c0d092c629d286708cb745ac3c9d7859`. Relative to v31 it adds only independently merged #335 (structured Google quota-window classification) and #334 (opt-in cross-process shared Google request pacing). Metric v13, released control `29a9e4be6273dbffeda324e15517dc64930ad315`, prompts/case semantics, and correctness/authority/admission/verification/finalization boundaries remain unchanged.

## Fresh surface

- corpus: `natural-language-e2e-v32`
- seed: `99584`
- 13 fresh synthetic cases
- scoring: `natural-language-e2e-scoring-v32-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `eafa6015c0d092c629d286708cb745ac3c9d7859`

## Prospective execution infrastructure

Google lanes set `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=1500` and a shared pacer path so concurrent `reason` subprocesses serialize request starts through one lane. Only `google/gemma-4-31b-it` may run adjacent eligible stateless investigation cases with at most two workers. Adaptive follow-up, MCP non-promotion, and session cases remain serial. Reports are materialized in manifest case-index order.

This changes throughput only. It does not change provider retry counts, semantic retries, scoring, case replacement, result-dependent rescheduling, or paired control-before-candidate ordering.

## Release discipline

Do not create the freeze/tag or attach live labels until every deterministic pre-live gate and normal CI is green. After freeze, launch Mistral paired exactly once; only a PASS permits the cross-model gate. Candidate operational failure remains a hard FAIL, cross-model averaging is forbidden, and whole-run retry is forbidden.
