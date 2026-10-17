# #520 — Engine 0.7.3 retraction/non-event successor

[日本語](launch-retraction-520.ja.md) | English

## Latest repair, independent evaluation and remaining work

Candidate `3ccda6b` repairs the20 consumed v23 misses. Same frozen60 development cases: both APIsTP30/FP0/TN30/FN0,precision/recall/F1=1.000; original firstF1 .500 unchanged. Workspace1578PASS/0FAIL/8existing ignored,strict Clippy/fmt,known1200/original14 PASS,historical597 whole-output byte equality and all60v23v29 unchanged. A newly authored independentv24 on this fixed candidate firstFAIL 47/60: v17 TP17/FP0/TN30/FN13, Precision 1.000000/Recall 0.566667/F1 0.723404; v30 TP17/FP0/TN30/FN13, Precision 1.000000/Recall 0.566667/F1 0.723404. Negative violationsv17=0/v30=0;operational/incomplete0. FirstEngine60 oneprocess/scorer1;retry/rescore/relabel0. All old first records remain untouched. Actual-answer F1 unmeasured;merge/release/CLI adoption NO-GO because fresh source gate fails. See[decision and next work](engine073-v24-decision.md).

[Real-answer metrics preparation](engine073-realworld-metrics-preparation.md) defines claim-grounding precision,required-claim recall/F1,citations,correct/avoidable abstention,usefulness,operational missingness and per-provider accounting. PREPARATION_ONLY;notM0/M1 and no answers scored. New scorer,actual callback/dependency closure,numeric live reserve/headroom/currency self-budget and an active-feature ordinary CLI path remain. [Previous01dc same-host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md) records identical grounded answer/acquisition1/public relevance API0 for one fixed Node path;directv17/v30 controls confirm counters. This neither establishes all paths inactive nor the repair's final-answer effect. #541 shared runner is merged;#540 live preparation remains OPEN. #531 freshreal24=0/24/adoption1/6,Groq prohibition,provider/model/Oracle additional0 remain.

