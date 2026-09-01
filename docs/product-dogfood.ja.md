# Product dogfood: 素のmodel vs Reasoning Harness

NL-5では、凍結済みresearch holdoutとは別のproduct workloadで実利用評価します。

runnerは`reason-product-dogfood`です。同じtask/contextを同じprovider/modelへ渡し、2 armを比較します。

```text
raw arm:      task/context -> model -> structured answer
harness arm:  task/context -> model candidate -> verify -> bounded resolution -> render -> final-claim coverage
```

`fixtures/product-dogfood-v1`には2 workload classがあります。

- incident analysis（障害分析）
- architecture review（構成レビュー）

それぞれに、最初からground可能なcase、意図的に根拠不足なcase、bounded resolutionで初めてground可能になるcaseがあります。research calibration/holdoutとは混ぜません。

`reason-product-dogfood-v1` reportでは次を測ります。

- unsupported grounded assertionの件数/率
- correct abstention / missed insufficiency
- 本来groundできるcaseでのfalse abstention
- final-claim coverage平均
- bounded resolutionの試行数/成功率
- rawとHarnessのtoken/latency overhead

`user_comprehension`は`not_automated_manual_review_required`と明示します。model出力だけから「人間に分かりやすかった率」を捏造しません。

provider keyがある環境では次で実行できます。

```bash
cargo run -p reasoning-harness-cli --bin reason-product-dogfood -- \
  --provider mistral \
  --model ministral-8b-latest \
  --fixtures fixtures/product-dogfood-v1 \
  --output /tmp/reason-product-dogfood.json
```

GitHub Actionsのmanual `product-dogfood` workflowはrepository secretを使い、JSON reportをartifactとして保存します。Harness armで外部へ露出するunsupported grounded claimが0であることをgateします。live結果はそのmodel/workload sliceの実測であり、普遍的な正しさの主張ではありません。
