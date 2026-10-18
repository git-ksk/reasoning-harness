# 対象APIを通す実験ホストの接続確認

2026-10-19。[実験コードと生記録](../evaluation/engine073-active-source-host-development-v1/README.md)を保存した。通常CLIを変更せず、既存のNode.js資料・観測済み質問を、0.6.1と現在の修正source ab3c632へ同じ入力・同じhostで渡した。新規合成問題・独立実24問・model/provider呼出しは0。Engine公開versionは0.7.2のまま。

| 切り分け | 両armの実測結果 | 意味 |
| --- | --- | --- |
| v17→v30→条件付きsource-local表示 | 各APIをhostから1回ずつ呼び、関係はunresolved、関連性はambiguous。最終保留・引用0 | 対象APIが実行され、保留が表示まで伝わる。肯定結果や改善は得ていない |
| exact quote表示部品だけの確認 | 明示的な手動source bindingでqualifiedな原文引用と構造化出典1件 | 表示部品単独の接続成功。関連性gateの成功へ加点しない |
| 引用本文の改変 | 元の全文byte位置と一致せず、API前に停止・stdout空 | 改変した引用を元資料の引用として通さない |

使用した原文のBuffer subclass記述では、両EngineのDefinition判定が保留になった。手動のexact proposal/raw qualificationで上書きせず、そのまま保存した。これは既知の開発入力による接続診断であり、新規の意味goldを付けたFNや回帰必須ケースではない。ここからEngine修正や合成60件追加へ戻らない。

表示部品の対照は関連性APIを呼ばない独立した切り分けで、通常CLIの出力でも、新しい質問に対する正答でもない。source/evidence/binding、URL・section・取得時点・source versionと原文を結び、truth claim/verification receiptはいずれも0。原資料の全文と正規化本文はhashと引用位置を照合してprivateに保持する。時間・scopeの意味的正しさやpublisher認証は未裁定。

baselineは4abee90/Engine0.6.1のGit archiveからoffline buildし、全1970 tracked fileをGit blobと照合。candidateは先の通常build成果物をhash確認して再利用した。production source hashは2e9207…、package versionは0.7.2。host・入力・rlib・実行ファイル・compiler・API入出力・exitを保存。ビルドprofile差があるため、速度・費用の比較には使わない。全928原参照と旧主張算術/表示部品は不変。

**実回答F1は未測定、real24=0/24、採用判定NOT_EVALUATED。** 同じ結果の両armから品質改善を認定しない。通常CLI対照・対象API経路・単独rendererを分けて、外部の主張分割/支持/矛盾/訂正/引用裁定とcase品質・原子的主張F1、欠測会計、runner依存/数値予算を接続する。独立部品レビューと完全M0の後に新実24問を著作する。Groq停止、合成60追加停止、source merge/Engine公開/CLI採用NO_GOを維持する。
