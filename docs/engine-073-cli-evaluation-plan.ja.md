# 通常CLI中心の評価計画 — 2026-10-26

日本語 | [English](engine-073-cli-evaluation-plan.md)

ユーザーの指示により、次回以降は通常CLIの実モデル評価と、補助APIのモデルなし契約検証を分ける。これは次回計画であり、旧v2の採点条件・結果・入力・凍結receiptを書き換えない。現時点の採用判断は保留。新しい実API実行やリリースをこの文書だけで許可しない。

## 評価対象

| 対象 | 方法 | 判定すること |
| --- | --- | --- |
| 通常CLI | 実モデルで質問→調査→根拠確認→最終表示 | 利用者が安全で有用な回答を得られるか。CLI採用の主評価 |
| 補助API | モデルも認証情報も使わず、仕様から定めた入力と期待結果を直接比較 | 正常処理、拒否すべき入力、権限、改変耐性、再現性、元artifact保全、互換性 |
| 旧helperモデル連携実験 | 既存結果を参考として保存 | AI提案＋制御ホスト＋API表示という組合せの結果。API単体の合否や通常CLIの合否へ流用しない |

補助API自身はモデルを呼ばず、信頼した呼出し側の照合記録を検査・適用する。自然文の一般的な意味一致を発見するモデルとして評価しない。AIの助言だけに照合承認権限を与えない。

## 現在の証拠と残り

