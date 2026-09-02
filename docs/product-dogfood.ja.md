# Product dogfood: 素のmodel vs Reasoning Harness

NL-5では、凍結済みresearch holdoutとは別のproduct workloadで実利用評価します。

runnerは`reason-product-dogfood`です。同じtask/contextを同じprovider/modelへ渡し、3 armを比較します。

```text
raw arm:                   task/context -> model -> structured answer
harness baseline arm:      task/context -> shared model candidate -> verify -> bounded resolution -> render -> final-claim coverage
harness+D3+sufficiency:    同じshared candidate -> 同じHarness -> D3/sufficiency answer gate -> bounded resolution または abstention
```

`fixtures/product-dogfood-v1`には2 workload classがあります。

- incident analysis（障害分析）
- architecture review（構成レビュー）

それぞれに、最初からground可能なcase、意図的に根拠不足なcase、bounded resolutionで初めてground可能になるcaseがあります。research calibration/holdoutとは混ぜません。

`reason-product-dogfood-v3` reportでは次を測ります。

- unsupported grounded assertionの件数/率
- correct abstention / missed insufficiency
- 本来groundできるcaseでのfalse abstention
- final-claim coverage平均
- bounded resolutionの試行数/成功率
- 3 armそれぞれのtoken/latencyと、D3/sufficiency追加分のoverhead
- successor armのanswer-safety runtime identityとtargetごとのsafety observation

v3 reportではv2のcase-level abstention指標をそのまま残したうえで、fixtureのHarness-owned `input.hypotheses`だけをtask targetとしてtarget-level指標を追加します。supportedなnon-target factを返すsafe partial answerは、task targetを断言していなければtarget abstention成功として別集計します。grounded-target coverage / missed target insufficiency / false target abstention / safe-partial retentionを分離し、最初のv2 pilot結果は書き換えません。

`user_comprehension`は`not_automated_manual_review_required`と明示します。model出力だけから「人間に分かりやすかった率」を捏造しません。

provider keyがある環境では次で実行できます。

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --output /tmp/reason-product-dogfood.json
```

successor runtimeはHarness-owned requirement policyも`generic-answer-sufficiency-requirements-v1`としてidentityに固定します。これはfrozen holdout corpusそのものではなくproduct wiringです。holdoutは「与えられたrequired_informationに対するclassifier」を検証し、NL-5ではこの固定product policyが過剰abstentionを起こさず役立つかを別に測ります。

GitHub Actionsのmanual `product-dogfood` workflowはrepository secretを使い、JSON reportをartifactとして保存します。baseline / D3+sufficiency両Harness armで外部へ露出するunsupported grounded claimが0であることと、runtime identityをgateします。`sufficient`はauthorityを増やさずno-op、`insufficient` / `mixed`だけがverification・bounded resolution・abstention方向へ作用します。live結果はそのmodel/workload sliceの実測であり、普遍的な正しさの主張ではありません。
