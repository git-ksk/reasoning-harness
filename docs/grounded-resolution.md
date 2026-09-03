# Bounded grounded resolution and finalization

Issue #22 adds a provider-neutral control loop around the existing reasoning harness. The loop can turn unresolved typed state into a bounded resolution request, accept acquired data only through an explicit evidence-admission boundary, re-run the ordinary verification pipeline, and finalize output only after typed factual-claim coverage is checked.

This is a runtime protocol, not a web-search or RAG implementation.

## Authority boundaries

Four boundaries are intentionally separate:

1. `ResolutionResolver` performs acquisition, candidate revision, or human-review routing. It can return raw `AcquiredEvidence`, but raw acquired evidence has no `EvidenceMetadata` and is not trusted evidence.
2. `EvidenceAdmissionPolicy` is harness-owned. It may attach trusted metadata to acquired evidence, but the runtime rejects an admission implementation that changes the acquired ID, source, observation, or structured facts while doing so.
3. `TrustedResolutionVerifier` is a separate authority-bearing interface. A generic resolver cannot manufacture `VerificationReceipt` values. Trusted receipts still obey the existing receipt validation contract, including evidence binding.
4. `FinalAnswerRenderer` produces an untrusted final-answer candidate. `finalize_answer` checks every typed factual claim against verified artifact state before text can be emitted as grounded output.

The default evidence admission policy is `RejectAllEvidenceAdmission`. Retrieval therefore cannot become authority accidentally.

## Resolution requests

`ResolutionRequest` carries:

- stable request ID;
- typed reason;
- target;
- requested resolver class;
- optional per-request attempt/token/time budget.

Targets can represent a proposition, causal relation, evidence-qualification requirement, claim revision, or explicit human review. The default planner first considers exact Harness-owned unresolved targets from `ReasoningArtifact.hypotheses` and `evidence_requirements`, preserving an exact evidence requirement as an `EvidenceQualification` target when one exists. Only after those task-owned targets does it consider unsupported-premise findings, other evidence-qualification findings, and unresolved generated claims. This ordering prevents unrelated candidate claims from consuming the bounded resolution budget before an exact requested target is attempted; it does not infer targets from model prose or fuzzy proposition matching. Already exact `Known`/`Supported` targets are not re-requested, while contradiction remains governed by the existing reject/revision policy. The causal target is part of the provider-neutral contract, while automatic causal-evidence acquisition remains deferred because `CausalEvidence` is still a separate observational contract.

Target priority does not change authority. Resolver output must still pass the configured admission boundary and the ordinary verification pipeline before it can change epistemic state, and temporal/scope/authority qualification remains attached to the exact Harness-owned requirement.

## Bounded execution

`GroundedResolutionPolicy` owns the run-wide controls:

- maximum attempts;
- added-token budget;
- elapsed-time budget as reported by adapters;
- allowed resolver classes;
- required evidence authority class;
- whether hard refutation may request candidate revision;
- whether human review is allowed;
- qualified-partial finalization policy.

Per-request budgets are enforced separately and `GroundedResolutionOutcome.request_usage` preserves request-level attempt/token/time accounting. Budget exhaustion never changes epistemic state. It terminates as `exhausted` with the current verified state intact.

Terminal statuses are:

- `resolved_supported`;
- `resolved_qualified`;
- `resolved_refuted`;
- `exhausted`;
- `unavailable`;
- `human_review_required`.

## Re-verification invariant

Every state-changing contribution re-enters the same correctness path:

- admitted evidence is appended to harness input and the candidate is materialized and verified again;
- candidate revision replaces only the untrusted candidate and is normalized, validated, verified, diagnosed, and decided from scratch;
- trusted verifier receipts pass through the ordinary trusted-receipt pass;
- evidence-qualification requirements are preserved or strengthened before newly acquired evidence may produce a hard structured-fact receipt.

A resolver returning relevant-looking data is therefore insufficient to resolve an unknown.

## Finalization coverage

Finalization operates on `ReasoningArtifact`, never raw provider prose. A final claim marked `grounded` must match a `known` or `supported` typed artifact proposition. A claim marked `uncertain` must still map to an artifact proposition with an admissible non-contradicted epistemic state.

If a renderer introduces a new factual proposition, finalization returns `requires_verification` and withholds its text. The bounded runtime converts that proposition into a new harness-owned hypothesis and routes it through resolution and ordinary verification before it may appear as grounded output.

The deterministic `CanonicalFinalAnswerRenderer` is the current default. Model-backed renderers can implement the same interface later without gaining authority.

## Controlled resolution benchmark

`fixtures/resolution/` contains deterministic resolution variants tied to stable corpus-v1 base identity. The initial nine scenarios all reuse `claim:missing-evidence` and cover:

- newly acquired supporting evidence;
- explicit refutation;
- stale evidence;
- wrong-scope evidence;
- insufficient-authority evidence;
- conflicting evidence;
- no resolver result;
- malformed resolver output;
- valid-looking but untrusted resolver output.

Run:

```bash
cargo run -p reasoning-harness-cli -- eval-resolution fixtures/resolution --format human
```

The resolution aggregate is separate from ordinary correctness and repeated diagnostic stability. It reports initially-unknown recovery, unsafe emitted final answers, blocked unverified finalizations, final factual-claim coverage, terminal distribution, attempts, and adapter-reported token/time cost.

The committed deterministic baseline has nine passing scenarios: one unknown-to-supported recovery, one explicit refutation, seven exhausted cases that preserve unknown, zero unsafe final answers, and full typed final-claim coverage. These fixture-oracle results are process regression tests, not empirical evidence of open-world resolver quality.

## Deferred integration scope

The core does not ship a generic web crawler, RAG pipeline, database resolver, MCP resolver, human-review backend, or provider-specific resolution policy. Real integrations must implement the adapter contracts and preserve the same admission/verifier boundaries. Live multi-provider resolution research is also separate from the deterministic CI baseline.
