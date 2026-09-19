# Action materialization v2 — 採用評価の混合結果

Issue #283 の2回目の fresh adoption holdout を `action-materialization-v2` として freeze して実行した。評価surface自体は有効で、両providerとも control/candidate 各5 trial を完走し、correctness-boundary violation は0、想定した control/candidate のアーキテクチャ差も観測できた。一方、事前固定した adoption gate は downstream finalization を planner-success predicate に含めていたため、provider間で結果が分かれた。

## Frozen coordinate

- freeze tag: `action-materialization-v2-freeze`
- freeze commit: `5a99912ffd6cd90afc71664cabadbe5f8727e2e2`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v2`
- evaluator: `reason-action-materialization-v2`
- scoring: `action-materialization-scoring-v2`
- canonical Actions run: `35429022032`
- Mistral job: `105860070988`
- Google job: `105860071151`
- primary seeds: `98221`–`98225`
- observed all-k group: `k=5`

surface、evaluator、scoring identity、provider/model coordinate、seed、success predicate、coordinate order、k group は初回provider call前にfreeze済み。semantic rerunは行っていない。

## アーキテクチャpathの観測

#283 に直接関係するアーキテクチャsignalは両providerで一貫した。

| provider/model | control complete | candidate complete | correctness violations, control | correctness violations, candidate | control legacy executable-action planner calls | candidate legacy executable-action planner calls | candidate intent calls | candidate Harness materializations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 5/5 | 5/5 | 0 | 0 | 20 | 0 | 30 | 30 |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | 0 | 0 | 16 | 0 | 20 | 15 |

両providerとも:

- control/candidateの全10 same-key-sibling caseで2つの別target identityを保持;
- controlは全10 caseでlegacy executable-action planner pathを使用;
- candidateは全10 caseで `reason-investigation-intent-v1` + `target-intent-materialization-v1` を使用;
- candidateのlegacy executable-action planner callは0;
- candidateのintent rejection / action rejectionは0;
- 関連するread-only capabilityを選択;
- correctness-boundary violationは0。

したがって、評価対象pathでは、candidateが安全境界を弱めずに mechanically safe な executable capability materialization を Harness control flowへ移したことを直接観測できた。

## Frozen gate の結果

v2でfreezeしたgateはproviderごとに異なる結果になった。

| provider/model | control planner success | candidate planner success | frozen acceptance |
| --- | ---: | ---: | --- |
| Mistral `ministral-8b-latest` | 0/5 | 0/5 | fail |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | pass |

この差は action-materialization path のcandidate regressionではない。

### Mistral

grounded Velmora caseはcontrol/candidateとも全trialで `requires_verification`、blocked unverified propositions=2 になった。same-key sibling targetが同一Propositionへ解決した場合、既存finalization bridgeは複数target candidateを曖昧として意図的にfail closedする。このためgrounded caseはv2 planner-success predicate上でfalse abstentionになった。

stale caseも全10 caseで `requires_verification` だったが、targetを誤ってgroundedにしていないためunknown-case側のplanner predicateは満たしている。

### Google

grounded Velmora caseはcontrol/candidateとも全trialで `grounded_answer` となり、freeze済みplanner predicateを満たした。stale Tarsenne caseは `unresolved` または `requires_verification` だが、targetは正しくungroundedのままだった。

各provider内ではcontrolとcandidateが同じ傾向を示している。したがってprovider差は、モデルが生成したtarget structureを downstream target/finalization が処理した結果であり、#283 candidate product commitが導入した差ではない。

## v2を最終採用判定にしない理由

v2 planner-success predicateは、次の2つを混ぜていた。

1. planner/action pathがtargetをrecallし、same-key siblingを保持し、関連capabilityを選び、legacy-vs-intent/materializationの想定pathを通ったか。
2. downstream #248 finalization bridgeが、その生成target structureからgrounded final answerを作れたか。

Issue #283の主題は mechanically safe action materialization の所有境界である。Issue #248は意図的に別の target-to-finalization authority bridge として残している。downstream final-answer grounding を #283 adoption gate の必須条件にしたため、provider依存のfinalization挙動が、一貫して観測できていたarchitecture-path signalを隠した。

freeze済みv2結果は測定どおり保存する。scoringを書き換えず、Mistral failureを後からPASS扱いしない。

## Cost / latency 観測

各provider 5 paired trialだけの記述的観測であり、一般化しない。

Mistral candidate vs control:

- provider calls: 60 → 70 (+16.67%)
- provider attempts: 77 → 84 (+9.09%)
- tokens: 61,563 → 58,033 (-5.73%)
- provider latency: 163,270 ms → 130,417 ms (-20.12%)
- wall time: 164,697 ms → 131,843 ms (-19.95%)

Google candidate vs control:

- provider calls: 51 → 55 (+7.84%)
- provider attempts: 57 → 61 (+7.02%)
- tokens: 34,959 → 33,839 (-3.20%)
- provider latency: 200,703 ms → 198,311 ms (-1.19%)
- wall time: 201,619 ms → 198,914 ms (-1.34%)

#283アーキテクチャはmodel call総数の削減を保証しない。直接測定しているbenefitは、対象pathからstochasticなexecutable-ID selectionを除去すること。total call/token/latencyへの影響はworkload/provider依存として扱う。

## Disposition

`action-materialization-v2` は有効なmixed observationとして保存し、rescoreもrerunもしない。

fresh successor `action-materialization-v3` では:

- control/candidate product commitは同じまま;
- case identity / fact key / fixture content / seedをfreshにする;
- correctnessとoperational completenessを独立したhard axisとして維持;
- #283 adoption gateは target recall、same-key sibling exposure、relevant capability selection、action/intent rejection、control legacy path exposure、candidate legacy path elimination、candidate intent/materialization conformanceだけで判定;
- downstream finalization status、grounded answer、false abstention、blocked propositionはadoption gateに入れず別metricとして記録;
- #248 finalization semanticsは変更しない;
- live observation前にfreezeし、providerごとにfirst canonical runを1回だけ行う。

## 保存したmachine report

- [Mistral raw machine report](observations/action-materialization-v2-mistral-run-35429022032-2026-09-19.json), SHA-256 `40f83ee1b75801eddc410c8ad8fd7c64c2f612bf7939deaa5bb4a6a7b687b982`
- [Google raw machine report](observations/action-materialization-v2-google-run-35429022032-2026-09-19.json), SHA-256 `f147f5c5ab53e8eb5cc035abf5ff3d17ae96eeb9d36dd0d990968f66b48c6b6d`
