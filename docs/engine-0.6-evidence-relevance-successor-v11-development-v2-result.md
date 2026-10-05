# Engine 0.6 evidence relevance successor v11 development v2 result

Status: immutable development PASS.

Freeze tag: engine-0.6-evidence-relevance-successor-v11-development-v2-freeze
Freeze commit: db38a7eb525a07208bfeb467cfa4d9d8a8a59f51
Run: 37215152913, attempt 1 only
Candidate: effective qualification v17 / materialization v30
Surface: 22 fresh independent development cases
Providers: Mistral + Google
Holdout acceptance evidence: false
Result: PASS

This run is immutable. Do not rerun, rescore, relabel, move or recreate the freeze tag, or reinterpret the run as holdout acceptance evidence.

## Provider results

Mistral / ministral-8b-latest:
- operational: 22/22, provider failures 0
- authority failures 0
- identity/risk failures 0
- materialization failures 0
- proposal exact: 7/22
- raw local qualification exact: 5/22
- effective local qualification exact: 13/22
- effective identity misses: 0
- effective relation-label differences: 9
- materialized exact: 22/22
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant-left-Ambiguous: 0
- utility misses: 0
- total tokens: 38,054
- PASS

Google / gemini-3.5-flash-lite:
- operational: 22/22, provider failures 0
- authority failures 0
- identity/risk failures 0
- materialization failures 0
- proposal exact: 11/22
- raw local qualification exact: 9/22
- effective local qualification exact: 15/22
- effective identity misses: 0
- effective relation-label differences: 7
- materialized exact: 22/22
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant-left-Ambiguous: 0
- utility misses: 0
- total tokens: 39,135
- PASS

## Identity objective

The development-v1 failure is repaired.

All six fresh explicit omitted-ownership controls preserve the frozen identity/risk contract under the v17 Harness-owned floor. Neither provider retained unsupported ExactTarget authority where the owner/product column, ownership field, row owner, or referent was explicitly unavailable.

The paired relation-only context-gap controls do not suffer broad identity demotion. Effective identity misses are zero for both required providers across all 22 cases.

## Relation diagnostics

Some context-gap and generic/no-relation cases remain conservatively Unresolved on the relation axis instead of the precommitted RequestedRelation or RelationAbsent label.

These differences are non-terminal and allowed by the frozen development gate:
- preserve_risk cases remain fail-closed through the typed scope blocker;
- forbid_different controls do not gain DifferentRelation authority;
- all require_different / require_requested / preserve_absence authority gates pass;
- final materialization is exact 22/22 for both providers.

The original holdout-v11 generic-no-relation defect remains repaired while the v2 ownership correction introduces no correctness or utility miss.

## Final development gate

- required provider completeness: PASS
- authority gate: PASS
- identity/risk gate: PASS
- materialization gate: PASS
- correctness gate: PASS
- utility gate: PASS
- overall development gate: PASS

## Next boundary

Do not author a fresh acceptance holdout from this observed development surface yet.

First freeze the observed v17/v30 semantics separately. The semantics-freeze commit may add freeze documentation/checksums only and must not alter the candidate semantic implementation or its observed development surface.

After the semantics freeze is tagged, prepare and freeze a dedicated next holdout runner before authoring a new fresh independent acceptance corpus. Groq remains reserved for that fresh acceptance holdout.

Issue #468 remains OPEN. PR #469 remains Draft.
