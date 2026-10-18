# Codex・OpenCodeから今回の修正に取り入れる点

確認日2026-10-18。一次資料を読み、#520の観測済みv25修正に適用した。製品やSDKの導入を行った記録ではない。

| 一次資料で確認したこと | このプロジェクトでの応用 | 状況 |
| --- | --- | --- |
| [Codex App Server](https://learn.chatgpt.com/docs/app-server)は項目ID、種別、状態、開始/完了イベントを公開し、完了時の項目を確定状態として扱う。 | case/signal/segmentと原文範囲を結び付けて、実際の判定経路を記録。API実行中のtraceと、その後の補助診断を分ける。 | 別コピーのテスト専用traceで実施。公開trace APIは追加していない。 |
| [Codexの修正ループ](https://developers.openai.com/cookbook/examples/codex/build_iterative_repair_loops_with_codex)はreview/repair/validationと各試行の結果を保存する。 | 修正仮説、元候補、source/build hash、F1、失敗対照を試作ごとに保存する。 | 誤採用2件を出した案を不採用として保存し、否定境界を修正後に測定。 |
| [OpenCode Server](https://docs.opencode.ai/docs/server/)と[SDK](https://docs.opencode.ai/docs/sdk/)はsession内のmessageとpartsを分け、message/part IDで個別要素を参照する。 | 役割宣言・訂正先・ownerを区別し、同じ役割の宣言を結び付ける。別役割や直前の属性からイベントownerを借りない。 | copular/colon宣言、同じ役割のowner、明示的な別製品の訂正を修正。 |

この応用は当プロジェクトの実装仮説であり、上記資料が自然言語の訂正先解析や今回のF1改善を実証したものではない。型付きの記録が正しいだけでは、文の意味や事実の正しさを保証しない。

最初の診断は候補9505300の観測済み25保留を対象とし、全件の最初のgoverning-context保留がtyped_reference_not_excludedだった。24件にはrole宣言が登録されず、残る1件には登録済みItemがあった。元初回effective tupleとの一致を確認した。診断に出す原文位置は、原文中で一意に一致したUTF-8 byte範囲だけ。再構成・重複で特定できない位置はnullとする。

実装では「The Object is …」を「Object: …」と同じrole処理へ接続し、valueの型とowner証明は維持。短いObject owner形式・作者の隣接/競合返信も同じ役割の一意性判定へ入れる。属性への訂正と別製品への明示的な否定は別に扱い、while/beforeの後にあるdenyingで対象否定が消えるのを防ぐ。新しい所有者の記述は、完全なイベントのownerと矛盾する場合にも保留する。

測定は元の独立初回、同じ観測済み60件の開発比較、新規独立受入、実資料/最終回答の4つを区別する。合格条件と元ラベルを変えない。残る既知保留を修正・検証するまで、新規v26は著作しない。[最新の測定と残件](engine073-v25-decision.ja.md)。Groq停止・provider/model/Oracle追加0、実資料24問0/24、正式公開/CLI採用NO-GOを維持する。
