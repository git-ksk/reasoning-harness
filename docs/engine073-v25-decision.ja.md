# v25独立評価の判定と次の修正

候補9505300/source SHA256 9d8d3fa48a67f197d32282314813d992a374b9f54a6b079c39498b9a67f9207f。新規独立の初回はFAIL/NO_GO。

| 軸 | TP | FP | TN | FN | Precision | Recall | F1 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| v17 requested_relation |5|0|30|25|1.000|0.167|0.286|
| v30 Relevant |5|0|30|25|1.000|0.167|0.286|

合成APIの指標。否定誤採用0でも、陽性25件を取りこぼしてRecall/F1下限に届かなかった。jointexact35/60は診断値で、全60成功を後付けで要求した不合格ではない。#547の両軸F1>=.90・Precision>=.95・Recall>=.85、安全性FP0、既知回帰100%・運用完全性を著作前に固定して判定した。

評価器M0 b172872d...を親が48純fixture結果とhashでレビュー後、新規30対60件を著作し、入力/ラベルM1を固定して観測。Engine60件一process、scorer1/helper1、再実行/再採点/再ラベル0。運用失敗/欠測0、候補/事前評価器/原結果不変。725source files＋別prep5＝730参照を照合。最終anchor f3ef8db2393aa6c458981e92099d23792856dd1e1431c5876bb51760aefab1aa。[初回原本](../evaluation/engine073-launch-independent-v25/README-results.md)と[親照合](../evaluation/engine073-launch-v25-parent-verification/review.json)。

失敗は意味上の非launch属性12件、複数の安全な文脈7件、別製品イベント6件。保存されたrawは全てexact_target/requested_relation/none。effectiveはexact_target/unresolved/noneが24件、context_gapが1件。これらのenumだけから内部risk喪失や最初の保留ruleを推定しない。

次の順序:

1. 今回の25件を開発入力として、別コピーのテスト専用traceで最初の実保留経路を確認する。元の初回を再実行・変更しない。
2. 役割宣言、訂正が指す対象、別製品owner、複数役割の参照を共通構造として修正する。対象/全体否定・不明/混在/競合owner・実欠落・raw riskの対照を維持する。context_gapの1件も実際の判定経路から調べる。
3. 同じ今回60件で両軸TP/FP/TN/FN・P/R/F1を測る。回帰対象は既知1264＋今回60＝1324、元監査14と歴史597全出力互換は別途維持。全体検証・strict Clippy/fmt、新commitと実Cargo artifactを固定する。
4. 新たな既知修正が完了するまで次の独立v26を著作しない。次世代でも評価器と#547方針を著作前に固定し、初回1回の原結果を保存する。
5. #520受入後に#540実host/完全scorer/数値予算/機能が働く通常CLI経路、#531新規実資料24問へ進む。実回答の主張F1とcase品質F1は合成API F1と別。

v24の同じ開発60件でF1.723404→1.000の修正効果は確認済みだが、新規v25での一般化は未達。異なる集合のF1差を改善率や退行率にしない。既知1264/全体1582成功・コード950と文書834のCI各19成功を保持する。

sourceマージ/Engine0.7.3公開/CLI採用NO-GO。実24問0/24、実資料M0/M1未完了、実回答F1未測定。公開Engine0.7.2/CLI0.5.5-Engine0.6.1は不変。Groq停止、provider/model/Oracle追加0。
