# Engine 0.6 source-attribution holdout v1

Status: fresh independent holdout corpus prepared and still unobserved. No acceptance provider generation has occurred.

Development semantics are frozen by the immutable v6 PASS at `engine-0.6-source-attribution-development-v6-freeze` / `2287594cce8c0885e14987a4660b6bbc7f664819`, canonical run `37294100665` attempt 1. The v6 result record is committed separately and remains development evidence only.

## Runner freeze

Dedicated binary: `reason-source-attribution-holdout-study`.

The runner binds the future holdout to Issue #463, 18 independently authored cases matching the issue evaluation families (6 positive / 9 safety / 3 mixed), the unchanged v6 utility thresholds and zero hard gates, and three required acceptance providers:

- Mistral `ministral-8b-2512`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`

The output marks canonical holdout observations as `holdout_acceptance_evidence=true`. Validate-only remains non-scorable and performs no provider generation.

The runner must be frozen and pass exact-head CI before the fresh holdout corpus is authored. Groq must not be invoked during runner preparation or corpus authoring.

## Freshness requirement

After runner freeze, author `fixtures/source-attribution-holdout-v1` from scratch. It must not reuse development v1-v6 case IDs, canonical entities, tasks, exact source excerpts/observations, or exact 8-token windows. The corpus and one-shot workflow receive a separate freeze coordinate only after freshness, checksum, validate-only, offline tests, fmt, Clippy, and CI pass.

The one-shot frozen holdout must require all three providers. Any required provider miss or hard-gate violation is immutable FAIL; the run, corpus, labels, scores, and freeze tag must not be rerun, rescored, relabelled, edited, or moved.

## Frozen holdout corpus

The post-runner-freeze corpus contains 18 cases: 6 positive, 9 safety, and 3 mixed, matching the Issue #463 evaluation categories. Six cases are utility-eligible model-transform opportunities. The frozen utility floors remain useful retention >= 0.90, avoidable abstention <= 0.10, and citation coverage = 1.0; every hard-gate counter remains required at zero.

Freshness is mechanically checked against every source-attribution development manifest. There is no reuse of development case IDs, exact tasks, or exact 8-token windows; all canonical synthetic entities and source text are new.

Holdout seed: `463601`. Intended corpus freeze tag: `engine-0.6-source-attribution-holdout-v1-freeze`. The tag-triggered workflow requires Mistral, Google, and Groq and rejects workflow reruns.
