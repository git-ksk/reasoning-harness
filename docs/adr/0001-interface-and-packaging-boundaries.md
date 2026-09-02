# ADR-0001: Interface and packaging boundaries

- Status: Accepted
- Date: 2026-08-29

## Context

This project is a native reasoning harness, not a prompt pack and not a tool that an
external agent may optionally call. Its primary job is to own the execution protocol
around a stochastic model: state transitions, validation, verification passes,
acceptance policy, retry/budget policy, and final epistemic status.

Several interfaces are plausible: a library API, CLI, evaluation runner, desktop UI,
and MCP server. Treating all of them as equal product surfaces would blur the trusted
boundary and encourage UI or integration concerns to leak into core semantics.

## Decision

The project will use the following responsibility order:

1. **Native Harness Runtime** — product/core correctness boundary.
2. **CLI** — first supported user and automation interface.
3. **Eval Runner** — first-class research and regression interface.
4. **Desktop UI** — deferred, thin human-inspection client.
5. **Embedding API** — supported after runtime contracts stabilize.
6. **MCP adapter** — optional and deferred; never required for correctness.

The runtime owns model invocation through a replaceable `ModelAdapter`. A model is a
candidate generator inside the harness; it does not own the harness loop.

## Runtime boundary

Conceptually, all interfaces converge on one operation:

```ts
run({
  input,
  model,
  passes,
  policy,
}): Promise<ReasoningArtifact>
```

The exact public API may evolve during v0.x, but these ownership rules are stable:

- Harness-owned input supplies task and evidence; model candidates cannot create trusted evidence.
- Model-proposed strong epistemic states are not final authority.
- The runtime owns execution order.
- The runtime validates state after every pass.
- The runtime decides `accept | reject | unknown` according to explicit policy.
- Model adapters cannot bypass validators or acceptance gates.
- Renderers cannot upgrade epistemic status.
- Interfaces cannot redefine core reasoning semantics.

## Interface responsibilities

### Native Harness Runtime

Owns:

- `ReasoningArtifact` and related typed intermediate state.
- epistemic state transitions.
- deterministic validators.
- reasoning/verification pass execution.
- model-adapter lifecycle.
- retry, timeout, and verification-budget policy.
- acceptance/rejection/unknown decisions.
- deterministic oracle integration.

Does not own:

- terminal presentation.
- desktop presentation.
- provider-specific product UX.
- MCP discovery semantics.

### CLI

The CLI is the first supported product surface because it is useful interactively,
in shell pipelines, and in CI without introducing another orchestration layer. v0.1.0 first exposed
structured contracts; the primary human-facing direction after that foundation is an AI-backed
natural-language command that internally constructs and traverses the same runtime boundaries. The
structured JSON commands remain supported advanced/integration/debug surfaces rather than being
removed or bypassed.

Initial command semantics:

```text
run       execute a harness workflow
verify    validate an existing reasoning artifact
explain   render a verified artifact for a target audience/style
eval      execute a fixture/benchmark suite
```

Command names are provisional during v0.x; their responsibility boundaries are not.

CLI requirements:

- machine-readable JSON output must be available.
- human-readable output must not be the only representation.
- non-zero exit status for structural/runtime failure.
- epistemic `unknown` is not automatically a process failure.
- CLI flags configure policy; they do not bypass invariant checks.

### Eval Runner

The eval runner is first-class rather than a test helper. It exists to measure whether
a harness change improves or degrades behavior independently of model marketing claims.

It owns:

- fixture loading.
- baseline/candidate comparison.
- deterministic metrics.
- token/latency/cost observations when adapters expose them.
- regression thresholds.
- explicit separation of hard metrics from soft model-judge metrics.

The eval runner uses the same native runtime as the CLI.

### Desktop UI

A desktop application is deferred until the runtime and artifact formats are useful
without a GUI.

Its intended role is human inspection and review:

- claim/evidence graph visualization.
- assumption and unknown inspection.
- pass timeline and rejection reasons.
- comparison between runs/models/policies.
- explicit human review where policy permits it.

The desktop app must remain a thin client over the native runtime. It must not acquire
its own hidden prompt flow, validator implementation, or competing state machine.

### Embedding API

The internal Rust crate boundaries already make embedding possible, but a stable public
library API is not a v0.1 goal. Public compatibility promises will be made only after
real CLI/eval use reveals the correct runtime contract.

When stabilized, the embedding API should expose the same runtime semantics rather than
low-level shortcuts that allow consumers to skip validation or acceptance policy.

### MCP adapter

MCP is explicitly optional and deferred.

MCP can later expose selected capabilities to other agents, but an MCP caller controls
whether a tool is invoked. Therefore an MCP server cannot, by itself, enforce this
project's native correctness process over the caller's entire agent loop.

Consequences:

- MCP is an integration adapter, not the core harness.
- MCP tools may invoke the full native runtime.
- MCP tools must not claim that the caller's overall reasoning is verified merely
  because one tool invocation passed.
- No v0.x core API will be distorted solely to fit MCP tool schemas.

## Packaging strategy

Do not introduce a monorepo or multiple publishable packages before there is a concrete
consumer need.

The repository is a small Cargo workspace with ownership boundaries that already exist in code:

```text
crates/reasoning-harness-core/   runtime, IR, validators, passes, eval primitives
crates/reasoning-harness-cli/    native `reason` executable
crates/reasoning-harness-providers/ provider-specific HTTP adapters
examples/                        executable artifacts
docs/                            architecture, ADRs, research notes
```

Additional provider, oracle, desktop, or integration crates are added only when they have a real independent dependency or ownership boundary. The workspace must not grow merely to mirror conceptual modules.

A future package split is allowed if independent versioning or external embedding makes
it valuable. Package topology is not part of the correctness model.

## Model-provider policy

Provider SDKs must not enter the trusted core semantics.

A provider adapter may:

- send a candidate-generation request.
- return candidate output and usage metadata.
- expose provider errors in a normalized form.

A provider adapter may not:

- mark a claim as verified.
- bypass a required pass.
- decide final acceptance.

This keeps low-cost, free-tier, local, and premium models comparable under the same
harness protocol.

## Rejected alternatives

### Skill as the primary product

Rejected because a skill primarily supplies instructions and context to an agent. The
agent still owns whether and how faithfully those instructions are followed.

### MCP as the primary product

Rejected because MCP supplies capabilities to an external agent; it does not own that
agent's full execution loop or acceptance policy.

### Desktop-first architecture

Rejected because UI concerns would mature before the reasoning protocol and would make
research automation and CI harder.

### Public library-first architecture

Rejected because early public API compatibility would freeze abstractions before the
CLI and eval workflows have validated them.

## Consequences

Positive:

- correctness semantics have one owner.
- CLI and CI can mature the harness cheaply and reproducibly.
- model/provider experiments remain replaceable.
- a future desktop UI can visualize stable artifacts rather than invent semantics.
- MCP can be added without changing the project's identity.

Costs:

- desktop usability arrives later.
- public embedding compatibility is intentionally delayed.
- some integrations may need to shell out to the CLI during early v0.x research.
