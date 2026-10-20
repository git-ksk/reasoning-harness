# 通常CLIの記録を実資料採点へつなぐ準備

2026-10-19。[評価用の照合部品](../evaluation/engine073-cli-trace-binding-development-v1/README.md)を実装した。実際の通常CLIが保存した出力・trace・取得journal・受入policy・検証receipt・表示文を結び、保存HTMLの全文hash→固定した正規化→引用byte位置まで照合する。Engineやproviderの新規実行、旧採点器の再実行、独立問題の追加は0。既存の主張算術/表示binding部品と独立初回は変更しない。

## 今回確かめた経路

既存の制御されたNode.js開発記録を使用した。共通CLI0.5.5 host ec6e55f、baseline Engine0.6.1/4abee90、当時のcandidate01dc78b。現在のab3c632の実行結果ではない。旧v2/v3のcanonical初回や、保存されていない元provider応答を再現・再採点したものでもない。

| 境界 | 保存済みの両armで確認したこと |
| --- | --- |
| 実行とmodel記録 | exit0、CLI引数の質問/provider/model、要求と応答のidentity/順序、最終traceと実出力が一致。5回はoffline callbackで、実API回数ではない |
| 取得→受入 | 同じ質問/targetの取得journalからevidence IDと実内容を照合。scope・適用期間・authorityはhost policyと整合。sourceの自己申告で昇格しない |
| 受入→検証→表示 | 同じ命題・evidenceに対するreceiptを確認。内部claimは2件だが表示命題は1件で、二重加点しない |
| 原資料と引用 | 保持していたHTML全文hash、正規化本文hash、実際の引用span10163–10228が一致。本文の再正規化も一致。publisherの認証・意味支持の裁定とは別 |
| 表示面 | 実表示はcanonicalな命題1件。モデルの自然文は表示されていない。表示引用は0で、内部evidence linkを引用成功へ転用しない |
| 機能の実行 | 当時の観測記録は通常経路の関連性API0回、別positive controlではcounter稼働。今回の原資料照合は新しい現在候補のruntime測定ではない |

[照合報告](../evaluation/engine073-cli-trace-binding-development-v1/saved-inspection/inspection.json)は `STRUCTURAL_MATCH` / `EXACT_CAPTURED_BYTES`。**実回答F1はnull、採用判定はNOT_EVALUATED**。意味注釈・引用裁定が未固定であり、引用のない表示や対象APIが動かない経路から、現在候補の有用性改善やF1合格を認定しない。原資料の全文はprivateのまま保持し、新しく公開していない。

## 次の作業

通常CLIは既存挙動の対照として保持し、対象のsource APIを実際に呼ぶ実験用host/profileを分けて用意する。取得資料の全文・引用span・対象・適用時点と、API入出力・表示を同じrun identityで保存する。通常回答と補助表示の分母・承認境界も分ける。Engineの依存versionを替えただけで新しいAPIが通常経路へ入ったとは扱わない。

その接続に、外部レビューによる主張分割・支持/矛盾/訂正/引用の裁定、case品質と主張F1、paired0.6.1比較、callback/dependency/containmentと数値予算を結合する。完全M0の後に未使用の独立実24問を著作し、初回で判定する。合成60件追加の修正ループは再開しない。Groq停止、実24問0/24・実回答F1未測定、Engine公開/CLI採用NO_GOを維持する。部品の独立レビューは未実施。
