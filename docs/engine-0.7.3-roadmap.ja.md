## 本評価の初回結果 — 2026-10-19 JST

**最新状況（2026-10-20、評価基盤v2の修正中）:** 修正済み共通CLIと両EngineのLinuxビルド、実CLIの正常終了・上限停止12対照・補助API3対照を無課金で確認した。独立レビューのruntime設定・再試行groundingの漏れも修正。最新の取得→受入→採点接続を実環境で確認中。次は既存24問で予算・完走を確認し、その後に完全M0固定と新規独立評価へ進む。今回の構造試験数は精度問題数に加算しない。旧初回の全体F1=null/NO_GOは不変、Groq延期・0.7.3未公開・CLI未採用。[修正と検証範囲](../evaluation/engine073-realworld-v2-development-preparation/README.md)。

以下は2026-10-19の初回結果です。
**最新状況（2026-10-19、初回実戦評価・独立採点済み）:** M0/M1を固定し、未使用だった公式12文書・6製品の実資料24問でMistral/Googleを実行したが、CLI論理呼び出し上限6と失敗時の使用量欠落で早期停止。通常回答は評価器の2段階終端照合にも不備があり、24問全体のF1は全provider/trackで`null`、**採用NO_GO**。採点可能なGoogle補助API1問/armのみ、baseline/候補ともcase F1=0.00・claim F1=1.00（最終判定conflict、改善なし）。これは全体性能ではない。[初回結果・独立レビュー・修正順序](../evaluation/engine073-realworld-first-v1a-results/README.md)。次は#540の結果回収・終端照合・失敗時診断を修正して実経路を確認し、別途固定した評価へ進む。Groqは認証前延期を維持。元raw/gold/予算/初回FAILは不変、モデル再試行・再採点なし。公開CLI0.5.5/Engine0.6.1、Engine0.7.3未公開。以下の日付付き準備記録は履歴。

### 以前の準備記録（初回評価前）

ユーザーの再指示により、未使用の実資料24問でbaseline0.6.1と候補ab3c632を比較し、実測のcase/claim P/R/F1を報告する。合成の新規60問ループは再開しない。

- 採点器の限定独立再レビューPASS。独立24構造試験・修正対照8件を保存。実際の両armのprovider模擬HTTP18件もPASS（Google応答fixtureの初回失敗は別保持）。精度評価の問題数へ加算しない。
- 採点器・通常CLI/補助host・閉じたLinux runtime・数値予算をM0で固定してから、別作成者の新規24問と全文source/goldをM1で固定する。各社/通常/helper/各armの288観測を別分母で報告する。
- 通常CLIの引用表示不足を事前宣言。case F1が高くても引用・有用性が不足すれば採用NO_GO。未根拠positiveをcase TPにせず、誤肯定negative/ambiguousをFP/hardとして保持。
- Groqは「他利用なし・残量フル」の新owner申告を記録。旧枯渇記録は不変。別の一度きり・微小予算のquota確認で実上限を読み、全cohortのheadroomが不足する場合は停止する。自動課金プラン変更はしない。

現時点で新規実24問0/24、actual-answer F1未測定、M0未完了、Engine公開/CLI採用はNO_GO。以下の過去記録と原FAILは保持する。

---

# Harness Engine 0.7.3 — 修正・測定・採用の順に進める

**実資料評価の接続準備（2026-10-19）:** 通常CLI保存記録と全文資料・引用位置の照合に加え、既存Node開発入力で対象APIを呼ぶ実験hostを保存した。0.6.1/現在候補とも関連性は保留・引用0。別の手動binding対照では出典付き原文表示を確認し、関連性成功へ混ぜない。新規独立問題0・実回答F1未測定。次は意味/引用裁定・欠測会計・数値予算を完全M0へ結ぶ。[実験host](engine073-active-source-host-development.ja.md)、[通常CLI記録の照合](engine073-cli-trace-binding-development.ja.md)。

**評価運用の見直し（2026-10-19）:** 合否は事前固定した独立F1/P/Rと安全性で判断する。既知対照の件数・満点は完成度にしない。着手済み6件の修正後は合成60件の追加・修正ループを止め、実資料・通常CLIの経路/traceと完全採点器へ重点を戻す。既知回帰は再発防止対象を著作前に定義し、新holdoutの全FNを自動で100%必須へ追加しない。凍結初回は不変。[詳細](engine073-evaluation-focus.ja.md)。

日本語 | [English](engine-0.7.3-roadmap.md)

未公開・未受入。公開CLI0.5.5/Engine0.6.1、最新独立Engine source release0.7.2は不変。[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)・[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)。0.7.3の版昇格・公開は別PRで判断する。

## 最新の修正・独立評価と残件

**#520 方針改定（2026-10-17）:** 今後の新規独立合成source gateは**60件すべて正解**を必須とせず、両公開API別に **F1>=0.90・Precision>=0.95・Recall>=0.85**、否定（撤回・明確な非実在/否認等）の誤採用0件、既知回帰100%（元14件監査を含む）、過去API互換と運用完全性を必須とする。独立60件以上（陽性/陰性各30以上）は維持。v24を含む凍結初回のFAILは変更せず、今後の評価契約だけに適用する。詳細は[新方針](../evaluation/engine073-source-gate-v1/README.md)。#531の実回答24問・各provider・実用性・安全性と#540のrunner要件は変更せず、公開/CLI採用もNO-GOのまま。



