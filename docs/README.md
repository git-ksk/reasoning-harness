# Reasoning Harness documentation

[日本語](README.ja.md) | English

This repository contains two different kinds of documentation:

1. **product documentation** for people using or integrating `reason`; and
2. **preserved research/evaluation evidence** used to justify or reject changes to the Harness.

You do **not** need to read the research record before using the product. Choose the path that matches what you are trying to do.

## I want to try `reason`

Start here:

1. [Getting Started](getting-started.md) — install the current release and run a grounded and intentionally-insufficient example.
2. [Native installers](native-installers.md) — one-command macOS/Linux/Windows distribution contract for Reason CLI 0.5.x.
3. [Package managers](package-managers.md) — Homebrew/WinGet channels and lifecycle ownership.
4. [Release provenance](release-provenance.md) — OIDC/Sigstore provenance, checksums, and immutable-release trust for `reason-v*`.
5. [Update / rollback / uninstall](update-rollback-uninstall.md) — provenance-verified lifecycle and retention/purge boundaries.
6. [Reason doctor](reason-doctor.md) — secret-free local diagnostics plus explicit bounded live readiness checks.
7. [Operational failure recovery](reason-recovery.md) — actionable human recovery plus additive machine remediation while preserving failure classes.
8. [Proxy, custom CA, and headless network environments](network-environments.md) — proxy presence, additive CA trust, and typed network diagnostics.
9. [CLI guide](cli.md) — commands, input, provider credentials, config, stdin, JSON output, and exit semantics.
10. [How Reasoning Harness works](how-it-works.md) — the proposal-vs-authority boundary in practical terms.
11. [Product support and compatibility](support.md) — supported platforms, contracts, provider posture, and v0.x compatibility.

If you only want to understand the project before installing it, read the root [README](../README.md) and then [How it works](how-it-works.md).

## I am integrating an application, agent, RAG system, or CI pipeline

Recommended order:

- [CLI guide](cli.md) — `reason run`, `reason verify`, JSON envelopes, stdin, and exit semantics.
- [How Reasoning Harness works](how-it-works.md) — candidate materialization, verification, diagnostics, and acceptance.
- [Grounded resolution](grounded-resolution.md) — bounded acquisition/re-verification/finalization.
- [External resolver adapters](external-resolvers.md) — external acquisition without granting resolver output authority.
- [Read-only MCP resolver](mcp-resolver.md) — MCP as bounded acquisition rather than correctness authority.
- [Trusted verifier](trusted-verifier.md) — explicitly trusted deterministic/oracle verification.
- [MCP product surface](mcp-product-surface.md) — calling the native runtime from an external MCP client.
- [Resumable sessions](session.md) — persisted reasoning state, replay, correction, and fork semantics.

## I want to understand the trust and architecture model

Start with:

- [Architecture](architecture.md)
- [Project trust](project-trust.md) — explicit activation boundary for executable/MCP project config.
- [Secure provider credentials](secure-credentials.md) — OS-native secret storage and deterministic environment precedence for the 0.5.x line.
- [Provider/model catalog](model-catalog.md) — curated compatibility evidence, credential readiness, and default switching for the 0.5.x line.
- [Reasoning policy](reasoning-policy.md)
- [Evidence qualification](evidence-qualification.md)
- [Exposed-text safety](exposed-text-safety.md)
- [Bounded investigation](investigation.md)
- [Terminology and naming](terminology.md)

Then use the ADRs for design rationale:

- [ADR-0001: interface and packaging boundaries](adr/0001-interface-and-packaging-boundaries.md)
- [ADR-0002: grounded resolution and finalization](adr/0002-grounded-resolution-and-finalization.md)
- [ADR-0003: reasoning control plane](adr/0003-reasoning-control-plane.md)
- [ADR-0004: Harness-owned investigation action materialization](adr/0004-investigation-target-intent-materialization.md)

## I want the current project status or roadmap

Use the short current views:

