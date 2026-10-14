# Harness Engine 0.7.2 — Launch evidence safety patch

[日本語](engine-0.7.2-release.ja.md) | English

## Release coordinates

- Package: `reasoning-harness-core 0.7.2`, `publish = false`.
- Intended tag: `engine-v0.7.2`, a **normal Engine-only source patch** (`prerelease=false`, `latest=false`) under the [release policy](release-policy.md). [#524](https://github.com/git-ksk/reasoning-harness/issues/524) tracks publication; this preparation is not itself a published tag.
- Implementation is identical to accepted [PR #523](https://github.com/git-ksk/reasoning-harness/pull/523), merged at `0e5b3389af61d55422205b83830c634f49d024bd`. This release preparation changes package identity, its lock record and documentation only.
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

## Final publication gate

Record the exact final release PR head and require locked workspace tests, strict all-target Clippy, format, standard fixtures, original archive and source-checkout version checks. Require final-head CI success, including four-platform CLI, native candidate/no-Rust consumer and lifecycle smoke. Verify the merged source/tree and its CI before tagging that exact merge commit. Implementation must remain byte-identical to accepted #523; old tags and frozen evidence must remain unchanged.

The GitHub release records the tag/merge SHA, release PR and CI links after these gates pass. Engine-only publication requests `latest=false`; the shared GitHub `/releases/latest` endpoint is not a CLI selection API. CLI consumers must use the `reason-v*` namespace and verified CLI/Engine pair. Close #524 and milestone #11 only after verifying actual normal publication.
