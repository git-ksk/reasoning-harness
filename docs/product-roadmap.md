# Product roadmap: Reason CLI and Harness Engine

[日本語](product-roadmap.ja.md) | English

This is the **current forward roadmap**. The previous release-by-release implementation ledger is preserved in [Product roadmap history](product-roadmap-history.md).

The project now separates product UX work from reasoning/correctness changes so a better CLI does not imply a different reasoning engine.

## Current coordinates

```text
Published split CLI:
  Reason CLI 0.5.2
  Harness Engine 0.4.2

Latest independent Engine source release:
  Harness Engine 0.5.0 (`engine-v0.5.0`)

Next integration:
  Reason CLI 0.5.x patch -> Harness Engine 0.5.0 (#455)

Final unified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2
```

`v0.4.2` is the final unified historical release. The split CLI line is already active (`reason-v0.5.0`, `reason-v0.5.1`, `reason-v0.5.2`); CLI, Engine, and machine-contract identities now advance independently. See [versioning](versioning.md).

## Product goal

Reasoning Harness is deliberately narrower than a general-purpose agent framework:

> Give users, developers, and automation a low-friction AI interface whose factual output remains bounded by inspectable evidence, verification, uncertainty, and failure semantics owned by the Harness rather than by model confidence.

The core product rule is unchanged:

```text
model proposes
Harness verifies / qualifies / abstains
```

## Track A — Reason CLI 0.5.x: general-use productization

**Published baseline through `reason-v0.5.2`: Harness Engine 0.4.2.** Existing release artifacts remain immutable; the next CLI patch adoption is tracked separately in #455.

The base `reason-v0.5.0` release and the 0.5.1/0.5.2 patch line are shipped. The milestone P0 release gate is complete. Remaining P1 work is #375 distribution follow-up plus #455 adoption of the released Harness Engine 0.5.0 into the next CLI patch. Homebrew physical acceptance is complete, while the WinGet community manifest has passed validation and CLA checks and is waiting for community moderator approval.

This track makes `reason` a mature terminal product without changing the underlying authority semantics.

### Phase 1: install, trust, and first answer

- native installers without requiring Rust/Cargo;
- release integrity/signing/notarization where appropriate;
- update, explicit rollback, and uninstall lifecycle;
- project trust for executable/authority-bearing local configuration;
- OS-native secure credential storage;
- `reason auth` and guided `reason setup`;
- provider/model discovery and explicit default selection.

### Phase 2: daily terminal UX

- no-argument interactive mode;
- simple continue/resume/session selection;
- crash/concurrency-safe managed session storage;
- clear verified / qualified / unresolved presentation;
- provider usage and enforceable budget visibility;
- discoverable config commands;
- progress/retry/cancellation UX;
- shell help/completion and accessible terminal rendering.

### Phase 3: external acquisition UX

- subprocess environment isolation so unrelated ambient secrets are not inherited;
- guided read-only MCP add/list/inspect/test/remove;
- secure remote MCP/OAuth lifecycle after the local path is stable.

MCP/resolver output remains acquired data rather than correctness authority.

### Phase 4: diagnostics and recovery

- `reason doctor`;
- actionable credential/model/quota/network/config/trust/MCP/session/update errors;
- explicit provider/model retirement/fallback policy with no silent execution-identity switch;
- proxy/custom-CA/headless diagnostics without insecure bypass guidance.

### Phase 5: fresh-install release gate — complete

`reason-v0.5.0` was tagged only after supported-platform fresh-install acceptance covered installation, trust, setup, secure credential handling, first answer, interactive follow-up, session resume, diagnostics, local privacy, subprocess secret isolation, update/rollback/uninstall, and existing JSON automation compatibility. The same gate remains regression coverage for the 0.5.x patch line.

The complete P0/P1 issue list and acceptance matrix live in the [Reason CLI 0.5.0 roadmap](reason-cli-0.5-roadmap.md).

### Next patch integration: adopt Harness Engine 0.5.0 (#455)

