# 実回答の採点準備と #542 品質ゲートの接続

本修正の作業中にmainへ入った[PR #542](https://github.com/git-ksk/reasoning-harness/pull/542)（c1cd554）は、将来の実資料評価へ品質下限を追加した。[独立担当の採点準備](engine073-realworld-metrics-preparation.ja.md)は、その前の12ファイルを読んだ時点の記録であり、[入力hash](../evaluation/engine073-realworld-metrics-preparation-v1/reviewed-input-sha256.json)を変更しない。本書は後から増えた要件の整理であり、新規実資料の著作・初回実行・採点を意味しない。

| 指標 | 正解と観測の単位 | 状況・用途 |
| --- | --- | --- |
| 合成API F1 | 固定ソースがv17 requested_relation / v30 Relevantであるべきか | #520の開発・独立source gate。実回答の品質下限へ転用しない。従来の60/60条件は過去の凍結評価のみ保持。#520の**今後の独立合成source gate**はF1>=0.90・Precision>=0.95・Recall>=0.85、否定誤採用0、既知回帰100%に改定（#531の実回答ゲートとは別） |
| #542 候補品質F1 | 独立goldのclear positive/negative/ambiguousと、candidateの回答/保留。通常は完全で根拠付きの回答、helperは別承認された対象への出典付き表示 | 必須provider・trackごとに完全24件、clear陽性8以上・陰性1以上、F1>=0.90・Precision>=0.95・Recall>=0.85を同時達成。F0.5>=0.95は診断目標。別provider/trackを合算しない |
| 主張grounding P・必要主張Recall/F1 | 実際に表示した原子的主張の支持、goldで必要なsupported主張groupの充足 | [採点準備](engine073-realworld-metrics-preparation.ja.md)で分母0・保留・欠測を定義。#542のcase二値F1と同じ値ではなく、未実装の完全scorer/rubricと接続してM0で用途・下限を固定する必要がある |

#542の純粋な計算器は、凍結済みgoldと外部で判定された行を計数する将来の統合部品。本文を読んで支持・引用・時点を判定する採点器や、表示span→取得→admission→evidenceの照合を自動実装したものではない。数値下限を満たしても重大違反0、既知未修正欠陥0、陽性退行0、対比較の有用性増加、通常CLIとhelperの区別、全provider完了、予算・配布の既存必須条件を代替しない。曖昧を陰性へ変換せず、不当な肯定は別の重大違反。運用欠測を意味的な保留成功にしない。

新しい主張F1の解釈を#542のcaseF1へ後付けせず、両方の固定単位・分母・採点権限・回答可能層・反例・集計と閾値を、独立24問の著作より前に完全に固定する。M0未完了、M1未作成、実24問0/24は変わらない。#542の設定を過去v2/v3や独立v1–v23へ後付けして再採点しない。合成APIの開発F1=1から、24問のcase品質ゲートや一般精度95%の達成は主張しない。
