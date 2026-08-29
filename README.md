# reasoning-harness

Experimental OSS primitives for making stochastic model output pass through an explicit, inspectable correctness process.

The project treats a model as a **candidate generator**, not an authority. Claims become usable only after deterministic checks and, where available, external oracles validate them.

> Stochastic intelligence, deterministic process.

## Research question

Can a small model become materially more reliable when its reasoning is forced through typed intermediate state, evidence binding, explicit uncertainty, adversarial passes, and deterministic acceptance gates?

This repository exists to measure that question rather than assume the answer.

## Current prototype

- Harness-owned `HarnessInput`: task plus immutable supplied evidence.
- Untrusted `ReasoningCandidate`: model-proposed claims, epistemic states, and inference edges; it cannot create evidence.
- `ReasoningArtifact`: harness-materialized task, evidence, verification receipts, claims, and inference edges.
- Epistemic states: `known`, `supported`, `inferred`, `assumed`, `contradicted`, `unknown`.
- Deterministic validation for provenance, receipt binding, and reference integrity.
- Typed adversarial findings for contradictions and counterexamples with explicit `hard` vs `soft` strength; discovery cannot directly force a verdict.
- Provider-neutral `AdversarialDetector` adapters; the first hard detector operates only on harness-owned structured facts.
- Trusted verification receipts for oracle-backed support promotion or contradiction; receipts are never model-owned or model-visible.
- A narrow deterministic Five Whys pass that removes lexical symptom-restatement edges without pretending to be a semantic causal judge.
- A pass-based harness runtime that fails closed when a pass produces invalid state.
- A first structured framework primitive for evidence-aware 5 Whys.
- Basic eval metrics for evidence coverage and unsupported accepted claims.
- Provider-neutral Rust `ModelAdapter`; model output is always outside the correctness boundary.
- Initial Mistral HTTP adapter in a separate Rust crate, using structured candidate output without granting provider authority.
- Native Rust CLI (`reason run`, `reason verify`, `reason eval`) sharing the exact same core validators and acceptance policy.
- Native runtime is the correctness owner; CLI and eval are the first supported interfaces.

## What this is not

- A prompt collection.
- A model-specific agent framework.
- A claim that LLM reasoning can be made mathematically correct in open-world tasks.
- A replacement for deterministic oracles such as compilers, tests, schemas, policy engines, or proof checkers.

## Model strategy

The harness is intentionally model-agnostic. Cheap/free inference can be useful for candidate generation because correctness comes from the surrounding protocol and validators, not from trusting a particular provider. A Mistral adapter is a reasonable early experiment, but provider code does not belong in the core correctness boundary.

## Development

Rust 1.88+ is the supported toolchain. The repository intentionally has no Node.js/TypeScript runtime dependency.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p reasoning-harness-cli -- run --input examples/input.json --candidate examples/candidate.json --format json
cargo run -p reasoning-harness-cli -- run --input examples/input.json --candidate examples/candidate.json --receipts examples/receipts.json --format json
cargo run -p reasoning-harness-cli -- verify examples/artifact.json --format json
cargo run -p reasoning-harness-cli -- eval examples/artifact.json
cargo run -p reasoning-harness-cli -- eval fixtures --format human

# Optional live candidate generation (requires MISTRAL_API_KEY)
cargo run -p reasoning-harness-cli -- run --input examples/input.json --provider mistral --model ministral-8b-latest --format json
```

See [project status](docs/project-status.md), [research plan](docs/research-plan.md), [benchmark design](docs/benchmark.md), [architecture](docs/architecture.md), [prior art](docs/prior-art.md), and [ADR-0001](docs/adr/0001-interface-and-packaging-boundaries.md).
