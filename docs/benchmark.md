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

### Repeated-trial stability semantics

`--trials N` keeps every fixture/trial observation as an individual case, but repeated-trial analysis is **not** inferred from the pooled `N × fixtures` comparison alone. Each trial is executed as one full-corpus pass; fixture concurrency is contained within that pass, and the next trial does not start until the current pass finishes. Final case output is still restored to the historical fixture/trial order. Live JSON also includes `stability`, which groups cases by trial index and reports an explicit per-trial correctness denominator and operational status. The pooled top-level `comparison` remains for backward-compatible case-level inspection; stable model comparisons must use `stability.correctness`.

A trial is `operationally_complete` only when every expected fixture generated and was evaluated without a provider/model operational failure. Partial trials remain visible under `stability.per_trial` with their successful correctness denominator and failure-class counts, but they are excluded from cross-trial correctness mean/min/max/stddev. Provider availability therefore does not masquerade as model correctness variance. If no complete trial exists, `stability.correctness` is omitted rather than synthesized from partial data.

For complete trials, baseline and harness distributions include verdict accuracy, accept/reject/unknown recall, unsafe-accept cases, deterministic verifier failure rate, contradiction detection, and counterexample detection. Every scalar distribution reports `count`, `mean`, `min`, `max`, and **population** standard deviation over the observed complete trials (`sqrt(sum((x - mean)^2) / N)`). No Bessel/sample correction is applied because the report describes the observed trial set rather than estimating an unobserved population parameter.

`stability.diagnostics` is a sibling report, not part of correctness. It aggregates provider-neutral diagnostic signals per fixture across operationally complete trials: adversarial contradiction/counterexample findings, assumption findings, evidence-qualification findings, candidate-normalization diagnostic codes, and causal finding/reason records when a caller supplies causal inspection output. Incomplete trials are excluded from diagnostic frequency/count distributions as a whole and remain visible through `operational_failures` plus `excluded_incomplete_trial_observations`. Per-fixture reports preserve exact occurrence counts and denominators, plus mean/min/max/population-stddev for diagnostic counts.

Diagnostic proportions receive a **95% Wilson score interval** only when at least five complete observations exist for that fixture. Smaller samples keep the exact frequency/denominator but omit the interval rather than implying statistical precision. Wilson intervals are descriptive uncertainty bounds only; the harness does not infer model rankings or significance from them.

Operational variability is reported separately. `stability.operational` includes successful-request total-token and latency distributions plus complete-trial total-token and summed-request-latency distributions. Missing provider token metadata is omitted from token distributions rather than converted to zero. Provider failures are classified operationally and never assigned a correctness verdict.

The manual workflow defaults Mistral and Google stability studies to 5 trials/model. A 10-trial follow-up is reserved for models whose 5-trial distributions still overlap materially. NVIDIA has a separate trial-count input and remains 1 trial by default because Hosted NIM is supplementary to the Issue #6 Mistral/Google study and is slower/more capacity-sensitive.

## Versioned corpus contract

`fixtures/corpus/v1.json` defines corpus `1.0.0` / score-compatibility ID `corpus-v1` across 41 active deterministic cases: 20 claim, 8 causal, 5 assumption, and 8 evidence-qualification cases. Stable suite-prefixed IDs, categories, difficulty strata, scoring modes, provenance/redistribution metadata, contamination notes, and lifecycle status are part of the manifest contract. Metamorphic seed fixtures remain unscored evaluation controls.

Recorded claim JSON keeps the historical top-level `comparison` unchanged and additionally exposes `corpus.stratification.by_category` and `by_difficulty`. Live runs preserve `corpus_version` and `score_compatibility_id` but omit pooled stratification so repeated/incomplete trial handling continues to be owned only by `stability.correctness`.

Direct score comparison requires the same `score_compatibility_id` and unchanged metric case/scoring contract. See [corpus versioning](corpus-versioning.md) for change discipline, contamination posture, and saturation warnings.

## Recorded corpus

The claim/verdict suite contains 20 fixtures: 5 expected `accept`, 6 expected `reject`, and 9 expected `unknown`. It covers:

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

### Evidence-aware causal diagnostic regression

Issue #4 adds a separate deterministic corpus under `fixtures/causal/`. It evaluates typed causal relations and per-edge support status for exact scoped support/refutation, association-only evidence, reverse-direction support, conflicting evidence, missing proposition bindings, partial multi-cause support, and scoped near-neighbors. The regression reports `supported | refuted | unknown` edge counts plus hard/soft causal finding counts.
Malformed harness-owned causal evidence is a fixture/input error rather than a scored unknown case, so broken oracle data cannot inflate conservative-edge counts.

