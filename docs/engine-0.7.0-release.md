# Harness Engine 0.7.0 — Engine-only source release candidate

**Status: under review. Do not create `engine-v0.7.0` or a GitHub Release until exact-head CI and final release checks pass.**

- Source package: `reasoning-harness-core 0.7.0`, still `publish = false`.
- Published Reason CLI 0.5.4 / Engine 0.6.1 artifacts stay immutable. CLI adoption, packaging, upgrade and rollback are a separate release.
- New opt-in source reconciliation APIs expose target-local **source-qualified compatible wording** only after separate host-owned trusted review, while retaining every original source quote, citation and `Conflict`. A model's equivalence advice alone is not authorization, and no `Known`/`Supported` external truth is promoted.
- Additive contracts: `harness-source-reconciliation-view-v1`, `harness-source-reconciliation-target-answer-v1`, `harness-source-compatibility-trusted-review-v1`.
- The development-only `reason-source-review-local` is **not** the published Reason CLI. The Mac keyring/signature/revoke E2E used automated OS-user input and **does not demonstrate independent human consent**; same-user automation remains a stated limitation.

### Frozen independent synthetic acceptance

Runner `engine-0.7-independent-v1-runner-freeze` (`1124e5a7c34e42c088d81908c3cdc75aae2db309`) was frozen before the fresh corpus `engine-0.7-independent-v1-corpus-freeze` (`78f99559bf044013e66025bde1a612895f2d41e6`). [First canonical GitHub run #38013006323](https://github.com/git-ksk/reasoning-harness/actions/runs/38013006323): Mistral +3, Google +6, Groq +6 additional compatible attributed target displays over the unchanged Engine 0.6.1 baseline, no provider failures and zero hard gate violations. Model mistakes on scope/version mismatches were vetoed. All first artifacts and exact pre-frozen score reproduction are archived under `evaluation/engine-0.7-independent-v1-first-observation/` and SHA-256 checked in CI.

[Detailed bilingual evaluation](engine-0.7-independent-v1-result.md).

These are fictional controlled source statements: **not external-world truth verification, measured real-user utility or independently authenticated human approval**.

### Explicit non-adoption

#488 origin-lineage/independence, #489 trusted temporal supersession and #491 new target-answerability/reacquisition controls were not adopted because the frozen 0.6.1 baseline did not establish concrete unsafe transitions or necessary new product behavior. They remain OPEN, requiring new measured use cases. Existing conservative 0.6.1 validity, scope, verification and acquisition floors are unchanged.

### Compatibility and pending release gates

The existing serialized artifact, reasoning thread, source attribution, CLI main and managed-session source files are unchanged from Engine 0.6.1. Local Thread replay 11/11, CLI resume/fork 1/1, reconciliation 5/5 and target-presentation 1/1 PASS. Four no-Rust fresh-install consumer platforms passed in PR #500.

Before tagging: require this exact release PR head's locked fmt/Clippy/workspace tests, OS matrix CI, correct Engine/CLI version identity, immutable observation integrity and unchanged previously published CLI artifacts. Only then tag verified merged main `engine-v0.7.0`, publish the Engine-only source prerelease and record exact evidence. Never retag Engine 0.6.1 or alter Reason CLI 0.5.4.
