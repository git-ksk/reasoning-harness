# Mistral 通常 CLI 48 実行の独立原因診断

担当: `/root/paired_evaluation_independent`。対象は実 API run `40248051474` の保存原本。追加 API、新しい意味注釈、F1 再計算、旧観測の修復は行っていない。

**低い回答率を Mistral の能力だけに帰する根拠はない。モデルの不適切な構造出力と、CLI が制約を伝えない／不正形状を受理する／取得を短絡する経路が重なっている。** 最初に計画上限の伝達とモデル候補 JSON の境界を直すのは妥当。ただし、この 2 点で全問題が解消したとは判断できない。

|確認対象|保存原本で確認できた事実|
|---|---|
|全通常経路|48 実行の candidate、plan、acquisition、resolution、finalization を確認|
|通信との対応|全 360 応答で wire request、journal、trace が一致。保存 HTTP 応答は全て 200|
|計画拒否|16 実行が資料取得 0 回。上限超過 14、target ID 重複 2|
|上限超過の内訳|有効上限 3 に対して 4 件の計画が 9、5 件が 2、6 件が 2、7 件が 1。空 targets は 0|
|候補の形状不正|top-level claims 欠落 11 応答を空候補として受理。初期 4、再生成 7。全件で既存 fallback を迂回|
|別の形状不正|claims はあるが、内部の evidence_ids 等が誤キーの 3 応答を受理|
|既存 fallback|候補生成の finish:error 82 と finish:length 1 では fallback が動作|
|再取得による停止|14 実行で同じ snapshot を 3 回目に再取得し、malformed_output → operational_terminal|
|取得前の短絡|6 実行で既存 Reject により pending acquisition が実行されず attempts=[]|
|欠測の区別|上記 14+6 が capture incomplete 20 件に一致。欠測を虚偽や安全成功へ変換しない|

候補生成は論理 97 回、応答 180 件。HTTP 200 は候補の妥当性を意味しない。また、この保存範囲に quota エラーがないことから将来の残量を推定しない。

## 最初に直す 2 点

1. **実効 plan 上限を送信する。** `rw-02-candidate` は 5 targets を返したが、request の prompt/schema に上限 3 がない。`rw-03-baseline` と `rw-21-baseline` は capability ID を複数の target に流用した。上限と target ID の一意性を明示し、現 admission を維持する。無断の切り捨てや再試行は不要。
2. **モデル wire の候補形状を厳密化する。** `ReasoningCandidate.claims` の `serde(default)` と未知フィールド許容により schema echo が成功扱いになった。`rw-20-candidate` round 2 では `properties.claims` 内に候補があるが top-level claims がなく、モデル候補が空へ置換された。正しい selector に束縛された host の limitation 事実だけが残り、それまで存在した command 候補は消えた。これは誤キーを推測して修復せず、既存の単回 fallback へ戻す境界不備である。

root が後から提示した private strict DTO 方針はこの診断と整合する。CLI 内の top/claim/proposition/inference の形状と schema を同じ DTO で定義し、公開 core 型の保存互換を変えず、明示的 `claims:[]` を許可する。実装の正否と現行 request/schema の互換性は別途 bounded review が必要であり、本診断は修正コードの PASS ではない。

3 件の nested typo は `rw-07-baseline` round 2、`rw-13-baseline` round 1、`rw-15-baseline` round 2。`rw-15` は host 事実補完で有用な出力を維持しているため、形状不正の件数をそのまま回答失敗件数にしない。

## 残る原因と最小対処

|原因|実際の経路|対処と守る境界|
|---|---|---|
|exact typed 値の言い換え／反証候補の混在|rw01 は requires=none を contradicted として残す。rw09 は foreach or while を同じ key の foreach/while に分割。rw19 は文を helpful/not required に分割。rw04 candidate は warning 値を言い換える|型付き key/value は admitted fact の厳密な参照、自然文とは分離。hard finding を捨てたり equality を緩めたりしない|
|Reject が次の取得も止める|rw04 candidate は warning 取得後に Reject。redirected-IO の source1 を選択しても resolver は呼ばれず、空 attempts が no_result として記録される|実行されなかった取得と実行済み no_result を分離。実際の attempt がない admission を記録しない。Reject 中の追加 read-only 調査を認める場合も、別途明示した bounded policy に限定し truth gate は維持|
|同じ資料の再取得を異常扱い|14 実行で 3 回目の evidence object は以前の object と完全一致。固定 ID の重複を core が malformed と判定|immutable cache/reuse/no-new-evidence を明示設計。現在の admission と target binding を検証し、同じ ID の改竄は拒否。ID をランダム化して新しい根拠に見せない|
|selector と capability の不整合|resolver は actual facts に primary lookup key を追加して宣言。rw03 の retry_all、rw10 の included、rw13 の PEER、rw20 の command は selector null になり得る。正しい資料が既にあっても target 未検証が残る|adapter の実際の fact family を正確に宣言。複数 key は明示的な選択が必要。gold や質問の単語から都合のよい key を推測しない|
|保留理由の消失|rw12/22/23/24 の final renderer は不足情報を説明するが、hard finalization 後の表示は汎用 not_acquired/no_progress 等のみ|trusted な plan 拒否・scope mismatch・未実行・現況情報不足を決定的に説明する。未検証のモデル原文をそのまま復活させない|

wrong product、古い資料からの現況推定、接続情報なしの private/live value の保留は維持すべきであり、回答率を上げるために破らない。

## 根拠・保全

- 完全報告: `diagnosis-report-v1.json`（各所見の trigger/cause/minimum fix/副作用、固定 source identity を含む）
- 全 48 行: `normal-row-diagnosis-v1.json` と `all-normal-flow-projection-v1.json`
- 通信の対応: `wire-trace-bindings-v1.json`
- 11 件の形状不正: `candidate-wrong-shape-events-v1.json`
- nested 3 件と length 1 件: `nested-claim-shape-events-v1.json`
- capture と実経路の区別: `runtime-capture-causal-links-v1.json`（再取得の完全一致、Reject のゼロ attempt を確認）
- 固定ソースの行番号付き根拠: `fixed-source-evidence-v1.json`
- 保全結果: `preservation-verification-v1.json`

元 normal raw 512 files、既存 observation/packet 384 files、独立注釈 192 files の hash は不変。既存 analysis-freeze と初回 report の hash も不変。共有 tracked file は変更していない。

core baseline/candidate は固定した別 commit、common CLI は `73bbd18173b4dfcd6664a26241de0385f2eb2e3e` と同一の resolution opt-in patch。両 arm でモデルの plan/候補自体が異なるため、ここで候補 core 単独の改善・退行を断定しない。consumed 24 ケースの原因診断であり、新しい F1、独立受入、採用 Go は出していない。
