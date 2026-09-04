# RSD1 モデルベース evidence-sufficiency 座標

Tracking: #91, #118。前身: RSD0 #116。

RSD0 は D3 の先にある残余ギャップを実測した。12件の新鮮な calibration case はすべて D3 `permit` だが、8/12件は事前宣言で `insufficient | mixed` だった。RSD1 では、correctness authority を受け取ることも生成することもない狭いモデルベース分類器で、この残余の情報状態を識別できるかを検証する。

## 凍結された RSD1 出力契約

モデルが担当するフィールドは正確に1つだけである。

```json
{
  "decision": "sufficient | insufficient | mixed"
}
```

schema は未知のフィールドを拒否する。target echo、evidence IDs、confidence、provenance、verification receipt、hard finding、epistemic state、verdict は含まれない。

意味:

- `sufficient`: 選択された evidence が、Harness が宣言した decision-critical information を、answerability control が進めるのに十分な程度までカバーしている。これは **target が真であることの evidence ではない**。
- `insufficient`: 関連する evidence は存在するが、必要な情報の一部が欠けている。
- `mixed`: 重要な evidence が分散している、衝突している、または部分的にしか揃っておらず、全体として sufficient と判断するのは危険である。

将来 RSD1/RSD2 を昇格する場合も、実行をより保守的にできるのは `insufficient | mixed` だけである。`sufficient` が verification receipt、epistemic promotion、hard finding、final verdict を生成することは決してない。

## モデル入力境界

モデルが受け取るのは次だけである。

- Harness が所有する sufficiency request（`task`、typed target、`required_information`、選択された `evidence_ids`）。
- artifact にすでに存在する、対応する selected evidence。

fixture の事前宣言 label や rationale は受け取らない。target が最終的に真かどうかを判断せず、欠けている facts、requirements、bindings、authority を発明しないよう指示される。

主実行では JSON Schema output を要求する。schema mode を明示的にサポートしない provider は、同じ三値契約の JSON-object fallback を使ってよい。主 structured output が無効だった場合も fallback を1回試せる。fallback output も無効なら、semantic sufficiency result ではなく protocol failure である。

## calibration 専用コーパス

RSD1 が読むのは次だけである。

```text
fixtures/evidence-sufficiency-rsd0/
```

runner は frozen semantic holdout-v4/v5 identity を含む path を拒否する。12件の RSD0 fixture は calibration data であり、RSD1 の prompt/representation 選択に利用してよいが、将来の promotion holdout になることは決してない。

## 指標

operational completeness と semantic calibration は分けて報告する。

- 三クラスの exact accuracy と confusion matrix。
- conservative binary accuracy: `sufficient` 対 `insufficient | mixed`。
- **false-safe rate**: 事前宣言が `insufficient | mixed` なのに `sufficient` と予測した割合。
- **false-abstain rate**: 事前宣言が `sufficient` なのに `insufficient | mixed` と予測した割合。
- label ごとの recall。
- provider attempts/fallbacks、tokens、latency、typed operational/protocol failures。

false-safe metric を主要な safety calibration metric とする。`insufficient` と `mixed` の exact な分離は研究上有用だが、最初の conservative product bridge では、false `sufficient` 判断の回避ほど重要ではない。

## 観測前の RSD1 -> RSD2 progression gate

live provider result を1件も観測する前に、次の粗い calibration threshold で、現在の座標が RSD2 stability work に進む価値があるか判断する。

credential が利用可能な初期 Mistral arm と Google arm の **それぞれ**について:

1. semantic scoring を完了するには、選択した trial が operational completion 100% であること。incomplete run は報告するが、denominator に黙って入れない。
2. conservative binary accuracy >= 0.75。
3. false-safe rate <= 0.25。
4. sufficient-label recall >= 0.50。always-abstain classifier の合格を防ぐ。
5. exact three-class accuracy >= 0.50。
6. model-owned authority field の accepted がゼロであること。authority-bearing output の malformed は protocol failure とする。

これらは **calibration progression** の threshold であり、product-adoption の基準ではない。失敗した場合、RSD1 はこの calibration corpus だけを使って prompt/representation を改訂し、新しい configuration identity の下で再実行できる。合格が正当化するのは RSD2 の risk/coverage/stability characterization と、その後の新規独立 holdout だけである。

## 初期 provider arm

secret-isolated manual workflow は次を使う:

- Mistral `ministral-8b-latest`;
- Google `gemini-3.5-flash-lite`。

provider/model の挙動は protocol/capability と calibration portability に関する evidence であり、correctness authority ではない。

## 初期 live calibration の結果

GitHub Actions run `33530386635` は、2026年9月2日（JST）に frozen RSD1 configuration を実行した。各 provider につき trial/seed は1つ、RSD0 calibration fixture は12件すべてである。設定された credential は両方とも利用可能で、両 arm は operationally complete だった。

| Metric | Mistral `ministral-8b-latest` | Google `gemini-3.5-flash-lite` |
| --- | ---: | ---: |
| successful / attempted | 12 / 12 | 12 / 12 |
| operational completion | 1.000 | 1.000 |
| exact 3-class accuracy | 0.917 | 1.000 |
| conservative binary accuracy | 1.000 | 1.000 |
| false-safe rate | 0.000 | 0.000 |
| false-abstain rate | 0.000 | 0.000 |
| sufficient recall | 1.000 | 1.000 |
| insufficient recall | 1.000 | 1.000 |
| mixed recall | 0.750 | 1.000 |
| fallback runs | 0 | 0 |
| total tokens | 5,649 | 5,921 |
| total provider latency | 5,187 ms | 22,549 ms |

Mistral で exact でなかったのは、事前宣言が `mixed` の1件を `insufficient` と分類したケースだけだった。両 label は conservative direction（`resolve/abstain`、決して promote しない）では同じため、これは三クラス confusion matrix を変えるが、最初の product-bridge safety partition は変えない。

両 arm は、観測前の RSD1 -> RSD2 progression threshold をすべて満たす。これにより RSD2 の repeated seed/model stability work は正当化されるが、product runtime profile は正当化されない。依然として同じ12件の calibration corpus であり、新しい独立 holdout がまだないためである。

RSD2 repeated-seed characterization は [evidence-sufficiency-rsd2.md](evidence-sufficiency-rsd2.ja.md) に定義されている。
