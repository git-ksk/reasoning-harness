# Harness Engine 0.7.1 — Engine-only source patch

[日本語](engine-0.7.1-release.ja.md) | English

## Release coordinates

- Source package: `reasoning-harness-core 0.7.1`, `publish = false`.
- Tag: `engine-v0.7.1`; normal Engine-only source patch following immutable `engine-v0.7.0`.
- [Release policy](release-policy.md): the initial GitHub prerelease label was a classification error carried over from the older v0.x convention. It is corrected without changing the immutable tag, commit or assets. Tagged documentation keeps its original snapshot; current classification is recorded in the GitHub notes and this policy.
- Published Reason CLI remains **0.5.4 / Engine 0.6.1**. CLI adoption and native distribution are separate releases. The CLI 0.5.5 / Engine 0.6.1 candidate remains isolated under #506.
- This release preparation changes package identity and documentation only; Engine implementation is identical to merged main `3d712f6f7e76606229e014bd90cb97fbcfc9a6b3`.

## Included corrections

- **#514 / #515:** recover directly target-owned affirmative launch statements such as `was launched` and `has been launched`, including bounded decimal versions and dotted maker/product names. Preserve clause ownership, negation, speculation, simulation, scope and identity controls. See [launch-source behavior and evidence](launch-source-frames-514-515.md).
- **#512:** recover the bounded Japanese AWS Lambda preparation question against an owned affirmative English excerpt. Validate original punctuation and governing documentary context before recovery; reject erased negation/strikethrough/quotes, unrelated service names and denial/simulation/retraction contexts. See [bilingual preparation behavior and evidence](bilingual-preparation-512.md).
- Corrections are confined to successor qualification v17/materialization v30. Historical v16/v29 results, frozen corpora, first observations, existing wire/session contract IDs and published tags remain unchanged.
- The source snapshot also includes already merged WinGet doctor/lifecycle diagnostic maintenance (#504), provider fixture diagnostics (#465), and routine Tokio/libc dependency updates. This Engine tag does not distribute a new CLI binary. Legacy MCP v1 transport limitations remain; supported deadline-bounded v2/v3 are preferred.

## Validation and evidence limits

The final pre-release implementation passed 1,468 workspace tests with 0 failures and 6 existing ignored tests, strict all-target Clippy and formatting. The standard 20-case fixture evaluation retained accuracy 1.000 and zero unsupported accepted claims; the original Engine 0.7 archive retained all 13 SHA-256 checks. All 288 #514/#515 independent v1-v9 regression controls and 21 #512 follow-up negatives pass. Evidence is retained in `evaluation/launch-514-515-final-validation/` and `evaluation/bilingual-preparation-512-followup/`.

The launch corpus/review and earlier #512 review are synthetic evidence. The final #512 punctuation/context follow-up has local deterministic validation; the earlier independent review applies to its earlier source. Its original 133 expectations remain immutable: 132 still match and one explicit `This claim is false.` disclaimer now intentionally abstains. No fresh independent model review, live-provider acceptance, real-user utility, or physical-device acceptance is claimed for this patch. The original 0.7.0 three-provider reconciliation observation remains historical evidence for that release.

Text recognition remains bounded and can conservatively abstain on unsupported wording. #488 source lineage, #489 trusted temporal supersession and #491 additional answerability/reacquisition remain explicitly deferred pending measured use cases.

## Publication gate

Before publishing, require the exact release PR head and verified merged main to pass locked formatting, strict Clippy, workspace tests, four-platform CLI smoke, native candidate/no-Rust consumer and lifecycle CI. Verify CLI 0.5.4 / Engine 0.7.1 source-checkout identity, the unchanged implementation and frozen archives, then tag that verified merge commit. The GitHub release records the exact tag SHA and CI links. Previously published Engine and Reason CLI artifacts are immutable.
