# v25独立評価の判定と次の修正

候補9505300/source SHA256 9d8d3fa48a67f197d32282314813d992a374b9f54a6b079c39498b9a67f9207f。新規独立の初回はFAIL/NO_GO。

| 軸 | TP | FP | TN | FN | Precision | Recall | F1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| v17 requested_relation |5|0|30|25|1.000|0.167|0.286|
| v30 Relevant |5|0|30|25|1.000|0.167|0.286|

合成APIの指標。否定誤採用0でも、陽性25件を取りこぼしてRecall/F1下限に届かなかった。jointexact35/60は診断値で、全60成功を後付けで要求した不合格ではない。#547の両軸F1>=.90・Precision>=.95・Recall>=.85、安全性FP0、既知回帰100%・運用完全性を著作前に固定して判定した。

評価器M0 b172872d...を親が48純fixture結果とhashでレビュー後、新規30対60件を著作し、入力/ラベルM1を固定して観測。Engine60件一process、scorer1/helper1、再実行/再採点/再ラベル0。運用失敗/欠測0、候補/事前評価器/原結果不変。725source files＋別prep5＝730参照を照合。最終anchor f3ef8db2393aa6c458981e92099d23792856dd1e1431c5876bb51760aefab1aa。[初回原本](../evaluation/engine073-launch-independent-v25/README-results.md)と[親照合](../evaluation/engine073-launch-v25-parent-verification/review.json)。

失敗は意味上の非launch属性12件、複数の安全な文脈7件、別製品イベント6件。保存されたrawは全てexact_target/requested_relation/none。effectiveはexact_target/unresolved/noneが24件、context_gapが1件。これらのenumだけから内部risk喪失や最初の保留ruleを推定しない。

初回FAIL後の作業順序（今回の実施状況は下記）:

1. 今回の25件の最初の実保留経路を別コピーのテスト専用traceで確認済み。現候補の残る7件も実経路を保存した。元の初回を再実行・変更しない。
2. 役割宣言、訂正が指す対象、別製品ownerの共通処理を修正済み。残る7件の参照・引用・riskを共通構造として修正する。対象/全体否定・不明/混在/競合owner・実欠落・raw riskの対照を維持する。context_gapの1件も実際の判定経路から調べる。
3. 同じ今回60件で両軸TP/FP/TN/FN・P/R/F1を測定済み（下記）。今後の修正でも同条件で測る。回帰対象は既知1264＋今回60＝1324、元監査14と歴史597全出力互換は別途維持。全体検証・strict Clippy/fmt、新commitと実Cargo artifactを固定する。
4. 新たな既知修正が完了するまで次の独立v26を著作しない。次世代でも評価器と#547方針を著作前に固定し、初回1回の原結果を保存する。
5. #520受入後に#540実host/完全scorer/数値予算/機能が働く通常CLI経路、#531新規実資料24問へ進む。実回答の主張F1とcase品質F1は合成API F1と別。

v24の同じ開発60件でF1.723404→1.000の修正効果は確認済みだが、新規v25での一般化は未達。異なる集合のF1差を改善率や退行率にしない。既知1264/全体1582成功・コード950と文書834のCI各19成功を保持する。

sourceマージ/Engine0.7.3公開/CLI採用NO-GO。実24問0/24、実資料M0/M1未完了、実回答F1未測定。公開Engine0.7.2/CLI0.5.5-Engine0.6.1は不変。Groq停止、provider/model/Oracle追加0。

## 前候補a3d772dの開発修正（2026-10-18）

候補`a3d772d`でrole宣言・訂正先・同じroleのownerを共通処理し、同じ観測済みv25の60件を開発比較した。v17はTP24/FP0/TN30/FN6・P1.000/R.800000/F1 .888889、v30はTP23/FP0/TN30/FN7・P1.000/R.766667/F1 .867925、両軸一致53/60。元初回の両API F1 .285714から改善したが、まだ基準未達・既知7保留が残る。以前の既知1264・元監査14成功、今回を含む既知1317/1324、歴史597全出力byte一致、全体1588成功/失敗0/既存ignore8・strict Clippy/fmt確認。Codex/OpenCodeの一次資料を参考に、ID/原文範囲付きtrace・修正/検証ループ・不採用案の保存を採用した。元の独立v25初回FAIL/NO_GO、全初回・ラベル・scorer・prepは不変。次は残る7件の実経路を修正し、新規v26著作はそれまで行わない。実24問0/24・実回答F1未測定、Groq停止・provider/model/Oracle追加0、sourceマージ/公開/CLI採用NO-GO。[判定と残件](engine073-v25-decision.ja.md)、[固定候補の検証](../evaluation/engine073-launch-development-v25-role-frames/README.md)、[参考資料と応用](engine073-codex-opencode-notes.ja.md)。

[初回25件の実経路](../evaluation/engine073-launch-trace-development-v8/README.md)は全件の最初のgoverning-context保留がtyped_reference_not_excluded。修正後の[残る7件の現候補trace](../evaluation/engine073-launch-trace-development-v9/README.md)は、031/035/037/043/045/051が同じrule、053はgoverning blockなしでRequestedRelation/ContextGap・v30保留。引用title、所有格の属性参照、同格role、印刷illustration、quotedイベントとowner返信、隣接Denial target、canonical Clipのrisk判定を共通構造として修正する。元rawのrisk消失と推定しない。

[不採用案と退行](../evaluation/engine073-launch-development-v25-role-prototypes/README.md)も保存。誤採用2件の初案、旧ケース退行、誤った新規parent対照の訂正を成功扱いにしない。元独立ラベルは変更なし。次の独立評価は新規著作前の評価器/方針固定と初回保存を維持し、既知1324件を回帰対象にする。現在は1317/1324なので着手しない。

## 現候補9082365の開発検証

候補`9082365`の同じ観測済みv25開発60件は、両API別にTP30/FP0/TN30/FN0・Precision/Recall/F1=1.000、両軸60/60となった。前候補a3d772dのF1 .888889/.867925から残る7件を解消。以前の既知1264＋今回60＋親の追加16＝1340成功、元監査14は別途成功、歴史597全出力byte一致、全体1594成功/失敗0/既存ignore8・strict Clippy/fmt確認。属性の外側の型、引用された発話、同じroleの参照・owner、独立した訂正節、明確な非実在、opaqueな正式名称を共通処理した。元の独立v25初回FAIL/NO_GO・F1 .285714と全初回/ラベル/scorer/prepは不変。これは開発比較であり、新規独立受入ではない。次は候補と評価器/方針を著作前に固定し、評価器を親レビューしてから新規v26を初回1回で観測する。実24問0/24・実回答F1未測定、Groq停止・provider/model/Oracle追加0、sourceマージ/公開/CLI採用NO-GO。[判定](engine073-v25-decision.ja.md)、[固定候補の検証](../evaluation/engine073-launch-development-v25-reference-frames/README.md)。

[不採用案・追加対照](../evaluation/engine073-launch-development-v25-reference-prototypes/README.md)も保存。高F1でも明確な非実在を誤採用したstage04は不採用。過去の安全な例を保留した中間案も受入扱いにしない。印刷タイトルに「否定・欠落」と書かれた16対照を元ラベルのまま8/16→16/16へ修正し、引用された作者の返信は実際の訂正節として解析する。新たな独立v26は1340件の回帰・元14件・全体検証の成功後、M0を親レビューしてから著作/初回観測する。旧FAILを後付けで合格にしない。
