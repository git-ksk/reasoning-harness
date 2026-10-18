# Harness Engine 0.7.3 — repair, measure, then adopt

[日本語](engine-0.7.3-roadmap.ja.md) | English

Unpublished/not accepted. Published CLI0.5.5/Engine0.6.1 and latest independent source0.7.2 remain unchanged. [Plan #538](https://github.com/git-ksk/reasoning-harness/issues/538)/[evaluation #531](https://github.com/git-ksk/reasoning-harness/issues/531). Version promotion/publication require a separate PR.

## Latest repair, independent evaluation and remaining work

**#520 policy revision (2026-10-17):** For **future newly frozen** independent synthetic source cohorts, unconditional 60/60 exact classification is no longer required. Instead each public API axis must satisfy **F1>=0.90, precision>=0.95, recall>=0.85**, **zero false Relevant/RequestedRelation on safety negatives**, 100% known regressions (including the original 14), historic API compatibility and complete operations. Keep at least60 independent examples with >=30 positives/30 negatives. Do not reinterpret or rescore v24/other immutable first FAIL archives. See [new source policy](../evaluation/engine073-source-gate-v1/README.md). #531 real24 per-provider/track final-answer safety/utility and #540 runner gates remain independent and unchanged; no Engine release or CLI adoption is implied.



Fresh independent v26 at candidate `9082365` is **FAIL / NO_GO**: each API TP22 / FP0 / TN30 / FN8, precision 1.000, recall 0.733333, F1 **0.846154**. Safety-negative admissions, operational failures and incomplete observations are zero, but fixed F1 0.90 and recall 0.85 floors fail. This is separate from consumed v25 development F1=1.000; different independent cohorts do not measure an improvement rate. First-only Engine 1 process / 60 inputs, scorer 1 / helper 1, retry/rescore/relabel0. Parent review reproduces59 pure fixtures; M0/M1 precede observation, all13 provenance checks pass, and parent verifies99 original new files plus730 old references /829 total corpus references. Original v25 FAIL and all older firsts stay unchanged. Actual consumed diagnostics show7 typed_reference_not_excluded blocks and057 unresolved_adverse_scope. Known controls are now1392/1400; original14, historical597 and workspace1594PASS are separate pinned908 checks. Next: repair shared property/ownership/local-reference handling, pass all1400 known controls, then freeze a new candidate/evaluator before fresh authorship. v27 is not authored. Real24=0/24, actual-answer F1 unmeasured, Groq stopped, source merge/Engine publication/CLI adoption NO_GO. See [decision](engine073-v26-decision.md),[original first](../evaluation/engine073-launch-independent-v26/README-results.md),[actual routes](../evaluation/engine073-launch-trace-development-v10/README.md).

| Order | Evidence required | Current state |
| --- | --- | --- |
| 1 | #520 source safety, known regressions and fresh source gate | 9082365 freshv26 first52/60 diagnostic,bothF1 .846154/FP0,FAIL/NO_GO;known1392/1400;repair8 misses and all regressions before new freeze/authorship;v27 not authored; originalv25 firstFAIL unchanged; [decision](engine073-v25-decision.md); all first records preserved |
| 2 | #540 shared stop/durable reserve/inflight/cancellation,actual callback/dependency/numeric budgets,normal CLI trace | #535/#537/#541 merged;47 offline/6 bounded independent controls PASS. Complete scorer/live closure/monetary budgets/external containment/active feature path remain |
| 3 | Small offline checks and only separately admitted provider development checks | Provider/model/Oracle additional0;oldv2/v3 firstFAIL preserved |
| 4 | CompleteM0 evaluator/host/model/budget/quality freeze→fresh independent real24→full snapshot/goldM1→same-host actual0.6.1/successor comparison | Real24=0/24,M0/M1 incomplete. Severe violations0,no positive regression,complete providers,paired useful-answer gain and quality floors required |
| 5 | Dedicated Engine0.7.3 source release after every required declared-scope acceptance/CI gate | Unpublished;consumed API improvement does not authorize normal release |
| 6 | Dedicated CLI adoption PR;Mac/Windows/Linux,distribution/install/update/rollback | Adoption1/6:development comparison/limits/NO-GO only |

## Real-answer preparation

[Real-answer metrics preparation](engine073-realworld-metrics-preparation.md) defines claim-grounding precision,required-claim recall/F1,citations,correct/avoidable abstention,usefulness,operational missingness and per-provider accounting. PREPARATION_ONLY;notM0/M1 and no answers scored. New scorer,actual callback/dependency closure,numeric live reserve/headroom/currency self-budget and an active-feature ordinary CLI path remain. [Previous01dc same-host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md) records identical grounded answer/acquisition1/public relevance API0 for one fixed Node path;directv17/v30 controls confirm counters. This neither establishes all paths inactive nor the repair's final-answer effect. #541 shared runner is merged;#540 live preparation remains OPEN. #531 freshreal24=0/24/adoption1/6,Groq prohibition,provider/model/Oracle additional0 remain.

Main's[#542 quality gates](engine073-quality-gate-integration.md) additionally require future per-provider/track caseF1>=.90/precision>=.95/recall>=.85 simultaneously. These differ from atomic-claim F1 and synthetic API F1;severe0/other gates remain. Complete semantic scorer/trace and both metric units/use must freezeM0 before independent real24 authorship.

## Preserved evidence and next repair

[Pinned development](../evaluation/engine073-launch-development-v24-structure/README.md),[prototypes and rejected controls](../evaluation/engine073-launch-development-v24-structure-prototypes/README.md),[actual trace](../evaluation/engine073-launch-trace-development-v7/README.md),and[originalv24 first record](../evaluation/engine073-launch-independent-v24/README.md) are preserved. [Fresh v25 first record](../evaluation/engine073-launch-independent-v25/README-results.md) is now closed FAIL. Follow[decision/next work](engine073-v25-decision.md). Cross-corpusF1 differences are not an improvement rate;measure each repair on the same consumed cases.

Groq prohibition cannot clear by time/TPM pacing/tiny readiness/model substitution. Published tags,historicalv16/v29,all first records and old scorers stay unchanged. Operational missingness never becomes successful negative or zero cost. #488/#489/#491 remain measured-needs-only;#465 infrastructure/#506WinGet separate. No0.8 feature expansion or Oracle.
