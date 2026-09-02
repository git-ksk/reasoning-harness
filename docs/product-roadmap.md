# Product roadmap: evidence-grounded AI CLI

Reasoning Harness is productized first as the native Rust `reason` CLI. v0.1.0 established the
structured correctness and automation contracts; the current primary end-user direction on `main` is an
**AI-backed natural-language CLI** over the same harness-owned runtime. Users do not need to construct
internal JSON just to ask the harness to reason.

The product goal is deliberately narrower than a general-purpose agent framework:

> Give users, developers, and automation a simple AI interface whose answers are produced through an
> inspectable evidence-grounded reasoning process, with typed uncertainty, abstention, and failure
> semantics owned by the harness rather than the model.

Research continues in parallel. New mechanisms graduate into the CLI only after independent
validation and operational stabilization; the product surface does not track every experiment.

## Primary UX after v0.1.0

Completed under Issue #107; external hardening continues under #90.

The intended default experience is natural-language-first and AI-backed:

```text
natural-language task
        |
        v
Reasoning Harness
        |
        v
model generates an untrusted candidate
        |
        v
evidence / verification / D3 / sufficiency gates
        |
        +--> missing support -> bounded resolution / regeneration -> re-verify
        |
        v
grounded answer | qualified answer | unknown
```

Target ergonomics are closer to a mature AI CLI than to a JSON protocol exerciser, for example:

```bash
reason "Analyze this incident and explain the most supported cause"
reason "Review this architecture" --file architecture.md --file template.yaml
cat error.log | reason "Find the most supported root cause"
```

Exact command spelling remains provisional during v0.x. Provider/model selection should normally come
from layered config, with explicit flags available when needed. Human-readable grounded output is the
default target; `--format json` remains available for automation and inspection.

The existing structured interfaces are **not removed**. `HarnessInput`, `ReasoningCandidate`,
`ReasoningArtifact`, schema discovery, `reason run --candidate`, and `reason verify` remain supported
advanced/integration/debug surfaces and internal representations. Product effort should no longer make
users understand those representations before they can use the main AI path.

Natural-language convenience must not weaken the correctness boundary. User prose, file content, model
extractions, tool output, and prior model output do not become trusted evidence merely because the CLI
accepted them. Evidence ingestion, admission, verification, D3/sufficiency diagnostics, bounded
resolution, re-verification, and final-claim coverage remain harness-owned.

## NL-1 through NL-5 implementation status

Tracking: #109 #110 #111 #112 #113 under #107.

- **NL-1 — implemented:** direct `reason "TASK"` uses the existing layered provider/model config and the same untrusted candidate/verification path. Existing structured commands remain compatible.
- **NL-2 — implemented first boundary:** `--file`/stdin are bounded provenance-bearing untrusted context; `--fact` is explicit structured evidence and `--hypothesis` is an explicit target. Arbitrary prose is not promoted to hard evidence.
- **NL-3 — implemented adapter slice:** `--resolver-fact` exercises the existing bounded `GroundedResolutionRuntime` through an explicitly trusted local fact-store adapter, admission policy, budgets, and mandatory re-verification. Network/search/database/MCP resolver integrations remain future adapters, not correctness-core shortcuts.
- **NL-4 — implemented:** the provider renders a typed final-answer candidate, then harness-owned final-claim coverage decides whether the text can be exposed as grounded/qualified; uncovered facts are blocked and can re-enter configured bounded resolution. Renderer failure falls back to a canonical safe renderer.
- **NL-5 — completed #113:** target-aware metrics, shared candidate/initial render, claim-local sufficiency v2, and exposed-text manual review are complete. Final v5 runs on Ministral 8B and Gemma 4 31B preserve zero unsupported Harness assertions and zero missed task-target insufficiency; Gemma preserves useful qualified partial facts with full target coverage on expected-grounded cases, while Ministral remains a model-specific low-coverage/false-abstention limitation rather than a successor-gate regression; follow-up is #139.

### D3 / sufficiency bridge and NL-5 closeout

The initial residual-sufficiency research/product sequence is complete. It progressed without retuning
D3 or consuming historical holdouts as product-tuning data:

