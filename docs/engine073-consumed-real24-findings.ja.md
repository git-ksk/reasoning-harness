# 消費済み実24問：実測から決めた次の修正

2026-10-20 JST。[初回開発スコア](../evaluation/engine073-realworld-v2-consumed-score-v1/README.md)は不変。Googleのみ全24問/arm/trackが完走し、通常F1 0.583333（旧版と同じ）、helper0.222222→0.400000。Mistral/Groqの全F1はnull、公開・CLI採用はNO_GOです。

この評価は `host_curated_facts_with_full_captured_context` です。公式資料全文を保持して独立裁定しましたが、実行時の型付き事実は評価hostが供給しています。自由なWeb探索や全文からの自動抽出、Engine単体の性能を測った値ではありません。今回のFNには、供給した事実が原文の条件を短くし過ぎた例と、CLIの最終化・調査終了の問題が含まれます。モデルの自然文をそのまま表示して解消する方針にはしません。

| 優先 | 保存済みの問題 | 最小の改善方向 |
| --- | --- | --- |
| 1 | rw02/06は条件・理由が短い型付き値に縮約。rw11/17は適用条件・未解決事項が最終化で消失。通常引用0/28。 | 取得・採用済み資料に結び付いた条件/理由/未解決facetと引用を最終表示まで保持する。特定gold文言をruntimeへ入力しない。 |
| 2 | rw10/13は比較の片側だけ取得後にAccept終了。rw16はstop後に未結合の説明だけ残り空のunresolved。 | 生き残ったclaimの合格と質問全体の充足を分ける。計画の必須対象を取得するか、未解決として明示する。同じ数値予算内で行う。 |
| 3 | rw04/20は未実行の明示、rw19は別の必須前提の注意が欠落。 | hostの実際の実行記録とsource-bound前提を表示し、未実施操作の実行済み・全前提充足と誤認させない。 |
| 4 | helper rw05/08/15はqualifiedだが同一origin/独立した裏付けではない旨が未表示。 | 同じURL/version/spanという確認済み情報から注意を表示する。未知の独立性を推定せず、重複資料への正しい拒否を緩めない。 |
| 5 | helper rw01/14/21は資料を取得済みでも公開APIの互換レビューが拒否。 | 条件・platform・要求された属性の比較範囲を調べる。advice/reference=trueだけで拒否を上書きしない。現時点では内部原因の断定なし。 |

Mistralは別枠です。PR549/main53be613は既知usage欠落だけを修正しました。256tokensでplanner JSONが途中切れになった問題、修正後の全量完走、残量確認は未完了。現在は追加API呼出しを保留し、保存済み失敗から出力設計を検討します。Groqも延期を継続します。

実host8525330の根拠：[早期Accept](https://github.com/git-ksk/reasoning-harness/blob/8525330ea3cadb897c50d6688228c4895558d3dc/crates/reasoning-harness-cli/src/main.rs#L3768)、[stop](https://github.com/git-ksk/reasoning-harness/blob/8525330ea3cadb897c50d6688228c4895558d3dc/crates/reasoning-harness-cli/src/main.rs#L3579)、[最終化](https://github.com/git-ksk/reasoning-harness/blob/8525330ea3cadb897c50d6688228c4895558d3dc/crates/reasoning-harness-cli/src/main.rs#L4297)、[partial表示](https://github.com/git-ksk/reasoning-harness/blob/8525330ea3cadb897c50d6688228c4895558d3dc/crates/reasoning-harness-cli/src/main.rs#L4348)。現在のmainとの比較は静的確認であり、現mainでの再実行ではありません。

独立診断の閉鎖anchorは `63156810a73981c68532f63c32d5039f301231c461e9e61e4f3938a8da3b0713`、stop行番号だけの別訂正票は `ed101902ea072f944518760ea9a196c67a503728c13d82cc83764776e1315f81`。完全raw/注釈/診断はprivateに保持。既存スコア・gold・注釈の再計算/変更、新しいEngine/モデル実行は0です。

#491は必要情報を取りこぼす実例が確認できたため、まずcoverageを保つ小さい修正を検討します。#488は既知originの表示と新しい独立性判定を分け、後者はまだ自動採用しません。#489のtrusted訂正・更新機能を新規実装する根拠は今回未確認です。

新合成問題を増やすことを進捗目標にせず、この既存24問で修正の効果と退行を開発測定します。新規独立受入は機能経路・完全M0・全providerのreadinessを先に固定してから著作します。F1≥0.90/P≥0.95/R≥0.85、重大安全性違反0、退行0、実測の有用性改善という基準は変更しません。上表は修正計画であり、修正済み/受入済みの主張ではありません。
