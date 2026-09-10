# Natural-language E2E v33

## Purpose

v33 is a fresh held-out successor to immutable v32. The v32 freeze (`natural-language-e2e-v32-freeze`, `62a93e7edb12337edb36cb488f4cca79f49ff685`, seed `99584`) is historical evidence and is not rerun, rescored, or reclassified. Its canonical release disposition is **VALID RELEASE FAIL**: Mistral paired PASS and Groq candidate-only PASS, while Gemini and Gemma were operationally incomplete under Google quota/provider failures. Observed correctness-boundary violations on both Google rows were zero; the incomplete rows therefore do not establish a reasoning regression.

The v33 candidate is `c34610863d23bb014c4b0cf49f801ab873f0f67f`, the main commit after #339/#340. Relative to the v32 candidate, the product-runtime delta is only the prospective Google adapter change that classifies conflict-free bounded structured `google.rpc.RetryInfo.retryDelay` (<=120s) as an existing short-window `RateLimit` signal when no daily/billing signal conflicts. Explicit daily quota still wins; generic/ambiguous quota remains fail-closed. Metric v13, released control `29a9e4be6273dbffeda324e15517dc64930ad315`, prompts/case semantics, and correctness/authority/admission/verification/finalization boundaries remain unchanged.

## Fresh surface

- predecessor: `natural-language-e2e-v32-freeze` -> `62a93e7edb12337edb36cb488f4cca79f49ff685`
- corpus: `natural-language-e2e-v33`
- seed: `323518`
- 13 fresh synthetic cases with collision-checked identities, source refs, fact keys, and markers
- scoring: `natural-language-e2e-scoring-v33-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `c34610863d23bb014c4b0cf49f801ab873f0f67f`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Google rate-limit accounting is project-scoped, so Gemini and Gemma canonical jobs are serialized at the GitHub Actions job layer (`max-parallel: 1`) rather than launched on two independent runners. Each Google job uses `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=3000`, corresponding to at most 20 request starts/minute before provider-side backoff. This is deliberately below the 30 requests/minute limit observed in immutable v32 and leaves boundary/jitter headroom instead of targeting the limit exactly.

Within the Gemma job only, adjacent eligible stateless investigation cases may use at most two workers. Their request starts remain serialized by one absolute shared-pacer path; the workers overlap response latency rather than multiply request-start rate. Adaptive follow-up, MCP non-promotion, and session cases remain serial. Reports remain materialized in manifest case-index order, and paired control -> candidate ordering is unchanged.

These are prospective operational controls only. They do not add semantic retries, malformed-output retries, whole-run retries, favorable case replacement, result-dependent rescheduling, or scoring changes. The v33 evaluation surface also carries the already-v32-frozen `paired-canonical-observation-v2` helper because that evaluator-only helper is not present on product `main`; this is packaging preservation, not a product-runtime or scoring delta.

## Release discipline

Do not create the v33 freeze tag or attach live labels until deterministic pre-live validation and normal PR CI are green. After freeze, launch Mistral paired canonical exactly once; only a PASS permits the cross-model gate. Each Google row remains independently required, candidate operational failure remains a hard FAIL, `INCONCLUSIVE` is non-releasable, cross-model averaging is forbidden, and an observed v33 outcome must never be followed by changing v33 settings and rerunning the same identity.
