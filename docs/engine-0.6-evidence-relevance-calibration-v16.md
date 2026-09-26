# Engine 0.6 evidence-target relevance calibration v16 — successor design

Status: design only. No v16 live observation has occurred. Independent holdout authoring remains blocked.

## Fixed surface

- fixed core: evidence-relevance-fixed-core-v1
- cases: 48, unchanged
- dispositions remain the same semantic labels selected before v14
- no case-ID, synthetic-entity, or exact-fixture-phrase branching
- v15 remains immutable and is not rescored under v16

## Why v16 exists

v15 failed even with a fully operational Mistral arm. The verifier both over-produced blockers and under-produced negative confirmations, while one spurious positive confirmation combined with materialization v10's unresolved-primary rescue created a wrong-target Relevant result. The next successor therefore changes the contract shape rather than tuning individual examples.

## Primary proposal v5

Keep the independent target/relation axes:
- target_binding: exact | different | unresolved
- relation_binding: exact | different | unresolved

Clarify the generic semantics:
- Harness canonical names and declared aliases are authoritative identity metadata when the substantive local material is scoped to them.
- Cross-language query wording does not weaken an explicit canonical/alias binding in the candidate.
- Staleness, factual disagreement, downstream authority, and untrusted candidate instructions do not downgrade target/relation binding.
- Multiple adjacent signals from one candidate may jointly establish one same-target relation.
- allow_semantic_equivalent may still classify a locally specific semantic equivalent as exact without a literal anchor.
- shared ownership, omitted product columns/referents, uncertain rename/alias/successor mappings, and visibly clipped identity remain unresolved.

The intent is to improve atomic proposal accuracy rather than compensating later with unsafe rescue authority.

## Local verifier v6

Replace binding_confirmation with three orthogonal fields:

identity_scope:
- exact_target
- distinct_target
- target_absent
- unresolved

relation_scope:
- requested_relation
- different_relation
- relation_absent
- unresolved

scope_risk:
- none
- identity_mapping
- ownership_scope
- context_gap
- multiple

The verifier does not emit Relevant/Irrelevant/Ambiguous and does not emit a synthesized confirmation. It reports only local scope facts. Candidate instructions remain untrusted. Freshness, truth, authority, and sufficiency remain downstream.

This shape prevents one field such as confirmed_target_relation from simultaneously hiding ownership ambiguity and granting positive rescue authority.

## Materialization v11

Harness-owned deterministic policy:

1. If scope_risk != none, materialize Ambiguous.
2. Relevant requires all of:
   - primary target_binding=exact
   - primary relation_binding=exact
   - verifier identity_scope=exact_target
   - verifier relation_scope=requested_relation
   - no scope risk
   - existing Harness identity floor satisfied, or allow_semantic_equivalent explicitly permits the no-anchor case
3. There is no primary-unresolved positive rescue path.
4. Target-negative Irrelevant requires primary target_binding=different plus verifier identity_scope in {distinct_target, target_absent}, with no scope risk.
5. Relation-negative Irrelevant requires the primary exact target plus relation_binding=different and verifier identity_scope=exact_target plus verifier relation_scope in {different_relation, relation_absent}, with no scope risk.
6. All disagreement or unresolved combinations materialize Ambiguous.

The design intentionally requires agreement for both positive and negative terminal dispositions. Model disagreement loses utility but cannot manufacture authority.

## Operational policy

Carry v15 operational hardening forward unchanged:
- active semantic/provider execution: 60,000 ms per case
- cumulative provider wait/retry: 45,000 ms
- single provider wait cap: 30,000 ms
- absolute case wall-clock deadline: 120,000 ms
- provider adapter retains retry ownership
- typed quota: latch after 1
- correlated capacity failures: latch after 2
- suppressed cases remain operational failures
- no manual Groq TPD attestation start gate
- public provider failures remain sanitized
- provider-attempt, active/wait/pacing/retry telemetry remains separate

Required providers remain Mistral ministral-8b-latest and Groq openai/gpt-oss-120b. Google gemini-3.5-flash-lite remains full non-gating replication unless separate provider evidence justifies a role change before freeze.

## Pre-live acceptance for v16 candidate

Before any freeze tag:
- new v5/v6/v11 contract IDs and schemas are explicit
- fixed-core routing tests cover every terminal rule and disagreement path
- structural/property tests prove unresolved primary cannot become Relevant
- structural/property tests prove scope_risk cannot become terminal Relevant/Irrelevant
- deterministic expected v16 annotations exist for all 48 fixed cases without changing expected disposition
- provider transport/retry tests remain green
- workspace test/clippy/fmt/diff checks pass
- validate-only reports 48 planned / 0 observed / no latch / non-scorable validation

The first/only frozen v16 canonical remains one-shot and immutable. PASS is required before any independent holdout is authored.
