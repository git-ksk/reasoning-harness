# Engine 0.6 evidence-target relevance calibration v19 — successor design

Status: design only. No v19 live calibration observation has been made. The v18 canonical result remains immutable FAIL and is not rescored, rerun, relabeled, or retagged.

## Frozen inheritance

v19 inherits without change:
- fixed core: `evidence-relevance-fixed-core-v1`, exactly 48 cases;
- primary proposal: `reason-evidence-relevance-binding-proposal-v5`;
- all v18 scored case labels and expected dispositions;
- v18 operational deadlines, adapter-owned retries, quota/capacity latches, provider-attempt telemetry, and public-log sanitization;
- one-shot canonical policy and holdout prohibition until calibration PASS.

## Why a materializer-only successor is insufficient

Required Mistral v18 completed 48/48 and materialized 47/48 exactly with zero wrong-target Relevant retention. However its raw local qualification was only 26/48 exact, with:
- scope-risk misses: 11;
- spurious scope risks: 0;
- identity-scope misses: 14;
- relation-scope misses: 14.

The v18 final gate required every one of those miss/spurious counts to be zero. A single new terminal rule for `14_sibling_product_overlap` could repair the disposition miss but would still leave the qualification gate red. v19 therefore must change the authority boundary explicitly rather than silently weaken acceptance.

## v19 authority split

v19 should expose two separate layers:

1. **Raw model qualification telemetry.** Keep verifier v8 output available exactly as observed and continue scoring it diagnostically. Do not overwrite or hide model mistakes.
2. **Harness-owned effective qualification.** Introduce a new versioned contract used by runtime materialization and by the v19 qualification gate. It combines deterministic local facts with bounded model evidence under fail-closed rules.

The first Harness-owned component is a typed deterministic local-risk classifier:
- `none`
- `identity_mapping`
- `ownership_scope`
- `context_gap`
- `multiple`

It generalizes the v18 boolean local-risk floor. Any non-none deterministic risk still forces Ambiguous and cannot be overridden by model output.

The effective identity/relation fields must also remain fail-closed. The implementation may normalize model output only from generic local structure and declared Harness identity metadata; it must never infer through clipped context, omitted ownership, or uncertain alias/rename/successor mapping.

## Pre-freeze acceptance for the effective qualifier

Before a v19 freeze tag can be created:
- effective qualification must be invoked/derived for all 48 fixed cases;
- effective identity-scope misses: 0;
- effective relation-scope misses: 0;
- effective scope-risk misses: 0;
- effective spurious scope risks: 0;
- raw verifier v8 metrics must still be emitted separately;
- no rule may reference case IDs, synthetic product names, or exact fixture phrases;
- property tests must cover the generic negative and risk boundaries.

If the effective qualifier cannot meet these requirements generically, v19 does not freeze. Do not relax the gate to consume a canonical run.

## Target-negative materialization

The frozen v18 audit supports one generic new terminal rule:

When all are true:
- deterministic local risk is none;
- effective `scope_risk=none`;
- effective `identity_scope=distinct_target`;
- primary `target_binding != exact`;

then materialize Irrelevant even when the primary target axis is unresolved.

The rule is prohibited when the primary target is exact, any effective/model safety risk is non-none, or deterministic local risk is present.

The fixed-core audit found no expected Ambiguous collision for this shape: every expected instance is Irrelevant.

Two other provider-observed negative shapes require separate property audit before adoption:
- exact target with independently established different relation;
- complete local relation absence with no risk.

They are candidates, not precommitted v19 behavior.

## Operational policy

Do not add a manual Groq daily-quota preflight. Runtime typed quota failure remains the authoritative signal and must latch the provider arm immediately as in v18. No retry storm, canonical rerun, or replacement observation is permitted.

Required providers remain Mistral and Groq; Google remains full non-gating replication unless independent provider evidence changes that policy before freeze.

Independent holdout authoring remains prohibited until a first/only frozen v19 canonical satisfies every required operational, correctness, utility, materialization, and effective-qualification gate.
