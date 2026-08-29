# Roadmap

## v0.1 — trustworthy intermediate state and native CLI
- stabilize HarnessInput / ReasoningCandidate / ReasoningArtifact schemas
- JSON Schema export
- provenance coverage gates
- harness-owned evidence / untrusted candidate authority boundary
- verification receipts / oracle-backed promotion for safely upgrading supported claims **implemented**
- explicit unknown/assumption handling
- fixture-based eval runner
- native CLI for run / verify / eval workflows; explain remains deferred until renderer semantics are defined
- JSON output and CI-safe exit semantics
- first provider adapter experiment (Mistral HTTP adapter + manual live benchmark implemented)
- offline fixture regression separated from live provider benchmark runs
- explicit hard-validator vs soft-judge metric classification

## P0 next — structured verifier binding
- [done] replace brittle exact-prose receipt matching with a typed `Proposition { key, value }` verification target
- define a provider-neutral verifier/oracle adapter contract that runs after candidate generation
- [done] bind verifier results to structured propositions plus harness-owned structured facts, never model self-asserted authority
- restore live accept/reject utility without increasing unsupported accepted claims
- [done] preserve exact-string receipt binding as a conservative compatibility mode

## v0.2 — adversarial reasoning passes
- generic contradiction discovery pass; trusted-oracle contradiction receipts are implemented
- counterexample generation + deterministic verification; trusted-oracle counterexample rejection is implemented
- assumption pass
- semantic-loss checks

## v0.3 — framework plugins
- extend the implemented lexical Five Whys restatement pass with evidence-aware semantic checks that remain explicitly soft unless oracle-backed
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


## Implementation constraint

All first-party components remain Rust-only. A future desktop application must use a Rust-capable native UI stack without requiring a JavaScript application runtime. Any future MCP adapter, if justified, is implemented in Rust and remains outside the core correctness boundary.
