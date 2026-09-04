# Gemma 4 セマンティック複製

`gemma4-31b-replication-v1` は、既存の semantic-judge research に対するファミリー横断の replication arm である。いずれの calibration fixture や holdout fixture も置換、再ラベル、チューニングせず、事前宣言された D3 adoption providers に Gemma 4 を遡及的に追加することもない。

## 固定モデル

- provider: `google`
- model: `gemma-4-31b-it`
- rationale: リポジトリで以前に実施した5試行の live benchmark では31Bモデルが5/5試行を完了した一方、26Bモデルには provider 側の copyright/recitation block があり、引き続き experimental である。

## 固定ステージ

| stage | corpus | seed range | trials | max tokens | provider calls |
| --- | --- | --- | ---: | ---: | ---: |
| R2 materialization | `fixtures/semantic-judges` | 2000-2004 | 5 | 512 | 180 |
| D2 decidability | `fixtures/semantic-decidability-d2` | 6000-6004 | 5 | 512 | 75 |
| v5 pilot | `fixtures/semantic-decidability-holdout-v5` | 7000-7004 | 5 | 512 | 120 |

固定された provider call の合計: 375。

R2 は既存の calibration-only materialization study の診断的 replication である。D2 は固定された deterministic decidability contract を再利用する。v5 は不変の holdout-v5 payload とその SHA-256 manifest を再利用する。provider の観測後、いかなる stage も labels、thresholds、seeds、model identity、corpus membership、semantic contracts を変更してはならない。

## 解釈

この replication では、Ministral 8B で観測された同じ定性的パターンが別の model family に移るかを問う。

1. 既存の R2 calibration surface で、model-backed semantic judge を測定する。
2. 固定された decidability gate が D2 で unsafe assertive decision をいくつ除去するかを測定する。
3. 変更していない v5 independent pilot を実行し、clear-case coverage/precision/recall、typed-insufficiency abstention、composed unsafe assertions、seed stability、ambiguous abstention diagnostics を比較する。

Gemma 4 の v5 pass は、model family をまたぐ一般化を支持する証拠である。ただし、それだけで遡及的な D3 adoption result になるわけではない。元の D3 provider set はこの replication arm が存在する前に固定されていたためである。

rate limit、daily quota exhaustion、provider unavailability、transport timeout などの運用上の失敗は、semantic failure とは別に扱う。study CLI は各ケースの完了時に bounded な `failure_class` を記録するため、workflow-level timeout により最終 JSON の組み立てが妨げられても、運用上の制限を診断できる。

## 観測結果

GitHub Actions run `33384957101` は、merged main commit
`14871a8375881f07a3813a4d584209859c30ac93` から、model、seeds、corpora、token
budget、semantic contracts を変更せずに固定 replication を実行した。

### R2 マテリアライゼーション

R2 stage は、18ケースの calibration corpus と5つの seed に対し、各 arm 90 observation を試行した。harness-materialized arm は90/90 call と5/5 trialを完了した。v3 full-JSON arm は87/90 callを完了し、3ケースが `representation_protocol` で失敗したため、v3 では2試行が運用上未完了となった。両 arm の運用上完全な trial では、labelled precision と recall はすべて1.000だった。materialized arm の ambiguous-abstention rate は5試行で 0.429、0.429、0.571、0.429、0.429 だった。87個の対応付けられた成功ペアのうち、2つの decision が変化し、decision-flip rate は0.023だった。

これは Gemma 4 31B における harness-owned materialization を支持する protocol evidence であり、ambiguity が解決されたことの証拠ではない。materialized arm は、意図的に ambiguous とした複数の fixture に対してなお assertive だった。

### D2 決定可能性

D2 stage は75/75 provider call と5/5 trialを完了し、運用上の失敗はなかった。eligible clear coverage、precision、recall の aggregate はすべて1.000だった。35個すべての typed-insufficiency variant は composition 前には assertive で、composition 後には abstain を強制された。すなわち base unsafe assertions 35 -> composed unsafe assertions 0、typed-insufficiency abstention 35/35 = 1.000、clear-case seed disagreement 0 である。eligible ambiguous abstention は15/20 = 0.750だった。

これらの D2 aggregate metric は、0.750 の ambiguous-abstention diagnostic を含め、以前の Ministral 8B D2 observation と一致する。

### ホールドアウト v5 パイロット

v5 stage は120/120 provider call と5/5 trialを完了し、運用上の失敗はなかった。eligible clear coverage、precision、recall の aggregate はすべて1.000だった。50個すべての typed-insufficiency variant は composition 前には assertive で、composition 後には安全に abstain した。すなわち base unsafe assertions 50 -> composed unsafe assertions 0、typed-insufficiency abstention 50/50 = 1.000、clear-case seed disagreement 0 である。eligible ambiguous abstention は20/40 = 0.500だった。

Gemma 4 31B v5 aggregate は、以前の Ministral 8B v5 aggregate と同一だった。全120個の対応付けられた semantic observation の直接的な case/seed 比較でも、base-decision difference は0だった。両 model family は、すべての seed で同じ4つの ambiguous fixture（`v5h05`、`v5h06`、`v5h23`、`v5h24`）で abstain し、同じ4つ（`v5h11`、`v5h12`、`v5h17`、`v5h18`）で `finding` を返した。

これは、固定された typed decidability composition が一つの Ministral 固有の decision pattern を補正しているだけではないことを示す、強い cross-family replication evidence である。ただし、universal model portability の主張ではなく、元の D3 adoption provider set を遡及的に変更するものでもない。4つの ambiguous v5 fixture に対する共通の assertiveness は、観測後に固定 corpus をチューニングする理由ではなく、model-facing semantic boundary として可視のまま残る。
