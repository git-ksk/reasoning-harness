# Natural-language E2E v35

## Purpose

v35 is a fresh held-out successor to immutable v34. The v34 freeze (`natural-language-e2e-v34-freeze`, `0aa45d4bfb72b445ad287e2a4f0c5b3e455a7ee2`, seed `940105`) remains historical evidence and is never rerun, rescored, or reclassified. v34's release disposition was **FAIL**: Mistral paired and Groq candidate-only passed; Gemini hard-failed because the candidate had two Google HTTP 429 quota generation failures at an observed free-tier request limit of 15; Gemma was INCONCLUSIVE because its candidate completed 13/13 with zero operational failures while the released v0.4.1 control was operationally incomplete from Google HTTP 500/provider-unavailable failures. Observed correctness-boundary violations were zero on those rows.

The v35 candidate is `dd66a4372cfb462f876ac3169ba91df8d4a7f436`, main after #346/#347. Provider runtime crates are unchanged from the v34 candidate `0741e2ba06618b5be37fd90b85495dbd36c0b166`; the narrow prospective Harness/evaluation change is canonical Google pacing policy. The shared Google request-start floor increases from 3000 ms to **6000 ms**, limiting sustained canonical starts to at most 10/minute and leaving 5 requests/minute (~33%) request-count headroom below the 15 RPM ceiling observed in v34. Google quota classification, provider retry semantics, max provider attempts, request timeout, transient-5xx backoff, worker counts, model ordering, inter-case delay, metric v13, scoring, and correctness/authority/admission/verification/finalization boundaries are unchanged.

## Fresh surface

- predecessor: `natural-language-e2e-v34-freeze` -> `0aa45d4bfb72b445ad287e2a4f0c5b3e455a7ee2`
- corpus: `natural-language-e2e-v35`
- seed: `776466`
- 13 synthetic cases with fresh identities, source refs, fact keys, and markers collision-checked against all observed predecessors including v34
- scoring: `natural-language-e2e-scoring-v35-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `dd66a4372cfb462f876ac3169ba91df8d4a7f436`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Gemini and Gemma canonical jobs remain serialized at the GitHub Actions job layer (`max-parallel: 1`). Each Google job uses `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=6000`. All request starts, including provider retries, pass through the same per-model shared pacer. Only Gemma may use two workers for eligible stateless investigation cases; adaptive follow-up, MCP non-promotion, and session/stateful cases remain serial. Paired execution remains control then candidate, and both coordinates run under the same pacing environment. Inter-case delay remains 3000 ms and is not used as the provider limiter.

The repository-wide policy and the v35 workflow/manifest wiring are mechanically checked by `validate_google_canonical_pacing.py` before credentials are exposed. No generic human-readable quota message is promoted into retryable status; ambiguous quota remains fail-closed.

## Release discipline

Do not create the v35 freeze tag or attach live labels until deterministic pre-live validation and normal PR CI are green. After freeze, launch the Mistral paired canonical exactly once; only a PASS permits the cross-model gate. Each Google row remains independently required, candidate operational failure remains a hard FAIL, `INCONCLUSIVE` is non-releasable, cross-model averaging is forbidden, and no observed v35 outcome may be followed by changing v35 settings and rerunning the same identity.
