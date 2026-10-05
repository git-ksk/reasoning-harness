# Engine 0.6 evidence relevance successor-v11 development v2

Status: pre-observation fresh development candidate after immutable successor-v11 development-v1 FAIL.

Historical boundary:
- holdout-v11 canonical FAIL: run 37182677114 at 4343e7939e964e91d466a0788358733d629e92f2
- development-v1 freeze: engine-0.6-evidence-relevance-successor-v11-development-v1-freeze / b97bf46f5e0badf434e5fccd8e3581fc8e77450f
- development-v1 run: 37212349800, immutable FAIL
- development-v1 result record: 861730fc0e46b96106e5001bfe9f546e03cd1525
- no rerun, rescore, relabel, or freeze-tag movement

## Why v2 exists

Development-v1 repaired the original holdout-v11 generic-no-relation defect: both Mistral and Google completed 22/22 with authority failures 0, materialization exact 22/22, wrong-target Relevant 0, and utility misses 0.

Both providers independently exposed one identical identity-axis miss on `sv11d_19_risk_clipped_column`. The candidate named the Harness target, but explicitly said the product column was clipped and ownership was not shown. Effective v16 retained `exact_target`; the frozen local-scope contract required `unresolved`. The blocking `context_gap` and final Ambiguous disposition were preserved.

The v2 change is orthogonal to the v16 relation-authority correction.

## Candidate semantics

Effective qualification v17 derives from v16 unchanged except for one narrow Harness-owned identity floor.

When effective identity is `exact_target`, scope risk is `context_gap` or `multiple`, and the bounded candidate explicitly says an ownership-bearing product or owner column, ownership field, row owner, or referent is omitted, clipped, outside the supplied unit, missing, or not shown, v17 demotes only identity from `exact_target` to `unresolved`.

The rule is one-sided. It cannot create `distinct_target` or `target_absent`, cannot clear or weaken scope risk, does not change the independent relation axis, and does not create requested- or different-relation authority.

Ordinary relation-only truncation remains `exact_target` when target ownership is locally visible. Historical replay explicitly checks this boundary.

Materialization v30 preserves v29 full composition and synchronizes proposal identity/relation bindings to the v17 effective state before delegating to frozen v23. A stale advisory Exact target binding cannot recreate target ownership removed by the v17 floor.

## Immutable replay

The v17/v30 replay against development-v1 observations changes effective identity only for:
- Mistral `sv11d_19_risk_clipped_column`: `exact_target -> unresolved`
- Google `sv11d_19_risk_clipped_column`: `exact_target -> unresolved`

Relation scope and scope risk are unchanged for those observations. No development-v1 terminal disposition changes; both remain Ambiguous.

Recent immutable holdout-v10, successor-v10 development v2/v3/v4, holdout-v11, and successor-v11 development-v1 provider observations replay to their frozen terminal contracts under v30.

## Fresh development-v2 surface

Suite: `evidence-relevance-successor-v11-development-v2`
Cases: 22
Protocol: effective qualification v17 + materialization v30
Fixed core: `evidence-relevance-fixed-core-successor-v11-development-v2`

Distribution:
- preserve_risk: 10, consisting of six explicit omitted-ownership identity gaps and four relation-only context gaps that must retain exact identity
- forbid_different: 4
- require_different: 4
- require_requested: 3
- preserve_absence: 1
- final Ambiguous: 14
- final Irrelevant: 6
- final Relevant: 2

Freshness is checked against observed holdout-v1-v11 and successor-v5-v11 development-v1 surfaces by case ID, canonical entity, task, exact signal, and 8-token signal window.

Property controls require deterministic risk to match every preserve-risk expectation, ownership gaps to demote adversarial ExactTarget votes, relation-only context gaps to retain ExactTarget, generic controls to resist model-only DifferentRelation, and true alternate relations to recover DifferentRelation under conservative model output.

## Provider policy

Development-v2 observation is Mistral + Google only. Groq remains excluded from candidate shaping and may re-enter only after successor semantics are separately frozen and a new fresh independent holdout is authored.

Intended development-v2 freeze coordinate: `engine-0.6-evidence-relevance-successor-v11-development-v2-freeze`.

No provider observation is permitted before exact-head CI passes and the annotated development-v2 freeze tag is pushed once.
