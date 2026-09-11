# v36補足：Harnessなしのraw model比較

日本語 | [English](v36-raw-baseline-supplement.md)

これは**リリース後の補足比較**です。freeze済みのv0.4.2 v36 release acceptanceを書き換えたり、再採点したりするものではありません。

元のv36では直接測っていなかった、次の疑問を確認します。

> 同じv36の13ケースで、Harnessの受け入れ判定・検証・最終回答の権限管理を外し、modelへrawなtask/contextだけを渡すと何が変わるか？

## 固定した元データ

ケースは次のfreeze済み座標から機械的に作っています。

- tag: `natural-language-e2e-v36-freeze`
- freeze commit: `57bea659d472a103cc48d86ddee7dfe4a41de790`
- v0.4.2 candidate: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- seed: `738214`
- 13ケース: **答えを確定できる想定 8件 / 根拠不足として未確定にすべき想定 5件**

元のv36 acceptanceは変更しません。canonical Harness artifactはActions run IDとSHA-256を`evaluation/v36-raw-baseline/harness-reference-v1.json`へ固定しています。

## Harnessなし側に何を渡す？

raw modelには次を渡します。

1. freeze済みの同じuser task;
2. 設定されていた取得結果またはsession状態変更を、加工せずcontextとして並べたsnapshot;
3. evaluatorが持つ正解値はpromptへ渡さない。ただし、取得データそのものに値が含まれる場合は当然そのまま見える。

調査ケースでは、resolverの観測値に加えて、鮮度・scope・authority・sourceなどのmetadataや`no_result`もraw contextとして渡します。MCPケースでは固定された`Cargo.toml`内容をtool contextとして渡します。sessionケースでは、同じuser入力による状態変更を渡します。

raw側にはtool selectionをさせません。したがってこの比較で見るのは、planner性能ではなく**最終回答の有用性と安全性**です。

## 実測値の意味

| 指標 | 意味 |
| --- | --- |
| **v36 target contract一致率** | v36で`grounded`想定の8件のうち、evaluatorが固定したtarget valueと一致する明確な回答を出せた割合。高いほどこのsynthetic surface上のutilityが高い。一般的な正答率ではない。 |
| **target contract未達** | `grounded`想定8件のうち、v36 target valueまで到達しなかった件数。元v36のfalse abstentionに対応するutility指標。少ないほど良い。 |
| **根拠不足ケースの未確定維持率** | stale、scope違い、authority不足、source不一致などの5件で、断定せず`unknown`を維持できた割合。高いほど安全。 |
| **根拠不足の見逃し** | 未確定にすべき5件で、それでも明確な回答を出した件数。少ないほど安全。 |
| **自信を持った誤答** | 明確に回答したが期待値と一致しなかった件数。少ないほど良い。 |

元のv36で使った`target recall`、`tool selection`、`trigger exposure`、`avoidable stall`はplanner/follow-up機構を見る別指標です。この補足比較では意味を変更せず、再採点もしません。

## 比較対象となるv36 Harness実測

保存済みのcanonical v36 candidate artifact 4モデル（Mistral / Groq / Gemini 3.5 Flash-Lite / Gemma 4 31B）は、この13ケースを最終回答の観点で読み直すと全モデル同じ値です。

- 答えを確定できるケースの正答: **`1/8 = 0.125`**
- 不要な棄権: **`7`**
- 根拠不足ケースの未確定維持: **`5/5 = 1.0`**
- 根拠不足の見逃し: **`0`**
- unsupported exposed assertion: **`0`**
- unsupported structured claim: **`0`**

かなり保守的です。このraw追加測定では、Harnessを外すとv36 target contractへの到達が増えるのか、同時に危険な断定も増えるのかを数字で確認します。なおv36にはplanner/follow-up検証用のsynthetic identity/valueが含まれるため、このcoverageを一般的な「質問への正答率」と読み替えません。

## 読み方

この結果を新しいv36 release結果として扱ってはいけません。**同じfreeze済みtask surfaceを使った、後日追加の別実測**です。

raw modelの正答率が高くても安全とは限りません。逆にHarnessが安全でも、棄権が多ければ有用性には改善余地があります。必ず**有用性と安全性を並べて**読みます。
