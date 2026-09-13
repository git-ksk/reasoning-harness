# Reason CLI 0.5.0 fresh-install release acceptance

**Status:** Phase 5 / Issue #374 release gate for **Reason CLI 0.5.0** on unchanged **Harness Engine 0.4.2**.

This gate is intentionally a composition of existing product contracts plus one new fresh-install lane. It does not duplicate every lower-level test in another monolithic suite, and it does not require a billable live-provider call.

## Gate structure

The required evidence is produced by these existing/new required workflows:

| Evidence lane | What it proves |
| --- | --- |
| `fresh-install-acceptance` | Builds the four release-shaped native archives, then runs each archive in a separate consumer job with no Rust toolchain action and a PATH that cannot resolve `cargo`/`rustc`. It checks CLI/Engine identity, empty-home setup, explicit provider/model selection, config inspection, local-only doctor, typed one-shot recovery, JSON compatibility, lifecycle dry-run/fail-closed behavior, private Unix config permissions, and recursive secret-persistence scanning. No live provider request is made. |
| `installer-smoke` | Production `install.sh` / `install.ps1` contract across supported platforms, including the immutable historical install path; split-release installer tests fail closed on provenance, checksum, version, and unsupported-platform errors. |
| `cli-platform-smoke` | Linux/macOS/Windows product CLI compatibility, MCP product surface, JSON/non-interactive regression, and subprocess environment isolation. |
| `credential-store-smoke` | Native credential-store round trips on macOS/Windows plus scoped MCP OAuth credential lifecycle. Linux headless/unavailable behavior is covered by deterministic secure-credential tests and never falls back to plaintext. |
| `lifecycle-smoke` | Update/rollback/uninstall product contracts on every supported platform. |
| `ci` | `cargo fmt --check`, workspace Clippy with `-D warnings`, full workspace tests, deterministic CLI/fixture regression, and the unchanged Engine correctness suite. |

The public `reason-v0.5.0` tag and GitHub Release are **not** created by Phase 5 acceptance. That publication action happens only after this gate is green. Therefore the pre-tag gate cannot download a release URL that intentionally does not exist yet. Instead, the candidate workflow packages the exact release archive layout, while the production installer/updater provenance policy is tested deterministically and the release workflow remains responsible for GitHub/Sigstore attestations at publication time.

## Acceptance matrix

| Requirement | Automated evidence |
| --- | --- |
| macOS/Linux/Windows fresh install | `fresh-install-acceptance` four-platform producer/consumer matrix |
| install native binary without Rust | consumer jobs install no Rust action and run with a PATH where `cargo`/`rustc` are unresolvable |
| separate CLI/Engine identity | packaged `reason --version`; `reason doctor` requires CLI `0.5.0` and Engine `0.4.2` |
| untrusted project cannot silently activate executable/MCP/verifier config | `project_trust` integration tests in full `ci`; `cli-platform-smoke` exercises product surfaces |
| credential setup without argv/history/plaintext secret | fresh-install environment credential path; auth/setup contract tests; native credential-store smoke; recursive secret scan |
| explicit provider/model selection, no silent fallback | fresh-install model switch/restore plus model-lifecycle contract tests |
| one-shot natural-language path | packaged binary reaches the configured-default natural command path and exercises typed credential recovery without a provider call; deterministic natural execution semantics remain covered by workspace tests |
| interactive mode and follow-up | deterministic interactive tests cover TTY entry, follow-up directives, context addition, status/evidence/usage, and persistence boundaries; no billable provider call is required |
| persisted create/continue/resume, crash/concurrency safety, identity pinning | managed-session, interactive identity-pinning, and product session contracts in full `ci` / platform smoke |
| `reason doctor` local diagnostics and optional live boundary | packaged local-only doctor plus doctor network/live-check deterministic tests; Phase 5 does not spend provider quota |
| actionable failure recovery | packaged one-shot credential failure plus Phase 4 remediation contract coverage |
| credential/provider/quota/config/trust/MCP/session/network/update/distribution recovery families | Phase 4 remediation catalog, doctor tests, lifecycle tests, MCP tests, and installer/provenance contracts |
| project trust | dedicated `project_trust` integration suite |
| MCP/resolver/verifier secret isolation | `cli-platform-smoke` subprocess sentinel tests and provider isolation tests |
| usage/resource limits | `usage_budget` deterministic pre-call/budget exhaustion and JSON tests |
| privacy, purge, ephemeral/no-persist | `local_privacy`, interactive ephemeral, managed-session purge, uninstall retain/purge contracts |
| update provenance / Engine-change confirmation | lifecycle + release provenance contracts; Engine identity is separately represented |
| rollback | lifecycle contract and packaged historical-boundary fail-closed smoke |
| uninstall | packaged dry-run plus lifecycle cross-platform smoke; data/credentials retained unless explicitly purged |
| JSON/non-interactive compatibility | packaged `run`/`verify` plus `cli-platform-smoke` and `ci` |
| proxy/custom CA/headless | Phase 4 doctor/network tests, reqwest proxy behavior, custom-CA fail-closed diagnostics, Linux headless credential test |
| zero secret leakage | stdout/stderr checks, recursive fresh-home scan, auth/setup/doctor/MCP tests, subprocess sentinel isolation |
| artifact/distribution tamper resistance | installer checksum/provenance failure tests, signed-manifest contract, release workflow GitHub/Sigstore attestation policy |
| Engine 0.4.2 correctness semantics unchanged | package coordinate assertion (`CLI=0.5.0`, `Engine=0.4.2`) plus full workspace/fixture regression; protected eval/holdout inputs are not modified by this gate |

## Manual / physical status

No physical-device or GUI-only acceptance is required for Issue #374. All supported release platforms are covered by native GitHub-hosted runners. Public release publication is intentionally **post-gate**, not a manual acceptance blocker.

## Release boundary

Phase 5 is green only when all required checks above pass on the Phase 5 PR and again on the merged `main` commit. The gate must not be marked complete if any required workflow is skipped because of an implementation change, if a P0 issue remains open, or if protected evaluation/holdout material changed as part of product acceptance.
