# reasoning-harness

Experimental OSS primitives for making stochastic model output pass through an explicit, inspectable correctness process and, over time, converge on grounded final answers.

The project treats a model as a **candidate generator**, not an authority. Claims become usable only after deterministic checks and, where available, external oracles validate them. The native harness runtime owns the reasoning protocol around the model rather than acting only as a post-hoc grader.

> Stochastic intelligence, deterministic process.

## Research question

Can a small or inexpensive model become materially more reliable when its reasoning is forced through typed intermediate state, evidence binding, explicit uncertainty, adversarial passes, deterministic acceptance gates, and bounded resolution/re-verification before finalization?

The long-term product question is not only whether the harness can diagnose bad reasoning. It is whether initially unsupported reasoning can be converted into a grounded answer by identifying exactly what support is missing, acquiring or verifying additional evidence through external adapters, re-running the same authority boundaries, and refusing to fabricate completion when support cannot be established.

This repository exists to measure those questions rather than assume the answer.

## Product direction

Reasoning Harness is evolving toward an **evidence-grounded reasoning runtime**:

```text
task + harness-owned evidence
          |
          v
candidate generation
          |
          v
ground + verify + diagnose
          |
          +--> supported enough --> finalization --> grounded answer
          |
          +--> unresolved --> resolution request --> external evidence/verifier
          |                                      |
          |                                      v
          +--------------------------- revise/regenerate --> re-verify
          |
          +--> refuted --> discard/revise --> re-verify
```

`unknown` remains a valid outcome. The runtime must be allowed to stop, qualify an answer, or abstain when a trusted resolver is unavailable or a configured budget is exhausted.

Retrieval, web search, databases, tests, compilers, MCP servers, and human review may supply candidate evidence or verifier results through adapters. They do not automatically become correctness authorities. The harness keeps evidence provenance, verification, state transitions, and finalization policy inside the native runtime boundary.

See [ADR-0002](docs/adr/0002-grounded-resolution-and-finalization.md) for the target resolution and finalization loop.

## Current prototype

- Harness-owned `HarnessInput`: task plus immutable supplied evidence, explicit assumptions, and optional evidence-qualification requirements.
- Untrusted `ReasoningCandidate`: model-proposed claims, epistemic states, and inference edges; it cannot create evidence.
- `ReasoningArtifact`: harness-materialized task, evidence, verification receipts, claims, inference edges, and typed diagnostics.
- Epistemic states: `known`, `supported`, `inferred`, `assumed`, `contradicted`, `unknown`.
- Deterministic validation for provenance, receipt binding, and reference integrity.
- Typed adversarial findings for contradictions and counterexamples with explicit `hard` vs `soft` strength; discovery cannot directly force a verdict.
- Provider-neutral `AdversarialDetector` adapters; the first hard detector operates only on harness-owned structured facts.
- Trusted verification receipts for oracle-backed support promotion or contradiction; receipts are never model-owned or model-visible.
- Evidence-aware causal diagnostics with explicit support/refutation/unknown assessments while keeping causal findings outside final-verdict authority.
- Harness-owned explicit assumptions plus unsupported-premise diagnostics that distinguish trusted support, allowed assumptions, unsupported typed premises, and unbound premises.
- Provider-neutral temporal/scope/provenance evidence qualification; stale, out-of-scope, or insufficient-authority facts cannot create built-in hard receipts when requirements are present.
- A pass-based harness runtime that fails closed when a pass produces invalid state.
- Provider-neutral bounded resolution with explicit resolver/admission/verifier boundaries, per-run and per-request budgets, mandatory re-verification, terminal-state accounting, and grounded final-claim coverage.
- Composable `ReasoningPolicy` constraints for authority/scope/time/resolver capabilities plus immutable-snapshot dependency invalidation when policy changes.
- Durable `ReasoningThread` event/checkpoint replay with fail-closed interrupt/resume, non-destructive fork lineage, and abstract persistence storage.
- Deterministic metamorphic robustness checks, repeated-trial diagnostic stability reporting, calibrated soft semantic-judge evaluation, and an optional model-backed soft judge over the same provider-neutral `ModelAdapter` boundary.
- A frozen semantic-runtime identity layer with `soft-semantic-v3` rollback and a staged `semantic-decidability-d3-v1` profile; D3 runtime adoption is intentionally separate from stabilization.
- A versioned corpus-v1 manifest covering 41 committed claim, causal, assumption, and evidence-qualification cases with stable IDs and category/difficulty strata.
- Provider-neutral Rust `ModelAdapter`; model output is always outside the correctness boundary.
- Provider adapters in a separate Rust crate for Mistral, Google Gemini/AI Studio, and NVIDIA Hosted NIM; all provider output remains outside the verification authority boundary.
- Native Rust CLI (`reason run`, `reason verify`, `reason eval`, `reason eval-resolution`) sharing the same core correctness contracts.
- Native runtime is the correctness owner; CLI and eval are the first supported interfaces.

The provider-neutral bounded resolution/finalization core is implemented and covered by deterministic controlled scenarios. Concrete web, database, MCP, and human-review resolvers remain external integrations rather than correctness-core features.

## What this is not

- A prompt collection.
- A model-specific agent framework.
- A post-hoc LLM judge that can self-certify another model's output.
- A claim that LLM reasoning can be made mathematically correct in open-world tasks.
- A replacement for deterministic oracles such as compilers, tests, schemas, policy engines, or proof checkers.
- A general-purpose web crawler or RAG framework embedded in the correctness core.

## Model strategy

The harness is intentionally model-agnostic. Cheap/free inference can be useful for candidate generation because correctness comes from the surrounding protocol and validators, not from trusting a particular provider. Provider code does not belong in the core correctness boundary.

In the grounded runtime, the same rule also applies to repair and rendering: a model may regenerate reasoning or render a final answer, but every new factual proposition remains untrusted until it crosses the normal harness-owned verification boundary.

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
cargo run -p reasoning-harness-cli -- eval-resolution fixtures/resolution --format human
cargo run -p reasoning-harness-cli -- eval-judges fixtures/semantic-judges --format human

# Optional live candidate generation
# Mistral: requires MISTRAL_API_KEY
cargo run -p reasoning-harness-cli -- run --input examples/input.json --provider mistral --model ministral-8b-latest --format json

# NVIDIA Hosted NIM: requires NVIDIA_API_KEY
cargo run -p reasoning-harness-cli -- run --input examples/input.json --provider nvidia --model nvidia/nemotron-3.5-lightning-30b-a3b --format json
```

See [project status](docs/project-status.md), [research plan](docs/research-plan.md), [benchmark design](docs/benchmark.md), [corpus versioning](docs/corpus-versioning.md), [semantic judge calibration](docs/semantic-judge-calibration.md), [reasoning policy](docs/reasoning-policy.md), [evidence qualification](docs/evidence-qualification.md), [grounded resolution](docs/grounded-resolution.md), [live benchmark CI](docs/live-benchmark.md), [architecture](docs/architecture.md), [prior art](docs/prior-art.md), [ADR-0001](docs/adr/0001-interface-and-packaging-boundaries.md), and [ADR-0002](docs/adr/0002-grounded-resolution-and-finalization.md).
