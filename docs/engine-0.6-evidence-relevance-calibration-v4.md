# Engine 0.6 candidate: evidence-target relevance calibration v4

Status: fresh unobserved successor after frozen v3. v1-v3 remain immutable.

## Why v4 exists

Frozen v3 run `35996093336` separated two remaining problems:

1. Mistral produced `target=unresolved` and `relation=different` for an uncertain rename case, and the v3 materializer incorrectly let relation-level `different` destroy unresolved target identity;
2. Google completed only 9/26 cases because the Harness-owned 15-second assessment deadline expired on 17 cases before a completed provider attempt was recorded.

## v4 semantic change

The binding proposal contract remains `reason-evidence-relevance-binding-proposal-v2`.

The final Harness-owned materialization policy advances to `target-evidence-relevance-binding-materialization-v3` with hierarchical precedence:

1. target `different` => `irrelevant`;
2. target `unresolved` => `ambiguous`;
3. target `exact` + relation `different` => `irrelevant`;
4. target `exact` + relation `unresolved` => `ambiguous`;
5. target `exact` + relation `exact` => `relevant`, subject to the unchanged deterministic identity floor.

This prevents relation-level rejection from overriding unresolved target identity.

## v4 operational change

The explicit assessment elapsed budget increases from 15,000 ms to 30,000 ms. The model-call budget remains 2 and max output remains 192 tokens. The assessment is still bounded and fails closed on timeout.

This is not a retry of v3. It is a new frozen evaluation identity with a changed Harness-owned operational budget.

## Fresh v4 corpus

- suite: `evidence-relevance-calibration-v4`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: excluded from tuning

The semantic case set remains comparable to v3. Expected model bindings remain unchanged; only the materializer precedence and explicit elapsed budget differ.

## Acceptance

Both canonical Mistral and Google arms must be operationally complete with:

- wrong-target / unresolved-binding materialized as `relevant`: **0**;
- utility misses: **0**;
- provider failures: **0**.

Proposal exact accuracy remains diagnostic.

No independent holdout is authored until v4 passes and #462 semantics are frozen.
