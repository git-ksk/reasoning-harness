# Product external-information v4 cross-model replication

Issue #208 replicates the frozen `product-external-info-v4` matched-context four-arm measurement across additional model families. This is post-observation replication only: the v4 corpus, target propositions, scoring, evaluator semantics, MCP boundary, admission policy, and finalization logic remain byte-for-byte frozen from semantic head `e324ccbff6e818d205a734f06ccc8cac4b587588`.

## Models

- Mistral `mistral-small-latest`
- Mistral `ministral-14b-latest`
- Google-hosted `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`

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
