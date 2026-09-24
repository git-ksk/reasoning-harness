# ADR-0005: Target-local evidence-need routing before acquisition

Status: candidate implementation accepted for calibration; final adoption requires fresh live calibration and a separately frozen independent holdout.

## Context

Released Harness Engine 0.5.0 owns evidence admission, qualification, verification, bounded resolution, finalization, answer safety, and replay-safe reasoning threads. It does not yet own the earlier decision of whether an exact requested target needs external evidence at all.

This gap can cause a context-local task such as summarizing or explaining supplied material to enter external acquisition unnecessarily. The opposite failure is also unsafe: a current-state or explicit verification request must not be downgraded merely because related supplied context exists.

Issue #461 therefore introduces a provider-neutral pre-acquisition boundary for the Engine 0.6 candidate. It is additive to Engine 0.5.0 and does not rewrite released semantics.

## Decision

Add a narrow target-local evidence-need contract before acquisition.

The ordered evidence modes are:

1. no_factual_evidence
2. context_only
3. external_optional
4. external_required
5. trusted_verification_required

Evidence need and acquisition disposition are separate. A target may remain external_required while acquisition is reuse_existing when existing Harness-owned evidence still satisfies the current freshness, scope, authority, and policy requirements.

The model-facing object is only an EvidenceNeedProposal containing an existing target ID and a proposed mode. It has no evidence, authority, source, freshness, scope, tool, or verdict field.

Harness-owned target kind is typed as non_factual, content_local, external_world, or ambiguous. content_local has a context_only semantic floor; external_world and ambiguous have an external_required semantic floor. This makes claims-about-content versus claims-about-world auditable rather than implicit in prose.\n\nHarness-owned EvidenceNeedTargetPolicy owns:

- the exact target identity and question;
- the deterministic baseline mode;
- the minimum mode;
- whether any model downgrade is permitted and its lowest allowed floor;
- explicit user verification intent;
- current-state requirements;
- trusted-verification requirements;
- supplied-context state and target-local sufficiency;
- existing-evidence reuse status.

A model proposal may escalate a requirement. A downgrade is accepted only inside an explicit Harness-owned downgrade floor and is then rechecked against all hard floors. Without that permission, a lower model proposal is ignored.

## Responsibility boundaries

EvidenceNeedTargetPolicy decides the pre-acquisition epistemic job. It does not decide truth.

EvidenceRequirement remains the proposition-level freshness, scope, and minimum-authority requirement used by qualification and verification after evidence exists. It is not replaced by evidence-need routing.

InvestigationTarget remains the bounded acquisition target. Evidence-need routing decides whether a target should enter that acquisition path; it does not create a second resolver abstraction.

EvidenceAdmissionPolicy and EvidenceQualificationPass remain authoritative after acquisition. A source being retrieved or reused does not make its content true.

ReasoningPolicy may supply conservative run/domain constraints to a future integrated policy builder, but the target-local decision remains separately auditable rather than becoming a sticky whole-run mode.

GroundedResolutionRuntime remains the existing resolution authority. The #461 candidate is not wired into released Engine 0.5.0 behavior on this branch until evaluation demonstrates safe adoption.

ReasoningThread replay must persist only serializable decisions and validity identities. It must never persist executable resolver callbacks or replay an external tool call.

## Context semantics

Context completeness and target-local sufficiency are distinct.

Partial or truncated context must never be interpreted as proof that information is absent. A partial excerpt may still be sufficient for an exact question about that excerpt. A request about the complete source with truncated input remains insufficient and must not silently authorize a context-only world claim.

Supplied context, including prompt-injection-like instructions, is data. It cannot mutate EvidenceNeedTargetPolicy or authority rules.

Claims about content and claims about the external world remain separate propositions. context_only authorizes only the former.

## Monotone floors

The materializer applies the following Harness-owned floors after considering any proposal:

- explicit external verification intent => at least external_required;
- current-state requirement => at least external_required;
- trusted exact verification => trusted_verification_required;
- insufficient or unknown context for a local/optional factual target => external_required;
- configured policy minimum => never weakened.

Resolver unavailability does not change these modes. If acquisition is required but unavailable, downstream runtime behavior remains unknown/abstain or a typed operational terminal; it never falls back to context-only authority.

## Evidence reuse

Reuse is permitted only when prior Harness-owned state says the exact target still satisfies the effective requirement.

External-required work may reuse evidence satisfying external requirements. Trusted-verification-required work may reuse only evidence that already satisfies trusted verification.

Stale, scope-mismatched, policy-mismatched, or ambiguous evidence cannot prevent required acquisition.

The evidence-need module does not independently infer freshness, scope, or authority from adapter payloads. Those inputs must come from existing Harness-owned qualification/verification state.

## Multi-target, follow-up, and replay

Every target is materialized independently. Mixed requests therefore may contain context_only and external_required targets in the same turn.

Evidence mode is recomputed on follow-up turns. Prior context-only status is not sticky.

Serialized EvidenceNeedDecision state contains no external action. Replay reconstructs the decision without executing a resolver. Any future invalidation of freshness, scope, policy identity, or target identity must invalidate reuse before new acquisition decisions are made.

## Evaluation and release boundary

The motivating production incident is a gap report only and is excluded from tuning fixtures.

Fresh calibration identity: evidence-need-routing-calibration-v1.

Acceptance criteria are frozen before independent holdout authoring. Correctness and utility are scored separately. Hard gates include zero unsafe skipped acquisition, zero context authority laundering, zero invalid model weakening, zero invalid evidence reuse, and zero replayed external side effects.

The candidate remains part of the Harness Engine 0.6.0 research line. No Engine release version is promoted until fresh live calibration and a separately frozen independent holdout pass. Engine 0.5.0 remains immutable.
