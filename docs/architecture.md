# Architecture

## Product boundary

Reasoning Harness is a native correctness runtime around stochastic candidate generation. Its current core materializes and evaluates a `ReasoningArtifact` and also owns a provider-neutral bounded protocol that can turn unresolved reasoning into resolution requests, re-verify admitted evidence or revised candidates, and finalize an answer only from adequately covered propositions.

The model is never part of the trusted computing base. A model may propose facts, claims, links, transformations, repairs, or rendered prose; the harness decides whether the resulting state is structurally admissible and what level of support can be claimed.

### Current implemented execution path

```text
source/task
   |
   v
candidate generation (model, optional)
   |
   v
ReasoningArtifact / framework trace
   |
   +--> deterministic validation
   +--> evidence / provenance gates
   +--> trusted verification receipts from deterministic/external oracles
   +--> narrow deterministic framework passes
   +--> contradiction, assumption, causal, and adversarial diagnostics
   |
   v
accept | reject | unknown
```

### Implemented provider-neutral grounded execution loop

`accept | reject | unknown` is an epistemic/policy decision, not necessarily the permanent end of a product run. The provider-neutral core defined by [ADR-0002](adr/0002-grounded-resolution-and-finalization.md) now adds bounded resolution and finalization:

```text
task + harness-owned evidence
          |
          v
candidate generation
          |
          v
materialize + validate + verify + diagnose
          |
          +--> supported enough ----------------------------+
          |                                                 |
          +--> unknown / insufficient support               |
          |          |                                      |
          |          v                                      |
          |     typed resolution request                    |
          |          |                                      |
          |     external evidence / verifier adapter        |
          |          |                                      |
          |          v                                      |
          |     revise / regenerate                         |
          |          |                                      |
          |          +----------> re-run harness -----------+
          |                                                 |
          +--> refuted --> discard/revise --> re-run -------+
                                                            |
                                                            v
                                                       finalization
                                                            |
                                                            v
                                                  claim coverage check
                                                            |
                                                            v
                                              grounded answer | abstain
```

The bounded control loop is implemented in core. Existing diagnostics can drive typed requests, but concrete web/RAG/database/MCP/human acquisition remains adapter work outside core. The repository therefore does not claim open-world resolution quality merely because the control protocol exists.

## Design rules

1. `unknown` is a successful epistemic outcome.
2. No `known` or `supported` claim without evidence.
3. Frameworks produce typed traces, not prose-only explanations.
4. Deterministic checks beat model judges whenever a deterministic oracle exists.
5. Soft semantic judging must be separately identified from hard validation.
6. A failed pass cannot silently continue with partially invalid state.
7. Provider/model adapters stay replaceable and outside core semantics.
8. Schema-valid model output is still only a candidate until validation and acceptance policy run.
9. Live model quality and deterministic contract regression are separate execution modes.
10. Retrieval or tool output is acquired data, not authority by default.
11. Every repaired/regenerated candidate crosses the same validation and verification boundary as the original.
12. A final renderer cannot upgrade epistemic state or introduce unsupported factual propositions.
13. Resolution is budgeted; budget exhaustion yields an explicit unresolved/abstain outcome rather than fabricated completion.

## Interfaces

The native runtime is the correctness boundary. CLI and eval are the first supported interfaces. A desktop UI is a deferred thin inspection client, the public embedding API is stabilized only after real usage, and MCP is an optional integration adapter rather than part of the correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md) and [ADR-0002](adr/0002-grounded-resolution-and-finalization.md).

For durable policy/session control, see [ADR-0003](adr/0003-reasoning-control-plane.md).

## Implementation language boundary

All first-party executable and library components are implemented in Rust. This includes the native runtime, CLI, evaluation tooling, model adapters, and any future desktop client or optional integration adapter. Model providers remain external services and are reached through Rust adapters. No JavaScript/TypeScript runtime is part of the correctness boundary.

## Runtime decision boundary

The runtime validates the input artifact before the first pass and after every pass. A policy then maps the valid artifact to `accept | reject | unknown`. The initial strict policy rejects explicit contradictions and preserves `assumed` or `unknown` claims as an `unknown` outcome. This policy is intentionally conservative and will evolve only with fixture evidence.

In the grounded runtime, that policy result additionally determines whether the run may finalize, should emit a typed resolution request, should revise/regenerate, or must stop unresolved. Policy may choose to stop immediately on `unknown`; resolution is an explicit capability, never an obligation to manufacture an answer.

