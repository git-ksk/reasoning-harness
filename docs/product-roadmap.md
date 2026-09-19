# Product roadmap: Reason CLI and Harness Engine

[日本語](product-roadmap.ja.md) | English

This is the **current forward roadmap**. The previous release-by-release implementation ledger is preserved in [Product roadmap history](product-roadmap-history.md).

The project now separates product UX work from reasoning/correctness changes so a better CLI does not imply a different reasoning engine.

## Current coordinates

```text
Current split preview:
  Reason CLI 0.5.2
  Harness Engine 0.4.2

Final unified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2

Active engine line:
  Harness Engine 0.5.0 — Verified Investigation Utility
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

**Engine baseline: fixed at 0.4.2.**

The base `reason-v0.5.0` release and the 0.5.1/0.5.2 patch line are shipped. The milestone's P0 release gate is complete; the only remaining milestone issue is the P1 distribution follow-up #375. Homebrew physical acceptance is complete, while the WinGet community manifest has passed validation and CLA checks and is waiting for community moderator approval.

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

## Track B — Harness Engine 0.5.0: verified investigation utility

This track owns changes that may alter reasoning/correctness behavior and therefore require fresh evidence.

Current status/order:

- **#248 finalization/grounding — complete:** PR #435 merged with fresh frozen `issue-248-finalization-e2e-v2` acceptance (3/3 cases, zero correctness-boundary violations, zero session external-call replay);
- **#247 evaluator semantics — complete:** frozen v11 separates path observability, product utility, hard correctness, and operational completeness without rescoring frozen v9;
- **#282 repeated-trial planner reliability — next:** characterize stochastic planner/action behavior with frozen repeated-trial identities and exact denominators;
- **#283 deterministic Harness-owned action materialization — after #282 baseline:** evaluate moving mechanically safe executable-action ownership into deterministic Harness control flow, with a separate fresh adoption holdout.

The exact engine milestone/issue definitions remain the source of truth for scope. No Engine 0.5.0 semantic/utility change should land under the label of a CLI-only productization change.

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
