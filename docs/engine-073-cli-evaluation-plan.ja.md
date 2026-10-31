# 通常CLI中心の評価計画 — 2026-10-26

**2026-10-31の次工程:** 通常v2実行は不正なモデル本文によって2完了/1失敗/45抑止で停止し、全量F1は未測定。[結果と独立RCA](../evaluation/cli-normal-real24-development-v2-results/README.md)を固定した。[次期測定契約](cli-model-output-measurement.md)に従い、モデル本文失敗の型付き診断と品質invalid集計を先に接続する。旧結果を変更せず、新しい独立runtime受入・receiptの後に通常48件を実測する。

日本語 | [English](engine-073-cli-evaluation-plan.md)

ユーザーの指示により、次回以降は通常CLIの実モデル評価と、補助APIのモデルなし契約検証を分ける。これは次回計画であり、旧v2の採点条件・結果・入力・凍結receiptを書き換えない。現時点の採用判断は保留。新しい実API実行やリリースをこの文書だけで許可しない。

## 2026-10-30 入力サイズ修正後の再測定準備

[#569](https://github.com/git-ksk/reasoning-harness/issues/569) / [PR570](https://github.com/git-ksk/reasoning-harness/pull/570)は独立source確認・CI22件成功後に`d61d7b8`でマージ済み。共通CLIの通常・JSON fallbackでbuilder由来JSONの整形空白だけを削減する。実保存要求は13,365→11,186bytes、fallback13,683→11,015で、全JSON値・利用者本文・権限/条件・出力形式・seed・上限を維持した。公開Coreは変更しない。

[旧run40873570944](https://github.com/git-ksk/reasoning-harness/actions/runs/40873570944)は2完了・1失敗・45抑止で保持し、全量F1は未測定。直前8件のHTTPは全200・usage既知、第9HTTPは送信前サイズ上限で未送信だった。元のusage_budget_unmeasurable記録は修正せず、診断を#540で別に追う。

[通常v2準備](../evaluation/cli-normal-v2/README.md)は[PR571](https://github.com/git-ksk/reasoning-harness/pull/571)で独立source確認・CI5件成功後にマージし、最初の[run40924323574](https://github.com/git-ksk/reasoning-harness/actions/runs/40924323574)はproof作成で停止した記録を保持し、出力形式だけの修正[PR572](https://github.com/git-ksk/reasoning-harness/pull/572)を独立確認・CI5件成功後にマージ済み。新しい[run40928520250](https://github.com/git-ksk/reasoning-harness/actions/runs/40928520250)は両Engineビルド・全offline controls・既存proof・archiveまで成功し、実イメージの正式独立確認はPASS。新Mistral接続確認は1GET/200/生成0、真正receipt03b86beの48件plan・予約callbackも独立HTTP0確認PASS。実行条件を固定し、評価記録PRのCIと並行して通常48件を起動する。固定v1 treeから変更箇所数・前後hashを照合して別identity/tag/workflow/normal_v2を導出し、両Engineの共通hostを`54eee37`へ固定する。予算・model・tier・pacing・停止条件は同一。v2の独立source確認と実AMD64 runtime/image/proof受入、新metadata/readiness、真正receipt・48件plan/callback確認後に、別の初回実モデル測定を行う。既知24問は開発診断であり、新規独立精度受入・CLI採用は別途必要。

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
- [PR565](https://github.com/git-ksk/reasoning-harness/pull/565)は2026-10-29に`a7de5a4`でマージ済み。Mistral専用48jobsの通常profileに、実run/artifact/receipt/rawと採点の結合、およびダウンロードimageを起動する前のhost側照合を追加した。最終`4bab9cd`の独立source限定レビューはPASS（host境界3・通常20・継承85）。初期`7f0508e`のFAILと修正過程はそのまま保存する。実AMD64 image/proofを[offline run40787270979](https://github.com/git-ksk/reasoning-harness/actions/runs/40787270979)で確認中で、正式運用承認・新しいモデル生成・F1はまだ未完了。

## 2026-10-28 の実行前追加修正

#566の後継CLIは[PR567](https://github.com/git-ksk/reasoning-harness/pull/567)の`dbad2dc`に固定し、ソース限定の独立確認と両EngineのLinux build・実重複取得loopbackを通過した。CI22件すべて成功後、`41fbd17`でマージし#566をcloseした。macOSのキャンセルはrunner未割当による容量不足で、該当jobだけ再実行した。最終runtime/image受入と実モデル評価は別の未完了項目として残す。


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