- [Project status](project-status.md) — what is released, what is active, and the main known product gaps.
- [Product roadmap](product-roadmap.md) — current forward product/engine tracks.
- [Engine0.7 real-world evaluation and CLI adoption](engine-07-realworld-evaluation.md) — #531, preserved v2/v3 FAIL, pending fresh real-source/provider/E2E acceptance.
- [Harness Engine0.7.3 plan](engine-0.7.3-roadmap.md) — #538 ordered repair, safe runner #540, fresh evaluation and separate release/adoption gates.
- [Engine0.7.3 retraction repair](launch-retraction-520.md) — Draft #539, known regressions and immutable independent first outcomes; source release/CLI acceptance remain separate.
- [Research-informed Engine0.7.3 repair](engine-073-research-informed-repair.md) — public Claude/Codex engineering and NLP research, archive-only diagnostics and the next scope/reference prototype.
- [Harness Engine 0.7 roadmap](engine-0.7.0-roadmap.md) — published 0.7.0 scope, deferred mechanisms and historical acceptance gates.
- [Harness Engine 0.7.1 release](engine-0.7.1-release.md) — previous source patch, #512/#514/#515 corrections, evidence and limits.
- [Harness Engine 0.7.2 release](engine-0.7.2-release.md) — current normal source patch, scope, final validation and publication coordinates.
- [Harness Engine 0.7.2](engine-0.7.2-roadmap.md) — completed launch-relevance safety patch and remaining deferred 0.7 hypotheses.
- [#520/#521 repair evidence](launch-context-markers-520-521.md) — original failed observations, bounded behavior and final source validation.
- [Engine 0.7 local reviewer host development v1](engine-0.7-local-reviewer-v1.md) — development-only OS-keyring reviewer bridge; same-user automated E2E is not independent human consent, and published CLI integration remains separate.
- [Engine 0.7 source reconciliation development v1](engine-0.7-source-reconciliation-v1.md) — opt-in trusted-review overlay shipped in the 0.7.0 source release.
- [Engine 0.7.0 baseline v1 result](engine-0.7-baseline-v1-result.md) — frozen 0.6.1 residual baseline (29 deterministic development cases, not independent holdout acceptance).
- [Product roadmap](product-roadmap.md) — current forward product/engine tracks.
- [Reason CLI 0.5.0 roadmap](reason-cli-0.5-roadmap.md) — general-use terminal productization.
- [Provider/model retirement and fallback policy](model-retirement-policy.md) — explicit lifecycle state, no silent identity switching, and session pinning.
- [Release channels and promotion policy](release-policy.md) — normal vs candidate gates, v0.x compatibility, immutable versions and separate Engine/CLI scope.
- [Versioning](versioning.md) — separate Reason CLI, Harness Engine, and machine-contract coordinates.

The previous long-form ledgers were preserved as [project-status history](project-status-history.md) and [product-roadmap history](product-roadmap-history.md).

## I am reviewing the research or evaluation evidence

Start from the current release evidence, then move backward only if you need provenance:

- [Planner reliability v1 result](planner-reliability-v1-result.md) — frozen #282 repeated-trial baseline before #283 action-materialization changes.
- [v0.4.2 final v36 release acceptance](natural-language-e2e-v36-result.md) — immutable final unified / Engine 0.4.2 release evidence.
- [Harness Engine 0.6.1 release](engine-0.6.1-release.md) — historical source patch, fresh cross-provider launch safety acceptance, and the residual legacy v1 limitation.
- [Harness Engine 0.6.0 release](engine-0.6.0-release.md) — historical 0.6.0 baseline and the frozen #461/#462/#468/#463 acceptance boundary.
- [Harness Engine 0.5.0 release](engine-0.5.0-release.md) — previous independent Engine coordinate, release boundary, and accepted semantic delta.
- [Harness Engine 0.5.0 final cross-model acceptance](engine-0.5-final-v3-result.md) — fresh final semantic-delta gate across the current curated model catalog.
- [Product dogfood](product-dogfood.md) — product-oriented comparative evaluation.
- [Benchmark](benchmark.md) — benchmark/evaluation methodology and interpretation.
- [Research plan](research-plan.md) — research questions and promotion discipline.
- [Corpus versioning](corpus-versioning.md) — frozen case identity and score-compatibility rules.

Historical E2E, holdout, semantic-judge, RSD, replication, and provider-study files are intentionally retained for provenance. They are **not** a recommended onboarding path and should not be read as one global version sequence. See [Terminology and naming](terminology.md).

## I want to contribute

Read:

- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [SECURITY.md](../SECURITY.md)
- [Architecture](architecture.md)
- [Project trust](project-trust.md) — explicit activation boundary for executable/MCP project config.
- [Secure provider credentials](secure-credentials.md) — OS-native secret storage and deterministic environment precedence for the 0.5.x line.
- [Project status](project-status.md)

Changes that affect reasoning/correctness semantics should include appropriate fixtures/evaluation evidence and must preserve the distinction between untrusted model output and Harness-owned authority.

## Product vocabulary in one minute

| Term | Meaning |
| --- | --- |
| **Reason CLI** | The user-facing `reason` executable and terminal/distribution experience. |
| **Harness Engine** | The reasoning/correctness runtime that owns admission, verification, finalization, and authority boundaries. |
| **candidate** | Model/agent-proposed reasoning or answer data. Untrusted by default. |
| **evidence** | Input that may support a proposition; authority depends on admission/qualification/verification. |
| **grounded** | A factual claim is covered by Harness-owned verified state. |
| **qualified** | Useful supported facts can be exposed while a stronger unsupported conclusion stays uncertain. |
| **unknown** | The Harness does not currently have enough trusted support to expose the requested conclusion. |

For exact machine identities and historical labels, use the [terminology guide](terminology.md).
