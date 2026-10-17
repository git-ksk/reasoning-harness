# Harness Engine 0.7.3 — 修正・測定・採用の順に進める

日本語 | [English](engine-0.7.3-roadmap.md)

**未公開・未受入の後継候補です。** 2026-10-17 JST。[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)／[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)／[Milestone #12](https://github.com/git-ksk/reasoning-harness/milestone/12)。

公開製品はReason CLI0.5.5 / Engine0.6.1、最新の独立ソースはEngine0.7.2です。0.7.2の現在形の撤回・非実在の判定漏れ[#520](https://github.com/git-ksk/reasoning-harness/issues/520)がCLI採用を止めています。0.7.3は新しい修正版の目標で、0.7.2を付け替えません。版の昇格・公開は別リリースPRで判断します。

## 進める順序と完了条件

| 順序 | 作業と必要な証拠 | 現状 |
| --- | --- | --- |
| 1 | #520を修正。撤回・否認・取り下げ・明示的な非実在を対象と主張の範囲で扱い、事実の肯定・準備表現・中立文・無関係な対象や別の性質は維持する。元14入力は開発用の再現として確認し、後継候補SHAを固定する。 | 候補7431333：既知1080件・関連98/全体1562成功、歴史597出力一致。元v21初回FAIL48/60を保持し、保留12件は開発再現で解消。新規独立v22初回FAIL30/60（陽性0/30・陰性30/30、安全性の漏れ0）。[証拠](launch-retraction-520.ja.md)。 |
| 2 | [#540](https://github.com/git-ksk/reasoning-harness/issues/540)で安全な評価runnerを完成。両Engineと補助APIで停止制御を共有し、予約・実行中・停止状態の永続化と再起動時停止、呼び出し全体の保守的予算、強制終了前のキャンセルを実装する。通常CLIの実際のEngine経路をtraceで確認し、Mistral rw03は固定応答で切り分ける。 | #535/#537/#541はmerge済み。実行基盤の最終head CI5/5・オフライン47件・限定独立再レビューPASS。同一CLIホストの経路と固定応答を保存。live callback/dependency・数値予算のfreezeは未完了。 |
| 3 | オフラインを先行し、必要ならMistral・Googleで小さな開発確認をする。通常回答の有用性と補助APIの表示増加は別集計にする。 | 未完了。過去v2/v3の初回不合格は保持。 |
| 4 | 完全な評価器・ホスト・Engine SHA・モデル設定・数値予算・採点基準を固定した後、別作成者が新規の実資料24問を作る。同一ホスト条件で実際の0.6.1と後継候補を比較。重大な安全性違反0、陽性退行0、有用性の実測改善、providerごとの完全な結果を要求する。 | 新規受入0/24。全受入protocolはdraftのまま。 |
| 5 | 宣言した範囲の全必須ゲートとCIを通して、専用PRでEngine0.7.3をソース公開する。リリース方針に従い、検証範囲・限界を記録する。 | 未公開。必須受入が未完了の間は通常リリースしない。 |
| 6 | 採用版を確定して専用CLI統合PR。その後Mac・Windows・Linux、配布、インストール、更新・ロールバックを検証する。 | 採用1/6：開発比較・限界・NO-GO記録のみ完了。 |

Groqは既知の日次quota枯渇状態を引き継いで停止中です。新runnerは認証取得前に停止状態を読み、実利用に基づく残量・回復・共有組織の利用影響を確かめる必要があります。分単位の間隔、時間経過だけ、少量readinessの成功では解除しません。モデル・tier・retry・予算を黙って増やしません。省略・途中終了・未測定は不完全として残し、意味的な不明や費用0の成功にしません。

## 研究知見を反映した次の修正

[研究資料と実装状況](engine-073-research-informed-repair.ja.md)にClaude/CodexとCheckList/HANS/FActScore/ALCEを記録。初回FAILと実際の経路を使い、候補7431333で完全な別製品の属性owner・引用外の証明・同じ役割の参照を修正。既知1080件・関連98/全体1562成功。新規独立v22初回FAIL30/60（陽性0/30・陰性30/30、安全性の漏れ0）。汎用graph/公開API拡大は条件付き。実24問0/24・採用1/6は不変。

## 保持する証拠と範囲

[開発用v2/v3初回結果](engine07-realworld-development-results.ja.md)は不合格・測定不十分のまま保持します。Mistral・Googleの通常回答の改善は0件、補助表示の各2件増加は別指標です。今回のソース修正だけで最終回答の誤り防止・外部事実の正しさ・実利用の効果・独立した人間の承認を証明したことにはなりません。実資料・providerの必須ゲートは[#531](engine-07-realworld-evaluation.ja.md)で継続します。

公開tag・初回結果・固定v2/v3評価器・過去v16/v29・独立評価アーカイブは変更しません。固定後の意味的な変更には新しい候補と新規受入が必要です。#488/#489/#491は実測で必要な場合のみ、#465の基盤と#506のWinGet受入は別工程。0.8.0の新機能拡大やOracle評価はこの計画に含めません。

## 指標を固定して測る

[F1等の集計](../evaluation/engine073-launch-first-metrics-v1/README.md)では、陽性を「v17 requested_relation / v30 Relevantが必要な固定ラベル」と定義し、API別にTP/FP/TN/FN、Precision、Recall、陽性F1、specificity、balanced accuracy、通過率を記録する。分母0は未定義null。v22は両APIともTP0/FP0/TN30/FN30、Precision未定義・Recall0・F1 0・specificity1・balanced accuracy0.5・通過率0。これはCLIの実回答率/最終回答F1ではない。運用失敗・未観測を陰性成功に変換せず、別コーパスのF1を改善率とせず、既知再現と新規独立を分ける。安全性FP0はF1にかかわらず必須。実資料24問では主張ごとの根拠対応と回答有用性も別に採点し、現時点では未測定。


再開した独立担当が最終保存を完了し、全551ファイルを照合。anchor359a6712b07bc1a58fce4332f77f190578b6fec0bad4f9aab0e022c6b3fef160、初回FAILは不変。[親の最終照合](../evaluation/engine073-launch-v22-parent-final-verification.json)・[初回指標の集計](../evaluation/engine073-launch-first-metrics-v1/README.md)を保存。[別コピーの実経路診断](../evaluation/engine073-launch-trace-development-v4/README.md)では型付き参照27・対象否定2・未解決範囲1が最初の保留箇所。次は完全な主張の包みを扱うが、内側の命題が独立に既存の属性確認を通る場合だけ認める。同義の冠詞/未所有表現は、一意な同じroleのforeign ownerがある場合だけ結ぶ。対象/全体否定、競合/未知owner、歴史/rawリスク制限を維持し、同じ入力の開発F1と誤通過数を測る。新規受入とは別で、マージ・公開・採用はしない。
