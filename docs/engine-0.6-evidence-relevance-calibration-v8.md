# Engine 0.6 candidate: evidence-target relevance calibration v8

Status: fresh semantic successor calibration after frozen v7 FAIL/incomplete plus the independently frozen identity-ambiguity and Google operational diagnostics.

## Successor semantics

v8 keeps the binding proposal v2 contract and introduces Harness materialization policy v4.

The only semantic change from v3 is negative target identity:

- `target=exact` keeps the v3 relation rules;
- `target=unresolved` remains `ambiguous`;
- `target=different` no longer forces `irrelevant` by itself;
- when primary binding says `target=different`, the Harness runs a separate one-sided negative-target confirmation;
- `confirmed_distinct_entity` may authorize `irrelevant` when the supplied local material affirmatively binds substantive content to a distinct entity/product;
- `confirmed_target_absent` may authorize `irrelevant` when the supplied local material establishes no target-specific local content;
- `not_confirmed`, missing confirmation, timeout, or protocol failure never becomes semantic rejection. `not_confirmed` materializes `ambiguous`; operational failures remain typed failures.

The confirmation model cannot create aliases, provenance, truth, authority, freshness, verification, or final relevance.

## Evidence before freeze

Fresh identity ambiguity diagnostic v1 run `36029430165` used 12 independently authored synthetic cases across three matched seeds on both Mistral and Groq. The primary binding assessor falsely emitted `target=different` on 11 expected-unresolved Mistral observations and one Groq observation. The one-sided distinctness candidate produced zero false confirmations, zero misses on explicit-different controls, and 36/36 gated dispositions on each provider.

Google full requalification v1 run `36078211994` then completed the full 26-case historical request-shape set 26/26, every call on attempt 1 with HTTP 200, zero retry/cancellation/429/503, and p50/p95/max latency 695/843/950 ms. This operationally requalified Google without erasing v4-v6 historical tail/capacity instability.

## Fresh calibration corpus

v8 has 32 synthetic calibration cases:

- the historical 26 calibration cases are retained for comparability;
- six fresh cases cover explicit not-a-rename distinctness, generic target absence, explicit target absence, unknown alias, unknown successor, and structured distinctness;
- expected negative-target confirmation is frozen only for cases whose expected primary target binding is `different`;
- production motivating product names/content remain excluded.

The case budget remains 60,000 ms. Primary binding and negative-target confirmation each have their own bounded structured-output attempt budget of at most two model calls, while both stages share the same overall case elapsed-time envelope. Confirmation output is capped at 96 tokens.

## Provider roles

Required semantic arms:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Non-gating full replication arm:

- Google `gemini-3.5-flash-lite`

Google uses the same 32-case semantic surface and expected labels. Its result is preserved and compared but does not determine the v8 release gate. Attempt-level HTTP telemetry remains enabled for Google.

## Acceptance

Each required arm must independently satisfy on the first/only frozen observation:

- planned/completed: 32/32;
- operational abort: none;
- failed provider cases: 0;
- wrong-target / unsafe relevance admission: 0;
- false relevance rejection: 0;
- expected-relevant left ambiguous: 0;
- utility miss: 0;
- negative-target confirmation exact on every expected confirmation case.

Proposal exact accuracy remains diagnostic. Materialized disposition is the semantic release gate.

Google replication reports the same semantic metrics plus operational/attempt telemetry, but cannot rescue or fail the required gate.

If v8 fails, it remains immutable FAIL and is not rerun. Independent holdout authoring remains blocked until a fresh canonical calibration passes.