See [prior art](prior-art.md) for external design patterns considered without adding runtime dependencies.

## Candidate authority boundary

Model output is represented as `ReasoningCandidate`, not as a finalized `ReasoningArtifact`. The candidate contains proposed claims, proposed epistemic states, and inference edges, but it cannot supply evidence. The runtime combines the candidate with harness-owned `HarnessInput` and initially materializes model-proposed `known`, `supported`, `inferred`, or `contradicted` states as `assumed`. Only harness-owned verification passes may later establish stronger states. A model may preserve `unknown` because uncertainty is a safe epistemic outcome.

This prevents a provider from fabricating its own evidence records, self-certifying a claim as supported, or forcing a final contradiction verdict merely by emitting a schema-valid label.

The same rule applies to future repair/regeneration. A model receiving diagnostic feedback may propose a better candidate, but the replacement candidate starts untrusted and receives no authority from the fact that it was generated in a repair phase.

## Verification receipt boundary

`VerificationReceipt` is authority-bearing data and is deliberately absent from `ReasoningCandidate`. A trusted verifier creates receipts only after candidate generation. The preferred hard-verification contract binds a typed `Proposition { key, value }` to structured facts owned by harness evidence. Inputs without evidence qualification requirements retain `StructuredFactVerifier` compatibility behavior. Inputs with requirements use `QualifiedStructuredFactVerifier`, which filters structured facts through harness-owned temporal/scope/provenance requirements before a hard receipt can be created and withholds a receipt when multiple qualified values conflict. Missing or unqualified facts preserve uncertainty. When a receipt is applied, the authoritative claim text is canonicalized to `key = value` so model-authored prose is never presented as verifier-endorsed wording. Exact statement-bound receipts remain available only as a conservative compatibility path for external verifiers.

A receipt is not a semantic score. It represents a hard verifier result whose authority comes from the verifier named by the caller. The current fixture benchmark uses explicit `fixture_oracle` receipts to test process correctness under known oracle coverage; this must not be reported as generic reasoning accuracy.

## Resolution boundary — implemented core

The resolution layer converts unresolved verified state into typed requests for additional work. The request describes the missing support; it does not invent the missing fact.

Expected request families include:

- proposition evidence acquisition;
- causal relation evidence acquisition;
- temporal/scope/provenance qualification;
- deterministic external verification;
- candidate revision after hard refutation;
- explicit human review where policy permits it.

The runtime owns request identity, attempt history, budget, allowed resolver class, and the state transition back into verification. External systems own domain-specific acquisition mechanics.

Web search, retrieval pipelines, databases, MCP tools, compilers, tests, policy engines, and humans may act as resolver adapters. Their output only gains authority according to the same harness-owned evidence or verifier contract used elsewhere. A retriever returning a document is not equivalent to a verifier proving the proposition that motivated the retrieval.

No resolution implementation may silently convert `unknown` into `supported` merely because a resolver returned something.

`ResolutionResolver` cannot return trusted metadata or receipts. Raw `AcquiredEvidence` crosses `EvidenceAdmissionPolicy` before entering `HarnessInput`, and `TrustedResolutionVerifier` is a separate authority-bearing interface. The default admission policy rejects all acquired evidence. Per-run and per-request attempt/token/time budgets plus resolver-class allowlists are owned by the runtime. Every admitted-evidence or candidate-revision step re-runs the ordinary normalization, validation, verification, diagnostic, and decision path.

See [bounded grounded resolution and finalization](grounded-resolution.md) for the concrete contracts and deterministic benchmark.

## Finalization boundary — implemented core

Finalization is distinct from verification and from presentation style.

The finalizer receives verified artifact state and produces a grounded answer, qualified partial answer, unresolved result, abstention, or a `requires_verification` result according to policy. A model may be used as a renderer, but the renderer cannot create authority.

The required target invariant is **final claim coverage**: factual propositions that appear in the final answer must map to supported artifact propositions or be explicitly represented as unresolved/assumed according to policy. If a renderer introduces a new factual proposition, that proposition must re-enter the ordinary candidate/verification loop before it may appear as grounded output.

This makes `ReasoningArtifact` the source of truth and prevents a fluent final-generation step from undoing the correctness work performed earlier in the run.

## Narrow deterministic framework checks

