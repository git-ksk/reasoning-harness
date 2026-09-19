# Planner reliability v1

Issue #282 の目的は、Issue #283 で executable action のownershipを変更する前に、現在残っているstochastic investigation planner / action selectorのreliabilityをdistributionとして記録することです。

## 座標

- corpus: `planner-reliability-v1`
- evaluator: `reason-planner-reliability-v1`
- scoring: `planner-reliability-scoring-v1`
- 観測対象: Reason CLI 0.5.2 / Harness Engine 0.4.2、commit `bcbd326e147fae21f06a601f988e6ab060ca41bb`
- providerは別々に報告:
  - Mistral / `ministral-8b-latest`
  - Google / `gemini-3.5-flash-lite`

v17 / v18やfreeze済みv0.4.2 release evidenceのretrofit / rescoreではありません。

## freezeするtrial plan

primaryはSurface Aで、事前固定した5 seedを使います。

    86101, 86102, 86103, 86104, 86105

各surfaceはfreshな3 roleです。

1. exact-key read-only candidate 2個 + nonmatching distractor 1個のdirect grounded case;
2. exact-key cache / registry 2個 + distractor 1個のno-result follow-up case;
3. exact-key stale source 2個 + distractor 1個のsafe-stop case。

どのcapabilityにも `selection_priority` を設定しません。各targetにはexact compatible read-only capabilityが2個以上あるため、既存のglobally unique pair selectorやpriority precedenceだけではaction selection全体を自動処理できません。つまり#283が対象にするstochastic action selectorをbaseline上で実際に残します。

Surface Bは情報構造を同じにしつつ、名称・key・value・capability ID・sourceをfreshにしたsurfaceです。seed 86105だけをSurface A trial 5とmatched pairとして観測します。これはsurface sensitivityのdescriptive comparisonであり、primary 5 trialへ平均したりfinal holdoutとして扱いません。

## metricとdenominator

attemptしたtrialは失敗も含めraw reportへ全て残します。3 caseすべてがprocess / typed action / generation failureなしで完了した場合だけoperationally completeです。

semantic / utility distributionはcomplete primary trialだけをdenominatorにします。incomplete trialはsemantic distributionから除外し、failure classとtrial IDを別に報告します。

reportには次を含めます。

- trial completion count / rate;
- complete primary trialをdenominatorにしたstrict planner pass@1;
- 事前固定したk=5 groupのobserved all-k reliability;
- target recall、valid action shape、relevant tool selection、trigger exposure、trigger exposed時だけの#249 conformance、avoidable stall、false abstention、action rejection reason、stop reasonのcase別frequency;
- trial-level metricのmean / min / max / raw values;
- observed provider/model identityとusage telemetry;
- seed 86105のSurface A/B matched delta。

strict planner-successは、complete trialについて correctness-boundary violation 0、全case target recall、全case relevant tool selection、action validation rejection 0、irrelevant acquisition 0、avoidable follow-up stall 0、false abstention 0、grounded caseのgrounding、stale caseのsafe non-grounding、さらにno-result triggerを実際に踏んだ場合だけ#249 conformanceを要求します。

trigger exposureそのものはsuccess条件ではありません。registryを先に選んで直接groundできる場合はutility failureではなく、#249はcache/no-result predecessorを実際に踏んだ場合だけ評価します。

## pass^kの解釈

事前固定した5 trial全部がsuccessしたかをobserved all-fiveとして報告します。観測がIIDだとは仮定しないため、pass@1からp^5を計算する派生値は出しません。

5 trialは、このfreeze済みprovider/model/surface sliceのdescriptive characterizationです。production SLA、母集団reliability保証、model rankingには使いません。

## measurement acceptanceとutilityを分離

live laneのacceptance条件は:

- primary 5 trialが全てoperationally complete;
- matched Surface A/B pairがcomplete;
- correctness-boundary violation 0。

planner pass@1やall-five successが低くてもmeasurement自体はvalidです。低いutilityこそ#283前のbaselineとして保存すべき観測であり、評価失敗へ潰して隠しません。

repeated observationはvoteやmajorityでevidence / authority / truthを作りません。provider reportは別々に保存し、cross-model averageもしません。

## freeze discipline

surface、evaluator、scoring identity、provider/model座標、seed、k、success predicate、matched pairをfirst provider call前にfreezeします。providerごとのfirst live launchがcanonicalです。live boundary後のsemantic / evaluator / surface変更にはsuccessor identityが必要です。pre-live infrastructure failureだけはevaluatorがlive-boundary markerを書く前ならretry可能です。

#283のfinal adoption evaluationでは、この#282 baselineを再利用せず、別のfresh successor / holdout identityを使います。