This corpus is deliberately separate from the 20-case claim-verdict benchmark. The existing `causal_edge_quality` metric above remains the historical fixture-label metric for retained bad inference IDs; it is **not** evidence-grounded causal accuracy. Causal diagnostic results do not enter Issue #6 verdict-correctness denominators or operational-completeness calculations. The provider-neutral repeated-trial diagnostic report now accepts causal support/refutation/unknown assessments plus finding/reason observations without placing them in verdict-correctness denominators. A live causal-generation/input contract remains separate; adding one must not grant causal diagnostics verdict authority.

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

`gemma-4-26b-a4b-it` remains an experimental/allow-failure matrix entry. In the Issue #6 five-trial study it generated 98/100 cases: two requests were rejected by the provider with HTTP 400 copyright/recitation blocks, leaving 3 operationally complete trials and 2 incomplete trials. The stability report therefore excludes those incomplete trials from correctness variance while retaining both failures as provider operational observations.

### Repeated-trial stability results

The Issue #6 stability study ran five full-corpus trials per Mistral/Google model (20 fixtures per trial). Correctness statistics below use only operationally complete trials; incomplete trials remain operational observations and are not folded into the accuracy distribution.

| Model | Complete trials | Incomplete | Harness accuracy mean | Min-max | Pop. stddev | Accept recall | Reject recall | Unknown recall | Unsafe accepts | Verifier failures |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-8b-latest` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `ministral-14b-latest` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `gemini-3.1-flash-lite` | 5/5 | 0 | **1.000** | 1.00-1.00 | 0.000 | 1.000 | 1.000 | 1.000 | 0 | 0 |
| `mistral-small-latest` | 5/5 | 0 | 0.990 | 0.95-1.00 | 0.020 | 1.000 | 1.000 | 0.978 | 0 | 0 |
| `gemini-3.5-flash-lite` | 5/5 | 0 | 0.980 | 0.95-1.00 | 0.024 | 1.000 | 1.000 | 0.956 | 0 | 0 |
| `gemma-4-31b-it` | 5/5 | 0 | 0.950 | 0.95-0.95 | 0.000 | 0.800 | 1.000 | 1.000 | 0 | 0 |
| `gemma-4-26b-a4b-it` | 3/5 | 2 | 0.867 | 0.85-0.90 | 0.024 | 0.800 | 1.000 | 0.815 | 0 | 0 |
| `ministral-3b-latest` | 5/5 | 0 | 0.750 | 0.75-0.75 | 0.000 | 0.000 | 1.000 | 1.000 | 0 | 0 |

The five-trial result changes the interpretation of the earlier one-run matrix. Ministral 3B is not merely noisy around a lower point estimate: it was consistently over-conservative, with harness accuracy 0.75 and accept recall 0 in every trial while preserving reject/unknown recall and safety. Mistral Small and Gemini 3.5 are strong but show small unknown-class variance. Gemma 31B is stable at 0.95 but consistently misses one accept-class case. Gemma 26B combines lower correctness with an operational provider-blocking issue, so its three complete trials must not be compared as though five complete trials existed.

The models still tied on all primary harness correctness metrics after five trials were `ministral-8b-latest`, `ministral-14b-latest`, and `gemini-3.1-flash-lite`. They therefore received the targeted 10-trial follow-up required by the research plan:

| Model | Complete trials | Incomplete | Harness accuracy | All class recalls | Unsafe accepts | Verifier failures | Mean summed request latency / trial | Mean tokens / trial |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `ministral-8b-latest` | **10/10** | 0 | **1.000 ± 0.000** | 1.000 | 0 | 0 | 80.9s | 29,356 |
| `ministral-14b-latest` | **10/10** | 0 | **1.000 ± 0.000** | 1.000 | 0 | 0 | 95.8s | 28,419 |
| `gemini-3.1-flash-lite` | **9/10** | 1 | **1.000 ± 0.000** over complete trials | 1.000 | 0 | 0 | 154.8s | 8,731 |

The Gemini 3.1 follow-up attempted all 200 fixture generations. One `contradictory-evidence` generation in trial index 4 failed operationally because the Gemini Interactions response contained no model text output (`protocol`); the other 199 generations succeeded. The nine complete trials remained perfect on harness accuracy and all class recalls. This run is intentionally **not retried to erase the failure**: operational instability is part of the observation and is exactly why correctness and provider reliability are reported separately.

Accordingly, the 10-trial follow-up does **not** separate Ministral 8B, Ministral 14B, and Gemini 3.1 on complete-trial harness correctness. It does separate them operationally in this observation: both Mistral models completed 10/10 trials without a generation failure, while Gemini 3.1 completed 9/10 because of one protocol failure. Between the two Mistral models, 8B used slightly more tokens per trial but lower summed request latency than 14B. These are observations under this corpus/provider state, not universal model rankings.

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


## Assumption diagnostic regression

Issue #12 adds a separate deterministic corpus under `fixtures/assumptions/`. It measures premise support classification rather than final task correctness. The initial five cases cover a trusted supported premise, a harness-owned explicit input assumption, an introduced typed unsupported premise, semantic reuse of one unsupported premise across multiple inference edges, and an unbound premise.

The aggregate reports supported/explicit/unsupported/unbound premise counts, hard and soft finding counts, unsupported-premise detection rate, and explicit-assumption recognition rate. These cases never enter the 20-case verdict denominator or the eight-case causal denominator. `AssumptionFinding` is observational: even a hard unsupported-premise finding does not directly force `reject`; final authority remains with verification and acceptance policy. Assumption signals are also available to the provider-neutral repeated diagnostic report introduced by #11.

See [assumption diagnostics](assumption-diagnostics.md) for the boundary between explicit assumptions, unknown claims, and unsupported causal edges.

## Evidence qualification regression

Issue #16 adds a separate deterministic corpus under `fixtures/evidence-qualification/`. The initial eight cases cover exact qualification, stale and not-yet-valid evidence, disjoint scope, unsupported scope expansion, insufficient authority, conflict between otherwise qualified structured values, and missing temporal/scope/provenance metadata.

The aggregate reports qualified/disqualified/unknown evidence counts, hard/soft finding counts, and expected finding-reason detection rate. These cases do not enter the 20-case verdict denominator, the eight-case causal denominator, or the five-case assumption corpus. When ordinary benchmark inputs contain `evidence_requirements`, the built-in structured verifier uses the qualification-aware path; missing or disqualified evidence and conflicting qualified values cannot create a hard receipt.

See [evidence qualification](evidence-qualification.md) for the exact temporal, scope, provenance, and trusted-receipt boundaries.

## Metamorphic robustness regression

Issue #10 adds a deterministic metamorphic layer that measures whether trusted outcomes remain invariant under semantics-preserving representation changes. The initial suite covers six transform families across verdict, adversarial, and causal behavior. Its invariance metrics are separate from raw 20-case claim accuracy and from the eight-case causal corpus; transformed cases never enter either correctness denominator.

See [metamorphic-testing.md](metamorphic-testing.md) for the semantic/non-semantic field contract and reporting rules.

## Bounded resolution and finalization regression

Issue #22 adds a separate deterministic suite under `fixtures/resolution/`. Resolution scenarios are **variants**, not new corpus-v1 primary cases: every initial scenario records a stable `base_case_id` and must match the path/fixture identity already committed in `fixtures/corpus/v1.json`. The initial nine variants reuse `claim:missing-evidence`, so direct one-shot, diagnose-only, and bounded-resolution observations share the same base-case identity without changing the 20-case claim denominator.

Run:

```bash
cargo run -p reasoning-harness-cli -- eval-resolution fixtures/resolution --format json
```

The initial deterministic aggregate is 9/9 expected scenarios. All nine begin `unknown` in the diagnose-only harness. Bounded resolution produces one `resolved_supported`, one `resolved_refuted`, and seven `exhausted` outcomes for stale, wrong-scope, insufficient-authority, conflicting, no-result, malformed, and untrusted resolver conditions. Unsafe emitted final answers remain 0, blocked unverified finalizations are reported separately, and mean typed factual-claim coverage is 1.0. Ten total controlled resolver attempts are recorded.

These values measure protocol regression only. Resolver outputs and trusted metadata are fixture-controlled, so the recovery rate is not an empirical claim about web retrieval, model self-correction, or external tool quality. Resolution metrics are serialized separately from ordinary `BenchmarkComparison` and `stability.diagnostics`.

See [bounded grounded resolution and finalization](grounded-resolution.md) for the acquisition/admission/verifier and finalization boundaries.
