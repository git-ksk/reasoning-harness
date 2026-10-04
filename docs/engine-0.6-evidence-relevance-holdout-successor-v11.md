# Engine 0.6 evidence relevance holdout successor v11

Status: pre-freeze successor after immutable holdout-v11 FAIL, development-v1 FAIL, and development-v2 PASS. Candidate semantics are effective qualification v17 plus materialization v30. Fresh acceptance-holdout authoring is prohibited until the successor-v11 semantics-freeze tag exists.

## Historical boundary

Canonical holdout-v11 run 37182677114 remains immutable FAIL at `engine-0.6-evidence-relevance-holdout-v11-freeze` / `4343e7939e964e91d466a0788358733d629e92f2`. Mistral and Google passed; Groq alone missed `v11h21_ambiguous_generic_no_relation`, hardening generic target-owned documentation with no classifiable alternate relation from relation_absent/Ambiguous to different_relation/Irrelevant. No rerun, rescore, relabel, or tag movement is permitted.

Development-v1 run 37212349800 remains immutable FAIL at `engine-0.6-evidence-relevance-successor-v11-development-v1-freeze` / `b97bf46f5e0badf434e5fccd8e3581fc8e77450f`. It repaired the original negative-relation defect but exposed a Harness-owned identity boundary on explicit omitted ownership.

Development-v2 run 37215152913 is immutable PASS at `engine-0.6-evidence-relevance-successor-v11-development-v2-freeze` / `db38a7eb525a07208bfeb467cfa4d9d8a8a59f51`, recorded by `1bc44a16c601d285a8e383c9ad4b66da8c4059df`. It is development evidence only; `holdout_acceptance_evidence=false`.

## Frozen candidate semantics

Effective qualification v17 preserves v16 symmetric negative-authority hardening. Model output alone cannot manufacture DifferentRelation for an exact target without Harness-owned affirmative alternate-relation evidence.

v17 additionally applies a one-sided ownership/identity floor only when an already typed ContextGap or Multiple risk coincides with explicit local evidence that the owner/product column, ownership, row owner, or referent is omitted, clipped, missing, hidden, or otherwise unavailable. In that bounded state ExactTarget becomes Unresolved. The rule cannot create DistinctTarget or TargetAbsent, cannot clear a blocking scope risk, and does not manufacture relation authority.

Materialization v30 preserves the v29/v23 composition while synchronizing advisory proposal target/relation bindings to the v17 effective state. A stale model Exact binding therefore cannot recreate target ownership removed by the v17 floor.

## Development-v2 evidence

The fresh 22-case development-v2 surface was authored before provider observation and excludes case ID, canonical entity, task, exact-signal, and 8-token-window reuse against the predeclared historical surfaces.

One-shot run 37215152913 used Mistral `ministral-8b-latest` and Google `gemini-3.5-flash-lite` only. Both completed 22/22 with provider, authority, identity/risk, and materialization failures 0; final materialization was exact 22/22 for both; wrong-target Relevant and utility misses were 0. Groq was not observed and remains reserved for fresh independent acceptance.

The v1 omitted-ownership defect is repaired without broad identity demotion on paired relation-only context-gap controls. Conservative relation-label differences remain non-terminal and satisfy the frozen authority contract.

## Pre-freeze audit

At development-evidence commit `1bc44a16c601d285a8e383c9ad4b66da8c4059df`:
- exact-head GitHub CI is 8/8 PASS;
- branch HEAD matches origin and the working tree is clean;
- the development-v2 freeze tag still dereferences to `db38a7eb525a07208bfeb467cfa4d9d8a8a59f51`;
- immutable holdout-v11 and development-v1 freeze coordinates remain unchanged;
- recorded Google and summary artifacts are byte-identical to canonical run 37215152913 artifacts;
- the original development-v2 surface checksum still validates all candidate files at the observed freeze commit; roadmap entries changed only when immutable PASS evidence was recorded.

The semantics-freeze commit may add only freeze evidence/documentation. It must not change implementation code, tests, runners, workflows, or any observed development surface.

Intended freeze coordinate: `engine-0.6-evidence-relevance-successor-v11-semantics-freeze`.

Only after that annotated tag is pushed may a dedicated next-holdout runner be prepared and frozen. Only after runner freeze may a fresh independent acceptance corpus be authored. Acceptance must restore Mistral + Google + Groq as required providers and must not reuse observed development cases.
