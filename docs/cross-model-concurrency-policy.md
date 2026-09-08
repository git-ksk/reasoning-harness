# Cross-model live concurrency policy

Cross-model evaluation uses **provider-aware lanes**. A common deterministic/no-network preflight may gate the live work, but once credentials and live observations begin, unrelated providers must not be collapsed into one global serialization merely for convenience.

## Scheduling rule

1. Run one shared preflight when the evaluation surface is common.
2. Fan out into independent provider lanes after preflight. Different providers may execute concurrently unless a concrete shared external resource requires otherwise.
3. Set model-job parallelism independently for each provider from documented quota scope and repository observations.
4. Treat in-model fixture concurrency as a separate control. A provider may serialize model jobs while still overlapping fixtures inside one model when that runner and provider combination has been validated.
5. Keep provider-owned pacing, retry, `Retry-After`, token budgeting, and rate-limit telemetry authoritative regardless of GitHub Actions parallelism.
6. Report quota/rate-limit/availability failures as operational evidence, not semantic or correctness failures.
7. Never modify or rerun a frozen observation solely to adopt a newer scheduling policy.

## Repository defaults

| Provider | Model-job default | In-model default | Rationale |
| --- | ---: | ---: | --- |
| Mistral | 1 | 1 | Repository-level live concurrency group protects shared account-level limits. |
| Google | 1 | 1 | Model jobs remain serialized. Fixture concurrency may use a separately validated bound; semantic-judge surfaces currently have evidence for 2. |
| NVIDIA Hosted NIM | 1 | 4 for the routine validated Nemotron Lightning runner | Avoid multiplying account pressure across models while overlapping slow requests within one validated model. |
| Groq current Free-tier replication targets | up to 3 current model jobs | 1 | The current three targets use per-model quota controls; each model retains request/token pacing and bounded retry. |

These are repository operational defaults, not universal claims about provider quotas. Any increase requires evidence for the active plan/model scope; any decrease should preserve provider lanes instead of serializing unrelated providers globally.

## Frozen historical exceptions

Issue #208 / the frozen product-external-info v4 replication predates this standard and used one Mistral+Google `max-parallel: 1` matrix. Its semantic surface remains immutable.

Issue #256 / `natural-language-e2e-v11-cross-model-v1-freeze` used one mixed Google+Groq matrix with `max-parallel: 1`. Its first live observation had already entered the canonical boundary when this policy gap was identified. That workflow is therefore an explicit historical exception: preserve it unchanged and apply this policy to successor replication identities.

`config/cross-model-concurrency-policy.json` is the machine-readable policy. `scripts/validate_cross_model_concurrency_policy.py` rejects new mixed-provider globally serialized cross-model workflows unless they are exact frozen historical exceptions, and also checks the known Mistral, Google, and Groq controls.
