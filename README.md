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
- Provider-neutral `ModelAdapter`; model output is always outside the correctness boundary.

## What this is not

- A prompt collection.
- A model-specific agent framework.
- A claim that LLM reasoning can be made mathematically correct in open-world tasks.
- A replacement for deterministic oracles such as compilers, tests, schemas, policy engines, or proof checkers.

## Model strategy

The harness is intentionally model-agnostic. Cheap/free inference can be useful for candidate generation because correctness comes from the surrounding protocol and validators, not from trusting a particular provider. A Mistral adapter is a reasonable early experiment, but provider code does not belong in the core correctness boundary.

## Development

Node.js 22+ is the supported runtime.

```bash
npm ci
npm test
npm run demo
```

See [docs/research-plan.md](docs/research-plan.md) and [docs/architecture.md](docs/architecture.md).
