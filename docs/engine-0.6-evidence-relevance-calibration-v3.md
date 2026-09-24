# Engine 0.6 candidate: evidence-target relevance calibration v3

Status: fresh unobserved successor authored after frozen v2. v1/v2 remain immutable and are not rerun or rescored.

## Why v3 exists

Frozen v2 run `35992854291` reached zero correctness violations on both Mistral and Google, but utility still failed because a small model sometimes collapsed unresolved binding into destructive `irrelevant`, while Google conservatively retained one generic page as `ambiguous`.

The v2 result shows that asking the model to own the final three-way policy disposition is too broad. v3 therefore narrows model authority instead of continuing prose-only tuning.

## Binding proposal contract

The model no longer proposes final `relevant / irrelevant / ambiguous`.

It returns only two advisory bindings:

- `target_binding`: `exact | different | unresolved`;
- `relation_binding`: `exact | different | unresolved`.

Harness-owned materialization then determines the final disposition:

- `exact + exact` => `relevant`, subject to existing strict identity floors;
- any `different` => `irrelevant`;
- otherwise any `unresolved` => `ambiguous`.

A missing proposal remains `ambiguous`.

This preserves the destructive rejection decision inside Harness-owned deterministic policy. The model cannot create target identity, source authority, trust, freshness, verification, truth, or a verdict.

## Semantic boundaries

`different` requires affirmative evidence that the candidate concerns another target or another requested relation. Missing, partial, truncated, mixed, uncertain rename/alias, or unresolved local applicability remains `unresolved`.

Factual disagreement about the same exact target and relation is still `exact + exact`; contradiction and truth remain downstream concerns.

Strict Harness-owned identity anchors remain unchanged. URL-only and navigation/footer-only identity cannot satisfy strict target identity. Semantic-equivalent policy remains explicit and Harness-owned.

## Fresh v3 corpus

- suite: `evidence-relevance-calibration-v3`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: excluded from tuning

The semantic families remain comparable to v2, while expected model output is re-authored as target/relation bindings under the new proposal contract.

## Acceptance

Canonical Mistral and Google arms must both be operationally complete with:

- wrong-target / unresolved-binding materialized as `relevant`: **0**;
- utility misses: **0**;
- provider failures: **0**.

Binding proposal exact accuracy remains diagnostic. Deterministic safety overrides remain observable. The lexical baseline remains diagnostic only.

## Freeze discipline

Before first live observation:

1. deterministic v3 binding materialization must pass all 26 cases;
2. core/runner tests and clippy must pass;
3. the exact core/runner/fixture surface must be checksummed;
4. the live workflow must be committed;
5. `engine-0.6-evidence-relevance-calibration-v3-freeze` must bind the first/only canonical v3 observation.

No independent holdout is authored until v3 passes and #462 semantics are frozen.
