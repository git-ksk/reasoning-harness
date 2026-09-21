# Project status

[日本語](project-status.ja.md) | English

This page is the **short current view**. It is intentionally written for users and contributors who need to know where the project is now without reading the full research chronology.

For the preserved long-form provenance ledger, see [Project status history](project-status-history.md). For navigation across product docs and research evidence, see the [documentation index](README.md).

## Current release

**Reason CLI 0.5.3 on Harness Engine 0.5.0** is the current published CLI preview. Harness Engine 0.5.0 remains independently released from the accepted semantic line as `engine-v0.5.0`; `reason-v0.5.3` is the distribution release that adopts it.

```text
Published CLI: Reason CLI 0.5.3 / Harness Engine 0.5.0
Latest Engine source release: Harness Engine 0.5.0
```

`v0.4.2` remains the immutable final release where the CLI and Engine shared one SemVer coordinate. `reason-v0.5.0` shipped the first split general-use CLI, followed by the 0.5.1/0.5.2 patch line on Engine 0.4.2. `reason-v0.5.3` then adopted the independently accepted Engine 0.5.0 under a new CLI coordinate.

The central trust boundary remains unchanged: model output is an untrusted candidate/renderer, while evidence admission, qualification, verification, bounded resolution, and exposed factual-claim authority remain Harness-owned. The final frozen Engine 0.4.2 natural-language release evaluation passed independently across Mistral, Groq, Gemini 3.5 Flash-Lite, and Gemma 4 31B. See [v36 release acceptance](natural-language-e2e-v36-result.md).

## What users can do today

The supported native product surface includes:

- bare `reason` for managed interactive terminal use, plus `reason "TASK"` for the one-shot natural-language verified path;
- `reason setup`, `reason auth ...`, and provider/model/config commands for guided setup and secure daily use;
- `reason session ...`, `-c`, and `-r` for persisted/resumable reasoning state;
- `reason doctor` for local diagnostics and explicit bounded live readiness checks;
- `reason mcp ...` for guided local/remote read-only MCP lifecycle under the existing acquisition-only boundary;
- `reason update`, explicit rollback, and `reason uninstall` for the verified CLI lifecycle;
- `reason run` for structured candidate/application integration;
- `reason verify` for deterministic artifact validation;
- `reason semantic-check` for soft semantic diagnostics;
- `reason schema` for versioned machine contracts;
- bounded external resolver acquisition;
- allowlisted read-only MCP acquisition;
- explicitly trusted deterministic command verification;
- optional `reason-mcp` delegation to the native runtime.

Supported provider adapters include Mistral, Google Gemini/AI Studio, NVIDIA Hosted NIM, and Groq. Provider/model output never becomes verification authority merely because the provider call succeeded.

## Current product track: Reason CLI 0.5.x

The first split general-use release (`reason-v0.5.0`) has shipped, and the current published patch coordinate is `reason-v0.5.3` on Engine 0.5.0. `reason-v0.5.2` remains an immutable Engine 0.4.2 artifact; the newer Engine was adopted only through the new 0.5.3 release coordinate.

The 0.5.0-0.5.2 productization line delivered ordinary terminal usability on the fixed Engine 0.4.2 baseline. Reason CLI 0.5.3 keeps those product surfaces while adopting the separately accepted Engine 0.5.0, including:

- native installation without requiring a Rust toolchain;
- signed/verifiable distribution and update/rollback/uninstall lifecycle;
- OS-native secure credential storage and `reason auth`;
- guided `reason setup` with provider/model discovery;
- explicit project trust for executable/authority-bearing configuration;
- no-argument interactive use plus simple continue/resume flows;
- understandable verified/qualified/unknown presentation;
- provider usage/budget visibility;
- guided read-only MCP management;
- actionable `reason doctor` and recovery-oriented errors;
- private local session/state handling and subprocess secret isolation;
- fresh-install acceptance across supported platforms.

The detailed acceptance plan is in the [Reason CLI 0.5.0 roadmap](reason-cli-0.5-roadmap.md).

## Separate engine track: Harness Engine 0.5.0

Reasoning/correctness changes are intentionally separated from CLI product UX work.

Harness Engine 0.5.0 is release-complete. Final-v3 canonical run `35457038163` passed all six independently required rows and all 18 fresh cases with zero correctness-boundary violations and zero session external replay. The release includes deterministic explicit-fact correction continuity, deterministic admitted exact-fact investigation materialization, and the finalization-correctness/planner-utility evaluator separation. See [Engine 0.5.0 final-v3 result](engine-0.5-final-v3-result.md).

`reason-v0.5.3` release provenance binds CLI 0.5.3, Engine 0.5.0, and merge commit `e9148c737c6f9bf29ce7c9258d549f5c526dfb4a`. Supported-platform package/no-Rust/lifecycle acceptance passed, and live 0.5.2 <-> 0.5.3 update/rollback acceptance verified fail-closed Engine-change consent. See [Product, engine, and contract versioning](versioning.md) and the [product roadmap](product-roadmap.md).

## Current trust boundary

The project currently makes these product-level commitments:

1. **Model output is untrusted.** A model cannot create evidence, verification receipts, or final authority by self-labeling a claim.
2. **Acquisition is not verification.** Retriever, resolver, and MCP output remains acquired data until admitted and verified.
3. **Unknown is valid.** Missing support is not converted into a confident answer merely to complete the task.
4. **Operational failure is separate from epistemic uncertainty.** Provider/quota/transport/protocol failure does not silently become semantic `unknown`.
5. **Final factual text is authority-bound.** Renderer fluency cannot introduce stronger unsupported factual claims into the exposed grounded answer.
6. **Historical evaluations remain historical.** Frozen/observed studies are not rewritten after the fact to make a later implementation look better.

## Main current gaps

The general-use CLI productization work is shipped. Native installation, guided setup/auth, interactive/continue/resume UX, config/model/MCP discovery, progress/cancellation, diagnostics, private local state, and lifecycle management are all part of the current 0.5.x product line.

The remaining product-side distribution follow-up is #375: Homebrew 0.5.3 physical upgrade/test acceptance is complete; the WinGet community manifest has passed Microsoft validation and CLA checks and is waiting for community moderator approval. This external review does not block Harness Engine work.

Harness Engine 0.5.0 is implementation-, acceptance-, and release-complete. Final hardening #445/#446/#450 was validated by `engine-0.5-final-v3-freeze`; canonical run `35457038163` passed all six independently required rows and all 18 fresh cases with zero correctness-boundary violations and zero session external replay. See the [Engine 0.5.0 final-v3 result](engine-0.5-final-v3-result.md). The independent Engine source release is `engine-v0.5.0`; Reason CLI 0.5.3 now distributes it, while Reason CLI 0.5.2 remains immutable on Engine 0.4.2.

## Research posture

Reasoning Harness does **not** claim that open-world reasoning is solved. Hard correctness still depends on deterministic structure and trusted evidence/oracles where a hard answer exists. Model-backed semantic mechanisms remain advisory/restrictive unless separately promoted through a measured authority-safe path.

For the detailed historical record—including earlier semantic studies, holdouts, RSD lines, provider investigations, product dogfood, and natural-language E2E iterations—use [Project status history](project-status-history.md) and the current [documentation index](README.md).
