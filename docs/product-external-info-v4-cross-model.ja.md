# Product external-information v4 モデル横断replication

Issue #208では、freeze済み`product-external-info-v4`のmatched-context 4-arm測定を追加model familyで再現する。これは観測後のreplicationだけを目的とし、v4のcorpus、target proposition、scoring、evaluator semantics、MCP boundary、admission policy、finalization logicはsemantic head `e324ccbff6e818d205a734f06ccc8cac4b587588`から1 byteも変更しない。

## 対象モデル

- Mistral `mistral-small-latest`
- Google-hosted `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`

run `34000216929`のMinistral 8B結果はv4のcanonical初回観測のまま保持し、比較表の基準行としてのみ利用する。

## 固定条件

- corpus: `product-external-info-v4`
- evaluator: `reason-product-external-info-v4`
- comparison: `matched-target-context-four-arm-v4`
- seed: `28000`
- max tokens: `1024`
- 21ケース: semantic 18 + typed operational 3
- 各model run内ではarm 3とarm 4がcaseごとに同じdecoded acquisition snapshotを共有する
- raw/Harnessへ同じtask、exact target hypothesis、evidence requirement、authority policyを渡す

v4にはfreeze後のsnapshot injection interfaceを追加しないため、live external acquisition自体はmodelごとに再実行する。したがってモデル横断では同じfrozen endpointとsemantic contractを比較するが、external response byteまで完全一致することを保証するのは各run内のarm 3/4間だけである。matrix開始直前にmodel-free acquisition preflightを必須とする。

## 報告指標

各modelについてarm 3（`raw_model_with_external`）とarm 4（`harness_with_mcp_external`）のexpected-grounded target coverage、expected-unknown preservation、false target abstention、unsupported grounded claims、missed target insufficiency、Harness unsafe-admission/authority-promotion counter、typed rejection telemetry、model token、model latency、accounted end-to-end latency、operational failureを報告する。

provider/protocol failureはsemantic scoreと分離する。観測結果を理由にv4を変更して修復してはならない。
