# Harness Engine 0.7.3 — 修正・測定・採用の順に進める

日本語 | [English](engine-0.7.3-roadmap.md)

未公開・未受入。公開CLI0.5.5/Engine0.6.1、最新独立Engine source release0.7.2は不変。[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)・[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)。0.7.3の版昇格・公開は別PRで判断する。

## 最新の修正・独立評価と残件

候補`01dc78b`で残る21件を修正した。同じ固定60件の開発比較はv17/v30ともTP30/FP0/TN30/FN0、Precision/Recall/F1各1.000（c20のv30 F1 0.461538から改善）。全体1572成功/失敗0/既存ignore8、strict Clippy/fmt、既知1140・元監査14、歴史597全出力byte一致とv29の60件不変を確認。これは観測済み合成APIの開発測定であり、実回答F1や一般精度ではない。新規独立v23は 初回FAIL40/60（肯定10/30・否定30/30、両APIのPrecision1.000・Recall0.333333・F1 0.500、誤通過0・肯定保留20・運用/不完全0）。元v22初回FAIL30/60・全過去初回は不変。マージ・公開・CLI採用はNO-GO。新しい20保留を現コードで診断し、同じ開発入力の指標を追って修正する。

| 順序 | 必要な証拠 | 現状 |
| --- | --- | --- |
| 1 | #520の対象/全体否定・未知ownerを保留し、完全な別属性/別製品イベントの肯定を維持。既知回帰・新規独立を確認 | 01dc78bは既知1140成功、新規v23はFAIL40/60。次は[新20保留の診断・修正](engine073-remaining-v23-repair.ja.md)。元v22/v23初回は不変 |
| 2 | #540共有停止・永続reserve/inflight・キャンセル、actual host callback/dependency/数値予算、通常経路trace | #535/#537/#541 merge、47 offline controls/限定独立6境界PASS。新scorer・live callback/dependency/数値予算・外部containment・featureが働く経路は残件 |
| 3 | 小規模offline開発確認、必要かつ別途開始条件を満たしたprovider確認 | 今回provider/model/Oracle追加0。旧v2/v3初回FAILを維持 |
| 4 | 完全M0評価器/host/model/予算/品質基準を固定→別作成者の新実資料24問→全文/gold M1固定→同hostで0.6.1/後継を比較 | 実24問0/24、M0/M1未完了。重大違反0・陽性退行0・provider別完全結果・有用性改善と品質下限が必須 |
| 5 | 宣言した全必須受入・CI後に専用Engine0.7.3 source release PR | 未公開。合成API F1=1だけで通常リリースしない |
| 6 | 専用CLI採用PR、Mac/Windows/Linux・配布・install/update/rollback | 採用1/6（開発比較・限界・NO-GO記録のみ） |

## 実回答の評価準備

[実回答の採点準備](engine073-realworld-metrics-preparation.ja.md)で主張grounding Precision、必要主張Recall/F1、引用、正しい/過剰保留、有用性、運用欠測、provider別会計を定義した。PREPARATION_ONLY、M0/M1未固定・実回答未採点。新scorer、実host callback/dependency closure・数値予算/headroom・金額自己上限、featureが働く通常CLI経路が残る。[現候補の同一host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md)は固定Node1経路で両Engineの同じgrounded回答・取得1・関連性API呼出し0、直接v17/v30対照ではcounter動作を確認した。全経路の不活性や今回修正の実回答効果を確定した結果ではない。#541共有runnerはmerge済みだが#540のlive準備はOPEN。#531新規実24問0/24・採用1/6、Groq追加起動禁止、provider/model/Oracle追加0を維持する。

mainの[#542品質ゲート](engine073-quality-gate-integration.ja.md)は、将来の各provider/trackのcase品質F1>=0.90・Precision>=0.95・Recall>=0.85を同時要求する追加下限。原子的主張F1とも合成API F1とも別の指標で、重大違反0・既存ゲートを緩和しない。完全scorer/traceと両単位の用途をM0で固定してから新規独立実24問を著作する。

## 保持する証拠と次の修正

[現在の開発検証](../evaluation/engine073-launch-development-v22-structure/README.md)・[不採用FP1を含む各試作](../evaluation/engine073-launch-development-v22-structure-prototypes/README.md)・[新規独立v23初回](../evaluation/engine073-launch-independent-v23/README.md)を保存。v23陽性20保留の原因は実経路で確認し、表面の分類だけで内部原因としない。[研究知見](engine-073-research-informed-repair.ja.md)に沿ってowner/型/引用/各節の証明を保ち、実装ごとに同じ観測済み集合のP/R/F1/FPを測る。既知1200と歴史597・元監査14を確認して新意味コードを固定した後にだけ新規独立を著作する。

Groqの既知禁止は時間経過・TPM間隔・小さいreadiness・モデル交換で解除しない。公開tag・歴史v16/v29・全初回・旧評価器は不変。運用欠測は成功の陰性や費用0にしない。#488/#489/#491は実測で必要な場合のみ。#465基盤・#506WinGetは別。0.8.0機能拡大・Oracleは含めない。