Historical c20-era snapshot: [Plan #538](https://github.com/git-ksk/reasoning-harness/issues/538) / [Draft PR #539](https://github.com/git-ksk/reasoning-harness/pull/539). Current repair candidate `c20c5136d59cb3a0bfdac4fb8ca9dade2d93e076`, source SHA256 `6b5aa28035bf23e6ad5eded264b323109786963354f3769a74fffd41ebe54a31`. Same60 consumed inputs give v30 development F1 0→0.4615, positive9/30, negative30/30, with21 positive misses remaining. Independentv22 firstFAIL30/60 at7431333 is immutable; fresh evaluation of this new source is not run. Engine0.7.3 is unpublished/unadopted; package remains0.7.2.

## Behavior and boundary

Published0.7.2 can accept `Silver Lens was launched. We retract this claim.` The successor withholds target launch/event and whole-source denial, including present/past/first-person retraction, withdrawal, contracted denial, non-events and target nonexistence. It preserves a separate target-owned affirmative launch when every correction is explicitly a non-launch property or a fully identified foreign event.

The adverse view checks reason/contrast/timing/reporting clauses independently. Positive literal reporting shells preserve the inner denial's scope. Explicit reporting modifiers and nested reports retain inner target/global denials. Parenthetical and coordinated corrections require a complete scope for every object. Printed literal event words are lettering only with an outside property/target binding; event propositions cannot be retyped by an apposition. Typed Object/Subject/Item definitions, role-specific complete owner supplements and adjacent same-role references cannot cross-bind, overwrite a target owner or lend a physical property's type to a launch reference. Complete appositions, physical/printed attributes and document properties retain their own scopes. Unresolved/mixed/collective references and source-wide withdrawals stay blocking.

Narrow canonical-name and scope-annotation views correct only false coarse context markers, with exact independent bindings, risk-free raw input, an untouched factual target anchor, proven original clause scopes and clean recomputed risk/identity. Actual missing/clipped ownership/context and conflicting target denials remain blocking. A source's explanatory annotation alone cannot establish safety.

Changes affect current v17/v30 and successor-only helpers. Published tags and historical v16/v29 remain unchanged. This bounded grammar does not establish arbitrary-language/coreference, property truth, Known/Supported promotion, final-answer utility or real-world acceptance.

## Preserved first observations

| Identity | Pinned implementation | Original result |
| --- | --- | --- |
| v1 | b6c1598 | Operational FAIL; zero-budget input, no semantic output |
| v2 | b6c1598 | FAIL50/60; unsafe1; positive misses9 |
| v3 | d8c7776 | FAIL39/60; unsafe2; positive misses19 |
| v4 | 4e36fa5 | FAIL46/60; v17 residual4/v30 unsafe0; positive misses10 |
| v5 | 7e107f1 | FAIL43/60; unsafe2; positive misses15 |
| v6 | 06d6c0b | FAIL55/60; unsafe1; positive misses4 |
| v7 | a80391e | FAIL41/60; unsafe0; positive misses19 |
| v8 | dd5b488 | FAIL50/60; unsafe0; positive misses10 |
| v9 | 9229307 | FAIL39/60; unsafe5; positive misses16 |
| v10 | e06c4ec | FAIL43/60; unsafe0; positive misses17 |
| v11 | 26c3baa | NOT_EVALUATED; parent found2 unsafe compound exclusions before freeze/semantics |
| v12 | 20e372d | FAIL40/60; unsafe3; positive misses17 |
| v13 | 2b75f04 | FAIL40/60; unsafe0; positive misses20 |
| v14 | 5edd26f | FAIL49/60; unsafe0; positive misses11 |
| v15 | 9a40b92 | FAIL45/60; unsafe1; positive misses14 |
| v16 | 854ef6d | FAIL50/60; unsafe0; positive misses10 |
| v17 | f3f1362 | NOT_EVALUATED; parent found3 unknown-owner loans before generation/freeze/semantics |
| v18 | 258667b | FAIL45/60; unsafe1; positive misses14 |
| v19 | f3d9ad1 | FAIL49/60; unsafe0; positive misses11 |
| v20 | fd091b1 | FAIL51/60; unsafe3; positive misses6 |
| v21 | d647ca4 | FAIL48/60; unsafe0; positive misses12 |
| v22 | 7431333 | FAIL30/60; unsafe0; positive misses30 |
| v23 | 01dc78b | FAIL40/60; v17/v30 TP10/FP0/TN30/FN20; P1/R.333333/F1.500 |
| v24 | 3ccda6b | FAIL47/60; v17 TP17/FP0/TN30/FN13、Precision 1.000000・Recall 0.566667・F1 0.723404 / v30 TP17/FP0/TN30/FN13、Precision 1.000000・Recall 0.566667・F1 0.723404 |

Original inputs, labels, scorer, raw observations and anchors remain immutable. Consumed sets are seen development coverage. Different-corpus rates are not measured improvement; local/CI success did not override independent failures. V11/v17 have no semantic observation and are not PASS/FAIL corpora. Independentv19 followed the f3d9ad1 freeze and failed49/60: negative30/30 withheld, positive19/30 retained. It is not accepted. One agency-negation pair was corrected before complete freeze/any Engine observation, preserving its initial draft/labels/static output; all polarities and the other58 remained unchanged.

## Current development verification

[Current candidate verification](../evaluation/engine073-launch-development-v22-foundation/README.md):c20c513 workspace1564 PASS/0 FAIL/8 existing ignored; strict Clippy/fmt;20 fixtures;13 original archive files. Old1080 pass;original14/14;597 historical outputs byte-identical;all60 v22 historicalv29 objects unchanged. Consumedv22 development39/60 remainsFAIL; measured F1/recall gain is not fresh independent acceptance. The first repair raised F1 to0.45 but had unsafeFP1 and was rejected; [its outputs/partial checks/limits](../evaluation/engine073-launch-development-v22-foundation-rejected/README.md) are preserved. New-source CI requires separate confirmation; old7431333 CI19 SUCCESS is historical.

[Preceding source-commit CI](../evaluation/engine073-launch-source18-ci-status.json) was19/19 SUCCESS at f3d9ad1 and did not override v19 FAIL. [Preceding fd091b1 CI](../evaluation/engine073-launch-source19-ci-status.json) is19/19 SUCCESS; independentv20 remains FAIL. [The consumed failure triage](../evaluation/engine073-launch-development-v19/README.md) indexes11 positive misses: literal printed content2, complete-property references4, foreign-event ownership/quote/apposition4 and several typed roles1. These are input-composition groups, not verified implementation causes. [Actual-rule development tracing](../evaluation/engine073-launch-trace-development-v1/README.md) found typed-reference5, target-adverse3 and unresolved-adverse3 blockers. The fd091b1 repair binds property anaphors to adjacent complete propositions/exact roles, keeps coordinated roles until an actual predicate and honors explicit foreign owners. Temporary trace hooks were removed before the repair. All11 consumed positives now pass, with the new boundary groups and preserved negatives. [Research and implementation status](engine-073-research-informed-repair.md) distinguishes these source checks from fresh v20 and real24 acceptance.

[#540 safe runner](https://github.com/git-ksk/reasoning-harness/issues/540) was merged with final-head CI5/5,47 offline controls and an independent bounded repair PASS on six new boundaries in [PR #541](https://github.com/git-ksk/reasoning-harness/pull/541). The original three P1 first failures remain immutable. Paired instrumented offline CLI-path evidence uses actual0.6.1/development5edd and a new controlled transcript; the exact ordinary case calls no public relevance API and both arms answer identically. This does not establish current-source quality or reconstruct the unsaved original Mistral responses. Live full-callback/dependency/numeric-budget freeze and real-source acceptance remain open; synthetic300 tokens are not a live budget.

Full [#531 fresh real-source24-task/provider acceptance](engine-07-realworld-evaluation.md) remains0/24; adoption stays1/6 for the preserved development comparison/limits/NO-GO record. Groq stays stopped in its recorded exhausted state. Source release and dedicated CLI integration/distribution follow [the0.7.3 roadmap](engine-0.7.3-roadmap.md).

## Repair and new acceptance after v20

[Firstv20](../evaluation/engine073-launch-independent-v20/README.md) ran/scored once, retry/relabel0; parent verified all478 files and anchor `9b0edb72270648f270c497b525658d9d17aa8b71c1375b018242786ab62b1396`. After [actual-rule diagnosticv2](../evaluation/engine073-launch-trace-development-v2/README.md), sourced647ca4 prevents distinct unresolved events from borrowing prior foreign owners and checks self-contained unknown ownership and role-prefixed target nonexistence. Complete property appositions, exact-role accuracy/owner supplements and comma predicates bind to their actual roles. Balanced literal quotes retain their closing delimiter. The existing successor-only view replaces only quoted bodies of proven typed printed properties, preserving all outside text, actual omissions and raw-risk eligibility guards.

All9 failures pass as consumed replays, with7 new boundary groups and preserved regressions/historical checks. Initial comma/topic-owner and literal-boundary RED remain saved. A parent library binding mistake was detected before fresh independent execution and [preserved as rejected evidence](../evaluation/engine073-launch-development-v20-buildbinding-rejected/README.md). Explicit build settings and actual cargo compiler-artifact metadata establish the corrected rlib, followed by a new freeze/output/binary identity and the final checks above. Originalv20 first results are unchanged.

Fresh independentv21 firstFAIL48/60 (unsafe0, positive misses12), authored after the correct source/core/serde/opaque freeze. HEAD/source/normal target remain fixed through final save, while full validation uses a separate target. [Sourced647ca4 CI](../evaluation/engine073-launch-source20-ci-status.json) is19/19 SUCCESS; this does not establish fresh independent acceptance. Fresh acceptance, real24/CLI/provider/human acceptance remain open; different-corpus scores are not improvement rates.

## Repair and fresh evaluation after v21

[Firstv21](../evaluation/engine073-launch-independent-v21/README.md) remains FAIL. Author-stage bare-role references were made explicit with the before full generation/observation; initial author/AST/diff and unchanged meaning/polarities are retained. No changes after complete freeze. Parent verified all516 files including old478, anchor `9db10bc88ef24b76cd6dd360bd0b23b90b32a19dac74cbbcb7c6f6dd851ee913`. [Actual-rule tracev3](../evaluation/engine073-launch-trace-development-v3/README.md) records typed-reference9, target-adverse2 and unresolved-scope1.

Candidate7431333 binds complete foreign material/document attributes to their property owner, without transferring publisher or quoted-event names to event ownership. Compound properties, outside printed-field owner/type proofs, exact-role supplements/finite references and complete-property anaphors retain independent denials. All12 consumed misses pass, with six new boundary groups and the full/historical checks above. Fresh independentv22 firstFAIL30/60 (positive0/30, negative30/30), with one first Engine/scorer run and no retry/relabel. Acceptance gates are unchanged.


Risk contract clarification: model raw risk is advisory; historicalv1–v16 derive effective risk from actual local scope/omission cues. A non-None raw risk makes the new projections ineligible and retains inheritedv16 effective risk; it does not require copying the raw enum into effective risk. Actual omissions, unknown owners and target/global denials block. A visible relation may coexist with blocking risk. Two new root test expectations were corrected to this existing contract; independent inputs/labels/first results were unchanged.

## V22 record and next diagnostic

[Saved first result](../evaluation/engine073-launch-independent-v22/first-summary.json) remains FAIL. The independent evaluator stopped at its usage limit after first save; [parent closure](../evaluation/engine073-launch-v22-parent-closure/README.md) hash-verified old516 plus29 existingv22 files, total545, without edits. This does not replace the independent final review. The same evaluator subsequently resumed and completed final save; the interruption snapshot is preserved. Candidate7431333 [CI19/19 SUCCESS](../evaluation/engine073-launch-source21-ci-status.json) does not override FAIL.

All30 positives were withheld. Trace those consumed cases in a separate source copy/target, checking complete property type/owner, outside quoted proofs and exact-role references at actual branches. Record causes and repair scope before semantic changes. Additional vocabulary alone is not completion; target/global denials, unknown owners and distinct roles remain controls. Same-frozen-input before/after F1 is development comparison, separate from fresh independent F1.

## Fixed metric definitions

[Metrics](../evaluation/engine073-launch-first-metrics-v1/README.md) define positives as frozen labels requiring requested_relation forv17 / Relevant forv30. Record TP/FP/TN/FN, precision, recall, positive-class F1, specificity, balanced accuracy and predicted-positive coverage per API. Zero denominator means undefined/null. V22 both APIs:TP0/FP0/TN30/FN30;precision undefined, recall0, F1 0, specificity1, balanced accuracy0.5, coverage0. This is not CLI answer rate/final-answer F1. Operational/missing results are never successful negatives; different-corpus F1 is not a measured improvement rate; consumed and fresh independent coverage stay separate. SafetyFP0 remains mandatory regardless of F1. Real24 will separately score claim-grounding and useful final answers; currently unmeasured.


The resumed independent evaluator completed final save: all551 files verified, anchor359a6712b07bc1a58fce4332f77f190578b6fec0bad4f9aab0e022c6b3fef160; firstFAIL unchanged. [Parent final verification](../evaluation/engine073-launch-v22-parent-final-verification.json) and [read-only first metrics](../evaluation/engine073-launch-first-metrics-v1/README.md) are saved. [Separate-copy actual tracing](../evaluation/engine073-launch-trace-development-v4/README.md) found first blockers typed-reference27, target-adverse2, unresolved-adverse1. The next bounded repair handles complete claim wrappers only when the inner proposition independently passes property checks, and equivalent determiner/unowned forms only with a unique same-role foreign owner. Target/global denials, conflicting/unknown owners and historical/raw-risk guards remain required. Measure same-input development F1 and safety counts; fresh acceptance remains separate. No merge/release/adoption.

## Historical c20 limited repair and remaining work

[Candidatec20c513](../evaluation/engine073-launch-development-v22-foundation/README.md) compares the same60 inputs: v30 F1 0→0.4615,precision undefined→1.0 (nine positive predictions),recall0→0.30,API coverage0→0.15,FP0. V17 F1 0.5/recall1/3 differs because one visible relation still has blocking risk; keep APIs separate. This does not rescore any original first result. The first repairFP1 was rejected; retaining competing owner declarations restoresFP0. Workspace1564,old1080 and historical597 checks pass.

Next diagnose the remaining21 on current source, then bound physical/document has/prints/uses/measured values, printed literals with outside type/owner proof, foreign-event quotes/appositions and the canonical Clip false-gap case. Record same-development-setF1/recall/precision/FP after each change. Resolve positive/negative regressions, repin source, then author another fresh independent corpus; do not consumev23 while known misses remain. #531 real24 final-answer/claim-grounding/citation/usefulness metrics are unmeasured and cannot be replaced by synthetic API scores. Merge/release/CLI adoption remain blocked.
