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
- [done] normalize malformed untrusted inference edges with explicit `candidate_diagnostics` rather than failing unrelated claims

## v0.2 — adversarial reasoning passes
- [done] provider-neutral `AdversarialDetector` contract with typed contradiction/counterexample findings
- [done] explicit `hard` vs `soft` finding strength; findings never own verdict authority
- [done] deterministic structured-fact contradiction/counterexample detector
- [done] counterexample detection metric and adversarial fixture coverage
- semantic/model-backed discovery remains soft until independently verified
- assumption pass
- semantic-loss checks

## v0.3 — framework plugins
- extend the implemented lexical Five Whys restatement pass with evidence-aware semantic checks that remain explicitly soft unless oracle-backed
- first principles
- Feynman/simplification renderer
- framework plugin contract

## v0.4 — reproducible research
- [done] cross-model benchmark matrix across Mistral, Google, and NVIDIA Hosted NIM
- [done] token/latency/cost accounting for live provider observations
- [done] fixture-level live concurrency with provider-owned pacing/retry semantics preserved
- [done] repeated-trial stability reporting with per-trial operational isolation and mean/min/max/stddev
- [done] 5-trial Mistral + Google stability matrix plus targeted 10-trial follow-up for tied models
- deterministic vs soft-verifier reporting
- public benchmark corpus

### v0.4 research policy
- required CI remains deterministic and credential-free; live provider studies remain manual/secret-gated
- provider/model output remains an untrusted candidate and never owns verification or final-verdict authority
- operationally incomplete trials are reported explicitly and excluded from cross-trial correctness variance
- single live runs remain diagnostic observations and must not be presented as stable rankings
- NVIDIA routine coverage remains `nvidia/nemotron-3.5-lightning-30b-a3b`; other Hosted NIM model IDs are ad-hoc research inputs


## Deferred interfaces

These are intentional non-goals until the native runtime, CLI, and eval contracts mature:

- desktop UI: thin visualization/review client after artifact formats stabilize.
- public embedding API compatibility: after real consumer pressure validates the contract.
- MCP adapter: optional agent integration; never a required correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).


## Implementation constraint

All first-party components remain Rust-only. A future desktop application must use a Rust-capable native UI stack without requiring a JavaScript application runtime. Any future MCP adapter, if justified, is implemented in Rust and remains outside the core correctness boundary.
