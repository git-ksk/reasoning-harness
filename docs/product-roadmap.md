# Product roadmap: Reason CLI and Harness Engine

**2026-10-30:** Lossless final-request JSON compaction in [#569](https://github.com/git-ksk/reasoning-harness/issues/569) / [PR570](https://github.com/git-ksk/reasoning-harness/pull/570) merged at `d61d7b8` after bounded independent source review and22 passing CI checks. The saved request shrinks13,365→11,186bytes (fallback13,683→11,015) without removing information. A distinct [ordinary v2 preparation](../evaluation/cli-normal-v2/README.md) applies the same host `54eee37` to both Engine arms and preserves all v1 source/workflow/receipt/raw records. Next: independent v2 source review, actual runtime/image/proof admission, fresh readiness/receipt, then48 Mistral jobs and new F1. The old run retains2 complete/1 failed/45 suppressed and unavailable full F1. New model execution, fresh independent accuracy acceptance and CLI adoption remain incomplete.

**2026-10-29:** #566 merged via [PR567](https://github.com/git-ksk/reasoning-harness/pull/567) after independent source review and22 passing CI checks. Ordinary preparation [PR565](https://github.com/git-ksk/reasoning-harness/pull/565) also merged, with authenticated run-bound scoring and pre-start image verification independently checked at source level. Next: independently accept the [actual AMD64 runtime](https://github.com/git-ksk/reasoning-harness/actions/runs/40787270979), then measure new Mistral F1. Model execution, new F1 and fresh independent accuracy acceptance remain incomplete.

**2026-10-28:** CLI repairs merged in [PR561](https://github.com/git-ksk/reasoning-harness/pull/561) after bounded independent reviews and22 green CI checks. [PR565](https://github.com/git-ksk/reasoning-harness/pull/565) prepares48 ordinary Mistral jobs. Bind scoring to the actual run/artifact/raw before re-review so historical observations cannot enter the new measurement. Model-free API verification is complete; new F1, fresh acceptance and CLI adoption remain incomplete.

<!-- ordinary-cli-policy-20261026-start -->
**Next policy (2026-10-26, user approved):** Follow the [ordinary CLI plan](engine-073-cli-evaluation-plan.md). Real-model ordinary F1/P/R, safety, citations, non-regression and efficiency drive CLI evaluation; auxiliary APIs receive model-free contract/authority/tampering/replay/compatibility checks. Retain the old0.400000 model-assisted helper F1 as a reference, not an API-unit or independent CLI adoption gate. First check APIs offline and diagnose saved Mistral outputs, repair reproduced defects, then run an ordinary-only comparison. Gemini0.933333 on23 tasks is encouraging but full final-candidate and fresh independent acceptance remain incomplete. Groq held; adoption pending. Historical v2 conditions/results remain immutable.
Model-free auxiliary API contract review passed. Independent diagnosis of48 saved Mistral ordinary runs led to CLI request-limit/identity and candidate-shape repairs for Issues562/563, pending bounded review. No new F1 yet.
<!-- ordinary-cli-policy-20261026-end -->

<!-- real24-v2-current-start -->
**Current, 2026-10-26:** [Actual-API v2 and independent first F1 (Japanese)](cli-paired-real24-v2-20261026.ja.md) are recorded. Both providers completed72 jobs;192 new observations were independently annotated and scored once with the frozen driver. Helper case F1 improved0.222222→0.400000 on both providers (precision1.000/recall0.250), below the gate. Google ordinary descriptive F1 is0.933333 on the same23 complete tasks in both arms. Mistral ordinary subsets differ (15/13 complete); no full-cohort improvement claim is valid. Ordinary22 missing observations remain intact and full F1 stays null. Future capture repairs for repeated snapshots and empty action events passed bounded independent review. Groq held; fresh accuracy cases0; Engine release/CLI adoption NO_GO.
<!-- real24-v2-current-end -->

**Checkpoint, 2026-10-25:** [PR555](https://github.com/git-ksk/reasoning-harness/pull/555) is merged after bounded independent PASS and CI22/22. The first PR556 review found three evaluator defects: withheld-answer projection acceptance, keyless-target rejection and a stale preflight expectation. Fixed source `552a6d7` passed bounded independent re-review. Both Linux arms/all ten binaries, 17 actual CLI controls, 18 provider audit controls, 51 evaluator checks, seven cancellation/reservation/restart controls and registry parity pass. See the [Linux checkpoint](../evaluation/cli-repair-paired-v1/linux-checkpoint.json).

The [GitHub paired-runtime build also completed](cli-paired-runtime-checkpoint-20261025.ja.md). Next: independently verify the successor planner/runtime profile, a new execution receipt and current account readiness, then measure consumed24 F1/P/R/citations/efficiency. This checkpoint adds zero model requests, accuracy questions or F1 recomputations. The owner now reports Mistral can resume; Mistral/Google still require a new readiness/receipt. Original scores, the Groq hold, fresh independent acceptance and Engine adoption gates remain unchanged.

**Checkpoint, 2026-10-24:** PR551 and [PR552](https://github.com/git-ksk/reasoning-harness/pull/552) are merged. A same-proposition unacquired registry candidate could abort an otherwise valid answer; [#553](https://github.com/git-ksk/reasoning-harness/issues/553) / [PR555](https://github.com/git-ksk/reasoning-harness/pull/555) repair that boundary, with CI22/22 green. PR555 remains Draft because the final independent review stopped at the reviewer's usage limit.

The [next common-CLI comparison preparation](../evaluation/cli-repair-paired-v1/README.md) is saved. Both historical core trees build locally with the identical acquisition-control patch; 17 inert connection controls and 44 evaluator controls pass. These checks do not measure F1 improvement; original scores remain unchanged. Final Linux containment, independent review, account readiness and a new execution freeze precede the consumed24 development measurement. Fresh independent acceptance and CLI adoption remain separate. Mistral/Groq holds remain; Google's new execution conditions are incomplete. See the [offline checkpoint](../evaluation/cli-repair-paired-v1/offline-checkpoint.json).

**2026-10-22:** PR551 merged; [#491](https://github.com/git-ksk/reasoning-harness/issues/491) / [PR552](https://github.com/git-ksk/reasoning-harness/pull/552) now repairs measured early acquisition completion and missing plan targets. [Boundaries](acquisition-coverage.md). F1 improvement remains unmeasured.

**2026-10-21 component work:** [#550](https://github.com/git-ksk/reasoning-harness/issues/550) / [PR551](https://github.com/git-ksk/reasoning-harness/pull/551) preserves host-bound source conditions, citations and unresolved inputs through ordinary CLI finalization. [Contract and scope](cli-answer-context.md). Saved-artifact presentation checks do not change the frozen F1; new paired measurement and Engine adoption remain separate gates.

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

## Next development — evaluate real use first

[#531](https://github.com/git-ksk/reasoning-harness/issues/531) / [milestone #12](https://github.com/git-ksk/reasoning-harness/milestone/12) order **real-data0.6.1/0.7.2 comparison → necessary fixes → fresh reacceptance → CLI adoption → distribution**. A new present-tense retraction/non-event residual reproduces in published0.7.2; #520 is reopened and adoption is NO-GO. #488/#489 remain conditional on measured need. The measured acquisition-control subset of #491 is now in PR552; its broader feature and fresh acceptance gates remain open.

Normal question-to-final-answer E2E and opt-in helper evaluation have separate denominators. [First three-provider development v2/v3 results](engine07-realworld-development-results.md) are recorded; both FAIL with incomplete measurement. No normal-answer utility gain is established; helper display gain2 for Mistral/Google is separate. Groq daily quota is exhausted and additional starts are blocked. #535 diagnostics, wiring next-run operational controls, a pinned #520 successor and fresh independent acceptance remain. Adoption1/6 (comparison record only). Published CLI0.5.5 / Engine0.6.1 and #506 package acceptance remain separate.

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

[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10) / [parent #486](https://github.com/git-ksk/reasoning-harness/issues/486) remain open for explicitly deferred [#488 source lineage](https://github.com/git-ksk/reasoning-harness/issues/488), [#489 trusted revision supersession](https://github.com/git-ksk/reasoning-harness/issues/489) and [#491 additional answerability/reacquisition](https://github.com/git-ksk/reasoning-harness/issues/491). The broader new mechanisms remain deferred. The October22 saved baseline/candidate artifacts establish a narrower #491 control-flow/display failure, now repaired in PR552; this does not adopt the wider sufficiency/reacquisition feature. Freeze separate measurement and acceptance before claiming its benefit. They do not block the narrow source patches.

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