The Five Whys restatement pass removes a causal edge only when a deliberately narrow lexical heuristic recognizes that the proposed cause substantially restates the effect. The conclusion remains uncertain. This avoids turning a string heuristic into semantic causal authority.

## Candidate normalization boundary

`ReasoningCandidate` is untrusted syntax, not trusted reasoning state. Structurally invalid inference suggestions (for example, missing premises or references to non-existent claims) are removed before artifact validation and recorded as `candidate_diagnostics`. This is not silent repair: the artifact preserves an inspectable record of every dropped edge. Claims themselves still pass through the normal downgrade and hard-verification boundary, so normalization cannot promote a claim or create authority.

## Adversarial discovery boundary

`AdversarialDetector` produces typed `AdversarialFinding` records with `contradiction | counterexample` kind and `hard | soft` strength. Discovery is observational: findings are recorded in the artifact but do not mutate claim epistemic state and cannot directly force `reject`. Hard authority remains in deterministic `Verifier` implementations and trusted verification receipts. The initial `StructuredFactConflictDetector` reads only harness-owned structured facts. A future model-backed semantic detector must emit `soft` findings until an independent hard verifier confirms them.

This separation prevents a model-generated contradiction label or counterexample suggestion from becoming self-authenticating evidence.

## Reasoning policy and invalidation boundary

`ReasoningPolicy` constrains admissibility and escalation; it does not decide truth. Global/domain/run policy layers compose conservatively for authority, scope, derived-support capability, and resolver-class permission. Soft semantic findings may request additional work, but no policy rule can create evidence, receipts, hard findings, epistemic promotion, or verdict authority.

A policy change creates a new `ReasoningArtifact` snapshot rather than mutating history. Hard state is preserved only when its authority remains reconstructable under the effective policy. Invalidated receipts propagate to claims, dependent inference edges, downstream claims, and finalization. Affected edges are removed from the new accepted snapshot, policy-sensitive qualification/assumption diagnostics are recomputed, and `StrictAcceptancePolicy` is re-evaluated. The old artifact remains unchanged for future thread/history ownership.

`constrain_resolution_policy` reuses #22 and can only tighten resolver classes and required evidence authority. See [reasoning policy and dependency invalidation](reasoning-policy.md) and [ADR-0003](adr/0003-reasoning-control-plane.md).

## Soft semantic-judge boundary

`SoftDiagnosticJudge` is an explicitly non-authoritative discovery/calibration boundary. It emits `finding | no_finding | abstain` observations tied to a typed diagnostic request and stable judge/model/configuration identity. `SoftSemanticFinding` deliberately has no verification receipt, verdict, epistemic-state mutation, or hard-strength field, and it is not stored in `ReasoningArtifact` by the initial calibration implementation.

Calibration reports precision/recall, decision coverage, disagreement, abstention, pairwise categorical agreement, and nominal Krippendorff alpha separately from final harness correctness. Ambiguous labels are retained but excluded from positive/negative precision/recall. Abstention remains explicit and is treated as missing data for alpha rather than being majority-voted into a finding.

A future policy/thread layer may record a soft observation or use it to request additional evidence, but only existing harness-owned evidence/verifier boundaries may create hard authority. See [soft semantic-judge calibration](semantic-judge-calibration.md).

## Assumption diagnostic boundary

`HarnessInput.assumptions` is harness-owned input and is deliberately absent from `ReasoningCandidate`. It names propositions that the task is allowed to take as premises without claiming that those propositions were independently verified. This is distinct from `hypotheses`, which identify propositions the task asks the candidate to evaluate.

`AssumptionInspector` examines propositions actually used as inference premises after trusted verification passes have run. `known`/`supported` premises are trusted, and `inferred` premises count as derived support only when their inference chain bottoms out in trusted support or an explicit input assumption. A candidate's own `inferred` label is therefore insufficient. Typed premises with no trusted support and no explicit input assumption produce hard `unsupported_premise` process findings; premises without a proposition binding produce soft `unbound_premise` findings. Findings remain observational and cannot create evidence, verification receipts, or verdict authority.

In the resolution loop, these findings may motivate a resolution request or candidate revision, but they do not gain additional authority by becoming actionable.

## Evidence qualification boundary

`Evidence.metadata`, `EvidenceRequirement`, and `EvidenceAuthorityPolicy` are harness-owned and absent from `ReasoningCandidate`. They let the runtime test whether a structured fact is applicable at an explicit time, scope, and minimum opaque authority rank without embedding domain-specific source names in core logic. Deterministic mismatches are hard findings; missing metadata stays soft/unknown.

