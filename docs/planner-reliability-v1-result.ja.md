# Planner reliability v1 — 結果

Issue #282では、Issue #283でexecutable action ownershipを変更する前に、release済みReason CLI 0.5.2 / Harness Engine 0.4.2に残るstochastic investigation planner / action selectionのreliabilityを測定しました。

## Freeze座標

- freeze tag: `planner-reliability-v1-freeze`
- freeze commit: `ce0842dd6e6f27a2004659cc9f2a47483f8cd4b5`
- 観測対象product runtime: `bcbd326e147fae21f06a601f988e6ab060ca41bb`
- corpus: `planner-reliability-v1`
- evaluator: `reason-planner-reliability-v1`
- scoring: `planner-reliability-scoring-v1`
- canonical Actions run: `35423433892`
- Mistral job: `105845143753`
- Google job: `105845143801`
- primary seed: `86101`, `86102`, `86103`, `86104`, `86105`
- observed all-k group: `k=5`
- information-equivalent Surface A/B matched seed: `86105`

surface、evaluator、scoring identity、provider/model座標、seed、success predicate、k groupはfirst provider call前にfreezeしました。観測対象product runtime自体は記録済みbase commitから変更していません。

## 結果概要

| provider/model | complete primary trial | pass@1 | observed all-5 | correctness-boundary violation | matched Surface A/B |
| --- | ---: | ---: | --- | ---: | --- |
| Mistral `ministral-8b-latest` | 5/5 | 1.00 | true | 0 | complete、両方planner success |
| Google `gemini-3.5-flash-lite` | 5/5 | 0.80 | false | 0 | complete、両方planner success |

両provider jobとも、事前宣言した**measurement acceptance**をPASSしました。primary 5 trialがすべてoperationally complete、matched Surface A/B pairもcomplete、correctness-boundary violationは0です。planner utilityは意図的にmeasurement acceptance gateへ含めていません。

観測をIIDとは仮定しないため、pass@1から導出した`p^5`は報告しません。事前固定したobserved all-five groupをそのまま保存します。

## Mistral観測

Mistralはprimary 5 trialすべてでstrict planner successでした。

5 complete trial全体で:

- target recall: 毎trial `3/3`;
- relevant tool selection: 毎trial `3/3`;
- valid action shape: 毎trial `3/3`;
- action validation rejection: 全trial `0`;
- avoidable follow-up stall: `0`;
- false abstention: `0`;
- no-result trigger exposure: follow-up case `5/5`;
- #249 exact-target follow-up conformance: `5/5`;
- correctness-boundary violation: `0`。

matched Surface A/B seed-86105 pairもcompleteで、事前宣言したplanner metric deltaはすべて0でした。

## Google観測

Googleもprimary 5 trialすべてoperationally completeで、correctness-boundary violationは0でしたが、1 trialだけstrict planner-success predicateを満たしませんでした。

seed `86101`, `86102`, `86103`, `86105`はPASS。seed `86104`のstale-unknown caseでaction validation rejectionが1仰ありました。

- plannerはまず正しいstale sourceを2つ選択;
- 両stale observationは設計どおり通常のadmission boundaryでreject;
- その後stochastic action selectorがtarget `check_window_one`に対して`myrador-region-distractor`を提案;
- Harnessは`unsupported_target_key`としてreject;
- caseは安全にungroundedのまま`no_progress`で停止。

したがって:

- complete trial: `5/5`;
- planner success: `4/5 = 0.80`;
- observed all-five success: `false`;
- trialごとのaction rejection count: `[0, 0, 0, 1, 0]`;
- invalid-shape rejection: `0`;
- avoidable follow-up stall: `0`;
- false abstention: `0`;
- no-result trigger exposure: `5/5`;
- trigger exposed時の#249 conformance: `5/5`;
- correctness-boundary violation: `0`。

これは狙っていたdiagnostic separationです。stochastic executable-action selectionにはutility上の残余が観測されましたが、Harness validationが安全性/correctness boundaryを維持しました。

matched Surface A/B seed-86105 pairは両方completeかつplanner-successで、事前宣言metric deltaはすべて0でした。

## 解釈

これは**baseline characterization**であり、model ranking、SLA、母集団reliability保証ではありません。

product上の重要な観測は限定的です。

1. release済みHarnessは、このfreeze済みplanner surfaceを両providerで安全かつoperationally completeに実行できた;
2. useful evidence acquisition後でもstochastic action selectorはinadmissibleなtarget/capability pairingを生成し得る;
3. Harnessはそれをauthority/evidenceへ昇格せず正しくrejectした;
4. Issue #283は、deterministic Harness-owned action materializationの前後比較に使えるpre-change baselineを得た。

#282 surfaceはこれでconsumed diagnostic evidenceです。#283でこの同じsurfaceへtuningした後、そのままadoption claimに使ってはいけません。adoption decisionには別fresh successor / holdout identityが必要です。

## 保存したmachine report

- [Mistral raw machine report](observations/planner-reliability-v1-mistral-run-35423433892-2026-09-19.json), SHA-256 `453012a07c8b2f8f807fa04b4f92d8e4012f9ce6f8f0681a1c81872916f82a3e`
- [Google raw machine report](observations/planner-reliability-v1-google-run-35423433892-2026-09-19.json), SHA-256 `b5b6ef82f8c2dbf019da95adb94ec13f3bece2e1282f42199fa3d688a6ffb533`

freeze済みv17/v18とv0.4.2 release rulerは変更・rescore・repairしていません。
