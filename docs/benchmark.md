# Benchmark design

## Goal

Measure the reliability difference caused by the harness itself rather than accidentally measuring a different model, provider, or prompt budget.

The initial comparison therefore uses the **same `ReasoningCandidate` in both arms**:

- **naive baseline**: treat the model-proposed epistemic states as authority and apply no deterministic verification process;
- **harness arm**: combine the same candidate with harness-owned evidence, downgrade unverified strong states, run required passes, and apply the harness acceptance policy.

This isolates process effects. A later supplementary experiment may compare free-form direct answers, but it must be reported separately because output format and prompt differences become confounders.

## Recorded fixtures vs live model runs

Committed fixtures contain synthetic recorded candidates designed to exercise specific failure modes. They are **deterministic regression tests, not empirical evidence of model quality**.

A live model study uses the same fixture inputs but replaces `recorded_candidate` with fresh provider output:

```bash
reason eval fixtures --provider mistral --model ministral-8b-latest --trials 5
reason eval fixtures --provider google --model gemma-4-26b-a4b-it --trials 5
```

Use `--seed` when a provider supports it. Trial N uses `base_seed + N`. Live runs are intentionally not part of the required CI gate because network availability, provider behavior, quota, and cost are external variables.

## Recorded corpus

The committed regression corpus contains 20 fixtures: 5 expected `accept`, 6 expected `reject`, and 9 expected `unknown`. It covers:

- direct structured facts across booleans, counts, versions, regions, and HTTP status;
- missing facts and correctly preserved unknowns;
- environment, tenant, temporal, and population-scope overreach;
- contradictory structured observations;
- counterexamples to universal health, policy, and request-success claims;
- unsupported causal attribution;
- Five Whys symptom restatement.

The corpus is intentionally adversarial and is still small enough to inspect case-by-case. It is a deterministic protocol regression suite, not a statistically representative model benchmark.

## Metrics

| Metric | Basis | CI-safe |
| --- | --- | --- |
| unsupported accepted claims | golden fixture labels + typed epistemic state | yes |
| evidence coverage | deterministic structural measurement | yes |
| verdict accuracy / accept / reject / unknown recall | golden fixture verdict | yes |
| hidden assumption exposure | golden fixture labels + typed state | yes |
| contradiction detection | golden fixture labels + typed verdict/state | yes |
| counterexample detection | golden fixture labels + typed adversarial findings | yes |
| causal edge quality | golden bad-edge labels | yes |
| deterministic verifier failure rate | runtime result | yes |
| token usage | provider usage metadata | live only |
| latency | local wall-clock around provider call | live only |
| provider cost | token usage + explicit caller-supplied price rates | live only |
| model-judge semantic score | soft evidence, not yet implemented | no hard gate |

Provider pricing is not hard-coded into the runtime because prices change independently of harness semantics. A live run can supply explicit rates:

```bash
reason eval fixtures \
  --provider mistral \
  --input-cost-per-million <usd> \
  --output-cost-per-million <usd>
```

The resulting report records token counts, latency, and calculated cost when all required metadata is available.

## Current recorded-fixture baseline

With harness-owned structured facts and trusted verification where deterministic authority is available, the 20 recorded fixtures currently produce:

- naive baseline verdict accuracy: 8/20 (40%);
- harness verdict accuracy under deterministic fixture coverage: 20/20;
- unsupported accepted claims: 8 → 0;
- accept recall: 1.0 → 1.0;
- reject recall: 0.0 → 1.0;
- unknown recall: 0.333 → 1.0;
- hidden assumption exposure: 0.1 → 1.0;
- contradiction detection: 0.0 → 1.0 for labeled deterministic conflicts;
- counterexample detection: 0.0 → 1.0 for labeled deterministic counterexamples;
- known bad Five Whys edges retained: 1 → 0.

This **is not 100% generic model reasoning accuracy**. It is a deterministic process regression result under explicit structured-fact/oracle coverage. Harness-owned facts and hard verifier outputs cannot be created by the model. Cases without hard authority still resolve conservatively to `unknown`. Soft semantic discovery remains a research gap.

A first live Mistral run before receipt-backed verification was introduced used seven generations, 6,022 tokens, and roughly 17.2 seconds of provider latency. It confirmed the core safety trade-off: untrusted candidate states could accept an unsupported claim, while the harness eliminated unsupported acceptance but was over-conservative.