Evidence qualification itself is observational, but the built-in structured verifier consumes the same requirements before producing hard receipts. This prevents stale, out-of-scope, or insufficient-authority facts from silently becoming `supported`/`contradicted`. Conflicting qualified values produce a diagnostic conflict and no built-in hard receipt. Explicit external trusted receipts remain an independent oracle compatibility boundary and are not automatically reinterpreted by this layer.

In the implemented resolution loop, newly acquired evidence passes the same qualification boundary before it can resolve an unknown. Retrieval therefore cannot bypass time, scope, or provenance policy merely because it returned a relevant-looking record.

See [temporal, scope, and provenance evidence qualification](evidence-qualification.md) for scope semantics, authority-policy rules, and cross-diagnostic interactions.

## Evidence-aware causal diagnostic boundary

`CausalInspector` extends Five Whys inspection beyond lexical restatement without becoming a verdict authority. It canonicalizes a typed causal relation as cause proposition(s) -> effect proposition and matches that relation only against harness-owned `CausalEvidence` with explicit provenance. Exact support can mark an edge `supported`; exact trusted refutation can mark it `refuted`. Association-only evidence, partial support, reverse-direction support, conflicting evidence, missing relation evidence, and incomplete proposition bindings remain `unknown` with soft diagnostics.

Causal inspection is observational: it does not mutate claim epistemic state, create verification receipts, or directly alter the final `accept | reject | unknown` policy result. The existing lexical Five Whys cleanup remains a narrow deterministic fast path, but cleanup is now local to the exact offending inference edge and cannot downgrade an independently hard-supported claim. The dedicated deterministic corpus under `fixtures/causal/` is evaluated separately from the original claim-verdict benchmark and from repeated-trial correctness denominators.

See [evidence-aware causal diagnostics](causal-reasoning.md) for the detailed contract and deferred scope.

## Metamorphic evaluation boundary

Metamorphic transforms live in the evaluation layer, not the runtime authority boundary. A transform may reorder set-like records, consistently remap referential IDs, or add an explicitly unrelated control fact, but it may not change proposition meaning, trusted verification conclusions, causal direction/membership, or other oracle semantics.

The evaluator compares semantic diagnostic signatures separately from raw finding IDs. This is necessary because a valid stable-ID remap can change generated diagnostic identifiers while preserving the same hard finding. Final-verdict invariance, hard-finding invariance, soft-finding stability, and typed diagnostic-status invariance are reported independently and never replace the original benchmark correctness denominator.

See [metamorphic reasoning robustness](metamorphic-testing.md) for the current transform contract.

## Versioned corpus measurement boundary

`CorpusManifest` is evaluation metadata, not runtime authority. It fixes stable suite-prefixed case identity, category/difficulty strata, scoring mode, provenance/redistribution notes, contamination posture, and lifecycle status independently from model/provider output. `score_compatibility_id` makes direct score-comparison compatibility explicit.

Recorded claim stratification reuses the existing `BenchmarkComparison` aggregation and therefore cannot redefine correctness semantics. Live runs record corpus identity but do not pool category/difficulty scores across repeated or incomplete trials. Future resolution variants must reuse stable base case IDs so recovery metrics remain additions to, not replacements for, the original denominator.

See [versioned benchmark corpus](corpus-versioning.md) for compatibility and change rules.

## Repeated diagnostic measurement boundary

Repeated diagnostic aggregation is an evaluation/reporting boundary, not a verifier. `DiagnosticSignal` records adversarial findings, candidate-normalization codes, causal finding/reason observations, assumption signals, and evidence-qualification findings without granting any of them new authority. `stability.diagnostics` is serialized alongside, not inside, final correctness stability.

Only operationally complete trials contribute to diagnostic frequencies and count distributions. Partial successful observations from an incomplete provider trial are reported as excluded observations rather than interpreted as diagnostic absence. Confidence intervals use the documented 95% Wilson score method only after the minimum complete-observation threshold; exact counts and denominators are always retained.

Resolution-loop evaluation remains separate from both diagnostic stability and ordinary verdict accuracy. Recovery rate is useful only when paired with unsafe-final-answer rate, final claim coverage, resolution cost, and explicit exhaustion counts.
