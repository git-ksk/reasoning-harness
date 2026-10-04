# Engine 0.6 evidence relevance successor-v11 development v1 result

Status: immutable development FAIL.

Freeze tag: engine-0.6-evidence-relevance-successor-v11-development-v1-freeze
Freeze commit: b97bf46f5e0badf434e5fccd8e3581fc8e77450f
Run: 37212349800, attempt 1 only
Candidate: effective qualification v16 / materialization v29
Surface: 22 fresh independent development cases
Providers: Mistral + Google
Holdout acceptance evidence: false
Result: FAIL

This run is immutable. Do not rerun, rescore, relabel, move or recreate the freeze tag, or reinterpret the run as PASS.

## What passed

Both providers completed 22/22 with provider failures 0.

Mistral: authority failures 0, materialization failures 0, materialized exact 22/22, wrong-target Relevant 0, utility misses 0, total tokens 37,659.

Google: authority failures 0, materialization failures 0, materialized exact 22/22, wrong-target Relevant 0, utility misses 0, total tokens 38,531.

The original holdout-v11 defect is repaired on this development surface: generic/no-relation controls do not gain DifferentRelation authority, while true alternate-relation families remain recoverable.

## Gate failure

Both providers produced the same identity-axis mismatch on sv11d_19_risk_clipped_column.

Task: How is Topaz Grid priced?

Candidate: Topaz Grid pricing row is clipped before the product column, so ownership is not shown.

Frozen expectation:
- identity_scope: unresolved
- relation_scope: unresolved
- scope_risk: context_gap
- final: Ambiguous

Observed effective state for both Mistral and Google:
- identity_scope: exact_target
- relation_scope: requested_relation
- scope_risk: context_gap
- final: Ambiguous

Final materialization is safely Ambiguous because the context-gap floor is preserved. The development gate fails because explicit omitted product-column / ownership-not-shown context must keep local target ownership unresolved under the frozen scope contract.

This is not provider variance. Both required development providers exposed the same Harness-owned identity boundary.

## Adjudication

The frozen expectation is retained.

Existing Engine 0.6 scope semantics treat omitted product columns, unresolved ownership, and visibly clipped identity context as unresolved local identity / blocking context. A literal canonical-name occurrence in a sentence describing a clipped row must not by itself establish ownership of the unseen row.

The failure is orthogonal to the v16 negative-relation change. v16/v29 succeeds at the original generic-no-relation objective, but the fresh development surface exposes a pre-existing ownership-authority gap.

## Next direction

Do not tune or rewrite development-v1.

Successor-v2 should preserve v16 symmetric negative-authority behavior and add a narrow Harness-owned identity floor for explicit omitted product-column / ownership-not-shown local context. It may remove unsupported ExactTarget authority but must not create DistinctTarget authority. Requested-relation classification remains independent when relation kind is locally visible; context-gap and final Ambiguous remain preserved.

Replay immutable v1 and recent historical provider observations, then use a new fresh independent development-v2 surface with no reuse from observed v1.

Groq remains excluded from candidate shaping and may only re-enter after successor semantics freeze on a fresh independent holdout.

Issue #468 remains OPEN. PR #469 remains Draft.
