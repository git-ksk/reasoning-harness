# Harness Engine 0.7.3 — repair, measure, then adopt

[日本語](engine-0.7.3-roadmap.ja.md) | English

Unpublished/not accepted. Published CLI0.5.5/Engine0.6.1 and latest independent source0.7.2 remain unchanged. [Plan #538](https://github.com/git-ksk/reasoning-harness/issues/538)/[evaluation #531](https://github.com/git-ksk/reasoning-harness/issues/531). Version promotion/publication require a separate PR.

## Latest repair, independent evaluation and remaining work

Pinned candidate01dc78b resolves the21 known joint misses. Same fixed60 consumed development inputs: bothv17/v30 TP30/FP0/TN30/FN0,precision/recall/F1=1.000 (v30 F1 .461538 onc20 before this repair). Workspace1572 PASS/0FAIL/8 existing ignored,strict Clippy/fmt,known1140/original14,historical597 whole-output byte equality and all60v29 unchanged. These are consumed synthetic API metrics, not final-answer F1 or general precision. Fresh independentv23: firstFAIL40/60 (positive10/30,negative30/30;both APIs precision1.000/recall.333333/F1 .500,unsafe0/positive misses20/operational or incomplete0). Originalv22 firstFAIL30/60 and every old first result remain unchanged. Merge/publication/CLI adoption remainNO-GO. Trace the20 newly consumed misses on current source before repair and track same-input development metrics.

| Order | Evidence required | Current state |
| --- | --- | --- |
| 1 | #520 target/global/unknown-owner withholding;preserve proven separate-property/foreign-event positives;seen regressions and fresh acceptance | 01dc78b known1140 PASS,freshv23 FAIL40/60. Next[trace/repair20 misses](engine073-remaining-v23-repair.md);all old first results unchanged |
| 2 | #540 shared stop/durable reserve/inflight/cancellation,actual callback/dependency/numeric budgets,normal CLI trace | #535/#537/#541 merged;47 offline/6 bounded independent controls PASS. Complete scorer/live closure/monetary budgets/external containment/active feature path remain |
| 3 | Small offline checks and only separately admitted provider development checks | Provider/model/Oracle additional0;oldv2/v3 firstFAIL preserved |
| 4 | CompleteM0 evaluator/host/model/budget/quality freeze→fresh independent real24→full snapshot/goldM1→same-host actual0.6.1/successor comparison | Real24=0/24,M0/M1 incomplete. Severe violations0,no positive regression,complete providers,paired useful-answer gain and quality floors required |
| 5 | Dedicated Engine0.7.3 source release after every required declared-scope acceptance/CI gate | Unpublished;synthetic consumed API F1=1 does not authorize normal release |
| 6 | Dedicated CLI adoption PR;Mac/Windows/Linux,distribution/install/update/rollback | Adoption1/6:development comparison/limits/NO-GO only |

## Real-answer preparation

[Real-answer metrics preparation](engine073-realworld-metrics-preparation.md) defines claim-grounding precision,required-claim recall/F1,citations,correct/avoidable abstention,usefulness,operational missingness and per-provider accounting. PREPARATION_ONLY;notM0/M1 and no answers scored. New scorer,actual callback/dependency closure,numeric live reserve/headroom/currency self-budget and an active-feature ordinary CLI path remain. [Current same-host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md) records identical grounded answer/acquisition1/public relevance API0 for one fixed Node path;directv17/v30 controls confirm counters. This neither establishes all paths inactive nor the repair's final-answer effect. #541 shared runner is merged;#540 live preparation remains OPEN. #531 freshreal24=0/24/adoption1/6,Groq prohibition,provider/model/Oracle additional0 remain.

Main's[#542 quality gates](engine073-quality-gate-integration.md) additionally require future per-provider/track caseF1>=.90/precision>=.95/recall>=.85 simultaneously. These differ from atomic-claim F1 and synthetic API F1;severe0/other gates remain. Complete semantic scorer/trace and both metric units/use must freezeM0 before independent real24 authorship.

## Preserved evidence and next repair

[Current development verification](../evaluation/engine073-launch-development-v22-structure/README.md),[prototypes including rejectedFP1](../evaluation/engine073-launch-development-v22-structure-prototypes/README.md),[freshv23 firstFAIL](../evaluation/engine073-launch-independent-v23/README.md) are preserved. Inspect actual governing paths for20 new positive misses;input classification alone cannot establish internal causes. [Research-informed repair](engine-073-research-informed-repair.md) retains owner/type/quotation/clause proofs and records same-consumed-setP/R/F1/FP per implementation. Preserve known1200,historical597/original14;pin new semantics before another fresh corpus.

Groq prohibition cannot clear by time/TPM pacing/tiny readiness/model substitution. Published tags,historicalv16/v29,all first records and old scorers stay unchanged. Operational missingness never becomes successful negative or zero cost. #488/#489/#491 remain measured-needs-only;#465 infrastructure/#506WinGet separate. No0.8 feature expansion or Oracle.
