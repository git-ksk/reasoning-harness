# Engine 0.6 evidence relevance independent holdout v11 result

Status: immutable canonical FAIL.

Freeze tag: engine-0.6-evidence-relevance-holdout-v11-freeze
Freeze commit: 4343e7939e964e91d466a0788358733d629e92f2
Canonical run: 37182677114, attempt 1 only
Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
Required providers: Mistral + Google + Groq
Effective qualification: v15
Materialization: v28
Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, or reinterpret the canonical run as PASS.

## Provider results

Mistral / ministral-8b-latest:
- operational 26/26, provider failures 0
- authority failures 0
- identity/risk failures 0
- effective qualification exact 26/26
- materialized exact 26/26
- wrong-target Relevant 0
- utility misses 0
- PASS

Google / gemini-3.5-flash-lite:
- operational 26/26, provider failures 0
- authority failures 0
- identity/risk failures 0
- effective qualification exact 25/26
- materialized exact 26/26
- wrong-target Relevant 0
- utility misses 0
- diagnostic-only v11h22 variation: expected unresolved, effective relation_absent
- PASS

Groq / openai/gpt-oss-120b:
- operational 26/26, provider failures 0
- identity/risk failures 0
- effective qualification exact 24/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- relevant-left-Ambiguous 0
- utility misses 1
- FAIL

## Terminal miss

Case: v11h21_ambiguous_generic_no_relation

Task: What monthly fee applies to Moss Gallery?
Candidate: Moss Gallery documentation covers account setup and navigation examples.

Frozen expectation:
- exact_target
- relation_absent
- scope risk none
- Ambiguous
- authority mode forbid_different

Groq produced an advisory negative relation result that survived v15 as DifferentRelation. v28 synchronized that effective state and materialized Irrelevant.

The candidate contains target-owned generic documentation but no affirmative proposition about a coarse relation different from pricing. Missing requested-relation evidence is not itself different-relation evidence. The frozen v11 expectation is therefore retained.

This is not a wrong-target Relevant or positive-authority safety failure. It is over-rejection caused by excessive negative relation authority.

A second diagnostic difference, v11h20_ambiguous_instruction_definition, remained final Ambiguous and is not a gate failure.

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- qualification gate: FAIL
- materialization gate: FAIL
- utility gate: FAIL
- overall: FAIL

Mistral and Google pass the frozen acceptance contract. Groq is the only failing provider and the only terminal miss is v11h21.

## Successor direction

Do not tune, rerun, rescore, or relabel v11.

The successor must generically distinguish:
1. an affirmative target-owned proposition that clearly expresses another coarse relation; from
2. a complete target-owned unit that merely lacks the requested relation or contains only generic documentation.

Model-only DifferentRelation must not become terminal negative authority in case (2). The correction must be one-sided: it may remove unsupported negative authority, but must not create RequestedRelation authority.

Historical v15/v28 behavior, known true different-relation families, instruction controls, strict absence, context risk, identity scope, and all immutable run records must remain replayable before any fresh successor holdout.

Issue #468 remains open and PR #469 remains Draft.
