# Harness Engine 0.7.3 — repair, measure, then adopt

[日本語](engine-0.7.3-roadmap.ja.md) | English

**Planned successor; unpublished and not accepted.** Updated 2026-10-17 JST. [Plan #538](https://github.com/git-ksk/reasoning-harness/issues/538), [evaluation #531](https://github.com/git-ksk/reasoning-harness/issues/531), [milestone #12](https://github.com/git-ksk/reasoning-harness/milestone/12).

Published product: Reason CLI0.5.5 / Engine0.6.1. Latest independent source: Engine0.7.2. Its present-tense retraction/non-event residual in [#520](https://github.com/git-ksk/reasoning-harness/issues/520) blocks adoption. Target0.7.3 is a new successor, not a relabeling of0.7.2; version promotion and publication need a separate release PR.

## Ordered gates

| Order | Work and completion evidence | Current state |
| --- | --- | --- |
| 1 | Repair #520 with target/claim-scoped retraction, denial, withdrawal and explicit non-event handling. Preserve genuine launch/preparation, neutral prose and unrelated-target/property positives. Replay the original14-input failure as seen development coverage; pin successor SHA. | Candidate7431333:1080 seen cases/98 related/workspace1562 pass;597 historical outputs identical. Originalv21 firstFAIL48/60 retained;12 misses pass as consumed replays. Fresh independentv22 firstFAIL30/60 (positive0/30, negative30/30, unsafe0). [Evidence](launch-retraction-520.md). |
| 2 | [#540 safe evaluation runtime](https://github.com/git-ksk/reasoning-harness/issues/540): one guard for all baseline/candidate/helper calls; durable write-ahead reserve/inflight/latch and restart refusal; conservative whole-callback budgets; graceful cancellation before bounded hard kill. Trace actual normal CLI Engine wiring and isolate Mistral rw03 with fixed responses. | #535/#537/#541 merged. Runner final-head CI5/5,47 offline controls and bounded independent re-review PASS. Instrumented paired CLI path/controlled responses saved. Live callback/dependency/numeric-budget freeze remains open. |
| 3 | Small offline and, when justified, Mistral/Google development checks. Keep normal final-answer utility separate from helper display gains. | Pending; historical v2/v3 FAIL preserved. |
| 4 | Freeze complete evaluator, host/adapters, Engine SHA, model settings, numeric budgets and scoring before an independent author creates24 fresh real-source tasks. Compare actual0.6.1 with the successor under identical host conditions. Require zero severe safety violations, no positive regression and measured useful-answer gain, with complete per-provider results. | Fresh acceptance0/24; draft protocol remains draft. |
| 5 | Dedicated Engine0.7.3 source release PR after all required declared-scope gates and CI pass. Document tested scope and evidence limits under the release policy. | Unpublished; no normal release while required acceptance is incomplete. |
| 6 | Select accepted Engine and adopt through a dedicated CLI PR, then Mac/Windows/Linux, distribution, install, upgrade and rollback acceptance. | Adoption1/6: development comparison/limits/NO-GO record only. |

Groq remains blocked by the carried-forward daily-quota exhaustion state. The next runner must load that state before acquiring credentials and establish actual headroom, refill and material shared-organization usage constraints. Minute pacing, elapsed time alone or a tiny readiness success cannot clear the latch. No silent model/tier/retry/budget increase. Missing/suppressed/partial runs stay incomplete, not semantic unknowns or zero-cost successes.

## Research-informed next repair

[Research and implementation status](engine-073-research-informed-repair.md) records Claude/Codex and CheckList/HANS/FActScore/ALCE. Preserved firstFAILs and actual traces inform candidate7431333 repairs for complete foreign property owners, outside literal proofs and same-role references.1080 seen cases/98 related/workspace1562 pass. Fresh independentv22 firstFAIL30/60 (positive0/30, negative30/30, unsafe0). Generic graph/public API expansion stays conditional. Real24=0/24, adoption1/6 unchanged.

## Evidence and scope

[First development v2/v3 results](engine07-realworld-development-results.md) remain FAIL/incomplete. Mistral/Google normal useful-answer gain was0; helper display gain2 is a separate metric. This source repair does not establish downstream wrong-answer prevention, external truth, real-user benefit or independently authenticated human review. Required real-source/provider gates remain under [#531](engine-07-realworld-evaluation.md).

Do not modify published tags, first observations, frozen v2/v3 evaluators, historical v16/v29 or independent archives. A semantic change after freeze needs a new pinned candidate and fresh acceptance. #488/#489/#491 are conditional on measured supported needs; #465 infrastructure and #506 WinGet acceptance are separate. No0.8.0 feature expansion or Oracle review is part of this plan.

## Fixed metric definitions

[Metrics](../evaluation/engine073-launch-first-metrics-v1/README.md) define positives as frozen labels requiring requested_relation forv17 / Relevant forv30. Record TP/FP/TN/FN, precision, recall, positive-class F1, specificity, balanced accuracy and predicted-positive coverage per API. Zero denominator means undefined/null. V22 both APIs:TP0/FP0/TN30/FN30;precision undefined, recall0, F1 0, specificity1, balanced accuracy0.5, coverage0. This is not CLI answer rate/final-answer F1. Operational/missing results are never successful negatives; different-corpus F1 is not a measured improvement rate; consumed and fresh independent coverage stay separate. SafetyFP0 remains mandatory regardless of F1. Real24 will separately score claim-grounding and useful final answers; currently unmeasured.


The resumed independent evaluator completed final save: all551 files verified, anchor359a6712b07bc1a58fce4332f77f190578b6fec0bad4f9aab0e022c6b3fef160; firstFAIL unchanged. [Parent final verification](../evaluation/engine073-launch-v22-parent-final-verification.json) and [read-only first metrics](../evaluation/engine073-launch-first-metrics-v1/README.md) are saved. [Separate-copy actual tracing](../evaluation/engine073-launch-trace-development-v4/README.md) found first blockers typed-reference27, target-adverse2, unresolved-adverse1. The next bounded repair handles complete claim wrappers only when the inner proposition independently passes property checks, and equivalent determiner/unowned forms only with a unique same-role foreign owner. Target/global denials, conflicting/unknown owners and historical/raw-risk guards remain required. Measure same-input development F1 and safety counts; fresh acceptance remains separate. No merge/release/adoption.