A second live run after exact-statement-bound verification receipts used seven generations, 6,870 tokens, and roughly 19.8 seconds of provider latency. The naive arm reached 5/7 verdict accuracy while the harness arm reached 3/7. The harness still reduced unsupported accepted claims to zero, but four of seven harness cases failed closed because live model paraphrases did not exactly match fixture receipt statements. This is a useful negative result: exact natural-language string binding is safe but too brittle for a live verifier contract. The built-in verifier has since moved to a typed `Proposition { key, value }` target backed by harness-owned structured facts. Verified proposition claims are rendered canonically as `key = value`; exact prose binding remains compatibility-only. A fresh live run is required before treating the migration as complete.

Live runs are stochastic research observations and never replace the committed deterministic fixture baseline.

## Regression policy

`cargo test --workspace` contains a snapshot-style regression test for the recorded fixture aggregate. Intentional semantic changes must update both the implementation and the expected benchmark baseline. Live provider results never silently rewrite the committed baseline.

### Typed-target live result

A subsequent seven-case live Mistral run after typed proposition verification and malformed-inference isolation reached 6/7 harness verdict accuracy (85.7%) with zero deterministic verifier failures and zero unsupported accepted claims. Accept recall and unknown recall were 1.0; reject recall was 0.5. This materially improves on the exact-statement receipt run (3/7 with four deterministic failures) and shows that verifier binding is no longer the dominant failure mode. The remaining reject miss belongs to generic contradiction/counterexample discovery rather than hard-verifier transport or binding. As with every live run, these seven samples are diagnostic rather than a statistically stable model-quality estimate.

### Live metric hardening and cross-model matrix

Live benchmark labels no longer bind to provider-generated claim IDs. Unsupported/hidden-assumption labels bind to typed propositions, while contradiction/counterexample detection is measured at fixture level. `unsafe_accept_cases` separately counts only final `Accept` verdicts that contain an unsupported strong claim; this avoids conflating an internal strong claim with an overall `Unknown` verdict. The earlier 20-case report of one harness unsafe accept was a metric false positive in `partial-population`, whose final verdict was `Unknown`.

`HarnessInput.hypotheses` now carries harness-owned propositions that formalize hypotheses explicitly posed by the task. Candidates cannot add or mutate these targets. The runtime materializes a missing hypothesis as an assumed claim so deterministic structured-fact verification and adversarial discovery do not depend on the provider choosing the same proposition key.

The manual live workflow runs the same 20-case corpus against `ministral-3b-latest`, `ministral-8b-latest`, `ministral-14b-latest`, and `mistral-small-latest`. It also supports Google-hosted models through the Gemini Interactions API when `GEMINI_API_KEY` is configured: `gemma-4-26b-a4b-it`, `gemma-4-31b-it`, `gemini-3.1-flash-lite`, and `gemini-3.5-flash-lite`. Antigravity managed agents are intentionally excluded because they are not equivalent candidate generators. Every provider remains an untrusted candidate generator; provider output cannot grant verification authority or decide the final verdict. This matrix is diagnostic and is not a required CI gate. NVIDIA Hosted NIM is an additional optional matrix described below.

### First 20-case cross-model result

A one-trial manual matrix on the hardened 20-case corpus produced the following harness-arm results:

| Model | Baseline accuracy | Harness accuracy | Accept recall | Reject recall | Unknown recall | Unsafe accept cases | Contradiction detection | Counterexample detection | Tokens | Provider latency |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-3b-latest` | 0.60 | 0.80 | 0.20 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 7,414 | 19.0s |
| `ministral-8b-latest` | 0.65 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 27,221 | 71.5s |
| `ministral-14b-latest` | 0.85 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 32,033 | 109.5s |
| `mistral-small-latest` | 0.75 | 1.00 | 1.00 | 1.00 | 1.00 | 0 | 1.00 | 1.00 | 7,319 | 24.2s |

All four harness runs had zero deterministic verifier failures and zero unsupported strong claims for the typed benchmark targets. This is one stochastic trial per model, not a statistically stable ranking. The result nevertheless shows a useful boundary: the harness fully recovered the 8B, 14B, and Small runs on this structured corpus, while the 3B run remained over-conservative on direct-accept cases despite preserving reject/unknown safety.


### Gemma 4 adapter

The Rust `GemmaAdapter` uses the Google Gemini Interactions REST API (`/v1beta/interactions`) and the standard `GEMINI_API_KEY` credential. It maps the provider-neutral `ModelRequest` contract to `system_instruction`, `input`, `generation_config`, and `response_format`, including JSON Schema structured output. It parses only model text and token usage back into `ModelResponse`. The API key is sent only in the `x-goog-api-key` request header and is never included in diagnostics.

Gemma live CI is optional: if the repository secret is absent, the Gemma matrix reports a notice and skips provider calls rather than weakening required CI.

### First Gemma 4 cross-family result

The first live Gemma 4 run used `gemma-4-31b-it` through the provider-neutral Google adapter on the same hardened 20-case corpus. In one stochastic trial, baseline verdict accuracy was 0.85 and harness verdict accuracy was 0.95 (19/20). Harness accept recall was 0.80, reject recall 1.00, unknown recall 1.00, unsafe accept cases 0, contradiction detection 1.00, counterexample detection 1.00, and deterministic verifier failure rate 0. The run used 8,005 total tokens and approximately 103.2 seconds of provider latency.

This is the first successful live result from a non-Mistral model family and therefore provides an initial cross-family check of the provider-neutral boundary. It remains a single trial and does not establish a stable ranking against the Mistral models.

`gemma-4-26b-a4b-it` is also kept in the diagnostic matrix, but the current GitHub project received HTTP 403 for that model while 31B succeeded with the same credential and adapter. The 26B matrix entry is therefore experimental/allow-failure until provider access is resolved.

### NVIDIA Hosted NIM research outcome

The NVIDIA adapter uses the OpenAI-compatible Hosted NIM endpoint at `https://integrate.api.nvidia.com/v1/chat/completions` with one provider-level `NVIDIA_API_KEY`. Model IDs remain data rather than adapter branches. The adapter requests generic JSON mode for structured candidate generation and leaves schema validation to the harness-owned candidate parser and validators. NVIDIA output never gains verification or verdict authority.