1. **RSD0 — completed #116:** fresh 12-case calibration-only corpus demonstrates a measurable residual gap: all 12 cases are D3 `permit`, while 4 are predeclared `insufficient` and 4 are `mixed`; frozen holdout-v4/v5 remain untouched.
2. **RSD1 — completed #118:** schema-constrained `sufficient | insufficient | mixed` coordinate passed the frozen one-trial calibration progression gate on both Ministral 8B and Gemini 3.5 Flash-Lite with zero false-safe and zero false-abstain decisions; this is calibration evidence only, not product authority.
3. **RSD2 — completed #121:** five-seed characterization passed for Ministral 8B and Gemma 4 31B on the product-relevant `sufficient` vs `non-sufficient` boundary; Gemini 3.5 Flash-Lite remained an operational-gate failure and was not rescued by relaxing thresholds.
4. **Fresh independent holdout — completed #125:** a new 24-case / 8-family corpus was frozen before provider observation. Seeds 7000-7004 passed every predeclared promotion gate for Ministral 8B and Gemma 4 31B; false-safe and false-abstain were both zero.
5. **Product bridge — v1 implemented #129; v2 refinement #134:** v1 established the monotonic authority boundary. Target-aware/shared-render NL-5 then showed its task-level generic requirement policy could suppress useful supported partial facts without improving task-target safety. v2 keeps the same authority boundary but uses `claim-local-answer-sufficiency-requirements-v1`, rolls back explicitly to v1, and keeps baseline as the next rollback.
6. **NL-5 — completed #113:** #133 removed renderer confounding, #134 promoted claim-local `d3-sufficiency-answer-gate-v2`, and v5 preserved exposed text for manual review. Final runs: Ministral `33576517724`, Gemma 4 `33576520136`. The successor matched baseline task-target safety/coverage in both model slices; Gemma retained its safe qualified partial answer, while Ministral's 0.75 false-target-abstention rate was already present in baseline and remains a model/product-utility follow-up rather than evidence against the gate.

RSD3 selective/conformal abstention and RSD4 relation-level causal sufficiency remain optional follow-on research. They do not block the adopted natural-language D3/sufficiency integration.

NL-5 used three arms:

```text
A. raw model
B. same model + current deterministic/grounding Harness baseline
C. same model + Harness + promoted D3/sufficiency gate
```

This separated the value of the existing harness process from the incremental value of the D3/sufficiency bridge. The final v5 report records unsupported assertions, target-level insufficiency/abstention, grounded final-claim coverage, resolution success, token/latency overhead, operational failures, safe-partial retention, and the actual exposed text used for manual review. Frozen research holdouts remain immutable and are never product-tuning data.

D3 remains the adopted semantic diagnostic runtime. The residual sufficiency coordinate crossed its frozen research promotion gate, while product wiring is versioned independently. `d3-sufficiency-answer-gate-v2` is the adopted current default and still cannot create verification authority; `d3-sufficiency-answer-gate-v1` and baseline remain explicit rollback points. Research #91 remains the provenance for classifier evidence, while NL-5 measures product-policy value on separate workloads.

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

The CLI remains the first compatibility surface. The natural-language AI path should invoke the full
native runtime, while structured JSON commands remain the advanced compatibility surface for automation,
debugging, and third-party integrations. Neither path may invent lower-level bypass APIs.

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

Product readiness requires workloads that are not frozen research holdouts. Execute this acceptance
phase only after the D3/sufficiency bridge has either been promoted into the natural-language runtime or
explicitly rejected by the research promotion gate. For the promoted path, prefer a three-arm comparison:
**raw model vs current Harness baseline vs the same Harness with the promoted D3/sufficiency gate**.
Use separate dogfood/reference workloads and answer:

- does the harness reduce unsupported final assertions in realistic use?;
- how often does it correctly abstain, and how often does it abstain unnecessarily?;
- how often can bounded resolution convert an initially unsupported answer into a verified one?;
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
7. breaking-change policy and security/secret-handling guidance are explicit;
8. the natural-language AI path preserves the same verification/finalization authority boundaries and
   has product acceptance evidence against a raw-model baseline.

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

The adopted semantic baseline is D3, and the completed #91 residual-sufficiency program produced the
versioned `d3-sufficiency-answer-gate-v2` product bridge after a separate independent holdout and NL-5.
D3, the residual classifier, and the product requirement policy retain separate identities and rollback
boundaries. Frozen holdout-v4/v5 and the sufficiency holdout remain immutable research history and are
never product-tuning corpora.

## Deferred product surfaces

- **Public Rust embedding API:** after real CLI consumers validate the correct compatibility
  boundary.
- **MCP adapter:** optional integration invoking the full runtime; never evidence that the caller's
  entire agent loop is verified.
- **Interactive CLI (`reason shell` / `reason repl`):** demand-gated after CLI-4 dogfood. If adopted, it is a thin stateful session over `ReasoningThread`/checkpoint/replay and the same product runtime, not a separate chat authority or evidence shortcut.
- **Desktop UI:** thin inspection/review client only after artifact and CLI contracts are stable.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md),
[roadmap](roadmap.md), and [research plan](research-plan.md).
