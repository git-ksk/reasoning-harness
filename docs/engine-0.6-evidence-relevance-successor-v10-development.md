# Engine 0.6 evidence relevance successor-v10 development

Status: pre-observation fresh development candidate for Issue #468. No successor-v10 provider observation has occurred yet.

## Motivation

Canonical holdout-v10 run 37085574010 is immutable FAIL under frozen effective qualification v11 and materialization v23. All required providers completed operationally and retained the safety boundary, but Groq left one exact-target non-frame negative relation case Ambiguous rather than Irrelevant.

The successor must not relabel, rerun, rescore, or tune the observed holdout-v10 surface.

A previous post-v9 local experiment referred to as v12/v24 made negative relation authority broadly symmetric and regressed historical controls. That experiment remains rejected and uncommitted. This new bounded candidate therefore uses effective qualification v13 / materialization v25; v12/v24 are intentionally not reused.

## Candidate boundary

The successor is one-sided and restrictive.

A dedicated negative-relation verifier returns only:

- confirmed_different_relation
- not_confirmed

The verifier cannot create requested-relation authority, target identity, truth, freshness, verification, sufficiency, or final relevance.

v13 may change an existing v11 relation_absent or unresolved result to different_relation only when all of the following are true:

1. proposal and local qualification are both present;
2. deterministic scope risk is none;
3. effective identity is exact_target;
4. a Harness-owned exact target anchor is present;
5. the current relation is non-positive;
6. a bounded Harness-owned observable cue says another relation is affirmatively present;
7. no Harness-owned requested-relation authority is present;
8. there is no strict target/relation absence condition; and
9. the one-sided verifier returns confirmed_different_relation.

The bounded observable cue is deliberately narrower than general semantic classification:

- an already-supported conflicting coarse semantic frame;
- availability queries with clear feature/API/protocol-support content that is not deployment/geography support;
- limit/quota queries with numeric benchmark/telemetry observations that are not cap/maximum/quota frames.

v25 only changes a v23 Ambiguous result to Irrelevant when the v13 result is exact_target / different_relation / none and the one-sided confirmation is present.

It cannot promote anything to Relevant.

## Fresh development surface

fixtures/evidence-relevance-successor-v10-development/manifest.json contains 16 independently authored cases:

- 8 expected confirmed_different_relation;
- 8 expected not_confirmed.

The positive side spans limit-vs-observed telemetry, availability-vs-feature support, availability-vs-pricing, pricing-vs-limit, definition-vs-launch, benefit-vs-pricing, and launch-vs-definition.

The negative controls include true requested-relation content, generic no-relation content, explicit requested-relation absence, clipped context, prompt injection, omitted ownership/context, and same-relation controls.

Automated pre-observation checks require zero holdout-v10 case/entity/task/signal/8-token surface reuse.

## Pre-observation deterministic checks

Before any provider observation:

- core v25 unit controls must pass;
- all frozen successor-v9 relation controls and historical replay must remain green;
- holdout-v10 frozen expectation tests must remain green;
- fresh successor-v10 composition must be monotone:
  - every expected confirmation turns the synthetic v23 Ambiguous baseline into v25 Irrelevant;
  - every expected not_confirmed case preserves the v23 disposition;
- the probe runner must validate the 16-case manifest without provider calls.

## Live development gate

Required providers:

- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite
- Groq openai/gpt-oss-120b

Each provider runs 3 matched trials over all 16 cases: 48 observations per provider.

A provider passes only with:

- 48/48 successful observations;
- confirmation matches 48/48;
- false confirmations 0;
- missed confirmations 0;
- provider/protocol failures 0.

All three required providers must pass before candidate semantics can be frozen. A new independent holdout is prohibited until after that freeze.

The first provider observation is bound to annotated tag engine-0.6-evidence-relevance-successor-v10-development-v1-freeze; its tag push starts the one-shot workflow and workflow reruns are rejected.

## Historical integrity

- holdout-v10 remains immutable FAIL;
- frozen v11/v23 behavior is retained and replayed unchanged;
- the rejected broad v12/v24 symmetry experiment remains rejected;
- this development surface is not a replacement holdout.
