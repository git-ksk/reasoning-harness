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

## Current product path

The current default experience on `main` is natural-language-first and AI-backed:

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
evidence / verification / semantic + answer-safety gates
        |
        +--> missing support -> bounded resolution / regeneration -> re-verify
        |
        v
grounded answer | qualified answer | unknown
```

The existing structured interfaces are **not removed**. `HarnessInput`, `ReasoningCandidate`,
`ReasoningArtifact`, schema discovery, `reason run --candidate`, and `reason verify` remain supported
advanced/integration/debug surfaces and internal representations. Product effort should no longer make
users understand those representations before they can use the main AI path.

Natural-language convenience must not weaken the correctness boundary. User prose, file content, model
extractions, tool output, and prior model output do not become trusted evidence merely because the CLI
accepted them. Evidence ingestion, admission, verification, semantic/answer-safety diagnostics, bounded
resolution, re-verification, and final-claim coverage remain harness-owned.

## Active priorities

1. **Bounded resolver target closure (#159):** implemented in the successor candidate line: exact Harness-owned unresolved hypotheses/evidence requirements are prioritized ahead of candidate-owned unresolved claims, while resolver class, budget, admission, qualification, and mandatory re-verification remain unchanged.
2. **Renderer downgrade recovery (#160):** implemented in successor candidate `a020b5925497ff3fdf200a9622270fa1889a6aa1`: if the renderer emits the same exact requested authorized target as `uncertain`, deterministically recover from artifact authority while preserving `Unknown`/`Reject`, qualification, adversarial, and answer-safety boundaries.
3. **Dependency-aware target-local recovery (#164):** implemented in successor candidate `993874fa0051d06a02c8db8f7a220a2ac7773c17`: artifact-global `Reject` remains unchanged, while an exact directly verified target may be emitted only as target-only `QualifiedPartialAnswer` when typed blocker/dependency/evidence isolation is demonstrable; ambiguous coupling fails closed.
4. **Provider reliability and resumable evaluation (#126):** implemented as an operational-only successor layer: Google transient 5xx/isolated-empty-output retries are narrowly bounded, actual provider attempts are propagated through telemetry, and `reason-product-dogfood` supports exact-identity case-level checkpoint/resume while preserving interrupted operational failures outside semantic scoring. The semantic successor candidate remains `993874fa0051d06a02c8db8f7a220a2ac7773c17`.
5. **External CLI hardening (#90) and real-workload UX (#139):** closeout complete on current `main`: process-level compatibility is pinned across all four supported release platforms, current live semantic/runtime smoke is green, and the successor Ministral 8B product rerun recovered Harness target coverage from 0.25 to 1.00 with zero unsupported grounded claims and zero missed target insufficiency.

The current answer-safety behavior and semantic runtime have exact machine configuration IDs for rollback and reproducibility, but those IDs are not product phase names. See [Terminology and naming](terminology.md).

## Historical research provenance

Earlier work used issue-scoped labels such as `NL-1`–`NL-5`, `D1`–`D3`, and `RSD0`–`RSD4`. They remain useful when tracing the research record, but they are **not** a project-wide version sequence and are not used to name new active product phases.

The completed sequence established:

- the natural-language product path over the same verification/finalization boundary (#107/#109–#113);
- an independently calibrated semantic runtime and conservative rollback (#73/#84/#85);
- a residual evidence-sufficiency classifier that cannot create authority (#91/#116/#118/#121/#125);
- the current claim-local answer-safety configuration with explicit rollback (#129/#134);
- target-aware/shared-render product dogfood and exposed-text review (#113/#131/#133/#137).

Exact historical phase labels, frozen run identities, and machine configuration IDs remain in the research/evidence documents so provenance is not rewritten.

## Current baseline

Already available:

- external-preview `reason` v0.1.0 executable with supported `run`, `verify`, `semantic-check`, and `schema` product commands; research/evaluation commands remain separate;
- provider-neutral core runtime and typed `ReasoningArtifact`;
- provider adapters for Mistral, Google, and NVIDIA outside the correctness authority boundary;
- bounded resolution/finalization, evidence qualification, policy, checkpoint/replay, and typed
  diagnostics;
- current semantic runtime plus an explicit characterized rollback profile (exact machine IDs remain stable and documented);
- credential-free deterministic CI plus separate live provider smoke/research workflows.

v0.1.0 is the first externally consumable preview. Its versioned machine contracts and supported product commands are compatibility-tracked under the v0.x support policy, but this is not yet a v1.0 stability promise.

## Historical milestone: supported command and data contract

Tracking: Issue #90.

The first product milestone makes the existing CLI predictable for humans, shell pipelines, and CI:

- [implemented #90] define `run`, `verify`, and `schema` as supported product commands separately from research-only/evaluation commands;
- [implemented #90] stabilize `-` stdin plus file/stdout behavior for supported JSON inputs, with at most one stdin consumer per invocation;
- [implemented #90] define `reason-cli-output-v1` plus `reasoning-artifact-v1` / `reasoning-candidate-v1` machine-readable contract identities and schema discovery;
- [implemented #90] document exit-code semantics: successful `accept | reject | unknown` execution is exit 0, command/runtime/validation failure is exit 1, and CLI parse failure is exit 2;
- [implemented #93] expose the semantic runtime through the separate `reason semantic-check` product command, with canonical machine identity, explicit rollback, and typed operational failure kept outside semantic/final-verdict authority;
- [implemented #100] normalize machine-readable product failures for `run`/`verify` plus the existing `semantic-check` failure surface; JSON automation keeps input/config/harness/provider failure classes separate from epistemic outcomes;
- [implemented #94] schema-backed `reason-config-v1` layers explicit CLI flags > explicit config > current-project config > user config > defaults; `--no-config` supports hermetic runs, unknown fields fail closed, and provider secrets remain environment-owned by default;
- keep `--format json` suitable for automation and human output explicitly non-authoritative;
- add a short install/quickstart path and copy-paste shell/CI examples.

The CLI must never expose a flag that skips core validation, verification, acceptance, or
finalization invariants.

## Historical milestone: install, release, and compatibility

Make `reason` straightforward to obtain and safe to upgrade:

- [implemented #97] reproducible `cargo install --git` path plus tag-driven standalone GitHub Release artifacts containing only the supported `reason` binary;
- [implemented #97] release tags are required to match the CLI semver and releases include SHA-256 checksums;
- [implemented #97] credential-free product smoke covers Linux x64, macOS arm64, macOS Intel, and Windows x64;
- [implemented #90] cross-platform process-level compatibility tests pin `reason-cli-output-v1`, supported stdin behavior, schema contract IDs, `unknown` as exit 0, typed operational failure as exit 1, and CLI usage failure as exit 2;
- [implemented #102] changelog/migration discipline for intentional breaking changes during v0.x;
- [implemented #102] explicit product/platform/provider support policy separating provider operations from the provider-neutral correctness boundary.

A package split is not required. The current Cargo workspace remains the default until an actual
external consumer creates an independent versioning or dependency boundary.

## Integration and observability

The CLI remains the first compatibility surface. The natural-language AI path invokes the full
native runtime, while structured JSON commands remain the advanced compatibility surface for automation,
debugging, and third-party integrations. Neither path may invent lower-level bypass APIs.

Product telemetry should make the harness useful to operators without turning model confidence into
correctness authority. Active provider-reliability work is tracked in #126: retries must remain bounded,
typed, observable, and strictly operational rather than becoming semantic `unknown` or abstention.

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

## Real-workload adoption evidence

Product readiness requires workloads that are not frozen research holdouts. The natural-language acceptance discipline uses a three-arm comparison: **raw model vs current Harness baseline vs the same Harness with the current answer-safety gate**. The 2026-09-04 successor revalidation on the six-case incident-analysis + architecture-review product workload (Actions run `33822567155`, main `5c5701f77df9dd507c3949294708f8c07a054064`) closed #139: Ministral 8B raw target coverage remained 0.25, while both Harness arms reached 1.00 target coverage with false target abstention 0, unsupported grounded claims 0, and missed target insufficiency 0. Expected-unknown cases remained safely unresolved. The human `reason` path supplies deterministic evidence-insufficiency guidance for unresolved/verification-required states rather than promoting model prose into authority.

The #147 product-evaluation generation is now closed and frozen. Stage B completed on the unchanged 24-case matrix and Stage C used a separately SHA-256-frozen 16-case holdout authored only after selection. The final Stage-C semantic panel recorded target coverage `1.00` for Ministral 8B, Mistral Small, Gemma 4 31B, and Gemini 3.1 Flash-Lite, while Ministral 14B reproducibly recorded `0.875`. Every completed Stage-C run preserved unsupported grounded claims = `0` and missed target insufficiency = `0`; the 14B miss is a conservative utility failure, not an unsafe exposure. Gemini 3.5 Flash-Lite remained outside Stage C because its predeclared Stage-B replication was operationally quota-incomplete, not because of a semantic failure.

The current semantic generation remains frozen at candidate `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352`; current `main` may contain provider-transport reliability changes that do not alter semantic runtime/gate/holdout behavior. #150 is closed as the verified-utility-recovery milestone. Successor semantic work is deliberately split into #159, #160, and #164 and must receive a new runtime/evaluation identity rather than reusing the observed Stage-C holdout as a tuning surface.

The #159 semantic behavior change starts a distinct successor candidate at commit `79ec3b44971c32f9a8847d8173672675947c7288`. That identity records exact Harness-owned bounded-target priority only; it does not replace or reinterpret the frozen `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352` Stage-C candidate, and the observed Stage-C holdout is not rerun as a tuning surface. Any later semantic change for #160 or #164 receives its own successor identity before fresh evaluation.

The #160 renderer-downgrade change advances the successor candidate to `a020b5925497ff3fdf200a9622270fa1889a6aa1`. It reuses only exact Harness-owned target identity and already-existing artifact authority; renderer `uncertain` mode is a trigger, never evidence. Its own recovery helper does not override `Reject`, and `Unknown` remains a target-only qualified result.

The #164 dependency-aware target-local change advances the successor candidate to `993874fa0051d06a02c8db8f7a220a2ac7773c17`. It adds a separate `Reject`-scoped qualified lane rather than relaxing the global decision: exact targets require direct evidence-bound trusted `Supported` receipts; contradicted blockers require their own evidence-bound trusted contradiction receipts; same-key, untyped, shared-evidence, target-local qualification/adversarial/contradiction, and inference/dependency coupling all fail closed. The frozen Stage-C corpus/results are unchanged and were not rerun for tuning.

Issue #126 does not create another semantic candidate. It hardens the provider/evaluation control plane around `993874fa0051d06a02c8db8f7a220a2ac7773c17`: Google temporary 429 behavior remains bounded and quota-aware; 500/502/503/504 and isolated empty model text receive narrow capped retries; actual adapter attempts are observable; and the 16-case product dogfood/Stage-C runner can resume only an exact completed-case prefix under an exact fixture/provider/seed/config/runtime/executable identity. An interrupted active case is restarted from its beginning and its prior operational failure remains recorded. Historical RSD2/Stage-C outcomes are not rewritten.

Use separate dogfood/reference workloads and answer:

- does the harness reduce unsupported final assertions in realistic use?;
- how often does it correctly abstain, and how often does it abstain unnecessarily?;
- how often can bounded resolution convert an initially unsupported answer into a verified one?;
- which missing-support patterns recur in practice?;
- what are the latency/token/retry costs of the safety process?;
- can users understand and act on `unknown`, abstention, and failure telemetry?;

Real-workload failures may seed **new calibration corpora**, but they must never be used to repair or
retune observed frozen holdouts.

Real-workload evidence also decides whether an interactive session surface is worth productizing. Do not add a chat-like
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

As of readiness evidence baseline commit `5c5701f77df9dd507c3949294708f8c07a054064`, all eight readiness conditions have recorded evidence: the process-level compatibility contract landed in PR #170 and is green in CI run `33822514022` plus the four-platform run `33822514005`; the reproducible release/install path remains recorded by the v0.1.0 release workflow; current bounded live runtime/product smoke is green in runs `33822794171` and `33822567155`; incident-analysis and architecture-review both have product acceptance evidence; runtime/rollback/failure/secret-handling and research-surface boundaries are documented and tested. This marks the **readiness gate** complete for the current main line. It does not itself publish, tag, or promise a stable v1.0 release; that remains an explicit version/release decision using the normal provenance workflow.

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

The current semantic runtime and the completed #91 residual evidence-sufficiency program retain separate machine identities and rollback boundaries. The answer-safety configuration is versioned independently from the semantic runtime, and neither may create verification authority. Frozen holdout-v4/v5 and the sufficiency holdout remain immutable research history and are
never product-tuning corpora.

## Deferred product surfaces

- **Public Rust embedding API:** after real CLI consumers validate the correct compatibility
  boundary.
- **MCP adapter:** optional integration invoking the full runtime; never evidence that the caller's
  entire agent loop is verified.
- **Interactive CLI (`reason shell` / `reason repl`):** demand-gated after repeated real-workload dogfood. If adopted, it is a thin stateful session over `ReasoningThread`/checkpoint/replay and the same product runtime, not a separate chat authority or evidence shortcut.
- **Desktop UI:** thin inspection/review client only after artifact and CLI contracts are stable.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md),
[roadmap](roadmap.md), and [research plan](research-plan.md).
