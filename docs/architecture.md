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


## Interfaces

The native runtime is the correctness boundary. CLI and eval are the first supported
interfaces. A desktop UI is a deferred thin inspection client, the public embedding API
is stabilized only after real usage, and MCP is an optional integration adapter rather
than part of the correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).
