# Product roadmap: Reason CLI and Harness Engine

[日本語](product-roadmap.ja.md) | English

This is the **current forward roadmap**. The previous release-by-release implementation ledger is preserved in [Product roadmap history](product-roadmap-history.md).

The project now separates product UX work from reasoning/correctness changes so a better CLI does not imply a different reasoning engine.

## Current coordinates

```text
Published split CLI:
  Reason CLI 0.5.5
  Harness Engine 0.6.1

Latest independent Engine source release:
  Harness Engine 0.7.2 (`engine-v0.7.2`)

Latest completed integration:
  Reason CLI 0.5.4 -> Harness Engine 0.6.1 (#484)

Final unified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2
```

`v0.4.2` is the final unified historical release. The split CLI line is active through `reason-v0.5.5` (Harness Engine 0.6.1); CLI, Engine, and machine-contract identities advance independently. `reason-v0.5.2` remains immutable on Engine 0.4.2, while `reason-v0.5.3` adopts the independently released Engine 0.5.0. See [versioning](versioning.md).

## Next development — Engine0.7.3 repair and re-evaluation

[0.7.3 roadmap](engine-0.7.3-roadmap.md) / [plan #538](https://github.com/git-ksk/reasoning-harness/issues/538) / [evaluation #531](https://github.com/git-ksk/reasoning-harness/issues/531) define **#520 repair → safe runner and actual normal-path wiring → small development checks → fresh independent real-source acceptance → source release → dedicated CLI adoption/distribution**.0.7.3 is planned and unpublished;0.7.2 remains an immutable historical comparison arm and its residual blocks adoption. #488/#489/#491 remain conditional.

[First v2/v3 results](engine07-realworld-development-results.md) remain FAIL/incomplete. #535 diagnostic retention is merged; #537 operational bridge passes26 offline tests, but live runner integration is pending. Groq daily exhaustion still blocks starts. No normal-answer utility gain or fresh independent acceptance is established; adoption1/6 records development comparison/limits/NO-GO only. Published CLI0.5.5 / Engine0.6.1 and #506 package acceptance remain separate.

Fresh independent v27 at candidate `26c98ad` is **FAIL / NO_GO**: each API TP24/FP0/TN30/FN6,Precision1.000/Recall.800/F1 **.888889**. Unsafe admissions,operational errors and incomplete observations0,but fixed F1>=.90/Recall>=.85 fail. Joint54/60 is diagnostic. Preserve99new+829old=928 references byte-exact;retry/rescore/relabel0. Originalv26 F1.846154/NO_GO unchanged. Existing1400 controls,original14,history597,workspace1599PASS/0FAIL/8ignored and same-source evidencehead826c98d CI19/19 remain separate. Addingv27 yields1454/1460 known cases. Actual hooks show5 typed_reference_not_excluded and025 unresolved_adverse_scope. Repair shared complete printed types,same local role/owner and bounded parenthetical property corrections while preserving unknown/conflicting owner,target/global denials,then verify1460. New claim arithmetic/display hash/UTF-8 span preparation passes32 offline tests;it is not the complete semantic scorer or M0. Real24=0/24,actual-answer F1 unmeasured,Groq stopped,source merge/release/CLI adoption NO_GO. See [decision](engine073-v27-decision.md),[original first](../evaluation/engine073-launch-independent-v27/README-results.md),[claim component](../evaluation/engine073-answer-metrics-development-v1/README.md).

[Real-answer metrics preparation](engine073-realworld-metrics-preparation.md) defines claim-grounding precision,required-claim recall/F1,citations,correct/avoidable abstention,usefulness,operational missingness and per-provider accounting. PREPARATION_ONLY;notM0/M1 and no answers scored. New scorer,actual callback/dependency closure,numeric live reserve/headroom/currency self-budget and an active-feature ordinary CLI path remain. [Previous01dc same-host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md) records identical grounded answer/acquisition1/public relevance API0 for one fixed Node path;directv17/v30 controls confirm counters. This neither establishes all paths inactive nor the repair's final-answer effect. #541 shared runner is merged;#540 live preparation remains OPEN. #531 freshreal24=0/24/adoption1/6,Groq prohibition,provider/model/Oracle additional0 remain.

## Product goal

Reasoning Harness is deliberately narrower than a general-purpose agent framework:

> Give users, developers, and automation a low-friction AI interface whose factual output remains bounded by inspectable evidence, verification, uncertainty, and failure semantics owned by the Harness rather than by model confidence.

The core product rule is unchanged:

```text
model proposes
Harness verifies / qualifies / abstains
```

### Engine 0.6.1 adoption — Reason CLI 0.5.4 (#484)

The independently accepted Engine 0.6.1 is adopted under the new CLI 0.5.4 coordinate, with signed native archives, verified update/rollback, and immutable prior CLI 0.5.3 artifacts.

## Track A — Reason CLI 0.5.x: general-use productization

**Current published pair: `reason-v0.5.5` / Harness Engine 0.6.1.** `reason-v0.5.3` remains immutable on Engine 0.5.0 and `reason-v0.5.2` on Engine 0.4.2; #455 and #484 completed the separate Engine-adoption releases.

The base `reason-v0.5.0` release and the 0.5.1/0.5.2 patch line shipped on Engine 0.4.2; `reason-v0.5.3` shipped the accepted Engine 0.5.0, and `reason-v0.5.4` now ships Engine 0.6.1. The milestone P0 release gate and #455 Engine adoption are complete. The #375 P1 distribution acceptance criteria are now verified: Homebrew 0.5.3 physical tests passed and Microsoft published WinGet package `git-ksk.Reason` (latest 0.5.4), including physical Windows fresh-install acceptance. The remaining `doctor` link-detection correction is independently tracked in #504.

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

### Harness Engine 0.5.0 adoption — complete (#455)

`reason-v0.5.3` is published as **Reason CLI 0.5.3 / Harness Engine 0.5.0**. The adopting release preserved `reason-v0.5.2` immutability and did not reopen Engine semantics.

- release manifest/provenance binds CLI 0.5.3, Engine 0.5.0, and merge commit `e9148c737c6f9bf29ce7c9258d549f5c526dfb4a`;
- supported-platform package candidate, installer, no-Rust consumer, lifecycle, credential-store, and CLI smoke gates passed;
- live `reason-v0.5.2` -> `reason-v0.5.3` update and `reason-v0.5.3` -> `reason-v0.5.2` rollback both surface the Engine transition and fail closed without `--allow-engine-change`;
- explicit-consent update and rollback succeeded using the published provenance-verified releases;
- Engine 0.5.0 frozen evidence remains unchanged.

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

Engine 0.5.0 is now a closed release baseline and is distributed by Reason CLI 0.5.3. Further semantic Engine work requires a new measured gap and a new Engine identity; completed #455 adoption did not reopen or rewrite the 0.5.0 evidence.

## Track C — Harness Engine 0.6.0: target-local evidence semantics — release complete

Engine 0.6.0 promotes the measured production-gap sequence #461 -> #462/#468 -> #463 without rewriting Engine 0.5.0 or any frozen historical observation.

- **#460 structured-output transport helper:** bounded JSON-Schema -> JSON-object fallback preserves task/budget identity and does not create evidence or authority.
- **#461 evidence need:** independent holdout run `35965160995` passed Mistral + Google 26/26 with correctness violations 0, utility misses 0, and provider failures 0.
- **#462/#468 relevance + relation:** final independent holdout-v12 run `37218652869` passed Mistral / Google / Groq 26/26 with authority failures 0, wrong-target Relevant 0, false relevance rejection 0, and utility misses 0.
- **#463 source attribution:** independent holdout-v2 run `37326666360` passed Mistral / Google / Groq 18/18 with useful attribution 6/6, citation coverage 100%, provider failures 0, and every hard gate 0.
- **#472 release closeout:** `reasoning-harness-core` advances to 0.6.0 under `engine-v0.6.0`; release promotion changes no accepted runtime semantics. Published `reason-v0.5.3` remains on Engine 0.5.0; the separate `reason-v0.5.4` release has now adopted Engine 0.6.1.

## Track D — Post-0.6.0 maintenance (Engine 0.6.1 released)

The Engine 0.6.0 milestone and `engine-v0.6.0` release are closed and immutable. Work merged after that tag is **not** part of the already-published Engine 0.6.0 source release.

- **#474 bounded launch-language regression:** recognize the affirmative `has made … generally available` frame only with ordered, local wording and preserve fail-closed handling of negation. Regression tests cover the affirmative and multiple negative forms; this is post-release Engine source maintenance, not a retroactive 0.6.0 acceptance change.
- **#475 / milestone #9 Engine 0.6.1 closeout:** `engine-v0.6.1` is independently published. PRs #480/#481/#482 passed exact-head CI, including the release 23/23 checks. Independently precommitted v2 passed 30/30 deterministic, and Mistral/Google/Groq live 36/36 on run 37935245178; original independent v1 FAIL remains preserved.
- **#476 wrong-target launch binding:** require target-owned `has made … generally available` evidence, and reject cross-clause, other-subject, and negated wording without trusting advisory model classification.
- **#479 launch factuality:** latest v17/v30 requires target-owned affirmative launch evidence; conditional, denied, interrogative, fictional, and unverified statements fail closed while historical frozen relation semantics stay unchanged.
- **#477 legacy MCP timeout:** preserve the historically frozen v1 adapter and mitigate its unbounded stdin-write limitation by steering supported integrations to the already deadline-bounded v2/v3 successors; add a reproducible v2 blocked-stdin regression. This provider safety follow-up is included in the 0.6.1 **source release** scope. The v1 limitation itself remains a documented migration concern.
- **#465 CI fixture reliability:** the intermittent legacy MCP subprocess transport test remains tracked separately from semantic correctness and frozen evaluation.
- **#467 / #470 dependency maintenance:** routine Cargo dependency updates are independent of the Engine semantic acceptance record.
- **Engine 0.7 source line:** 0.7.0 shipped the accepted source-reconciliation scope; 0.7.1 packages merged #512/#514/#515 corrections. Published CLI adoption remains separate.

## Track E — Harness Engine 0.7: corrective patch and deferred hypotheses

[Engine 0.7.0](engine-0.7.0-release.md) shipped #490 opt-in host-reviewed target-local attributed presentation after #487 baseline, #496 reviewer-host controls and #492 frozen independent fictional-source acceptance. The original Conflict, claims and citations remain intact; model advice cannot authorize a review or establish external truth. Same-user automated keyring E2E is not independently authenticated human approval.

[Engine 0.7.1](engine-0.7.1-release.md) packages the merged #512 bilingual preparation and #514/#515 launch-frame corrections, preserving historical v16/v29, frozen observations and machine contracts. Its patch evidence and limits are documented separately from 0.7.0's provider observation.

**[Engine 0.7.2](https://github.com/git-ksk/reasoning-harness/releases/tag/engine-v0.7.2) is published as a normal Engine-only source patch**, at `296690a86e049b75ee693ae5032a71504366137c` via [release PR #525](https://github.com/git-ksk/reasoning-harness/pull/525). [#520](https://github.com/git-ksk/reasoning-harness/issues/520) and [#521](https://github.com/git-ksk/reasoning-harness/issues/521) repairs from #523 preserve governing fictional/retracted/false context and original strike/negation markers in v17/v30. Final release-candidate validation passed 1490 workspace tests (0 failures, 8 ignored), strict Clippy, format, 20 fixtures and 13 original archive checks; exact release-PR-head CI 23/23 and merged-source CI 22/22 passed. Fresh separately authored v10 50/50 and historical v16/v29 equality on 597 inputs remain preserved as accepted-source evidence. See [release notes](engine-0.7.2-release.md), [repair report](launch-context-markers-520-521.md) and [0.7.2 roadmap](engine-0.7.2-roadmap.md). [#524](https://github.com/git-ksk/reasoning-harness/issues/524) and [milestone #11](https://github.com/git-ksk/reasoning-harness/milestone/11) record publication closeout. Prior tags and first failed observations are unchanged. Current published CLI is0.5.5 / Engine0.6.1; synthetic/local/CI acceptance does not establish new live-provider correctness, external factual truth or real-user utility.

[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10) / [parent #486](https://github.com/git-ksk/reasoning-harness/issues/486) remain open for explicitly deferred [#488 source lineage](https://github.com/git-ksk/reasoning-harness/issues/488), [#489 trusted revision supersession](https://github.com/git-ksk/reasoning-harness/issues/489) and [#491 additional answerability/reacquisition](https://github.com/git-ksk/reasoning-harness/issues/491). No concrete supported product failure requiring these new mechanisms was established; demonstrate a measured use case and freeze separate acceptance before implementation. They do not block the narrow source patches.

The two repaired issues defined the 0.7.2 corrective scope; the three deferred items are possible improvements requiring experiments, not three additional confirmed bugs. This is not an exhaustive absence-of-defects claim. #465's legacy MCP CI reliability and #506's CLI distribution gate remain separate.

CLI 0.5.5 / Engine 0.6.1 is now published; #506 retains the separate WinGet/package acceptance. Engine 0.7 CLI adoption is gated by #531.

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
