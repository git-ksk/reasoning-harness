# Live benchmark CI

The live benchmark workflow is a manual research workflow. It is intentionally separate from required deterministic CI because provider availability, trial quota, rate limits, model catalog state, and stochastic output are external variables.

## Credentials

Repository secrets are provider-level credentials:

- `MISTRAL_API_KEY` for Mistral;
- `GEMINI_API_KEY` for Google Gemini/AI Studio;
- `NVIDIA_API_KEY` for NVIDIA Hosted NIM.

Normal `ci.yml` does not require any of these secrets. Google and NVIDIA jobs skip provider calls when their secret is absent. Credentials are sent only in provider authentication headers and must never be written to benchmark JSON, logs, committed fixtures, or issue comments.

## Manual model selection

`.github/workflows/live-benchmark.yml` exposes one selector per provider. Each selector accepts `all`, `none`, or one explicit model ID, so a single NVIDIA model can be exercised without running the full matrix. The NVIDIA `all` set is currently:

- `nvidia/nemotron-3.5-lightning-30b-a3b`;
- `nvidia/nemotron-3-ultra-550b-a55b`;
- `deepseek-ai/deepseek-v4-flash-0731`;
- `deepseek-ai/deepseek-v4-pro-0813`;
- `google/gemma-4-31b-it`.

NVIDIA jobs use `max-parallel: 1` to avoid creating unnecessary pressure on provider-managed trial limits. This is a scheduling precaution, not an asserted NVIDIA RPM limit. The matrix uses `fail-fast: false`, so one model job failing does not cancel the remaining models.

## Result and failure semantics

Live benchmark JSON makes the requested top-level provider and model explicit. Each successful generation records the returned provider model ID, latency, provider attempt count, and token usage when the API exposes it. Generation failures are retained as structured case records with fixture ID, provider, requested model, latency, failure class, and a bounded diagnostic message. Aggregate correctness metrics use only successfully generated/evaluated cases; `operational.attempted_runs`, `generated_runs`, and `failed_runs` make any reduced denominator explicit.

Within a model run, provider failures do not abort collection of later fixtures. After the report is produced, the workflow marks that model job failed when `operational.failed_runs` is non-zero. This preserves diagnostics while still making live smoke failures visible. Deterministic harness failures remain separate in `result.harness.deterministic_failure`; provider outage, quota, or timeout must not be misreported as a harness correctness failure.

The committed deterministic fixture regression remains the required correctness gate. Live results are diagnostic observations and never rewrite or override deterministic verification authority.

NVIDIA Hosted NIM calls use a conservative client-side minimum interval of 1.6 seconds (at most 37.5 request starts/minute per benchmark process). This is pacing, not a claimed provider quota: NVIDIA limits may vary by model/account, and HTTP `429` with `Retry-After` remains authoritative.
