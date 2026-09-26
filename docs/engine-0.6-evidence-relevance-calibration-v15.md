# Engine 0.6 evidence-target relevance calibration v15

Status: pre-freeze successor to immutable v14 FAIL. No v15 live model observation has occurred. Independent holdout authoring remains prohibited.

## Fixed evaluation surface

v15 keeps evidence-relevance-fixed-core-v1 unchanged at 48 cases. No case is added, removed, or selected in response to v14 model misses. The scored disposition balance remains 14 Relevant / 18 Irrelevant / 16 Ambiguous.

The semantic contracts are:
- primary binding proposal: reason-evidence-relevance-binding-proposal-v4
- independent verifier: reason-evidence-local-qualification-v5
- Harness materializer: target-evidence-relevance-binding-materialization-v10

The v15 verifier returns only:
- blocking_reason: none / identity_mapping / ownership_scope / context_gap / multiple
- binding_confirmation: none / confirmed_target_relation / confirmed_distinct_target / confirmed_different_relation / confirmed_local_absence

The second call replaces v14 model-authored explicit_local_absence. A primary negative target binding alone cannot force Irrelevant; a matching one-sided confirmation is required. A primary unresolved positive may be retained only when the independent verifier confirms the exact target/relation and the Harness-owned identity floor is satisfied. Any concrete blocker remains fail-closed to Ambiguous.

Harness canonical-name/alias matching is boundary-aware for ASCII aliases. An anchor remains an identity floor/check, not independent proof that a requested relation belongs to the target.

## Operational budget v15

Provider retry ownership remains inside the provider adapters. v15 does not add a second semantic retry loop and does not remove the existing bounded retry behavior.

Each case has independent limits:
- semantic/provider active execution budget: 60,000 ms from the frozen case policy
- cumulative provider wait/retry budget: 45,000 ms
- single provider wait cap: 30,000 ms
- absolute case wall-clock deadline: 120,000 ms

Mistral, Groq, and Google expose adapter execution telemetry:
- provider attempts started/completed
- active execution time
- pacing wait
- retry wait

A provider-declared wait beyond the single or cumulative wait budget fails typed rate_limit instead of sleeping until the semantic deadline. Daily quota remains typed quota and is not retried as a transient short-window limit.

At the run level:
- one typed quota failure latches the provider arm and suppresses further guaranteed-failure calls
- two consecutive capacity failures (rate_limit, provider unavailable, transport/timeout, assessment/absolute timeout) latch the provider arm
- suppressed cases remain operational failures and can never satisfy canonical acceptance

Persisted runner failure messages sanitize quota/provider identity details rather than storing organization/project/account identifiers or billing URLs in public artifacts.

## Groq full-run capacity gate

The v14 48-case Mistral arm consumed 67,610 total tokens for 96 model calls. v15 uses a conservative required Groq daily headroom of 160,000 tokens before the one-shot canonical begins.

A tiny successful probe is not sufficient evidence of this headroom. The canonical preflight requires the repository variable:

ENGINE_0_6_RELEVANCE_V15_GROQ_CAPACITY_ATTESTATION=engine-0.6-evidence-relevance-calibration-v15-freeze:160000

This value is an operator attestation for the exact freeze tag. It must be set only after authoritative quota state or an isolated fresh reset window establishes at least that headroom. If daily remaining capacity cannot be established, the canonical must not start.

## Provider set and acceptance

Precommitted provider roles remain unchanged from v14:
- required: Mistral ministral-8b-latest
- required: Groq openai/gpt-oss-120b
- non-gating full replication: Google gemini-3.5-flash-lite

A provider is operationally complete only with 48/48 successful cases, no provider-arm latch, no operational abort, and complete attempt telemetry.

Each required arm must also have:
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility misses: 0
- materialized exact: 48/48
- blocker misses: 0
- spurious blockers: 0
- binding-confirmation misses: 0
- spurious binding confirmations: 0

The first/only frozen canonical is never rerun, rescored, relabeled, or retagged. Holdout authoring remains blocked until that canonical passes.
