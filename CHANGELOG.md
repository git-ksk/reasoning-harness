# Changelog

All notable product-facing changes to the `reason` CLI are recorded here. Research-only binaries and fixture-study changes may be documented in the research notes instead.

The project follows semantic versioning for the executable, with the usual v0.x caveat that product interfaces are still being hardened. Machine-readable contract identities provide a stricter compatibility boundary than the executable version alone.

## [Unreleased]

### Changed

- Closed Issue #210's exposed-text correctness gap: `GroundedAnswer` / `QualifiedPartialAnswer` no longer expose model renderer `text`; Harness constructs the exposed text from accepted factual claims or typed recovery state under `harness-canonical-exposed-text-v1`.
- Bumped natural-language JSON output from `reason-natural-output-v2` to `reason-natural-output-v3` and added explicit `exposed_text` policy telemetry. Consumers that relied on renderer prose should use `finalization.text`; historical v2 behavior remains reproducible at commit `3a601c8` but is not available as a safety rollback because it would restore the P0 gap.
- Closed Issue #211's subprocess timeout gap with one shared absolute wall-clock deadline across `external_command`, `trusted_command`, and the v0.4 product successor `mcp_readonly_v2`, while frozen historical `mcp_readonly_v1` remains unchanged. The deadline covers spawn, complete stdin write, stdout read, process termination, and non-blocking cleanup handoff; blocked large writes and inherited descendant pipes cannot extend the caller beyond the configured timeout, and oversized stdout remains a typed protocol failure rather than being misclassified as timeout.
- Added Issue #212 bounded investigation planning to the natural-language path under `bounded-investigation-v1`: closed plan/action schemas, Harness-admitted untrusted targets, selection among configured read-only capabilities, typed follow-up/no-progress stopping, candidate regeneration after admitted evidence, and mandatory return through ordinary qualification/verification before grounded exposure. Static resolver lanes remain unchanged; investigation external commands use the separate `investigation_external_command_v1` / `reason-investigation-external-resolver-request-v1` request identity rather than extending `external_command_v1`.

## [0.3.0] - 2026-09-04

Third external-preview capability release. v0.3.0 adds bounded external evidence and resolution without changing the research generation, semantic runtime, or answer-safety identity.

### Added

- `external_command_v1` plus fail-closed `external_evidence_admission_v1` for Harness-owned source, freshness, scope, and authority policy.
- External-resolution budgets, typed operational failures, telemetry, and replay-safe records.
- Read-only `mcp_readonly_v1`, separate `trusted_command_verifier_v1`, and optional Rust-only `reason-mcp` native-runtime delegation.
- `external-resolution-acceptance-v1`; release acceptance kept unsupported grounded claims and missed target insufficiency at `0`, with two safe recoveries and a separate live AWS RSS `Unknown -> Accept` smoke.

### Preserved

- Frozen Stage-C/RSD2/historical holdouts remain unchanged.
- Semantic runtime remains `semantic-decidability-d3-v1`; answer safety remains `verified-target-answer-gate-v1`; MCP remains outside the correctness boundary.

## [0.2.0] - 2026-09-04

Second external-preview release of the native Reasoning Harness CLI. This is a **product capability release on the existing research/authority foundation**, not a rewrite of frozen Stage-C/RSD2 evidence.

### Natural-language AI CLI

- Added direct `reason "TASK"` AI-backed execution with the current `reason-natural-output-v2` JSON identity while preserving the v0.1 structured product commands.
- Added provenance-aware `--file` and piped-stdin untrusted context with bounded input size.
- Added explicit `--fact`, `--hypothesis`, and bounded `--resolver-fact` inputs without allowing arbitrary prose/model output to self-promote into trusted evidence.
- Added model-backed final rendering behind final-claim coverage, plus deterministic recovery for exact already-authorized targets when renderer output omits or weakens them.
- Added strict target-local qualified recovery for structurally isolated verified targets while preserving artifact-global `Reject`/`Unknown` and all existing authority checks.

### Product evaluation and reliability

- Added `reason-product-dogfood` with same-model raw vs Harness baseline vs current-safety comparison across incident-analysis and architecture-review workloads.
- Added bounded Google/Gemini transient retry for temporary 429, HTTP 500/502/503/504, and one isolated empty-model-text anomaly; credential, quota, deterministic 4xx/protocol, transport, and timeout failures remain fail-fast under the current policy.
- Added actual provider HTTP-attempt telemetry across adapters and structured-output fallback calls.
- Added `reason-product-dogfood-v10` exact-identity checkpoint/resume: only fully completed cases are reused, an interrupted active case restarts from its beginning, and preserved provider/protocol failures remain operational evidence rather than semantic abstention.
- Current Ministral 8B six-case product revalidation improved Harness target coverage from the historical 0.25 slice to 1.00 while preserving zero unsupported grounded claims and zero missed target insufficiency.

### CLI compatibility and distribution

- Added process-level compatibility tests that execute the real `reason` binary and pin `reason-cli-output-v1`, schema IDs, stdin behavior, epistemic `unknown` as exit 0, typed operational failure as exit 1, and CLI usage failure as exit 2.
- Run the compatibility contract on Linux x86_64, macOS arm64, macOS x86_64, and Windows x86_64.
- Kept v0.x releases explicitly in external-preview status even though the documented v1.0 readiness gate is now satisfied on current main.
- Release automation marks 0.x GitHub Releases as prereleases automatically.

### Research and authority provenance

- Preserved the frozen Stage-C candidate/holdout and historical RSD2 outcomes unchanged; v0.2.0 does not reinterpret prior provider failures as semantic success.
- The current successor semantic candidate remains `993874fa0051d06a02c8db8f7a220a2ac7773c17`; the semantic runtime remains `semantic-decidability-d3-v1` and the current answer-safety configuration remains `verified-target-answer-gate-v1`.
- Model output, retrieval prose, retry success, and checkpoint reuse remain outside verification authority.

## [0.1.0] - 2026-09-01

First external preview of the native Reasoning Harness CLI.

### Product CLI

- Added supported `reason run`, `reason verify`, `reason semantic-check`, and `reason schema` product surfaces; `eval*` remains research/evaluation.
- Added stdin (`-`) support for non-interactive JSON inputs with one-consumer protection.
- Added `reason-cli-output-v1`, `reasoning-artifact-v1`, `reasoning-candidate-v1`, `reason-config-v1`, and `semantic-check-input-v1` machine-readable contract identities/schema discovery.
- Added schema-backed layered non-secret config: CLI flags > explicit config > project config > user config > defaults, plus `--no-config` hermetic execution.
- Added machine-readable product failure envelopes and normalized provider/input/config/harness failure classes while preserving exit 1 for process failure and exit 0 for successful `accept | reject | unknown` outcomes.

### Semantic runtime

- Exposed the adopted `semantic-decidability-d3-v1` runtime through `reason semantic-check` without granting soft diagnostics final-verdict authority.
- Preserved explicit `soft-semantic-v3` rollback selection.
- Added typed operational failure output separate from semantic decisions.
- Live product smoke passed on Mistral Ministral 8B and Google-hosted Gemma 4 31B for D3 and v3 rollback.

### Distribution

- Added credential-free product smoke on Linux x86_64, macOS arm64, macOS x86_64, and Windows x86_64.
- Added `cargo install --git` installation for the single supported `reason` binary.
- Added tag-driven standalone GitHub Release archives and SHA-256 checksums; research binaries are not release artifacts.
