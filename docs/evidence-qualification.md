# Temporal, scope, and provenance evidence qualification

The evidence-qualification layer asks whether harness-owned evidence is applicable to a proposition at the time, scope, and source-authority level required by the harness. It does not retrieve evidence, rank real-world sources by name, or infer missing metadata.

## Harness-owned contracts

`Evidence.metadata` may contain:

- `temporal`: an optional validity window expressed as inclusive Unix-second bounds;
- `scope`: provider-neutral applicability dimensions whose coverage is either `any` or an explicit set of values;
- `provenance_class`: an opaque class label whose meaning is supplied by the harness.

`HarnessInput.evidence_requirements` binds one qualification requirement to a proposition key. A requirement may specify an `as_of_unix_seconds`, required scope, and minimum authority class. Validation permits at most one requirement per proposition key so every structured fact for that key is evaluated under one unambiguous qualification context.

`HarnessInput.authority_policy` maps opaque provenance classes to integer ranks. The core has no built-in knowledge that one named source is stronger than another. Domain-specific source taxonomy belongs outside the core.

All of these fields are harness-owned and absent from `ReasoningCandidate`. A model may observe the supplied context, but it cannot create evidence metadata, change requirements, or promote its own provenance class.

## Qualification semantics

A required as-of time is valid when it is inside the evidence validity window. An as-of time before `effective_from` is `not_yet_valid`; a time after `effective_until` is `stale`. Missing temporal metadata is `unknown`, not stale or current by assumption.

Required scope must be a subset of evidence coverage for every required dimension. Disjoint values are a `scope_mismatch`; partially covered or universal requirements backed only by narrower evidence are `scope_expansion`. Missing required scope metadata is `unknown`.

A provenance class qualifies when the harness policy ranks it at or above the required class. A lower rank is `insufficient_authority`. Missing or unranked provenance is `unknown`; the core does not guess an ordering.

Deterministic mismatches are hard findings. Missing bindings are soft findings because the harness lacks enough structured information to prove a mismatch.

## Verification interaction

Evidence qualification is observational, but the built-in structured-fact hard-verification path is qualification-aware when an input contains evidence requirements. `QualifiedStructuredFactVerifier` excludes stale, not-yet-valid, scope-mismatched, scope-expanded, insufficient-authority, and metadata-insufficient records from hard receipts. If no evidence survives, no receipt is emitted and the claim remains uncertain.

If multiple qualified evidence records for the same key contain conflicting values, the qualification layer emits a hard conflict finding and the qualified verifier withholds a receipt. It does not turn conflicting qualified evidence into a hard support or contradiction verdict.

Inputs with no evidence requirements retain the historical `StructuredFactVerifier` behavior for backward compatibility.

Explicit `TrustedVerificationPass` receipts are a separate external-oracle compatibility boundary. They are already harness-owned authority and are not automatically downgraded by evidence qualification. Callers that construct such receipts are responsible for enforcing any temporal/scope/provenance policy appropriate to that external verifier.

## Interaction with other diagnostics

Assumption diagnostics run after hard verification. Therefore a premise can become trusted support through the built-in structured verifier only when any applicable evidence requirement is satisfied; stale or otherwise unqualified evidence cannot bootstrap a derived-support chain.

`CausalEvidence` remains a separate harness-owned causal relation contract. Generic `Evidence.metadata` qualification does not silently rewrite causal support/refutation semantics. A future causal input adapter may map the same domain policy into causal evidence, but #16 does not merge the two authority types.

Evidence-qualification findings are also emitted as their own `DiagnosticSignal` family for repeated-trial reporting. They remain outside final verdict accuracy and causal-edge-quality denominators.

## Deterministic corpus

`fixtures/evidence-qualification/` is a separate credential-free eight-case regression corpus covering exact qualification, stale evidence, not-yet-valid evidence, disjoint scope, unsupported scope expansion, insufficient authority, conflicting qualified evidence, and missing metadata. The corpus measures qualification behavior only and never changes the historical 20-case claim-verdict or eight-case causal denominators.

## Non-goals

This layer does not perform open-world retrieval, web search, generic RAG orchestration, automatic recency-based truth selection, domain-specific source ranking, or model-generated provenance certification.
