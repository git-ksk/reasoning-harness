# Reasoning policy and dependency invalidation

Issue #27 implements the policy/invalidation track from ADR-0003. The policy layer controls **admissibility and escalation**, not truth.

A `ReasoningPolicy` cannot create evidence, verification receipts, hard findings, epistemic promotion, or a final verdict. It can only constrain which already-authoritative state remains admissible and which additional work the runtime may request.

## Policy composition

Policy layers are composed in explicit order, typically:

```text
global
  -> domain
  -> task/run
```

`ReasoningPolicyLayer` carries generic constraints only. Core contains no domain-specific source taxonomy.

Composition intentionally distinguishes authority-bearing restrictions from advisory control flow:

- minimum authority class: the highest ranked requirement wins;
- applicability scope: layers intersect; disjoint scope is a configuration error;
- derived-support capability: restrictive AND, so a stricter layer cannot be loosened later;
- allowed resolver classes: set intersection;
- evaluation `as_of` time: later contextual layer overrides, then qualification is re-run;
- soft-finding escalation: later layer may override because escalation requests work but does not establish truth.

`None` for allowed resolver classes means “no additional restriction from this policy layer”, not “deny every resolver”. An explicitly empty set denies all resolver classes.

The effective policy has a stable `version_id` and records its source layer IDs.

## Policy validation

Public `ReasoningPolicy` values are validated even when a caller deserializes them directly instead of using `compose_reasoning_policy`.

The runtime rejects:

- empty policy version IDs;
- empty source-layer IDs;
- authority classes absent from the harness-owned rank policy;
- empty scope dimensions;
- empty scope value sets or values.

This keeps policy configuration fail-closed independently of construction path.

## Relationship to evidence qualification

A policy may add generic temporal/scope/authority constraints to proposition evidence requirements. Existing task-specific requirements remain authoritative:

- an existing explicit `as_of` requirement is not overwritten by policy context;
- scope is intersected with policy scope;
- minimum authority is tightened to the stronger requirement.

When no requirement exists for a typed claim/hypothesis and the policy defines evidence constraints, the transition creates an effective requirement for that proposition key.

The transition then re-runs evidence qualification. Metadata relevance alone does not promote a claim.

## Hard-authority preservation

Policy changes operate on a **new artifact snapshot**. The historical input artifact is not mutated.

Hard state survives only when its authority remains reconstructable under the new policy:

- `supported` / `contradicted`: a matching retained verification receipt must remain admissible;
- `known`: direct evidence must still satisfy the effective evidence qualification;
- `inferred`: remains derived working state only when derived support is permitted and its dependency chain remains valid.

A `supported` state with evidence but no reconstructable receipt is downgraded during a policy transition rather than being trusted from its label alone.

Receipt binding uses the same matcher as normal verification/validation, avoiding a second policy-specific interpretation of receipt identity.

## Invalidation propagation

When a stricter policy makes upstream authority inadmissible, `apply_reasoning_policy` emits typed `PolicyInvalidation` records and constructs a new accepted-state snapshot.

Targets include:

- verification receipt;
- claim;
- inference edge;
- finalization.

Invalidation propagates through inference dependencies. If a premise becomes invalid and the conclusion has no independent retained hard receipt:

1. the dependent inference edge is invalidated and removed from the new snapshot;
2. the dependent claim is downgraded to `assumed`;
3. propagation continues through downstream inference edges;
4. finalization is invalidated;
5. the strict acceptance policy is re-evaluated.

Removed edges are still preserved in the old immutable snapshot. Durable historical lineage belongs to #28 `ReasoningThread`, not to mutation of the current artifact.

Policy-sensitive evidence-qualification and assumption findings are recomputed after invalidation so diagnostics do not reference removed inference edges.

## Soft semantic findings

A calibrated `SoftJudgeObservation` may trigger advisory policy actions:

- request evidence;
- request deterministic verification;
- request human review.

It cannot directly mutate the artifact or create hard authority. The eventual result still has to cross #22 evidence admission / qualification / verifier boundaries.

## Resolution policy

`constrain_resolution_policy` can only tighten the existing #22 `GroundedResolutionPolicy`:

- resolver-class allowlists intersect;
- required evidence authority is raised to the stricter class.

It does not create a second resolver abstraction or own acquisition logic.

## Deterministic policy regression

`fixtures/policy/` contains four provider-neutral regression scenarios:

- authority tightening;
- temporal re-evaluation;
- scope-expansion rejection;
- dependency propagation from invalidated support through an inference edge.

They assert that the new snapshot loses inadmissible receipts/claims/edges, finalization is invalidated, and the original artifact remains unchanged.

These fixtures are separate from corpus-v1 correctness, resolution recovery, and semantic-judge calibration denominators.

## Non-goals

- domain-specific evidence/source policy in core;
- generic agent approval UX;
- model confidence as policy authority;
- policy-generated verification receipts;
- hidden chain-of-thought policy or persistence;
- workflow graph orchestration.

Checkpoint/resume/fork and durable event history are intentionally left to #28.
