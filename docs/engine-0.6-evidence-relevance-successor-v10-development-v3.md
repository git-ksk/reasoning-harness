# Engine 0.6 evidence relevance successor-v10 development v3

Status: pre-observation fresh development candidate for Issue #468. No v3 provider observation has occurred.

## Motivation

Development v2 run 37100490212 is immutable FAIL. Google passed 18/18. Mistral had no authority or identity/risk failure, but v26 materialized 17/18 because the final materializer delegated to frozen v23 using the stale raw qualification rather than the corrected v14 effective qualification.

v3 keeps effective qualification v14 unchanged. Only final composition changes.

## Candidate materialization v27

v27:
1. derives the v14 effective local qualification from the original proposal and raw qualification;
2. delegates final relevance policy to frozen v23, but supplies the v14 effective qualification as the local qualification input;
3. relabels only the materialization policy ID to v27.

v27 adds no new semantic cue, no new model call, no requested-relation authority, and no direct Relevant/Irrelevant special case. Its purpose is only to make final materialization consume the already-authoritative v14 result instead of stale provider-local state.

## Historical replay

Before any v3 provider observation:

- frozen successor-v9, holdout-v10, and development-v2 expected contracts remain exact;
- all 78 canonical holdout-v10 observations replay under v27 with exactly one terminal change:
  - Groq v10h18: Ambiguous -> Irrelevant;
- all 36 immutable v2 observations replay under v27 with exactly one terminal change:
  - Mistral sv10v2_18: Irrelevant -> Relevant;
- the remaining 112 historical terminal dispositions are unchanged.

This is diagnostic replay, not a rescore of either immutable run.

## Fresh v3 development surface

fixtures/evidence-relevance-successor-v10-development-v3/manifest.json contains 20 newly authored cases:

- 8 require_different;
- 8 require_requested;
- 3 forbid_different;
- 1 preserve_risk.

The surface has zero case-ID, entity, task, exact-signal, or 8-token signal-window reuse against 60 previously observed cases from holdout-v10, development-v1, and development-v2.

The requested-relation controls deliberately include mixed-relation content such as availability plus feature support, limit plus pricing, pricing plus limit, and definition plus launch. These specifically exercise the v27 composition fix without reusing v2 wording.

## Live development gate

Required providers:
- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite

Groq remains excluded from candidate shaping and is reserved for the later fresh independent holdout.

Each provider observes all 20 cases once through the ordinary proposal + local-qualification path.

PASS requires:
- operational 20/20;
- provider/protocol failures 0;
- effective identity and scope risk exact;
- require_different -> different_relation;
- require_requested -> requested_relation;
- forbid_different -> not different_relation;
- preserve_risk retains expected risk;
- materialized disposition exact 20/20;
- wrong-target Relevant, false relevance rejection, relevant-left-Ambiguous, and utility misses all 0.

Raw proposal/local labels remain diagnostic.

## One-shot discipline

The first v3 observation is bound to annotated tag engine-0.6-evidence-relevance-successor-v10-development-v3-freeze. The tag is immutable and workflow reruns are rejected.

A fresh independent holdout is prohibited until v3 passes and successor semantics are separately frozen.
