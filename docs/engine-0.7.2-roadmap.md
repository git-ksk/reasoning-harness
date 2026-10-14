# Harness Engine 0.7.2 — Launch evidence safety patch

[日本語](engine-0.7.2-roadmap.ja.md) | English

**Source repairs accepted; normal Engine-only release preparation active, not yet published.** [Milestone #11](https://github.com/git-ksk/reasoning-harness/milestone/11) · [0.7 parent #486](https://github.com/git-ksk/reasoning-harness/issues/486). Updated 2026-10-14.

[PR #523](https://github.com/git-ksk/reasoning-harness/pull/523) repairs **two confirmed 0.7.1 relevance regressions** in successor source. Published `engine-v0.7.0` and `engine-v0.7.1` remain immutable. Published Reason CLI 0.5.4 continues to distribute Engine 0.6.1; #506's separate CLI candidate and any Engine 0.7 CLI adoption have their own acceptance.

## Required corrective scope

| Issue | Reproduced failure | Required behavior |
| --- | --- | --- |
| [#520 governing context](https://github.com/git-ksk/reasoning-harness/issues/520) | A fictional, retracted or explicitly false heading/title/other sentence governs an apparently affirmative launch excerpt, but v17/v30 accepts it | Preserve target-local governing context across signals and sentences before accepting an affirmative fact |
| [#521 original markers](https://github.com/git-ksk/reasoning-harness/issues/521) | Lossy normalization removes strikethrough or negation symbols and v17/v30 accepts the resulting affirmative sentence | Inspect original statement/marker scope before normalization; preserve legitimate punctuation and dotted names |

These are separate fixes with shared safety and positive-recall controls. Both must be resolved before a normal 0.7.2 release. They concern **relevance/acquisition qualification**; the audit did not demonstrate downstream wrong answers or external-fact `Known`/`Supported` promotion.

[The saved audit](../evaluation/engine-0.7.1-rereview-20261013/README.md) pins source revisions, inputs, public-API reproducers, outputs, hashes and validation logs. In 56 root-authored synthetic cases, 7 positives passed; 49 negatives included 28 context and 12 marker false-Relevant failures, plus 9 correct withholdings. All 40 failing inputs were Ambiguous at **released 0.7.0 v30**. This is development/audit evidence, not independent acceptance or a population error rate.

## Implementation and validation status

See the [repair report](launch-context-markers-520-521.md) and [validation bundle](../evaluation/launch-520-521-final-validation/README.md). Fresh separately authored v10 first observation against fixed source `2f12d4c61546e60d31026d9b691c9eb369c37957` passed 50/50 (25 positives, 25 negatives; zero false Relevant, positive misses or API errors). The v1–v8 RED first observations and original audit remain unchanged.

Locked workspace: 1490 PASS / 0 FAIL / 8 ignored; strict all-target Clippy, format, 20 standard fixtures, 13-file archive and 22 reconciliation controls pass. Full historical v16/v29 outputs are byte-identical to released 0.7.1 for 597 complete inputs. PR #523 exact-head and merged-main CI passed. The dedicated 0.7.2 release PR must separately pass final-head and merged-source checks. This is synthetic/local/CI evidence, not real-user utility or external factual truth.

The dedicated [release closeout #524](https://github.com/git-ksk/reasoning-harness/issues/524) prepares package 0.7.2 and [release notes](engine-0.7.2-release.md). Implementation remains identical to merged #523. Final release-candidate validation, immutable `engine-v0.7.2` publication and post-publication reconciliation remain required; no tag/release or CLI adoption is claimed by this preparation. Milestone #11 stays OPEN until publication.

## Implementation and acceptance order

1. Add committed direct v17/v30 regressions for both issues and preserve this failing observation. Implement only in the successor path; retain historical v16/v29 and frozen artifacts.
2. Retain factual affirmative launch, publisher attribution, decimal/dotted target and vendor names, bilingual preparation, neutral multi-sentence prose and denial/markers referring only to another target. Keep wrong-target, conditional, question, rumor, fictional and source-binding guards. No blanket multi-sentence or punctuation rejection.
3. Fix the candidate identity and acceptance criteria before a **fresh, separately authored** negative/positive matrix is executed. Record author/provenance, exact source SHA, first observation and scoring; require zero false Relevant and no positive-recall regression on that matrix. The current 56 cases do not count as fresh independent acceptance. No specific external review service is mandatory.
4. On the exact candidate, pass locked workspace tests, strict all-target Clippy, formatting, standard fixtures, original archive verification and applicable contract/replay checks. Record provider/operational failures separately; retain source-reconciliation authority and original Conflict/citation preservation.
5. Use a dedicated source-release PR under the [release policy](release-policy.md). Declare the bounded scope, evidence and limits; complete all required gates before a normal new immutable `engine-v0.7.2` tag. Any candidate publication with unfinished required acceptance needs an explicit prerelease suffix and measurable exit criteria. This roadmap does not authorize or report a release.

## What else remains in the 0.7 line?

The two issues above are the **reproduced regressions addressed by this patch**, not proof that all possible defects have been found. The rereview also passed 22 additional source-reconciliation authority/replay controls, existing 1,468 workspace tests (6 existing ignored), strict Clippy, formatting, 20 fixtures and the 13-file frozen archive check. These results cover their tested boundaries; fresh provider, real-user utility and independently authenticated human acceptance were not performed.

Three existing feature hypotheses remain deferred outside this patch:

| Deferred issue | Evidence required before adoption |
| --- | --- |
| [#488 source lineage / independent corroboration](https://github.com/git-ksk/reasoning-harness/issues/488) | A measured supported use case where origin duplication matters, with a trusted provenance producer; citation/URL count alone does not establish independence |
| [#489 trusted revision / supersession](https://github.com/git-ksk/reasoning-harness/issues/489) | A measured revision/as-of residual and a trusted explicit supersession relationship; retrieval recency alone is not truth |
| [#491 answerability / bounded reacquisition](https://github.com/git-ksk/reasoning-harness/issues/491) | A complete-task failure beyond existing need/qualification floors, plus measured safety and utility/efficiency benefit |

They remain under [milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10), whose original 0.7.0 source-release scope is already published. They are experiments to justify possible future work, not three known bugs or prerequisites for this corrective patch. #465 is a separate legacy MCP CI-fixture reliability investigation; #506 is a separate CLI distribution gate.
