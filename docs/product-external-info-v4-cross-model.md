# Product external-information v4 cross-model replication

Issue #208 established cross-model replication for the frozen `product-external-info-v4` matched-context four-arm measurement, and Issue #216 extends that operational replication surface to Groq. This is post-observation replication only: the v4 corpus, target propositions, scoring, evaluator semantics, MCP boundary, admission policy, and finalization logic remain frozen from semantic head `e324ccbff6e818d205a734f06ccc8cac4b587588`. The only allowed delta inside the v4 executable is provider wiring for `GroqAdapter`; `scripts/validate_product_external_info_v4_provider_wiring.py` mechanically removes exactly that allowlisted wiring and requires the remainder to match the frozen file byte-for-byte.

## Models

- Mistral `mistral-small-latest`
- Mistral `ministral-14b-latest`
- Google-hosted `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`
- Groq `qwen/qwen3.8-27b`
- Groq `openai/gpt-oss-20b`

The canonical Ministral 8B result from run `34000216929` remains the original v4 observation and is used only as the reference row.

## Fixed conditions

- corpus: `product-external-info-v4`
- evaluator: `reason-product-external-info-v4`
- comparison: `matched-target-context-four-arm-v4`
- seed: `28000`
- max tokens: `1024`
- 21 cases: 18 semantic + 3 typed operational
- within each model run, arm 3 and arm 4 share one decoded acquisition snapshot per case
- raw and Harness arms receive the same task, exact target hypothesis, evidence requirement, and authority policy

The live external acquisition is repeated per model run because v4 has no post-freeze snapshot-injection interface. Cross-model conclusions therefore compare the same frozen endpoints and semantic contract, while exact external response bytes are guaranteed matched only within each arm-3/arm-4 pair, not across provider runs. A model-free acquisition preflight must pass immediately before the matrix begins.

## Reporting

For each model, report arm 3 (`raw_model_with_external`) versus arm 4 (`harness_with_mcp_external`): expected-grounded target coverage, expected-unknown preservation, false target abstention, unsupported grounded claims, missed target insufficiency, Harness unsafe-admission/authority-promotion counters, typed rejection telemetry, model token usage, model latency, accounted end-to-end latency, and operational failures.

Provider/protocol failures remain separate from semantic scores. No result may be repaired by changing v4 after observation.


## Groq Free-tier operational extension — Issue #216

The Groq targets use the same frozen v4 corpus, seed `28000`, max-output budget `1024`, and four-arm scoring contract. Groq is connected through its OpenAI-compatible Chat Completions endpoint and `GROQ_API_KEY`; model IDs remain data rather than adapter-specific semantic branches.

The manual Groq lane targets `openai/gpt-oss-120b`, `qwen/qwen3.8-27b`, and `openai/gpt-oss-20b`. Groq currently publishes Free Plan limits of 30 requests/minute, 8,000 tokens/minute, 1,000 requests/day, and 200,000 tokens/day for each of these three models. The workflow therefore sets provider-local `REASON_GROQ_MIN_REQUEST_INTERVAL_MS=2100` and `REASON_GROQ_TOKENS_PER_MINUTE=8000`. The adapter combines minimum request-start spacing with actual prior-response token usage, honors `Retry-After`/rate-limit reset headers on HTTP 429, performs bounded retry, and can emit only non-secret rate-limit telemetry through `REASON_GROQ_RATE_LIMIT_TELEMETRY=1`. Groq Structured Outputs uses best-effort `strict:false` for the existing Harness-owned v4 schema rather than rewriting that schema to satisfy Groq strict-mode constraints; final typed validation/fallback remains Harness-owned. Because the Free Plan quotas for these targets are per-model, the workflow matrix may run all three models in parallel.

These pacing settings are workflow-local Free Plan controls, not universal Groq quota claims and not semantic tuning. They do not change prompts, output schemas, fixtures, acquisition, admission, verification, scoring, or finalization. Operational quota/rate-limit failure remains outside semantic denominators.

## First cross-model observation — 2026-09-06

GitHub Actions run `34001534798` reused the frozen v4 evaluation surface without modifying cases, targets, scoring, or evaluator semantics.

### Gemma 4 31B

`google / gemma-4-31b-it` completed all 21 cases.

