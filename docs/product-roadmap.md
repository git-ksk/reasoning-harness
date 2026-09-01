# Product roadmap: native CLI

Reasoning Harness is productized first as the native Rust `reason` CLI. The CLI is not a thin demo
around a research library: it is the first supported interface to the same harness-owned runtime,
validation, evidence, verification, abstention, resolution, and finalization boundaries used by the
research/evaluation surfaces.

The product goal is deliberately narrower than a general-purpose agent framework:

> Give developers and automation a reproducible way to run stochastic model output through an
> inspectable evidence-grounded reasoning process, with typed uncertainty and failure semantics.

Research continues in parallel. New mechanisms graduate into the CLI only after independent
validation and operational stabilization; the product surface does not track every experiment.

## Current baseline

Already available:

- external-preview `reason` v0.1.0 executable with supported `run`, `verify`, `semantic-check`, and `schema` product commands; research/evaluation commands remain separate;
- provider-neutral core runtime and typed `ReasoningArtifact`;
- provider adapters for Mistral, Google, and NVIDIA outside the correctness authority boundary;
- bounded resolution/finalization, evidence qualification, policy, checkpoint/replay, and typed
  diagnostics;
- adopted semantic runtime profile `semantic-decidability-d3-v1`;
- explicit `soft-semantic-v3` rollback profile;
- credential-free deterministic CI plus separate live provider smoke/research workflows.

v0.1.0 is the first externally consumable preview. Its versioned machine contracts and supported product commands are compatibility-tracked under the v0.x support policy, but this is not yet a v1.0 stability promise.

## CLI-1 — supported command and data contract

Tracking: Issue #90.

The first product milestone makes the existing CLI predictable for humans, shell pipelines, and CI:

- [implemented #90] define `run`, `verify`, and `schema` as supported product commands separately from research-only/evaluation commands;
- [implemented #90] stabilize `-` stdin plus file/stdout behavior for supported JSON inputs, with at most one stdin consumer per invocation;
- [implemented #90] define `reason-cli-output-v1` plus `reasoning-artifact-v1` / `reasoning-candidate-v1` machine-readable contract identities and schema discovery;
- [implemented #90] document exit-code semantics: successful `accept | reject | unknown` execution is exit 0, command/runtime/validation failure is exit 1, and CLI parse failure is exit 2;
- [implemented #93] expose adopted D3 through the separate `reason semantic-check` product command, with canonical runtime identity, explicit v3 rollback, and typed operational failure kept outside semantic/final-verdict authority;
- [implemented #100] normalize machine-readable product failures for `run`/`verify` plus the existing `semantic-check` failure surface; JSON automation keeps input/config/harness/provider failure classes separate from epistemic outcomes;
- [implemented #94] schema-backed `reason-config-v1` layers explicit CLI flags > explicit config > current-project config > user config > defaults; `--no-config` supports hermetic runs, unknown fields fail closed, and provider secrets remain environment-owned by default;
- keep `--format json` suitable for automation and human output explicitly non-authoritative;
- add a short install/quickstart path and copy-paste shell/CI examples.

The CLI must never expose a flag that skips core validation, verification, acceptance, or
finalization invariants.

## CLI-2 — install, release, and compatibility

Make `reason` straightforward to obtain and safe to upgrade:

- [implemented #97] reproducible `cargo install --git` path plus tag-driven standalone GitHub Release artifacts containing only the supported `reason` binary;
- [implemented #97] release tags are required to match the CLI semver and releases include SHA-256 checksums;
- [implemented #97] credential-free product smoke covers Linux x64, macOS arm64, macOS Intel, and Windows x64;
- compatibility tests for stable JSON/exit semantics;
- [implemented #102] changelog/migration discipline for intentional breaking changes during v0.x;
- [implemented #102] explicit product/platform/provider support policy separating provider operations from the provider-neutral correctness boundary.

A package split is not required. The current Cargo workspace remains the default until an actual
external consumer creates an independent versioning or dependency boundary.

## CLI-3 — integration and observability

The CLI remains the first compatibility surface. Integrations should initially call the full native
runtime through the CLI rather than invent lower-level bypass APIs.

Product telemetry should make the harness useful to operators without turning model confidence into
correctness authority:

- runtime/profile/config identity;
- `accept | reject | unknown` and abstention/unknown reasons;
- grounded final-claim coverage and unsafe-final-answer counters;
- deterministic gate interventions and prevented unsafe assertions where measurable;
- provider/protocol/quota/rate-limit/timeout failure classes;
- attempts, retries, tokens, and latency;
- explicit separation of semantic outcome from operational completeness.

Reference resolver/oracle integrations may be documented when they preserve evidence admission,
trusted verification, and mandatory re-verification. Public embedding compatibility and MCP remain
later adapters rather than correctness boundaries.

## CLI-4 — real-workload adoption evidence

Product readiness requires workloads that are not frozen research holdouts. Use separate dogfood and
reference workloads to answer:

- does the harness prevent unsupported final assertions in realistic use?;
- how often does it abstain unnecessarily?;
- which missing-support patterns recur in practice?;
- what are the latency/token/retry costs of the safety process?;
- can users understand and act on `unknown`, abstention, and failure telemetry?;

Real-workload failures may seed **new calibration corpora**, but they must never be used to repair or
retune observed frozen holdouts.

CLI-4 also decides whether an interactive session surface is worth productizing. Do not add a chat-like
REPL merely for parity with general-purpose agent CLIs. First observe whether real users repeatedly need
to add evidence, revisit an `unknown` result, inspect why the harness abstained, or continue the same
reasoning state across multiple commands. If that demand is measurable, design a thin `reason shell` /
`reason repl` layer over the existing runtime and `ReasoningThread` checkpoint/replay model. Interactive
turns must preserve the same authority boundaries: conversation history is not trusted evidence, prior
model output cannot self-promote, policy/evidence changes trigger re-validation, and every assertive
result still crosses the normal harness-owned verification/finalization path.

## v1.0 readiness gate

Do not present the CLI as stable/v1.0 until all of the following are true:

1. supported command, JSON, exit-code, and configuration contracts are compatibility-tested;
2. install/release/upgrade flow is reproducible and documented;
3. deterministic CI plus bounded live runtime smoke gates are green;
4. at least two distinct real workload classes have product acceptance evidence;
5. runtime identity, rollback, typed failures, and operational-completeness semantics are documented
   and tested;
6. research/eval commands are clearly distinguished from the supported product surface;
7. breaking-change policy and security/secret-handling guidance are explicit.

## Research-to-product promotion gate

The research track is allowed to move faster than the product track. A new reasoning mechanism does
not become part of the stable CLI merely because it improves calibration metrics.

Promotion order:

```text
fresh calibration-only hypothesis
  -> pre-observation spec/label review
  -> calibrated candidate
  -> fresh independently frozen holdout
  -> operational stabilization + typed failures
  -> explicit runtime profile + rollback
  -> CLI compatibility/observability coverage
  -> reversible product adoption
```

The currently adopted D3 profile is the product baseline while Issue #91 explores residual evidence
sufficiency. Frozen holdout-v4/v5 remain immutable research history and are never product-tuning
corpora.

## Deferred product surfaces

- **Public Rust embedding API:** after real CLI consumers validate the correct compatibility
  boundary.
- **MCP adapter:** optional integration invoking the full runtime; never evidence that the caller's
  entire agent loop is verified.
- **Interactive CLI (`reason shell` / `reason repl`):** demand-gated after CLI-4 dogfood. If adopted, it is a thin stateful session over `ReasoningThread`/checkpoint/replay and the same product runtime, not a separate chat authority or evidence shortcut.
- **Desktop UI:** thin inspection/review client only after artifact and CLI contracts are stable.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md),
[roadmap](roadmap.md), and [research plan](research-plan.md).
