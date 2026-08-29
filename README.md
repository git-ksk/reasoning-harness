# reasoning-harness

Experimental OSS primitives for making stochastic model output pass through an explicit, inspectable correctness process.

The project treats a model as a **candidate generator**, not an authority. Claims become usable only after deterministic checks and, where available, external oracles validate them.

> Stochastic intelligence, deterministic process.

## Research question

Can a small model become materially more reliable when its reasoning is forced through typed intermediate state, evidence binding, explicit uncertainty, adversarial passes, and deterministic acceptance gates?

This repository exists to measure that question rather than assume the answer.

## Current prototype

- `ReasoningArtifact`: claims, evidence, assumptions, and inference edges.
- Epistemic states: `known`, `supported`, `inferred`, `assumed`, `contradicted`, `unknown`.
- Deterministic validation for provenance and reference integrity.
- A pass-based harness runtime that fails closed when a pass produces invalid state.
- A first structured framework primitive for evidence-aware 5 Whys.
- Basic eval metrics for evidence coverage and unsupported accepted claims.
- Provider-neutral Rust `ModelAdapter`; model output is always outside the correctness boundary.
- Native Rust CLI (`reason verify`, `reason eval`) sharing the exact same core validators.
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
cargo run -p reasoning-harness-cli -- verify examples/artifact.json
cargo run -p reasoning-harness-cli -- eval examples/artifact.json
```

See [docs/research-plan.md](docs/research-plan.md), [docs/architecture.md](docs/architecture.md), and [ADR-0001](docs/adr/0001-interface-and-packaging-boundaries.md).
