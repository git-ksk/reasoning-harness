# Project status

## Current phase

The repository is an early research prototype. The core authority boundary, native CLI, deterministic fixture benchmark, one live provider adapter, trusted verification receipts, and a narrow Five Whys restatement pass are implemented.

This is not a claim that open-world reasoning is solved. Current correctness gains depend on deterministic structure and on trusted oracles where a hard answer exists.

## Implemented

- Rust-only core, CLI, eval, and provider adapter crates.
- Harness-owned evidence and untrusted `ReasoningCandidate` boundary.
- Deterministic structural/provenance validation.
- `accept | reject | unknown` policy.
- Trusted verification receipts that are never model-owned or model-visible.
- Receipt-backed support promotion and contradiction rejection.
- Narrow deterministic Five Whys lexical-restatement removal.
- Seven committed regression fixtures.
- Mistral live benchmark workflow, manually triggered and secret-isolated.
- GitHub CI, Dependabot configuration, contribution/security guidance, issue and PR templates.

## Known gaps

- Exact natural-language receipt binding was confirmed too brittle for live paraphrases. The current implementation now uses typed propositions and harness-owned structured facts for the built-in hard verifier; exact-string binding remains compatibility-only.
- P0 validation still requires a live Mistral benchmark after the typed-target migration before Issue #2 can be closed.
- No generic semantic contradiction detector: contradiction authority currently requires a trusted receipt/oracle.
- No generic counterexample generator/verifier.
- Five Whys pass is intentionally lexical and narrow, not a semantic causal judge.
- No assumption extraction, first-principles pass, semantic-loss verifier, or verification-budget policy yet.
- Live model results are too small and stochastic for broad model-quality claims.

## Release posture

No stable API guarantee is made yet. Breaking schema/runtime changes are acceptable while the research contracts are still being validated by fixtures and live experiments.
