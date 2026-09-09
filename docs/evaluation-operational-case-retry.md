# Operational-only case retry policy

This policy exists only to keep paired release evaluation scorable when a provider has a transient operational failure. It does not change evaluator or scoring semantics.

## Canonical observation

Each case has one frozen command identity: product coordinate, case identity, provider/model, seed, token budget, and config are unchanged across attempts. The driver may repeat that exact case command only after a narrowly typed retryable provider-operational failure.

The **first operationally complete or non-retryable result is the canonical semantic observation**. Earlier retryable operational failures remain audit records but are not scored, averaged, or used to choose among semantic answers. If the bounded retry budget is exhausted, the final operational failure is canonical and the case remains operationally incomplete.

Whole-run retries are forbidden because they would re-sample cases that already produced scorable semantic observations.

## Retryable classes

The driver accepts only provider generation failures classified as `transport`, `provider_unavailable`, `timeout`, plus released-v0.4.1 compatibility subtype `protocol` only when the provider contract says `Gemini Interactions response contained no model text output`.

The Google compatibility mapping is intentionally narrower than the generic `protocol` class. Invalid JSON, schema/serde failure, malformed semantic output, scoring/correctness failure, ordinary provider 4xx, quota, credentials, unsupported capability, resolver/tool/action operational failure, and rendering fallback are not retry triggers.

The default bound is two total case attempts. A frozen acceptance surface may predeclare another finite bound, but it must use the same bound for control and candidate before live credentials are exposed.

Refs: #260, #263, #311.
