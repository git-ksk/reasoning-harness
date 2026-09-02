# Product dogfood: 素のmodel vs Reasoning Harness

NL-5では、凍結済みresearch holdoutとは別のproduct workloadで実利用評価します。

runnerは`reason-product-dogfood`です。同じtask/contextを同じprovider/modelへ渡し、3 armを比較します。

```text
raw arm:                   task/context -> model -> structured answer
harness baseline arm:      task/context -> shared candidate -> deterministic pre-render Harness state -> shared initial render -> baseline finalization
harness+D3+sufficiency:    同じcandidate/state/render -> D3/sufficiency gate -> optional bounded resolution -> state変更時だけsuccessor側rerender
```

v4の比較契約は`shared-candidate-initial-render-v1`です。2つのHarness armはuntrusted candidate・deterministicなpre-render state・最初のfinal-answer renderを完全共有します。successor介入でstateが変わった場合だけC側追加rerenderを許し、renderer sampling差をD3/sufficiencyの効果として誤認しません。

`fixtures/product-dogfood-v1`には2 workload classがあります。

- incident analysis（障害分析）
- architecture review（構成レビュー）

それぞれに、最初からground可能なcase、意図的に根拠不足なcase、bounded resolutionで初めてground可能になるcaseがあります。research calibration/holdoutとは混ぜません。

`reason-product-dogfood-v5` reportでは次を測ります。

- unsupported grounded assertionの件数/率
- correct abstention / missed insufficiency
- 本来groundできるcaseでのfalse abstention
- final-claim coverage平均
- bounded resolutionの試行数/成功率
- 3 armそれぞれのtoken/latencyと、D3/sufficiency追加分のoverhead
- successor armのanswer-safety runtime identityとtargetごとのsafety observation

v5 reportではv2のcase-level abstention指標とv3のtarget-level指標をそのまま維持し、v4のshared-render比較契約も維持したまま、NL-5で必要なqualified/unknown出力のmanual comprehension reviewを行えるよう各armの実際のユーザー表示`exposed_text`を保存します。target-level指標はfixtureのHarness-owned `input.hypotheses`だけをtask targetとして使います。supportedなnon-target factを返すsafe partial answerは、task targetを断言していなければtarget abstention成功として別集計します。grounded-target coverage / missed target insufficiency / false target abstention / safe-partial retentionを分離し、最初のv2 pilot結果は書き換えません。

`user_comprehension`は`not_automated_manual_review_required`と明示します。model出力だけから「人間に分かりやすかった率」を捏造しません。

provider keyがある環境では次で実行できます。

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --output /tmp/reason-product-dogfood.json
```

現在のsuccessor runtimeは`d3-sufficiency-answer-gate-v2`で、requirement policyは`claim-local-answer-sufficiency-requirements-v1`です。product sufficiency判定を個別typed propositionへ限定し、Supported/Known claimへ既にbindingされたevidenceを優先します。broader taskはcontextであり、安全なpartial fact一つ一つにtask全体の完答を要求しません。旧`d3-sufficiency-answer-gate-v1` / `generic-answer-sufficiency-requirements-v1`はrollbackとして実行可能です。どちらもfrozen holdout corpusそのものではなく、NL-5でproduct wiringを別評価します。

## 最終NL-5 acceptance結果

最終v5 acceptanceは`shared-candidate-initial-render-v1`比較契約とclaim-local `d3-sufficiency-answer-gate-v2` successorで実施しました。

- Ministral 8B: Actions run `33576517724`
- Gemma 4 31B: Actions run `33576520136`

両model sliceとも、2つのHarness armはunsupported grounded claim 0、missed task-target insufficiency 0でした。Gemmaではbaseline/successorともexpected-grounded caseのmean target coverage 1.0、false target abstention 0、resolution 2/2成功で、unknown caseでもsupported non-target fact 2件を含むsafe qualified partial answerを1件保持しました。このrunでsuccessorのbaseline比overheadはtoken約+45.3%、latency約+12.2%です。Ministralはbaseline/successorのtask-target挙動が完全同値でしたが、両方ともfalse target abstention 0.75、resolution 0/2でした。これはsuccessor gateによる回帰ではなくmodel固有のproduct utility制約として#139で追跡します。

v5の`exposed_text`でmanual comprehension reviewも実施しました。Gemmaのroot-cause qualified answerは「database原因は未確定」と明示しつつ、HTTP 503とDB connection error 7件というverified observationだけを残し、correlationをcausationへ昇格していません。baseline/successor文面も完全一致です。Ministralのraw unknown文は不足根拠を分かりやすく説明しますが、Harness armはfinal textをwithholdするcaseが多く、安全ではあるものの説明性が弱いです。これらは対象model/workload sliceのproduct evidenceであり、普遍的なmodel品質主張ではありません。

GitHub Actionsのmanual `product-dogfood` workflowはrepository secretを使い、JSON reportをartifactとして保存します。baseline / D3+sufficiency両Harness armで外部へ露出するunsupported grounded claimが0であることと、runtime identityをgateします。`sufficient`はauthorityを増やさずno-op、`insufficient` / `mixed`だけがverification・bounded resolution・abstention方向へ作用します。live結果はそのmodel/workload sliceの実測であり、普遍的な正しさの主張ではありません。
