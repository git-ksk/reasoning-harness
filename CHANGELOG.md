# Changelog

All notable product-facing changes to the `reason` CLI are recorded here. Research-only binaries and fixture-study changes may be documented in the research notes instead.

The project follows semantic versioning for the executable, with the usual v0.x caveat that product interfaces are still being hardened. Machine-readable contract identities provide a stricter compatibility boundary than the executable version alone.

## [Unreleased]

### Natural-language AI CLI

- Added direct `reason "TASK"` AI-backed execution with `reason-natural-output-v1` JSON identity while preserving all v0.1 structured commands.
- Added provenance-aware `--file` and piped-stdin untrusted context with bounded input size.
- Added explicit `--fact`, `--hypothesis`, and bounded `--resolver-fact` inputs without allowing arbitrary prose/model output to self-promote into trusted evidence.
- Added model-backed natural-language final rendering behind final-claim coverage; uncovered renderer facts are blocked and may re-enter bounded verification.
- Added `reason-product-dogfood` and a manual live workflow for same-model raw-vs-harness product evaluation across incident-analysis and architecture-review workloads.

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
