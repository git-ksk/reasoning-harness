# Natural-language E2E v36

## Purpose

v36 is a fresh held-out successor to immutable v35. The v35 freeze (`natural-language-e2e-v35-freeze`, `17fa551b5a2aea1100083274f7e542f8bf30ded6`, seed `776466`) remains historical evidence and is never rerun, rescored, retuned, or reclassified. v35's release disposition was **FAIL** because INCONCLUSIVE is non-releasable: Mistral paired and Groq candidate-only passed; Gemini was INCONCLUSIVE after the candidate completed 13/13 cleanly while the released control had two structured-planner JSON EOF protocol failures; Gemma was INCONCLUSIVE / measurement not executed because both coordinates exited before any provider/model request on an eval-runner invariant bug. v35 also established that the 6000 ms shared Google request-start floor removed the v34 free-tier 429 failure mode at the observed ~9–9.7 calls/minute.

The v36 candidate is `9497b563ad914fada13d33e0c1a7fee549a1f1de`, main after #349/#350. Product/provider runtime crates are unchanged from the v35 candidate `dd66a4372cfb462f876ac3169ba91df8d4a7f436`. The prospective delta is eval infrastructure only: runner-side Gemma validation now checks Google request pacing against `parallel_execution_policy.google_request_start_interval_ms` and inter-case delay independently against `provider_policy.inter_case_delay_ms`. The shared pacer path remains absolute and required for Gemma workers=2.

## Fresh surface

- predecessor: `natural-language-e2e-v35-freeze` -> `17fa551b5a2aea1100083274f7e542f8bf30ded6`
- corpus: `natural-language-e2e-v36`
- seed: `738214`
- 13 synthetic cases with fresh identities, source refs, fact keys, and markers collision-checked against all observed predecessors including v35
- scoring: `natural-language-e2e-scoring-v36-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Google canonical request pacing remains 6000 ms and inter-case delay remains 3000 ms. Gemini and Gemma model jobs remain serialized (`max-parallel: 1`). Gemma alone may use two workers for eligible stateless investigation cases; adaptive follow-up, MCP non-promotion, and session/stateful cases remain serial. Request starts, including provider retries, continue through the shared Google pacer.

`validate_google_canonical_pacing.py` checks repository policy plus workflow/manifest wiring. `google_parallel_runner_policy.py` is additionally invoked by the v36 runner itself before any live provider/model request. The runner exposes a no-model `--runner-policy-only` path so the exact frozen `workers=2 + pacing=6000 + delay=3000 + absolute shared pacer` combination is exercised during pre-live validation.

## Release discipline

Do not freeze or launch v36 until pair validation, metric lock, pacing validation, runner-integration validation, fresh-collision tests, checksums, no-model capability probe, full Python tests, cargo fmt/clippy/test, and normal PR CI are green. After freeze, launch Mistral paired canonical exactly once; only a PASS permits the cross-model canonical. Groq, Gemini, and Gemma remain separate required rows. Candidate operational failure is a hard gate, `INCONCLUSIVE` is non-releasable, cross-model averaging is forbidden, and no observed v36 outcome may be followed by changing v36 settings and rerunning the same identity.
