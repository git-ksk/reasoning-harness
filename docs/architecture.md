# Architecture

## Boundary

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
   +--> contradiction and adversarial passes (planned)
   +--> external oracle checks when available
   |
   v
accept | reject | unknown
```

The model is never part of the trusted computing base. A model may propose facts, claims, links, or transformations; the harness decides whether the resulting artifact is structurally admissible and what level of support can be claimed.

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


## Interfaces

The native runtime is the correctness boundary. CLI and eval are the first supported
interfaces. A desktop UI is a deferred thin inspection client, the public embedding API
is stabilized only after real usage, and MCP is an optional integration adapter rather
than part of the correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).


## Implementation language boundary

All first-party executable and library components are implemented in Rust. This includes the native runtime, CLI, evaluation tooling, model adapters, and any future desktop client or optional integration adapter. Model providers remain external services and are reached through Rust adapters. No JavaScript/TypeScript runtime is part of the correctness boundary.

## Runtime decision boundary

The runtime validates the input artifact before the first pass and after every pass. A policy then maps the valid artifact to `accept | reject | unknown`. The initial strict policy rejects explicit contradictions and preserves `assumed` or `unknown` claims as an `unknown` outcome. This policy is intentionally conservative and will evolve only with fixture evidence.

See [prior art](prior-art.md) for external design patterns considered without adding runtime dependencies.

## Candidate authority boundary

Model output is represented as `ReasoningCandidate`, not as a finalized `ReasoningArtifact`. The candidate contains proposed claims, proposed epistemic states, and inference edges, but it cannot supply evidence. The runtime combines the candidate with harness-owned `HarnessInput` and initially materializes model-proposed `known`, `supported`, `inferred`, or `contradicted` states as `assumed`. Only harness-owned verification passes may later establish stronger states. A model may preserve `unknown` because uncertainty is a safe epistemic outcome.

This prevents a provider from fabricating its own evidence records, self-certifying a claim as supported, or forcing a final contradiction verdict merely by emitting a schema-valid label.