Engine 0.5.0 is already release-complete; #455 is a **CLI adoption/distribution** task, not new Engine semantic work.

- keep the published `reason-v0.5.2` artifacts immutable on Engine 0.4.2;
- use the next available Reason CLI 0.5.x patch coordinate (expected 0.5.3 if still available);
- publish provenance that identifies Engine 0.5.0 exactly;
- preserve explicit user consent when `reason update` crosses the Engine 0.4.2 -> 0.5.0 boundary;
- validate update/rollback, package-manager ownership, fresh-install, no-Rust consumer, and supported-platform packaging before release.

## Track B — Harness Engine 0.5.0: verified investigation utility — complete

This track owns changes that may alter reasoning/correctness behavior and therefore require fresh evidence.

Current status/order:

- **#248 finalization/grounding — complete:** PR #435 merged with fresh frozen `issue-248-finalization-e2e-v2` acceptance (3/3 cases, zero correctness-boundary violations, zero session external-call replay);
- **#247 evaluator semantics — complete:** frozen v11 separates path observability, product utility, hard correctness, and operational completeness without rescoring frozen v9;
- **#282 repeated-trial planner reliability — complete:** frozen `planner-reliability-v1` completed 5/5 primary trials on both routine provider targets; Mistral strict planner success was 5/5, Google 4/5 because one inadmissible action proposal was safely rejected, with zero correctness-boundary violations;
- **#283 deterministic Harness-owned action materialization — complete:** the accepted candidate moved mechanically safe exact-key read-only action materialization into Harness control flow with `reason-investigation-intent-v1` / `target-intent-materialization-v1`; frozen v3 adoption evidence passed the architecture-path gate without changing authority/finalization semantics.
- **#443 initial Engine 0.5 final cross-model acceptance — complete:** fresh `engine-0.5-final-v2-freeze` established the accepted baseline across the validated-required rows;
- **#445/#446/#450 final hardening — complete:** separated finalization correctness from planner target-recall utility, made explicit-fact session correction continuity deterministic, and materialized mechanically unique admitted exact facts under Harness control without weakening verification authority;
- **#452 final-v3/versioned release closeout — complete:** `engine-0.5-final-v3-freeze` canonical run `35457038163` passed all six independently required rows and 18/18 fresh cases with correctness-boundary violations 0 and session external replay 0;
- **Versioned Engine release — complete:** `engine-v0.5.0` points to `4fd6acc85511f286fd6b2f7c9439665b7819206a`, where `reasoning-harness-core` reports 0.5.0. Milestone #4 is closed with zero open issues.

Engine 0.5.0 is now a closed release baseline. Further semantic Engine work requires a new measured gap and a new Engine identity; CLI adoption of this already-released Engine belongs to #455 and must not reopen or rewrite the 0.5.0 evidence.

## Promotion rule

A new reasoning mechanism does not become a supported product behavior merely because it looks promising in one experiment.

Promotion requires, as appropriate:

1. a clearly stated authority boundary;
2. deterministic regression coverage;
3. calibration/development evidence separated from independent evaluation;
4. frozen or otherwise provenance-stable evaluation identity;
5. operational stabilization separate from semantic scoring;
6. explicit runtime/contract identity and rollback where needed;
7. supported CLI/API integration without weakening existing fail-closed behavior.

Historical failed or inconclusive studies remain evidence; they are not rewritten into passes after later fixes.

## What is deliberately not on this roadmap

The current general-use productization does not aim to turn Reason into:

- a write-capable coding agent;
- a background autonomous agent platform;
- a general browser/RAG crawler inside the correctness core;
- a system that silently changes models/providers to complete a task;
- a product that treats retrieved/tool/model text as trusted merely because it is convenient.

Those would require separate product and authority designs.

## Historical releases and research

For v0.1.0 through v0.4.2 implementation chronology, completed milestone details, old evaluation coordinates, and historical research provenance, use [Product roadmap history](product-roadmap-history.md).

For what is currently released and the main user-facing gaps, use [Project status](project-status.md). For the full document map, use the [documentation index](README.md).