修正候補 `ab3c632` は、同じ観測済みv27の開発評価で両API TP30/FP0/TN30/FN0、P/R/F1=1.000。元の独立v27初回F1 .888889/Recall .8/NO_GOは変更しない。完全な印字属性・同roleのowner/複数参照・括弧内の属性限定を共通処理で修正した。修正した不具合と安全性境界の退行を確認し、全体1605PASS/0FAIL/既存ignore8・strict Clippy/fmt、元14・歴史597byte互換を保存。過去1400は同じsourceの通常成果物で一度確認し、rootでは必要な60/14/597照合に絞った。件数・開発満点を完成度にせず、次の合成60件は作らない。次は実資料・通常CLIのfeature-active経路/traceと完全採点器・数値予算へ重点を戻す。主張算術/表示binding部品32検証は準備のみ、独立部品レビューは利用上限で未完了。全928原参照は不変。実24問0/24・実回答F1未測定・Groq停止、sourceマージ/Engine公開/CLI採用NO_GO。[方針](engine073-evaluation-focus.ja.md)、[保存済み修正](../evaluation/engine073-launch-development-v27-bounded-properties/README.md)、[原独立判定](engine073-v27-decision.ja.md)。

| 順序 | 必要な証拠 | 現状 |
| --- | --- | --- |
| 1 | #520の対象/全体否定・未知owner保留、既知回帰と新規独立source gate | ab3c632の消費済みv27開発F1=1、元独立v27 .888889/NO_GO不変。全体1605成功。次の合成60追加は止め、実資料/通常CLIの評価器と経路へ重点移行。元v25初回FAIL不変。[判定](engine073-v25-decision.ja.md)、全初回は不変 |
| 2 | #540共有停止・永続reserve/inflight・キャンセル、actual host callback/dependency/数値予算、通常経路trace | #535/#537/#541 merge、47 offline controls/限定独立6境界PASS。新scorer・live callback/dependency/数値予算・外部containment・featureが働く経路は残件 |
| 3 | 小規模offline開発確認、必要かつ別途開始条件を満たしたprovider確認 | 今回provider/model/Oracle追加0。旧v2/v3初回FAILを維持 |
| 4 | 完全M0評価器/host/model/予算/品質基準を固定→別作成者の新実資料24問→全文/gold M1固定→同hostで0.6.1/後継を比較 | 実24問0/24、M0/M1未完了。重大違反0・陽性退行0・provider別完全結果・有用性改善と品質下限が必須 |
| 5 | 宣言した全必須受入・CI後に専用Engine0.7.3 source release PR | 未公開。合成API F1=1だけで通常リリースしない |
| 6 | 専用CLI採用PR、Mac/Windows/Linux・配布・install/update/rollback | 採用1/6（開発比較・限界・NO-GO記録のみ） |

## 実回答の評価準備

[実回答の採点準備](engine073-realworld-metrics-preparation.ja.md)で主張grounding Precision、必要主張Recall/F1、引用、正しい/過剰保留、有用性、運用欠測、provider別会計を定義した。PREPARATION_ONLY、M0/M1未固定・実回答未採点。新scorer、実host callback/dependency closure・数値予算/headroom・金額自己上限、featureが働く通常CLI経路が残る。[過去候補01dcの同一host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md)は固定Node1経路で両Engineの同じgrounded回答・取得1・関連性API呼出し0、直接v17/v30対照ではcounter動作を確認した。全経路の不活性や今回修正の実回答効果を確定した結果ではない。#541共有runnerはmerge済みだが#540のlive準備はOPEN。#531新規実24問0/24・採用1/6、Groq追加起動禁止、provider/model/Oracle追加0を維持する。

mainの[#542品質ゲート](engine073-quality-gate-integration.ja.md)は、将来の各provider/trackのcase品質F1>=0.90・Precision>=0.95・Recall>=0.85を同時要求する追加下限。原子的主張F1とも合成API F1とも別の指標で、重大違反0・既存ゲートを緩和しない。完全scorer/traceと両単位の用途をM0で固定してから新規独立実24問を著作する。

## 保持する証拠と次の修正

[固定候補の開発検証](../evaluation/engine073-launch-development-v24-structure/README.md)・[不採用対照を含む試作](../evaluation/engine073-launch-development-v24-structure-prototypes/README.md)・[実経路trace](../evaluation/engine073-launch-trace-development-v7/README.md)・[元の独立v24初回](../evaluation/engine073-launch-independent-v24/README.md)を保存。[新規v25初回](../evaluation/engine073-launch-independent-v25/README-results.md)もFAILとして閉鎖した。次は[判定と作業順序](engine073-v25-decision.ja.md)に従う。異なるコーパスのF1差を改善率とせず、同じ観測済み集合で修正効果を測る。

Groqの既知禁止は時間経過・TPM間隔・小さいreadiness・モデル交換で解除しない。公開tag・歴史v16/v29・全初回・旧評価器は不変。運用欠測は成功の陰性や費用0にしない。#488/#489/#491は実測で必要な場合のみ。#465基盤・#506WinGetは別。0.8.0機能拡大・Oracleは含めない。
