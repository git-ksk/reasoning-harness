# ADR-0002: Implement all first-party components in Rust

- Status: Accepted
- Date: 2026-08-29

## Context

The project is still at the prototype stage and has no frozen public API. Its purpose is to place a deterministic, inspectable shell around stochastic model behavior. Splitting the implementation across TypeScript, Rust, and IPC boundaries this early would add protocol and packaging complexity before those boundaries provide research value.

The CLI is also expected to become a primary interface for local use, CI, reproducible evaluation, and eventually standalone distribution.

## Decision

All first-party components are implemented in Rust:

- harness runtime and state machine;
- Reasoning IR and epistemic types;
- deterministic validators and verification passes;
- evaluation and benchmark tooling;
- CLI;
- model-provider adapters;
- deterministic oracle adapters;
- any future desktop application;
- any future optional MCP or other integration adapter.

External models remain outside the trusted computing base and may be hosted by any provider. They are invoked through Rust adapter boundaries.

The repository does not require Node.js, TypeScript, Python, or another language runtime for normal build, test, CLI, or future desktop execution.

## Workspace shape

```text
crates/
  reasoning-harness-core/   trusted runtime primitives
  reasoning-harness-cli/    native `reason` executable
examples/                    language-neutral fixtures
```

Additional crates are introduced only when a real ownership boundary appears. The project will not create a large workspace merely to mirror conceptual modules.

## Desktop implication

The desktop client is deferred, but it must preserve the Rust-only decision. Candidate Rust-native/Rust-first UI stacks may be evaluated later; the toolkit is intentionally not frozen by this ADR.

## Why now

The migration cost is currently minimal: the prototype has only a small typed IR, validator set, one framework trace, and a few tests. Making the language decision after provider adapters, CLI contracts, or desktop work exist would be substantially more expensive.

## Consequences

### Positive

- one type system across runtime, CLI, eval, and future UI;
- no IPC boundary merely to connect the CLI to the harness;
- straightforward single-binary CLI distribution later;
- strong enums and exhaustive matching for epistemic state and verdicts;
- deterministic tooling can remain small and native;
- cross-platform CI can exercise the same implementation users run.

### Costs

- some model-provider SDKs may be more mature in Python or TypeScript;
- provider integrations may require direct HTTP implementations;
- rapid experimentation can require more explicit types than scripting languages.

These costs are acceptable because provider SDK convenience is not part of the research question. Provider-specific code must not define correctness semantics.
