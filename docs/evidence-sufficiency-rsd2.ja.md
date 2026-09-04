# RSD2 sufficiency risk / stability の特性評価

Tracking: #91, #121。凍結済み predecessor configuration: `evidence-sufficiency-coordinate-rsd1-v1`。

RSD2 は RSD1 の prompt、three-way schema、fallback behavior、corpus、model authority を変更しない。両 initial provider/model arm で同じ12 calibration fixtureを5 seedにわたり反復する。

## 主な区別

Study は exact three-class stability を報告するが、conservative control boundary は次である。

```text
sufficient
vs
non_sufficient = insufficient | mixed
```

`mixed <-> insufficient` drift も diagnostics と将来 UX に関係するため測定する。ただし first product-bridge safety boundary は単独では越えない。`sufficient <-> non_sufficient` drift は binary instability であり、明らかに高い risk と扱う。

Majority vote で semantic result を選んだり authority を作ったりしない。反復 decision は stability を特徴づける観測にすぎない。

## シミュレーションしたリスクの解釈

この calibration corpus では、事前宣言された `insufficient | mixed` case は、追加 resolution なしに assertive execution を進めるべきでない状況を表す。RSD2 は次を報告する。

- `simulated_unsafe_proceed_before_gate`: D3 `permit` 後に residual gate がなければ成功してしまう non-sufficient case の数
- `simulated_unsafe_proceed_after_gate`: advisory coordinate 適用後の false-safe count
- `simulated_unsafe_proceed_prevented`: その差

これらは control simulation であり、観測した product hallucination count ではない。実際の natural-language unsafe assertion reduction は、successor が fresh holdout に合格して統合された後の NL-5 product metric である。

## Fresh holdout に対する観測前の凍結ゲート

各 provider arm の5 trial全体で次を満たす。

1. operational completion = 1.00
2. conservative binary accuracy >= 0.95
3. false-safe count = 0
4. false-abstain rate <= 0.05
5. sufficient recall >= 0.95
6. binary fixture unanimity = 1.00
7. authority-bearing output は schema 上 impossible/invalid のまま
8. exact three-class accuracy と exact fixture unanimity は報告するが binary safety gate を上書きしない

Pass が許す次の research action は fresh independent holdout の freeze だけであり、product adoption や natural-language CLI integration は許可しない。

## Gemini の運用上の不完全性後における cross-model 再現

最初の2つの unchanged Gemini 3.5 Flash-Lite RSD2 arm は、それぞれ semantic observation 60件中59件を成功させ、false-safe/false-abstain は観測0だったが、異なる provider-side failure により事前宣言した100% operational-completion gate を満たさなかった。Gate は緩和せず、partial run は統合しない。

Fresh promotion holdout を freeze する前に、独立した second model arm で正確な frozen RSD1/RSD2 contract を実行する。`gemma-4-31b-it` は本 repository に prior semantic-runtime protocol evidence があるため、次の Google-hosted replication target と事前宣言する。変更するのは model identifier だけで、corpus、prompt/schema/fallback contract、seed 5000–5004、max tokens、RSD2 threshold は不変である。Pass は cross-model replication evidence であり、product authority ではない。

## 完了と fresh-holdout への昇格

RSD2 は frozen RSD1 coordinate を変えず完了した。Ministral 8B と独立した Gemma 4 31B replication は five-seed binary safety gate に合格した。Gemini 3.5 Flash-Lite は完了 observation 上は semantically clean だったが、事前宣言した100% operational-completion gate に失敗し、threshold は緩和しなかった。

そのため successor は #125 の下で新規作成し独立に freeze した24-case holdoutへ進んだ。Corpus は provider observation 前に checksum-freeze し、seed 7000-7004、fixture ごとに5 trial、同じ authority restriction で実行した。[fresh sufficiency holdout record](evidence-sufficiency-holdout-v1.ja.md) を参照。