The 2026-08-30 live research exercised several Hosted NIM models on the same 20-case corpus. Correctness metrics below are computed only over successfully generated cases, so operational success rate must be read first.

| Model / run | Concurrency | Generated | Operational failures | Baseline accuracy | Harness accuracy | Total tokens | Sum of request latency | Approx. live wall time |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: |
| `nvidia/nemotron-3.5-lightning-30b-a3b` first run | 1 | 19/20 | protocol 1 | 0.368 | 0.947 | 46,989 | 757s | 15m22s |
| `nvidia/nemotron-3.5-lightning-30b-a3b` repeat | 4 | **20/20** | **none** | 0.450 | **0.950** | 38,045 | 736s | **4m19s** |
| `openai/gpt-oss-20b` | 4 | 18/20 | protocol 2 | 0.667 | 0.944 | 40,462 | 244s | 2m02s |
| `google/gemma-4-31b-it` | 4 | 14/20 | timeout 5, protocol 1 | 0.857 | 0.929 | 19,045 | 780s | 10m53s |
| `deepseek-ai/deepseek-v4-flash-0731` | 10 | 0/20 | timeout 20 | n/a | n/a | n/a | n/a | ~6m40s |

The DeepSeek run hit the adapter's 180-second request timeout for every case; an earlier serial probe also spent roughly three minutes per case and eventually hit the 40-minute GitHub Actions job limit. The Gemma run returned useful results but still timed out on 5/20 cases. GPT-OSS was the fastest useful probe but produced two structured-output/protocol failures. Nemotron Lightning was the only model in this research set to complete all 20 cases without an operational failure, and its repeat run preserved harness accuracy while four-way fixture concurrency reduced wall time by roughly 3.5x.

For that reason, the routine NVIDIA live matrix is intentionally narrowed to **`nvidia/nemotron-3.5-lightning-30b-a3b` only**, with `--concurrency 4` as the workflow default. GPT-OSS, Gemma-through-NVIDIA, DeepSeek, and larger Nemotron variants remain available through the data-driven CLI adapter for ad-hoc research but are excluded from routine live CI to avoid making the diagnostic workflow slow or flaky. This is a repository policy based on these observations, not a claim that the excluded models are universally unavailable or slow.

Hosted model availability, latency, capacity, and trial quota are external provider state and can change without a repository change. The live workflow therefore remains manual and secret-gated and does not promise a fixed RPM or token quota.

NVIDIA rate-limit handling honors `Retry-After` in either delay-seconds or HTTP-date form and otherwise uses bounded exponential retry. Operational failures are classified separately as `credentials`, `rate_limit`, `quota`, `provider_unavailable`, `timeout`, `transport`, `provider_error`, `protocol`, or `unsupported_capability`. A failed live generation is retained as a benchmark case rather than being misreported as harness correctness.

The adapter also applies conservative 1.6-second request-start pacing (37.5 request starts/minute maximum per benchmark process). This is a client-side guardrail, not an asserted NVIDIA quota.

### Live fixture concurrency

Live fixture suites accept `--concurrency N` (1-10, default 1 at the CLI). Independent fixture generations may be in flight concurrently, while final output is restored to fixture/trial order before aggregation. All workers share the same provider adapter, so NVIDIA pacing and `Retry-After` handling apply across the run. The routine NVIDIA workflow defaults to concurrency 4 because Nemotron Lightning completed 20/20 without timeout, rate-limit, or protocol failures at that setting.
