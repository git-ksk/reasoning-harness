# Engine 0.6 evidence relevance successor-v10 development v2

Status: pre-observation fresh development candidate for Issue #468. No v2 provider observation has occurred.

## Motivation

Canonical holdout-v10 run 37085574010 is immutable FAIL under frozen effective qualification v11 and materialization v23. Its only terminal semantic miss was Groq case v10h18: an exact-target telemetry observation was left Ambiguous because the provider returned relation_absent instead of different_relation.

Successor-v10 development v1 tested a two-key design with a dedicated one-sided model verifier. Run 37097092197 is immutable FAIL: Mistral missed every expected positive verifier family and falsely confirmed the prompt-injection control. Google and Groq were cancelled after the all-provider gate became impossible.

v2 removes that failed model verifier entirely. The candidate uses only Harness-owned bounded local structure on top of the unchanged v11/v23 baseline.

## Candidate semantics

Candidate effective qualification is v14. Candidate materialization is v26.

v14 starts from v11 and may convert only relation_absent or unresolved to different_relation. Every condition below is required:

1. proposal and local qualification are both present;
2. deterministic scope risk is none;
3. effective identity is exact_target;
4. a Harness-owned exact target anchor exists;
5. the current relation is non-positive;
6. the candidate has no untrusted-instruction marker relevant to this deterministic path;
7. the other-relation cue is in a substantive segment where the Harness target name or alias is locally owning, not comparison/context-only;
8. the bounded cue affirmatively expresses another relation:
   - an existing conflicting coarse semantic frame,
   - availability query + feature/API/protocol support that is not deployment/geography support, or
   - limit/quota query + numeric benchmark/telemetry observation that is not a limit frame;
9. no requested-relation lexical cue is locally present;
10. no Harness-owned requested-relation authority is present; and
11. no strict target/relation absence rule applies.

The lexical exclusion is deliberate. Cross-frame wording such as "became generally available" can simultaneously resemble launch/change and availability. v14 preserves those cases as unresolved instead of manufacturing negative authority.

v26 delegates first to frozen v23. It changes only a v23 Ambiguous result to Irrelevant when v14 establishes exact_target / different_relation / none. It cannot promote anything to Relevant.

v13/v25 remain historical development-v1 semantics and are not rewritten.

## Fresh v2 development surface

fixtures/evidence-relevance-successor-v10-development-v2/manifest.json contains 18 newly authored cases:

- 8 require_different;
- 6 require_requested;
- 3 forbid_different;
- 1 preserve_risk.

The surface is independent of both canonical holdout-v10 and observed successor-v10 development v1. Pre-observation checks require zero case ID, entity, task, exact signal, and 8-token signal-window reuse against those 42 prior observed cases.

Recovery families include exact-target limit vs throughput/latency observations, availability vs feature/API support, pricing vs limit, definition vs launch, benefit/use-case vs pricing, and launch vs definition.

Controls include true requested-relation evidence, generic or explicit absence, context-gap telemetry, prompt injection with benchmark-looking text, and requested-relation content coexisting with another relation.

## Historical replay requirements

Before provider observation:

- frozen successor-v9 development expected contracts remain unchanged under v14/v26;
- frozen holdout-v10 expected contracts remain unchanged under v14/v26;
- replay of all 78 canonical holdout-v10 provider observations changes exactly one terminal disposition: Groq v10h18 Ambiguous -> Irrelevant;
- Mistral and Google canonical v10 dispositions remain unchanged;
- v10h25 cross-frame "became generally available" remains Ambiguous;
- comparison-only target mentions and target-title/other-entity-body layouts cannot create target-local negative relation authority.

Historical replay is diagnostic development evidence, never a rescore of holdout-v10.

## Live development gate

Required development providers are Mistral ministral-8b-latest and Google gemini-3.5-flash-lite.

Groq is deliberately excluded from candidate shaping and reserved for a later fresh independent holdout after successor semantics freeze.

Each provider observes all 18 cases once through the normal two-stage proposal + local-qualification path. No dedicated negative-relation model call is made.

A provider passes only if:

- 18/18 cases complete operationally;
- provider/protocol failures are 0;
- effective identity and scope risk match the precommitted contract;
- require_different cases end as different_relation;
- require_requested cases end as requested_relation;
- forbid_different cases do not end as different_relation;
- preserve_risk cases retain expected deterministic risk;
- final v26 disposition is exact on 18/18;
- wrong-target Relevant, false relevance rejection, relevant-left-Ambiguous, and utility misses are all 0.

Raw proposal and raw local-qualification exact labels remain diagnostic. Acceptance is defined by Harness-owned effective semantics and final materialization.

## One-shot discipline

The first v2 provider observation is bound to annotated tag engine-0.6-evidence-relevance-successor-v10-development-v2-freeze.

The tag is immutable and workflow reruns are rejected. A failed v2 development observation remains FAIL and is not rescored or relabeled.

A new independent holdout may be authored only after v2 development PASS and a separate successor-semantics freeze.
