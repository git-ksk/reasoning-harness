# Harness Engine 0.7.3 — 修正・測定・採用の順に進める

日本語 | [English](engine-0.7.3-roadmap.md)

未公開・未受入。公開CLI0.5.5/Engine0.6.1、最新独立Engine source release0.7.2は不変。[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)・[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)。0.7.3の版昇格・公開は別PRで判断する。

## 最新の修正・独立評価と残件

**#520 方針改定（2026-10-17）:** 今後の新規独立合成source gateは**60件すべて正解**を必須とせず、両公開API別に **F1>=0.90・Precision>=0.95・Recall>=0.85**、否定（撤回・明確な非実在/否認等）の誤採用0件、既知回帰100%（元14件監査を含む）、過去API互換と運用完全性を必須とする。独立60件以上（陽性/陰性各30以上）は維持。v24を含む凍結初回のFAILは変更せず、今後の評価契約だけに適用する。詳細は[新方針](../evaluation/engine073-source-gate-v1/README.md)。#531の実回答24問・各provider・実用性・安全性と#540のrunner要件は変更せず、公開/CLI採用もNO-GOのまま。



候補 `9082365` の新規独立 v26 初回は **FAIL / NO_GO**。両API別に TP22 / FP0 / TN30 / FN8、Precision 1.000 / Recall 0.733333 / F1 **0.846154**。安全性誤採用0・運用失敗0・欠測0だが、固定した F1 0.90・Recall 0.85 の下限未達。修正に使った v25 開発 F1=1.000 と区別し、異なる独立集合のF1差を改善率にしない。初回 Engine 1 process・60 inputs / scorer 1 / helper 1、retry/rescore/relabel0。M0親レビュー59純fixture、M0/M1事前固定と13 provenance checks、原99ファイル・旧730参照・全829 corpus参照を親でも照合した。原v25 FAILと全過去初回は不変。8件の実経路は7件がtyped_reference_not_excluded、057がunresolved_adverse_scope。現在の既知対照は1392/1400、元監査14・歴史597・全体1594成功は候補908の別検証。次は属性・所有者・局所参照の共通処理を修正し、既知1400件を全成功させてから新しい候補/評価器を著作前に固定する。新規v27は未著作。実24問0/24・実回答F1未測定、Groq停止、sourceマージ/Engine公開/CLI採用NO-GO。[独立判定](engine073-v26-decision.ja.md)、[原初回](../evaluation/engine073-launch-independent-v26/README-results.md)、[実経路](../evaluation/engine073-launch-trace-development-v10/README.md)。

| 順序 | 必要な証拠 | 現状 |
| --- | --- | --- |
| 1 | #520の対象/全体否定・未知owner保留、既知回帰と新規独立source gate | 9082365の新規独立v26初回は52/60診断・両API F1 .846154/FP0、FAIL/NO_GO。既知1392/1400の8保留を共通修正し、全成功後に新候補/評価器固定。v27未著作。元v25初回FAIL不変。[判定](engine073-v25-decision.ja.md)、全初回は不変 |
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
