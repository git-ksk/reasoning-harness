# Nemotron D3 プローブ

これは Issue #73 の bounded cross-family probe である。frozen D3 adoption plan を変更せず、追加の adoption arm でもない。

provider/model は観測前に NVIDIA Hosted NIM `nvidia/nemotron-3.5-lightning-30b-a3b` と固定する。以前の semantic-judge study は protocol-incomplete かつ finding-biased だったため、この model は診断上有用である。

変更していない D2 と holdout-v5 corpus、semantic contract を再利用する。

| stage | corpus | seed | trials | max output tokens | calls |
| --- | --- | ---: | ---: | ---: | ---: |
| D2 probe | `fixtures/semantic-decidability-d2` | 6000 | 1 | 512 | 15 |
| v5 probe | `fixtures/semantic-decidability-holdout-v5` | 7000 | 1 | 512 | 24 |

D2 を先に検査し、GitHub Actions job として D2 が完了した場合だけ v5 を実行する。provider 初期化前の holdout-v5 SHA-256 payload verification は必須である。

この probe が問うのは、既存の R2 materialized decision protocol と deterministic decidability composition が Nemotron 上で operationally usable か、また one-trial の clear-case、typed-insufficiency、unsafe-assertion、ambiguity、stability metric が既記録の Mistral/Gemma pilot と方向性を共有するかだけである。1 trial では cross-seed stability を確立できず、Nemotron を D3 adoption model と認定できない。

結果に応じて fixture、label、threshold、semantic prompt、decidability rule、holdout payload を変更してはならない。operational failure は semantic score ではなく operational evidence のままである。

## 観測結果

GitHub Actions run `33392183569` は merged main から frozen probe を実行した。D2 job は完了したが provider observation の成功は 7/15 (`0.4667` protocol completion) にとどまり、残り8件は `materialization_protocol` failure だった。各 failure は `materialization-r2-v1` が `decision` と任意の `advisory_note` だけを許すにもかかわらず、model-owned `finding` field を返した。そのため単一 D2 trial は operationally incomplete で semantic aggregate score を持たない。

依存する holdout-v5 job は payload hash と credential check を通過したが、40分の job timeout に達した。fixture 18/24 まで進み、8 observation は成功、10件は cancellation 前に同じ materialization-protocol class で失敗した。最終 JSON serialization 前に process が終了したため output artifact は zero-length だが、per-fixture log には operational failure class が残っている。

この結果は protocol compatibility の negative control である。Nemotron は有効な R2 base decision の前に頻繁に失敗するため、complete denominator 上で deterministic decidability composition を評価できない。救済のための provider-specific prompt/schema relaxation は導入しない。
