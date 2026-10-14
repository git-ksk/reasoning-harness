# Harness Engine 0.7.2 — Launch evidence safety patch

[日本語](engine-0.7.2-release.ja.md) | English

## Release coordinates

- Package: `reasoning-harness-core 0.7.2`, `publish = false`.
- Published tag: `engine-v0.7.2`, a **normal Engine-only source patch** (`prerelease=false`, `latest=false`) under the [release policy](release-policy.md). Published at `296690a86e049b75ee693ae5032a71504366137c` via [PR #525](https://github.com/git-ksk/reasoning-harness/pull/525); [#524](https://github.com/git-ksk/reasoning-harness/issues/524) records closeout. Tagged documents retain their preparation snapshot; these current notes and the GitHub release record publication.
- Implementation is identical to accepted [PR #523](https://github.com/git-ksk/reasoning-harness/pull/523), merged at `0e5b3389af61d55422205b83830c634f49d024bd`. This release preparation changes package identity, its lock record, pinned CI/version acceptance expectations and documentation only.
- `evidence_relevance.rs` SHA-256: `5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5`.
- Published Reason CLI remains **0.5.4 / Engine 0.6.1**. A source checkout identifies itself as CLI 0.5.4 / Engine 0.7.2. This release distributes no new CLI binary or crates.io package. CLI adoption and #506's CLI 0.5.5 / Engine 0.6.1 candidate remain separate acceptance tracks.

## Included corrections

- **#520 governing context:** preserve fictional, retracted and explicitly false context across headings, titles and sentences before accepting an affirmative launch fragment. Known adverse context with unresolved scope withholds conservatively; clearly foreign/local-feature context does not deny a legitimate target launch.
- **#521 original markers:** inspect original strikethrough and negation markers before normalization, including sentence-spanning strike scope. Preserve legitimate punctuation, dotted names and target ownership.
- Corrections apply to successor relevance qualification **v17** and materialization **v30**. They fix false `Relevant` / requested-relation qualification; they do not demonstrate downstream wrong answers or external-fact `Known` / `Supported` promotion.
- Historical v16/v29, machine/session contract IDs, frozen corpora, first observations and published Engine/CLI tags remain immutable. #488 source lineage, #489 trusted supersession and #491 extra answerability/reacquisition remain deferred outside this patch.

## Validation and limits

The [repair report](launch-context-markers-520-521.md) and [validation bundle](../evaluation/launch-520-521-final-validation/README.md) preserve source identities, the original 56-case audit and all first observations. Fresh separately authored v10 first observation passed **50/50** (25 positive, 25 negative; false Relevant, negative requested-relation acceptance, positive misses and API errors all zero) against candidate `2f12d4c61546e60d31026d9b691c9eb369c37957` after corpus freeze. Earlier RED first observations are retained, not rescored as new independent acceptance.

Accepted source validation: **1,490 PASS / 0 FAIL / 8 ignored**, strict all-target Clippy, format, standard fixtures **20/20** with zero unsupported accepted claims, original frozen archive **13/13** exact files, reconciliation authority/replay **22/22**, and full historical v16/v29 output equality against released 0.7.1 on **597 complete inputs**. Six ignored tests are pre-existing; two historical v1/v3 runners retain original expectations that conflict with the later factuality floor. Corrected expectations are tested separately as seen-data development controls.

This is bounded synthetic, local and CI evidence. No fresh live-provider, external factual-truth, real-user utility, physical-device or independently authenticated human acceptance is claimed. Text recognition may conservatively abstain on unsupported wording. Historical 0.7.0 provider evidence is not new patch acceptance. The separate CLI distribution's Windows physical gate does not apply to this Engine-only source tag.

## Final release verification

- [Release PR #525](https://github.com/git-ksk/reasoning-harness/pull/525): exact head `f6c80c1d6c65bd0031ed72733de9a75fe7e9987d`, **23/23 checks SUCCESS**.
- Published merge `296690a86e049b75ee693ae5032a71504366137c`: **22/22 checks SUCCESS**; merge tree is identical to the verified release head.
- Final release-candidate local validation: **1,490 PASS / 0 FAIL / 8 ignored**, locked offline workspace, strict all-target Clippy, format, standard fixtures **20/20**, unsupported accepted claims **0**, original archive **13/13**, source-checkout doctor **CLI 0.5.4 / Engine 0.7.2**. The local summary and unchanged-source/hash checks are recorded in PR #525.
- Implementation is byte-identical to accepted #523. Original frozen evidence and published Engine/CLI tag identities are unchanged. Initial packaging/version-pin CI failures remain visible; the corrected final head passed all checks.

- [credential-store-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132718): SUCCESS.
- [lifecycle-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132689): SUCCESS.
- [ci](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132621): SUCCESS.
- [fresh-install-acceptance](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132769): SUCCESS.
- [cli-platform-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132738): SUCCESS.

This Engine-only normal release requests `latest=false`. GitHub’s shared `/releases/latest` endpoint is not a CLI selection API; consumers must use `reason-v*` and the verified CLI/Engine pair. No new CLI binaries or crates.io package are distributed.

Merged-source macOS Intel CLI attempt1 failed before build because the runner could not connect to or resolve github.com. Only that job was retried once at the unchanged merge commit; the first failure is preserved in [attempt1](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132738/attempts/1). This is an operational CI retry, not independent semantic rescoring.
