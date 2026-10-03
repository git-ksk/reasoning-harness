# Engine 0.6 evidence relevance successor-v10 development v4

Status: pre-observation fresh development candidate for Issue #468. No v4 provider observation has occurred.

## Motivation

Canonical holdout-v10 remains immutable FAIL under frozen v11/v23. Successor-v10 development v1, v2, and v3 are also immutable FAIL development evidence.

v3 run 37105677785 established two remaining gaps:
- direct affirmative definition wording such as "is defined as" was not recognized as another relation for a launch query;
- advisory model-only DifferentRelation from instruction/control text could survive the inherited v11 negative path and become terminal Irrelevant once v27 correctly composed effective semantics.

v4 does not rewrite v14 or v27. It introduces a new effective qualification v15 and materialization v28.

## v15 negative-authority boundary

v15 derives from v14.

First, model-only DifferentRelation is demoted to Unresolved when:
- identity is exact_target;
- deterministic scope risk is none;
- the candidate contains untrusted instruction/control text;
- there is no independent Harness-owned negative-relation cue in a clean factual segment;
- there is no requested-relation Harness authority;
- there is no strict target/relation absence rule.

Instruction/control text is therefore inert. A separate clean factual segment may still carry bounded negative-relation evidence.

v15 keeps the frozen v14 marker vocabulary unchanged and adds a successor-only control-schema detector for model-facing labels/fields such as `relation_binding`, `relation_scope`, `final disposition`, and imperative outcome labels. This closes the case where control text can request `different_relation` / `irrelevant` without using the older natural-language injection phrases. The detector is deliberately one-sided: it can only remove model-only negative authority; a separate clean Harness-owned factual cue still survives.

Second, v15 adds a successor-only direct Definition cue. A target-owned substantive segment containing "is defined as" / "defined as" can establish Definition as another relation when the requested relation is not Definition and the ordinary v14 safety restrictions hold.

This does not modify the frozen global semantic frames used by v10/v11/v14.

## v28 materialization

v28 composes v15 through frozen v23.

Unlike v27, v28 synchronizes both advisory proposal bindings and the local qualification to the Harness-owned effective v15 semantics before delegating to v23. This prevents a stale model proposal from recreating a relation or identity decision that v15 has already corrected.

v28 cannot invent RequestedRelation or DifferentRelation on its own; it materializes the v15 effective state.

## Historical replay

Before any v4 provider observation, v28 must preserve all precommitted expected contracts and replay observed history.

Current deterministic replay covers:
- canonical holdout-v10 observations;
- immutable v2 observations;
- immutable v3 Mistral observations;
- the available cancelled-v3 Google partial observations;
- frozen successor-v9 / holdout-v10 / v2 / v3 expected contracts.

The intended terminal changes are bounded to previously adjudicated misses:
- canonical v10 Groq v10h18: Ambiguous -> Irrelevant;
- immutable v2 Mistral sv10v2_18: Irrelevant -> Relevant;
- immutable v3 Mistral sv10v3_08: Ambiguous -> Irrelevant;
- immutable v3 Mistral sv10v3_20: Irrelevant -> Ambiguous;
- available v3 Google partial direct-definition miss is corrected without unrelated terminal changes.

## Fresh v4 development surface

fixtures/evidence-relevance-successor-v10-development-v4/manifest.json contains 24 newly authored cases:
- require_different: 11
- require_requested: 9
- forbid_different: 3
- preserve_risk: 1

The surface has zero case ID, entity, task, exact signal, or 8-token signal-window reuse against the 80 previously observed holdout-v10 / development-v1 / development-v2 / development-v3 cases.

Fresh recovery families include:
- direct Definition wording against launch/pricing/availability/benefit queries;
- limit vs throughput/latency observation;
- availability vs feature support;
- pricing vs limit;
- definition vs launch;
- benefit vs pricing.

Controls include:
- true requested-relation evidence;
- a clean negative factual segment plus a separate instruction segment;
- requested-relation evidence plus a separate instruction segment;
- requested relation coexisting with another relation;
- numeric and definition-shaped prompt injection;
- generic no-relation content;
- clipped direct-definition context.

## Strengthened offline gate

Before provider observation:
- all v28 unit controls pass;
- v28 historical replay passes;
- v4 expected-path test passes;
- every require_different v4 case is forced through conservative model outputs and must still recover DifferentRelation / Irrelevant from Harness cues;
- prompt-injection controls are forced through adversarial model DifferentRelation votes and must remain Ambiguous;
- context-gap controls cannot be recovered to DifferentRelation;
- control-schema injection, comparison-only target mentions, distinct-target Definition text, and strict requested-relation absence cannot create exact-target negative relation authority;
- validate-only must report 24 planned / 0 completed with zero model/provider calls;
- the v4 workflow gates on v4 configuration/suite identifiers (not the predecessor v3 identifiers);
- frozen successor-v9, holdout-v10, v1, v2, and v3 tests remain green;
- fmt / Clippy / checksum / validate-only are green.

## Live development gate

Required development providers:
- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite

Groq remains excluded from candidate shaping and is reserved for a later fresh independent holdout.

Each provider observes all 24 cases once through the normal proposal + local-qualification path.

A provider passes only with:
- 24/24 operational completion;
- provider/protocol failures 0;
- exact identity and scope-risk contract;
- require_different -> different_relation;
- require_requested -> requested_relation;
- forbid_different -> not different_relation;
- preserve_risk -> expected risk preserved;
- final v28 disposition exact 24/24;
- wrong-target Relevant, false relevance rejection, relevant-left-Ambiguous, and utility misses all 0.

Raw proposal/local labels remain diagnostic.

## One-shot discipline

The first v4 provider observation is bound to annotated tag:

engine-0.6-evidence-relevance-successor-v10-development-v4-freeze

The tag is immutable and workflow reruns are rejected. A failed v4 observation remains FAIL and is not rescored or relabeled.

A fresh independent acceptance holdout may be authored only after v4 development PASS and a separate successor-semantics freeze.
