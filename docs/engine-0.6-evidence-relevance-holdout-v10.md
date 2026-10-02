# Engine 0.6 evidence relevance holdout v10

Status: runner-freeze preparation only. Semantic contract remains frozen v11 + v23 from engine-0.6-evidence-relevance-successor-v9-semantics-freeze. No holdout-v10 corpus has been authored or observed.

## Why v10 exists

Canonical holdout v9 is immutable FAIL. Post-run adjudication found two evaluation-surface expectations that drifted from the already frozen successor-v9 development contract, plus one independent Google operational timeout. The semantic implementation is not retuned. Holdout v10 is a fresh evaluation generation under unchanged semantics.

## Frozen semantic input

- semantics tag: engine-0.6-evidence-relevance-successor-v9-semantics-freeze
- semantics commit: 3e49a9fd2f7827b7c707516d2150e2f0f16e9e17
- effective qualification: v11
- materialization: v23
- issue binding: #468
- canonical holdout v9 remains immutable FAIL

## Runner wiring

Dedicated binary: reason-evidence-relevance-holdout-v10-study.

V10 profile:
- configuration: evidence-relevance-live-holdout-v10
- suite: evidence-relevance-holdout-v10
- annotation protocol: evidence-relevance-effective-qualification-v11
- fixed core: evidence-relevance-fixed-core-v10
- expected directory: fixtures/evidence-relevance-holdout-v10
- expected cases: 26
- issue: 468
- complete holdout only when all cases are observed

No holdout-v10 directory may exist at runner freeze.

## Evaluation-contract guard

Before corpus freeze, relation expectations must be explicitly reconciled with the frozen successor-v9 development contract. Non-frame controls that were frozen as different_relation must not be silently relabelled as unresolved.

Canonical v9 labels are not edited. This guard applies only to the new fresh surface.

## Runner freeze

Freeze coordinate: engine-0.6-evidence-relevance-holdout-v10-runner-freeze.

Only after that tag is pushed may a fresh 26-case corpus be authored. The corpus must have zero case/entity/task/signal/exact-8-token reuse against holdout v1-v9 and all prior successor development surfaces. Provider observation remains prohibited until the corpus itself is frozen.
