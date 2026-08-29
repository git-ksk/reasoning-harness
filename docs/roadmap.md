# Roadmap

## v0.1 — trustworthy intermediate state and native CLI
- stabilize ReasoningArtifact schema
- JSON Schema export
- provenance coverage gates
- explicit unknown/assumption handling
- fixture-based eval runner
- native CLI for run / verify / explain / eval workflows
- JSON output and CI-safe exit semantics
- first provider adapter experiment

## v0.2 — adversarial reasoning passes
- contradiction pass
- counterexample pass
- assumption pass
- semantic-loss checks

## v0.3 — framework plugins
- evidence-aware 5 Whys
- first principles
- Feynman/simplification renderer
- framework plugin contract

## v0.4 — reproducible research
- cross-model benchmark matrix
- token/latency/cost accounting
- deterministic vs soft-verifier reporting
- public benchmark corpus


## Deferred interfaces

These are intentional non-goals until the native runtime, CLI, and eval contracts mature:

- desktop UI: thin visualization/review client after artifact formats stabilize.
- public embedding API compatibility: after real consumer pressure validates the contract.
- MCP adapter: optional agent integration; never a required correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).
