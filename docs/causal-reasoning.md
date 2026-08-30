# Evidence-aware causal diagnostics

Issue #4 extends Five Whys beyond lexical restatement checks without turning a model or heuristic into causal authority.

## Canonical relation

A causal relation is represented as a non-empty set of cause propositions and one effect proposition. Proposition keys carry scope. Five Whys traces are stored as effect-premise to proposed-cause conclusion, but the causal inspector canonicalizes them to cause -> effect before matching evidence.

## Authority boundary

`CausalEvidence` is harness-owned and carries provenance plus a typed conclusion: `supports`, `refutes`, or `association_only`. Candidate/model output may suggest claims and inference edges, but it cannot create trusted causal evidence or hard causal findings.
Malformed harness-owned causal records fail at the causal input boundary: evidence IDs and sources must be non-empty, IDs must be unique, relations must contain at least one unique non-empty cause proposition, and the effect proposition must be non-empty. Invalid oracle input is not converted into an `unknown` edge result.

`CausalInspector` is observational. It emits per-edge assessments and findings but does not mutate claim epistemic state, create verification receipts, or directly change the final `accept | reject | unknown` verdict. The current final verdict remains claim-oriented; whole-artifact causal gating is deliberately deferred.

## Hard and soft diagnostics

An exact scoped support record yields `supported`. An exact trusted refutation yields `refuted` plus a hard `explicit_refutation` finding. Everything that lacks deterministic authority remains conservative:

- no exact causal evidence -> `unknown` + soft `missing_causal_evidence`;
- association-only evidence -> `unknown` + soft `association_only`;
- support for only part of a multi-cause relation -> `unknown` + soft `partial_support`;
- support for the reverse direction -> `unknown` + soft `direction_mismatch`, not a refutation;
- conflicting exact support and refutation -> `unknown` + soft `conflicting_evidence`;
- incomplete proposition binding -> `unknown` + soft `missing_proposition_binding`.

The existing lexical restatement heuristic remains a narrow deterministic fast path. Cleanup is local to the exact offending inference edge and cannot downgrade an independently `supported` claim.

## Deterministic causal corpus

`fixtures/causal/` is a separate credential-free regression corpus. It does not alter the original 20-fixture claim-verdict benchmark or the Issue #6 correctness denominator. The initial corpus contains positive and adversarial controls for exact support, exact refutation, association-only evidence, reverse-direction evidence, conflicting evidence, missing proposition binding, multi-cause partial support, and scoped near-neighbor evidence.

`causal_benchmark` reports edge assessments as supported/refuted/unknown and counts hard versus soft findings separately. These diagnostics are process-regression measurements, not generic model reasoning accuracy.

## Deferred scope

This implementation intentionally does not provide general causal discovery, SCM/do-calculus, learned process reward models, LLM-judge final authority, provider-specific causal branches, or semantic similarity as a hard gate. Future model-backed causal critics must remain soft unless independently verified by a deterministic or external trusted oracle.

Also deferred from #4 are candidate-supplied causal-evidence reference hints and a general temporal/domain-constraint reasoner. Issue #11 now provides a provider-neutral repeated-trial report that can aggregate causal finding/reason observations while keeping them outside Issue #6 correctness/availability denominators. A live causal-generation/input contract is still deferred; the current candidate schema has no causal-evidence authority fields, and no future reporting interface may grant candidate or model output hard authority.