- raw + external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `11/13 = 0.8462`; unsupported grounded claims `2`; missed target insufficiency `2`.
- Harness + MCP external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`; safety gate passed.
- The two raw unsafe cases were `authority-numpy-claim-cannot-self-promote-v4` and `insufficient-generic-content-no-envelope-v4`.
- Raw + external used 17,244 model tokens. Harness + external used 11,601 (`0.673x` raw tokens). Accounted end-to-end latency was 91,983 ms raw versus 106,611 ms Harness (`1.159x`). These are single-run operational observations, not stable performance rankings.

### Gemini 3.5 Flash-Lite

The first unpaced `google / gemini-3.5-flash-lite` attempt in run `34001534798` reached case 9 and then hit the AI Studio free-tier request quota (`15 requests/minute`). That incomplete attempt remains operational evidence and is not scored.

A provider-only paced retry in run `34002172470` used `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=4500`. The setting is opt-in, defaults to disabled, and changes request-start timing only; it does not alter v4 requests, responses, fixtures, scoring, admission, verification, or finalization. The retry completed all 21 cases.

- raw + external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`.
- Harness + MCP external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`; safety gate passed.
- Raw + external used 17,209 model tokens. Harness + external used 11,030 (`0.641x` raw tokens). Accounted end-to-end latency was 94,663 ms raw versus 97,917 ms Harness (`1.034x`). Provider pacing time is not fully represented by these model-call latency counters, so this remains an operational observation rather than a performance ranking.

For this model and this frozen corpus, the raw external arm was already fully safe and complete, so the Harness did not improve semantic scores; it preserved the same correctness boundary without reducing coverage.

### Mistral Small operational blocker

`mistral / mistral-small-latest` did not produce a scored report. The initial attempt and a later retry both failed on the first case with HTTP 429. Every observed retry returned `x-ratelimit-limit-req-minute=0` and `x-ratelimit-remaining-req-minute=0`, including after bounded backoff. Because no semantic case completed, this model is excluded from the cross-model correctness denominator. The failure is provider rate-limit state, not Harness semantic evidence.

### Ministral 14B

Run `34002627232` completed all 21 cases with `mistral / ministral-14b-latest` under the unchanged frozen-v4 contract.

- raw + external: grounded target coverage `4/5 = 0.8`; expected-unknown preservation `9/13 = 0.6923`; false target abstention `1`; unsupported grounded claims `6`; missed target insufficiency `4`.
- Harness + MCP external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`; safety gate passed.
- raw + external used 47,843 model tokens; Harness + external used 33,936 (`0.709x`). Accounted end-to-end latency was 187,730 ms raw versus 90,557 ms Harness (`0.482x`). These are single-run operational observations, not stable performance rankings.
- Live response headers reported `30` requests/minute and `937,500` tokens/minute for this model during the run. This is model/account-specific runtime evidence, not a universal Mistral tier claim.

The 14B result is important because larger parameter count did not monotonically eliminate unsafe raw external grounding: raw 14B still produced six unsupported grounded claims, while the Harness lane again retained full target coverage and zero unsafe exposure.

## First Groq observation — run `34008471577`

After switching Groq Structured Outputs to `strict:false` best-effort mode without changing the Harness-owned v4 schema, `openai/gpt-oss-120b` and `qwen/qwen3.8-27b` completed all 21 cases in the same run. `openai/gpt-oss-20b` remained operationally incomplete because of a provider-side JSON-validation HTTP 400 and is excluded from semantic scoring here; it is re-measured separately after adding bounded structured-output retry.

### GPT-OSS 120B

- raw + external: grounded target coverage `4/5 = 0.8`; expected-unknown preservation `10/13 = 0.7692`; false target abstention `1`; unsupported grounded claims `3`; missed target insufficiency `3`.
- Harness + MCP external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`; safety gate passed.
- Raw + external used 26,897 model tokens; Harness + external used 25,237 (`0.938x` raw tokens). Accounted end-to-end latency was 186,199 ms raw versus 160,182 ms Harness (`0.860x`).
- All four arms used 97,424 model tokens in total. This is a single-run operational observation, not a stable speed or cost ranking.

### Qwen 3.8 27B

- raw + external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `12/13 = 0.9231`; false target abstention `0`; unsupported grounded claims `1`; missed target insufficiency `1`.
- Harness + MCP external: grounded target coverage `5/5 = 1.0`; expected-unknown preservation `13/13 = 1.0`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`; safety gate passed.
- Raw + external used 17,660 model tokens; Harness + external used 8,382 (`0.475x` raw tokens). Accounted end-to-end latency was 82,842 ms raw versus 152,932 ms Harness (`1.846x`).
- All four arms used 45,964 model tokens in total. The Harness lane substantially reduced tokens while increasing latency in this single run, so token efficiency and wall-clock speed must not be conflated.

Across both models, the Harness lane retained full expected-grounded target coverage, restored expected-unknown preservation to 100%, and reduced unsupported grounded claims and missed target insufficiency to zero. In particular, the 120B raw external arm still produced three unsupported grounded claims and Qwen 27B produced one, consistent with the existing v4 evidence that model size or family alone does not guarantee safe external grounding.