- [旧v2の実行・独立初回F1](cli-paired-real24-v2-20261026.ja.md)は保存済み。通常Geminiは両Engineとも23/24問で参考F1 0.933333、Precision1.000、Recall0.875。数値上は基準内だが、欠測1問と最終修正の影響を含む確認が残る。Engineの改善量とは主張しない。
- Mistral通常はbaseline15/24、candidate13/24のみ採点可能。異なる集合の参考F1 0.428571/0.461538を改善量としない。まず保存済み全48通常出力から、計測器の欠測と実際の回答不足を分けて原因を調べる。
- captureの再取得/空イベント修正は[PR560](https://github.com/git-ksk/reasoning-harness/pull/560)で統合済み。85controlsは計測器の確認で、補助API単体のテスト結果や精度改善ではない。
- 旧helperのモデル連携F1 0.400000は当時の契約で未達のまま保持する。今後のCLI採用を止める独立の品質指標にはしない。仕様違反や安全性不具合を無視する意味ではない。
- Mistral/Googleを優先し、Groqは保留。未実行を成功・TN・0点へ変換しない。Groqまで含めた全provider受入を完了したとは主張しない。

## この作業の進捗

- 計画・日英ロードマップ・#531/#538/#540を同期済み（PR561）。
- [補助APIのモデルなし独立確認](../evaluation/cli-normal-v1/model-free-api-review/review.json)はPASS。既存core960成功/歴史的skip2、独立直接44グループ、旧API互換9ケース。対象source不変・外部model0。hostの実認証や一般的な意味同値判断までAPIが代行するとはしない。
- [Mistral保存48通常出力の独立診断](../evaluation/cli-normal-v1/mistral-independent-diagnosis/diagnosis-summary-v1.md)を完了。計画上限超過14・ID重複2と、candidate claims欠落11応答・内部誤キー3応答を確認した。
- 2026-10-28: #562/#563/#564のCLI修正は[PR561](https://github.com/git-ksk/reasoning-harness/pull/561)でマージ済み（`82b48a4`）。要求上限・candidate形状・再出力前の予算判定を修正し、局所独立レビューと最終headのCI22件が成功。共通評価hostは`c38d87d`に固定。公開core型やclaims:[]の正当な保留は維持する。
- 同じ資料の再取得による停止、Rejectによる取得短絡、selector/説明不足は別の残件として保持する。2修正だけで全問題が解消したとは判断しない。
- [PR565](https://github.com/git-ksk/reasoning-harness/pull/565)でMistral専用48jobsの通常profileを準備中。新規12・継承85のoffline controlsと無害な実CLI loopbackは成功したが、準備版`7f0508e`の独立source reviewで、旧観測を新runへ混入できる採点由来の欠落を確認した。実run/artifact/receipt/rawの結合を修正してから再レビューする。新しいモデル生成・F1はまだ0。

## 2026-10-28 の実行前追加修正

#566の後継CLIは[PR567](https://github.com/git-ksk/reasoning-harness/pull/567)の`dbad2dc`に固定し、ソース限定の独立確認と両EngineのLinux build・実重複取得loopbackを通過した。現在の評価profileはこのhostを事前指定するが、製品CIのmacOS runner割当と最終runtime/image受入は未完了で、実行許可ではない。


[Issue566](https://github.com/git-ksk/reasoning-harness/issues/566)を優先する。独立operational reviewで、同じ固定snapshotの再取得が既存coreでmalformed扱いとなり、新runnerが停止する経路をMistral14/48・Google2/48の保存済み通常payloadで再現した。採点・エラー分類を緩めず、現在保持する証拠と現在のadmission方針で完全同一と確認できる再取得だけを、共通CLIの新しい版でno-progressとして扱う。外部resolverの発行履歴だけでは採用済みと判断しない。実resolver呼出し・費用を保持し、別内容・不明なmetadata・権限拒否は停止させる。

順序は、CLI後継修正・境界テスト → 別担当レビューとCI → 共通hostを両Engineで再ビルド → 新runtime/image/proofの独立受入 → Mistral48jobs・新F1。準備版3086778のsource-only PASSやローカルDocker成功は、新しい実行の許可ではない。旧raw・旧F1・公開core・過去の凍結条件は変更しない。

## 作業順序

1. 本計画と日英ロードマップ、#531/#538/#540を先に同期する。
2. 候補Engine `ab3c6322921c8c738ec35bd018345917b5afa538` の補助APIを、モデルなしで独立確認する。既存の仕様テストを優先し、未検証の境界だけを追加する。失敗は再現して修正し、単なるtest件数を品質スコアとしない。
3. Mistralの保存ログを調べ、初期candidate・調査計画・資料取得・最終化のどこで回答が失われるか特定する。原因を再現できた箇所だけ修正する。共通CLIの修正は両Engineへ同一に適用する。
4. 通常CLIだけの新しい実行profileを固定する。各providerは24問×両Engine＝48jobs、補助API向けモデル呼出し0。旧72jobs/96観測profileやreceiptを再利用して開始しない。
5. 新source・実binary・image・入力・採点・独立review・実行条件を固定後、Mistralの通常CLIを再測定する。最終候補ではGeminiも全24問を確認する。既知24問は開発診断のまま扱う。
6. 新規独立資料24問で通常CLIの最終受入を行う。評価方式・候補固定後に別作成者が作り、初回実行前に元資料全文・引用・期待結果を確認する。過去の注釈やスコアは流用しない。
7. 候補Engineの版を確定し、専用CLI統合PR・Mac/Windows/Linux・配布/インストール/更新/rollback確認へ進む。評価だけで配布完了とはしない。

## 通常CLIの次回判定条件

- 必須に指定したproviderごとに全件を完了。case/claimそれぞれF1≥0.90、Precision≥0.95、Recall≥0.85。FP/FN/TN/TP、分母・保留・欠測を併記する。
- 重大な安全性違反0、引用の対応・coverage・literal fidelity1.0、既知の安全性不具合なし、既存の有用な回答に退行なし。
- baselineとの有用回答差・退行を対になった同じ問題で記録する。Engine差、共通CLIの改善、モデルの単回差を区別する。同等の成績を改善と呼ばない。
- 今回の目的は通常CLIが安全に使えるかの判定であり、未使用APIの表示増加を要求しない。旧v2の各trackでの改善件数ゲートは旧結果にのみ適用し、次回は通常CLIの絶対品質・非退行と改善量の報告を分ける。
- 効率は同じ完全pairで比較。平均logical calls差≤1、aggregate tokens比≤1.25、p95時間比≤1.5。欠測や未定義で合格としない。
- Geminiの旧23問F1だけで新規独立受入や最終採用を完了にしない。小標本から母集団精度を保証しない。

## 実行・記録の保護

既存のモデル/tier、単回HTTP、停止/キャンセル、durable予約、unknown usage停止、pacing、自己上限Mistral USD2.5/Google USD7.5を維持する。Groqの再開、上限引上げ、モデル切替、失敗の自動再実行は行わない。新しい実行条件とその根拠を固定する。旧v1/v2のraw/観測/注釈/初回report/receipt/tagは保持し、後付け再採点や欠測行の合格化をしない。

補助APIのモデルなしPASS、CLI通常F1の合格、Engine公開、CLI採用、配布受入は別の完了項目。#488/#489/#491の追加開発は通常CLIまたはAPI仕様の再現できる問題を根拠に必要な範囲へ絞る。
