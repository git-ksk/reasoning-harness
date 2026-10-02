# Engine 0.6 evidence relevance holdout successor v9

Status: pre-freeze successor after immutable holdout v8 FAIL and the non-accepted successor-v8/v10 development observation. Candidate semantics are effective qualification v11 plus materialization v23. Fresh holdout v9 authoring is prohibited until the successor-v9 semantics-freeze tag exists.

## Historical boundary

Canonical holdout v8 run 36946457925 attempt 1 is immutable FAIL at freeze commit 867b7a90b4911c09c1397e2d15642b3dc9895022 / tag engine-0.6-evidence-relevance-holdout-v8-freeze.

Mistral, Google, and Groq all completed 26/26 with provider failures 0. Materialization was 26/26 exact on all three providers, wrong-target Relevant was 0, false relevance rejection was 0, relevant-left-Ambiguous was 0, and utility miss was 0. The required qualification gate failed only because Mistral and Google each had one effective relation-scope miss on v8h15_negative_explicit_separate_service; Groq was exact.

The requested relation was availability. The fresh distinct-target proposition expressed deployment coverage through a semantic paraphrase rather than the historical bounded availability lexicon. Identity was already correct as distinct_target, so this successor owns the relation axis only. Historical v8 remains immutable FAIL and is not rerun, rescored, relabelled, or retagged.

## v10 development result

Effective qualification v10 introduced bounded positive semantic frames for Availability, Pricing, Limit, Change/Launch, Definition, and Benefit/Use-case while preserving identity and scope-risk semantics.

Fresh successor-v8 development run 36978705359 at candidate 706902c9e7ccb6213b8f021c945159ffe86bff41 remains FAIL. Mistral completed 14/14 with successor-owned relation authority exact. Google completed 14/14 but retained three relation-authority misses on non-frame controls. The observation showed that positive frame recall alone was insufficient because historical model-only RequestedRelation fallback could still manufacture relation authority when Harness-owned relation evidence was absent.

The run is captured under fixtures/evidence-relevance-successor-v8-development/ and remains development evidence only.

## v11 selective relation authority

v11 layers on v10 without changing frozen v9/v22.

Harness-owned positive requested-relation authority requires either a requested coarse-relation semantic frame in substantive local material, or a historical lexical requested-relation cue with no conflicting coarse semantic frame. If relation scope would otherwise become requested_relation only from advisory model output, v11 changes only that axis to unresolved. Deterministic different_relation / relation_absent authority, identity scope, and scope-risk are preserved.

The frame detector remains compositional and relation-specific. Controls distinguish deployment coverage from feature support, commercial pricing from incidental currency/number mentions, hard limits from observed counts, rollout events from unrelated phrasal uses, definitions from documentation references, and benefits/use-cases from topical mentions. Prompt-injection/control text is inert for factual relation authority. URL/navigation/source-title text remains outside the semantic-frame path.

## Materialization v23

v23 derives v11 effective qualification before delegating to frozen v22 behavior. If historical v22 would materialize Relevant while v11 has no Harness-owned requested-relation authority, v23 forces conservative Ambiguous. This is a positive safety floor only; it does not broaden negative authority or create truth, provenance, source authority, or target identity.

## Replay and controls

The successor-v9 surface covers all six coarse relation frames, lookalike/non-frame controls, availability vs general-availability/change conflicts, model-only requested-relation agreement, model disagreement, exact-target lexical history, mapping/context risks, prompt injection, and identity/relation orthogonality.

Historical replay requires v9 identity scope unchanged, v9 scope-risk unchanged, expected requested-relation regression 0, false requested-relation authority 0, and frozen materialization safety retained.

Canonical holdout-v8 replay under v11 has zero relation-scope misses. The only v9 to v11 changes are Mistral and Google on the original v8h15 boundary, both moving to the expected requested_relation. Groq is unchanged. Frozen v22 materialization stays exact and wrong-target Relevant stays 0. The failed v10 development observation is also replayed; v11 removes the three Google false-positive relation-authority cases without provider-specific branches.

## Fresh successor-v9 development

The fresh successor-v9 development surface was authored before any v11 provider observation. It contains 19 cases: 8 require_requested, 9 forbid_requested, and 2 scope-risk preservation cases. CI enforces no reuse of case IDs, tasks, canonical entities, or exact 8-token surface windows against prior evidence-relevance holdouts and successor development manifests.

Live run 37015407859 at candidate 78e635e1aea485e3f13fe78e01fa3a7f5ddd5e90 completed both iterative providers.

- Mistral ministral-8b-latest: 19/19 completed, provider failures 0, authority failures 0, identity/risk failures 0, materialization failures 0.
- Google gemini-3.5-flash-lite: 19/19 completed, provider failures 0, authority failures 0, identity/risk failures 0, materialization failures 0.
- Both: wrong-target Relevant 0, false relevance rejection 0, relevant-left-Ambiguous 0, utility miss 0.
- Final two-provider selective-authority development gate: PASS.

Full effective-qualification exactness is diagnostic when the contract intentionally permits conservative abstention. Mistral was 19/19 exact; Google was 14/19 exact with five conservative relation-scope differences. None violated a required/forbidden authority mode and all 19 materialized dispositions were exact.

Live observations and the final summary are captured under fixtures/evidence-relevance-successor-v9-development/ and replayed offline under unchanged v11/v23.

## Freeze blockers

Before semantics freeze: relation controls, historical replay, captured live replay, surface independence, immutable holdout-v8 replay, wrong-target Relevant 0, production special-case scan, frozen v9/v22 invariance, orthogonality and false-positive controls, full workspace tests, all-target Clippy -D warnings, rustfmt, workflow YAML parse, diff check, and exact-head GitHub CI must all be green.

## Pre-freeze audit

At capture commit 842b6f4b07fc99c3ee0add823f279df0b295eda8:

- full workspace tests PASS;
- all-target Clippy -D warnings PASS;
- rustfmt / workflow YAML parse / diff check PASS;
- production special-case scan finds 0 successor-v9 case/entity literals in the semantic source/exports;
- frozen derive_effective_evidence_local_qualification_v9 is byte-for-byte unchanged from the successor-v7 semantics freeze;
- frozen materialize_evidence_relevance_v22 is byte-for-byte unchanged from the successor-v7 semantics freeze;
- exact-head GitHub CI is 8/8 green at the capture commit.

Intended freeze coordinate: engine-0.6-evidence-relevance-successor-v9-semantics-freeze.

Only after that tag is pushed may a fresh independent holdout v9 runner/corpus be authored. The holdout must restore Mistral + Google + Groq as the one-shot cross-provider acceptance surface and must not reuse the successor-v9 development corpus.
