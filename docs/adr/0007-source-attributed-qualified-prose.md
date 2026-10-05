# ADR 0007: Source-attributed qualified prose

Status: proposed for Engine 0.6 under #463.

## Decision

Add a separate Harness-owned source-attribution lane. “Source S states X” never means “X is externally true/current/applicable.”

The Harness owns exact target identity and policy, admitted evidence/source binding, bounded UTF-8 source span, locator and retrieval/version metadata, authority ceiling, materialization policy identity, transform acceptance, conflict state, canonical exposed text, citations, validation, and replay persistence.

Providers may propose paraphrase, summary, or translation and may assess semantic preservation. They cannot choose target identity, evidence/source binding, authority, hard-verification policy, citations, or final exposed prose.

Exact quotes are constructed from the bound span. Transforms pass deterministic anti-strengthening checks plus an exact-binding preserved assessment. Translation is authority-neutral and remains bound to the original source.

Canonical finalization has no free-form renderer input. Compatible sources retain all citations. Conflicting statements remain separate. Source attribution cannot satisfy or repair a hard-verification target.

Accepted state is persisted in ReasoningArtifact, so replay performs no external refetch. Ordinary provider telemetry remains structural (attempt/status/token/timing) and does not require raw source/provider payload.

## Hard gates

All must be zero: source-binding violation; external truth promotion; renderer-only unsupported factual exposure; paraphrase/translation strengthening; wrong-target attribution; missing mandatory citation; replay external refetch.

Production code must not branch on fixture IDs, entities, or exact calibration phrases. Historical #461/#462/#468 observations and tags remain immutable.

## Evaluation freeze

Before live model observation, freeze one 18-case development surface and its scoring contract. Mistral and Google are required development providers. Groq is reserved for a separately authored fresh independent acceptance holdout after semantics freeze.

Per required provider, useful attributed-answer retention must be >= 90%, avoidable abstention <= 10%, and citation/source-binding coverage = 100%. Exact quotes use zero transform-model calls; transformed cases allow at most two model attempts. Token/latency/provider overhead is diagnostic only.

A development FAIL is immutable: no rerun, rescore, relabel, or result-driven case addition. Acceptance holdout authoring starts only after development observation and semantics freeze. A #463 PASS does not authorize Engine 0.6 release.
