# Reasoning Harness documentation

[日本語](README.ja.md) | English

This repository contains two different kinds of documentation:

1. **product documentation** for people using or integrating `reason`; and
2. **preserved research/evaluation evidence** used to justify or reject changes to the Harness.

You do **not** need to read the research record before using the product. Choose the path that matches what you are trying to do.

## I want to try `reason`

Start here:

1. [Getting Started](getting-started.md) — install the current preview and run a grounded and intentionally-insufficient example.
2. [CLI guide](cli.md) — commands, input, provider credentials, config, stdin, JSON output, and exit semantics.
3. [How Reasoning Harness works](how-it-works.md) — the proposal-vs-authority boundary in practical terms.
4. [Product support and compatibility](support.md) — supported platforms, contracts, provider posture, and v0.x compatibility.

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
- [Reasoning policy](reasoning-policy.md)
- [Evidence qualification](evidence-qualification.md)
- [Exposed-text safety](exposed-text-safety.md)
- [Bounded investigation](investigation.md)
- [Terminology and naming](terminology.md)

Then use the ADRs for design rationale:

- [ADR-0001: interface and packaging boundaries](adr/0001-interface-and-packaging-boundaries.md)
- [ADR-0002: grounded resolution and finalization](adr/0002-grounded-resolution-and-finalization.md)
- [ADR-0003: reasoning control plane](adr/0003-reasoning-control-plane.md)

## I want the current project status or roadmap

Use the short current views:

- [Project status](project-status.md) — what is released, what is active, and the main known product gaps.
- [Product roadmap](product-roadmap.md) — current forward product/engine tracks.
- [Reason CLI 0.5.0 roadmap](reason-cli-0.5-roadmap.md) — general-use terminal productization.
- [Versioning](versioning.md) — separate Reason CLI, Harness Engine, and machine-contract coordinates.

The previous long-form ledgers were preserved as [project-status history](project-status-history.md) and [product-roadmap history](product-roadmap-history.md).

## I am reviewing the research or evaluation evidence

Start from the current release evidence, then move backward only if you need provenance:

- [v0.4.2 final v36 release acceptance](natural-language-e2e-v36-result.md) — current release evidence.
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
