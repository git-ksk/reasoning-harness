# Project status

[日本語](project-status.ja.md) | English

This page is the **short current view**. It is intentionally written for users and contributors who need to know where the project is now without reading the full research chronology.

For the preserved long-form provenance ledger, see [Project status history](project-status-history.md). For navigation across product docs and research evidence, see the [documentation index](README.md).

## Current release

**Reason CLI 0.5.4 on Harness Engine 0.6.1** is the current published CLI preview; previous `reason-v0.5.3` artifacts remain immutable on Engine 0.5.0. **Harness Engine 0.6.1** is the latest independent source release under `engine-v0.6.1`; Reason CLI 0.5.4 has separately adopted that released Engine.

```text
Published CLI: Reason CLI 0.5.4 / Harness Engine 0.6.1
Latest Engine source release: Harness Engine 0.6.1 (`engine-v0.6.1`)
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

The first split general-use release (`reason-v0.5.0`) has shipped, and the current published patch coordinate is `reason-v0.5.4` on Engine 0.6.1. Older `reason-v0.5.3` (Engine 0.5.0) and `reason-v0.5.2` (Engine 0.4.2) artifacts remain immutable.

The 0.5.0-0.5.2 productization line delivered ordinary terminal usability on the fixed Engine 0.4.2 baseline. Reason CLI 0.5.3 preserved those product surfaces while adopting accepted Engine 0.5.0; CLI 0.5.4 further adopts Engine 0.6.1. The existing product surfaces include:

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

## Latest product integration: Reason CLI 0.5.4

Reason CLI 0.5.4 adopts independently released Harness Engine 0.6.1 under #484, with separate signed four-platform native artifacts and explicit Engine-change consent on 0.5.3 -> 0.5.4 upgrade and reverse rollback. Existing 0.5.3 binaries remain unchanged on Engine 0.5.0.

## Current Engine patch: Harness Engine 0.6.1

The source-only `engine-v0.6.1` release fixes target-owned positive ChangeOrLaunch authority in v17/v30. The original independent v1 FAIL is preserved; separate v2 deterministic acceptance passed 30/30, and Mistral/Google/Groq live acceptance passed 12/12 each (wrong-target Relevant 0, utility miss 0, provider failure 0; [run 37935245178](https://github.com/git-ksk/reasoning-harness/actions/runs/37935245178)). Release PR #482 passed 23/23 CI checks. Frozen MCP v1 retains its stdin-write limitation; supported v2/v3 successors are deadline-bounded. CLI 0.5.4 now distributes Engine 0.6.1; the older CLI 0.5.3 still contains Engine 0.5.0. See [release notes](engine-0.6.1-release.md).

## Separate engine track: Harness Engine 0.6.0

Reasoning/correctness changes are intentionally separated from CLI product UX work.

Harness Engine 0.6.0 is release-complete. The accepted delta adds target-local evidence-need routing (#461), evidence-target relevance and relation qualification (#462/#468), and source-attributed qualified prose without truth promotion (#463). The supporting bounded structured-output fallback (#460) changes transport compatibility only and does not create authority.

Independent frozen acceptance completed before promotion: #461 holdout run `35965160995` passed Mistral + Google 26/26 with zero correctness/utility failures; #462/#468 holdout-v12 run `37218652869` passed Mistral / Google / Groq 26/26 with zero authority, wrong-target, false-rejection, or utility failures; #463 holdout-v2 run `37326666360` passed Mistral / Google / Groq 18/18 with citation coverage 100% and every hard gate at zero. Historical failures remain immutable.

`engine-v0.6.0` is an independent Engine source release. Published `reason-v0.5.3` binaries remain immutable on Engine 0.5.0; the separate CLI 0.5.4 release has since adopted Engine 0.6.1. See [Engine 0.6.0 release notes](engine-0.6.0-release.md), [Product, engine, and contract versioning](versioning.md), and the [product roadmap](product-roadmap.md).

## Planned Engine 0.7.0 — not yet released

[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10) / [Engine 0.7.0 detailed roadmap](engine-0.7.0-roadmap.md) begin with a frozen Engine 0.6.1 cross-source residual baseline. Conditional workstreams cover trusted provenance lineage, explicit temporal revision relationships, conflict-preserving source synthesis, and target-local answerability. None is promoted into the released Engine until measurable benefit, separate fresh independent acceptance, and exact-head release gates pass. Existing Engine and CLI versions remain unchanged.

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

Harness Engine 0.6.0 is implementation-, acceptance-, and release-complete through #461/#462/#468/#463 and is published independently as `engine-v0.6.0`. The current distributed CLI is Reason CLI 0.5.4 / Engine 0.6.1. Legacy CLI 0.5.3 binaries retain Engine 0.5.0. #465 remains a separate, non-semantic CI-fixture reliability follow-up rather than an Engine correctness blocker.

## Research posture

Reasoning Harness does **not** claim that open-world reasoning is solved. Hard correctness still depends on deterministic structure and trusted evidence/oracles where a hard answer exists. Model-backed semantic mechanisms remain advisory/restrictive unless separately promoted through a measured authority-safe path.

For the detailed historical record—including earlier semantic studies, holdouts, RSD lines, provider investigations, product dogfood, and natural-language E2E iterations—use [Project status history](project-status-history.md) and the current [documentation index](README.md).
