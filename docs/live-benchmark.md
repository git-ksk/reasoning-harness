# Live benchmark CI

The live benchmark workflow is a manual research workflow. It is intentionally separate from required deterministic CI because provider availability, trial quota, rate limits, model catalog state, and stochastic output are external variables.

## Credentials

Repository secrets are provider-level credentials:

- `MISTRAL_API_KEY` for Mistral;
- `GEMINI_API_KEY` for Google Gemini/AI Studio;
- `NVIDIA_API_KEY` for NVIDIA Hosted NIM.

Normal `ci.yml` does not require any of these secrets. Google and NVIDIA jobs skip provider calls when their secret is absent. Credentials are sent only in provider authentication headers and must never be written to benchmark JSON, logs, committed fixtures, or issue comments.

## Manual model selection

`.github/workflows/live-benchmark.yml` exposes one selector per provider. The routine NVIDIA selector is intentionally narrow after the 2026-08-30 Hosted NIM research: its `all` set contains only `nvidia/nemotron-3.5-lightning-30b-a3b`. That model was the only NVIDIA research candidate to complete the full 20-case corpus without an operational failure.

Other NVIDIA Hosted NIM model IDs can still be exercised through the data-driven CLI adapter for ad-hoc research. They are not routine workflow choices: GPT-OSS 20B produced 18/20 generations with two protocol failures, Gemma 4 31B produced 14/20 with five timeouts and one protocol failure, and DeepSeek V4 Flash timed out on all 20 requests in the ten-way probe. See [benchmark.md](benchmark.md#nvidia-hosted-nim-research-outcome) for the recorded results.

NVIDIA jobs use `max-parallel: 1` at the model-job level. Within the selected model, the workflow defaults to fixture concurrency 4. This avoids multiplying account-level pressure across several NVIDIA models while still overlapping the slow Hosted NIM requests that proved safe for Nemotron Lightning. Neither value is an asserted provider quota.

## Result and failure semantics

Live benchmark JSON makes the requested top-level provider and model explicit. Each successful generation records the returned provider model ID, latency, provider attempt count, and token usage when the API exposes it. Generation failures are retained as structured case records with fixture ID, provider, requested model, latency, failure class, and a bounded diagnostic message. Aggregate correctness metrics use only successfully generated/evaluated cases; `operational.attempted_runs`, `generated_runs`, and `failed_runs` make any reduced denominator explicit.

Within a model run, provider failures do not abort collection of later fixtures. After the report is produced, the workflow marks that model job failed when `operational.failed_runs` is non-zero. Deterministic harness failures remain separate in `result.harness.deterministic_failure`; provider outage, quota, timeout, or malformed provider output must not be misreported as a harness correctness failure.

The committed deterministic fixture regression remains the required correctness gate. Live results are diagnostic observations and never rewrite or override deterministic verification authority.

NVIDIA Hosted NIM calls use a conservative client-side minimum interval of 1.6 seconds (at most 37.5 request starts/minute per benchmark process). This is pacing, not a claimed provider quota: NVIDIA limits may vary by model/account, and HTTP `429` with `Retry-After` remains authoritative.

Mistral live jobs share one repository-level GitHub Actions concurrency group, including the ordinary Mistral benchmark and a Mistral-backed semantic-judge run. They are serialized instead of competing for the same account-level rate limit. The Mistral adapter classifies HTTP `429` separately as `rate_limit` (or `quota` when the provider message explicitly indicates quota/billing exhaustion), honors `Retry-After`, and otherwise uses bounded exponential backoff for up to three retries. This is defensive client behavior, not a claim about a fixed Mistral quota.

## Repeated trials

The workflow exposes a `trials` selector (`1`, `5`, or `10`) for Mistral and Google and defaults it to **5** for stability research. NVIDIA uses a separate `nvidia_trials` selector with default **1** so the routine Hosted NIM diagnostic is not multiplied automatically.

For `--trials > 1`, read `stability.correctness` rather than treating the pooled top-level `comparison` as a stability estimate. Each trial is a full 20-fixture pass; `--concurrency` only overlaps fixtures inside that pass, and the next trial starts after the current one finishes. Only trials that generated/evaluated every expected fixture contribute to correctness mean/min/max/population-stddev. Incomplete trials remain in `stability.per_trial` with explicit denominators and failure classes. Token/latency distributions are reported separately under `stability.operational`.

`stability.diagnostics` is independent from correctness. It reports complete-trial-only per-fixture frequencies for typed diagnostic signals, diagnostic-count distributions, exact denominators, and 95% Wilson intervals when a fixture has at least five complete observations. Operationally incomplete trials are excluded from those diagnostic denominators and reported explicitly instead of being treated as diagnostic absence.

A one-trial live result remains a diagnostic point observation and is not enough to claim a stable ranking. Models whose 5-trial distributions materially overlap are candidates for a targeted 10-trial follow-up.

Each live model job preserves its raw JSON as a short-retention GitHub Actions artifact so the per-case/per-trial evidence can be reviewed after the workflow completes instead of relying only on console summaries.

Every live fixture-suite JSON also records the committed corpus identity (`corpus_version` and `score_compatibility_id`) when a manifest is present. Live output intentionally omits pooled category/difficulty stratification: complete-trial correctness remains owned by `stability.correctness`, so partial provider failures cannot silently change a stratum denominator.

## In-model concurrency

Use `--concurrency N` (1-10) to overlap independent fixture generations for one live model. Results are restored to fixture/trial order before aggregation, and one fixture failure remains isolated from other in-flight work. All workers share the same provider adapter, so NVIDIA request-start pacing and 429 `Retry-After` handling continue to apply across the run. The NVIDIA workflow defaults to 4 based on the successful 20/20 Nemotron Lightning repeat run.

## Live soft semantic-judge calibration

Issue #33 extends the manual workflow with an optional `semantic-judge` job. Set `judge_provider` to `mistral`, `google`, or `nvidia`, provide a compatible `judge_model`, and choose `judge_trials` (normally 5 before making any stability observation). The job runs:

```text
reason eval-judges fixtures/semantic-judges \
  --provider <provider> \
  --model <model> \
  --max-tokens <judge-max-tokens> \
  --seed 1000 \
  --trials <trials> \
  --concurrency <N> \
  --format json
```

Trials remain sequential stability samples, while `--concurrency` overlaps only independent fixtures inside the active trial. The workflow uses conservative provider-specific semantic fixture concurrency by default: Mistral 1, Google 2, and NVIDIA 4. NVIDIA workers share one adapter, so the existing 1.6-second request-start pacing and `Retry-After` handling remain authoritative across in-flight calls. Google starts at 2 rather than 3 so rate-limit behavior can be characterized before increasing parallelism.

`judge_max_tokens` defaults to 256 and can be raised to 512 or 1024 when calibrating reasoning-heavy models. Characterize token-budget changes on the calibration corpus before any new independent holdout measurement. A structured parse failure with `finish_reason=length` is truncation evidence, not semantic evidence.

Live semantic-judge requests also carry a provider-neutral `reasoning_preference=minimize` hint. This is not semantic prompt tuning: adapters may map the hint to a native request control when the provider supports one, while adapters without such a control preserve their existing behavior. NVIDIA maps it to `chat_template_kwargs.enable_thinking=false`; ordinary candidate generation keeps provider-default reasoning behavior.

The workflow stores the full JSON as a short-lived artifact and prints aggregate semantic metrics plus each failed run's fixture ID, trial, failure class, latency, and bounded provider-safe message. A provider/protocol failure does not become `no_finding`; any affected trial is excluded from semantic stability distributions. Precision/recall are accompanied by decision coverage and ambiguous-case abstention so aggressive decisions on intentionally uncertain cases stay visible. Ordinary Mistral and Google live benchmark summaries likewise print failed-run details instead of only a failure count. These metrics are independent of the ordinary live correctness benchmark. See [live soft semantic-judge study](live-semantic-judge-study.md) for the first repeated Mistral result and its holdout caveat.

For independent semantic-judge measurement, the manual workflow exposes `judge_corpus=holdout-v1` and `judge_corpus=holdout-v2`, mapping to their versioned observation-free fixture directories. Holdout-v1 remains frozen for the v2 configuration study; holdout-v2 is frozen for the first `soft-semantic-v3` study and must be merged before any provider result is observed on it. The live JSON reports corpus identity, complete-trial semantic stability, complete-trial family decision counts, operational fallback rate, successful-run fallback-reason counts, token usage, and latency. `fallback_reason_counts` distinguishes `not_needed`, `primary_json_schema_unsupported`, and `invalid_primary_structured_output`; it does not classify provider-internal retry attempts. Family semantic summaries exclude incomplete trials; operational metrics continue to describe all attempted calls.
