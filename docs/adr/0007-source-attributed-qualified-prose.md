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


## Development v1 feedback amendment

The first frozen development run showed that asking a model to echo Harness-owned target IDs, binding IDs, and the exact statement under assessment creates avoidable protocol/utility failure without adding authority. The contract is therefore tightened: model-facing proposal output contains transform content only, and assessment output contains disposition only. Harness code injects the exact target, binding set, statement, and requested translation output language after parsing.

This is a strengthening of the original ownership decision, not an expansion of model authority. The frozen v1 result remains immutable.

## Research-backed semantic gate amendment

The v2 development result showed that the remaining utility loss was dominated by a lexical modality heuristic rather than provider transport or citation binding. The successor design was cross-checked against established attribution/evaluation work before changing semantics:

- AIS (Measuring Attribution in Natural Language Generation Models) motivates evaluating whether a statement is attributable to an identified source independently from whether that statement is externally true.
- ALCE (Enabling Large Language Models to Generate Text with Citations) motivates keeping claim support, citation support/completeness, and multi-source citation precision as separate evaluation dimensions rather than one undifferentiated verdict.
- FActScore motivates evaluating support at atomic-proposition granularity so that one unsupported clause cannot hide inside an otherwise supported sentence.
- Inspect AI scoring policy motivates keeping task/grader semantics separate from operational/protocol failure so evaluation machinery does not silently become a semantic label.

Accordingly, transformed source-attributed claims now require two independent semantic conditions:

1. every atomic factual proposition in the transformed statement is fully supported by the jointly bound source excerpts; and
2. the transformation preserves modality, conditions, tense, quantity, scope, timing, causality, availability, benefits, and authority without strengthening.

Both conditions are advisory model verdicts bound to Harness-owned target/source/statement identity, and both must pass before materialization. They create no external-world authority.

Deterministic hard rejection remains for mechanically provable violations such as invalid target/source/span binding, non-canonical exact quotes, unsupported numeric precision, explicit current-state/causality/scope expansion, and hard-verification lane crossing. Cross-lingual or paraphrastic hedge-token disappearance is not itself treated as a proof of modality strengthening; semantic strengthening remains a hard gate, but is evaluated in the semantic assessment lane when it cannot be established mechanically.

This amendment does not change the immutable v1/v2 observations. Any successor development surface exists because the generic semantic/materialization contract changed, not because failed cases were relabelled or replaced.
