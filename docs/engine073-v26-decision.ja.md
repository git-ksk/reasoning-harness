# Engine 0.7.3 新規独立v26の判定

2026-10-18 JST。未公開候補9082365 / source SHA2564f82b2e41614394f1bd528f01c817f0e962f1c546d7212f75f15c2f9c5787999。package version0.7.2、公開CLI0.5.5 / Engine0.6.1は変更していない。

候補 `9082365` の新規独立 v26 初回は **FAIL / NO_GO**。両API別に TP22 / FP0 / TN30 / FN8、Precision 1.000 / Recall 0.733333 / F1 **0.846154**。安全性誤採用0・運用失敗0・欠測0だが、固定した F1 0.90・Recall 0.85 の下限未達。修正に使った v25 開発 F1=1.000 と区別し、異なる独立集合のF1差を改善率にしない。初回 Engine 1 process・60 inputs / scorer 1 / helper 1、retry/rescore/relabel0。M0親レビュー59純fixture、M0/M1事前固定と13 provenance checks、原99ファイル・旧730参照・全829 corpus参照を親でも照合した。原v25 FAILと全過去初回は不変。8件の実経路は7件がtyped_reference_not_excluded、057がunresolved_adverse_scope。現在の既知対照は1392/1400、元監査14・歴史597・全体1594成功は候補908の別検証。次は属性・所有者・局所参照の共通処理を修正し、既知1400件を全成功させてから新しい候補/評価器を著作前に固定する。新規v27は未著作。実24問0/24・実回答F1未測定、Groq停止、sourceマージ/Engine公開/CLI採用NO-GO。[独立判定](engine073-v26-decision.ja.md)、[原初回](../evaluation/engine073-launch-independent-v26/README-results.md)、[実経路](../evaluation/engine073-launch-trace-development-v10/README.md)。

## 固定方針による初回判定

| 軸 | TP | FP | TN | FN | Precision | Recall | F1 | 判定 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| v17 requested_relation |22|0|30|8|1.000|.733333|.846154|NO_GO|
| v30 Relevant |22|0|30|8|1.000|.733333|.846154|NO_GO|

[#547の方針](../evaluation/engine073-source-gate-policy-v1.json)とhelperは変更していない。各APIのF1>=.90 / P>=.95 / R>=.85 と FP0、既知回帰100%、完全な初回/事前固定を累積条件とする。joint52/60は診断であり、60/60を後付けで必須にして落としたのではない。今回の下限未達はF1とRecall。driver exit1は意味的なNO_GOで、recorder/scorer exit0・stderr空・運用エラー0。初回結果を再実行して合格にしない。

## 残る8件の実経路

001/027/033/041/043/047/055はtyped_reference_not_excluded、057はunresolved_adverse_scopeが最初の保留。別コピーのcfg(test)観測は元初回effective tuple全8件と一致した。001はprefix付きrole宣言が未収集、027は証明済みforeign role参照が除外できず、033/055は印刷属性の値がLaunch型、041はforeign claimがLaunch型、043はmaterial属性が未解決、047は直前のforeign productへの参照、057は直前のforeign claimへの参照を結び付けられない。raw/effective enumだけから内部risk喪失を推定したものではない。修正は各語の暗記ではなく、完全な宣言・所有者・同じ局所参照の共通処理へ限定する。明示的な対象/全体否定・未知/競合owner・本当の欠落・raw riskの境界を維持する。

現在の既知対照1340＋今回60＝1400件中1392件成功。新規v27を消費する前に8保留と既知全回帰を直し、元14件・歴史API byte互換・全体チェックを新候補で再確認する。元v26 FAILと入力/ラベル/方針/採点器は永久にそのまま保持し、開発比較は別prefixへ保存する。

## 保存と次工程

M0 ad980eba8479eec16e78f5ef2c2d108ac949ed1cf4b6fe2277df4dc0e3742510、M1 7627332228e422c58b64890434f681066469d77ab3326a8037e9b8379508c366、最終anchor8214db3607e32d707da797fb90bb4af77b1101db976b6f0d0a60f96d49cdb921。親レビュー59純fixture、first-onlyの実行順/hash/readonly13 checksとF1算術を照合。[親照合](../evaluation/engine073-launch-v26-parent-verification/final-copy-verification.json)、[算術報告](../evaluation/engine073-launch-first-metrics-v26-report.json)。利用上限での中断は著作startのみ・意味実行前であり、同じstartから再開した。原初回のretryではない。

908候補の全体1594PASS/0FAIL/既存ignore8・strict Clippy/fmt・fixtures/archiveと、証拠head1d7019aのCI19/19成功は別記録。新候補の合格へ流用しない。[現908の静的CLI参照準備v5](../evaluation/engine07-cli-path-preparation-v5/README.md)は42 core/62 host filesで対象APIの通常CLI literal参照を確認できなかった。完全call graphや全経路不活性の証明ではない。実資料M0前には対象機能が動く通常経路、取得/admission/claim/citation/display trace、完全意味scorer、actual callback/dependency/数値予算が必要。実24問は未著作・0/24、Groq停止。修正→新規独立受入→実資料評価→CLI採用の順序を維持する。

## 現在の開発修正 — 2026-10-19 JST

候補 `26c98ad` の同じ観測済みv26開発60件は両API別に TP30 / FP0 / TN30 / FN0、Precision/Recall/F1=1.000、両軸60/60。元初回 .846154→中間 .928571→1.000 は同じ集合の開発比較で、独立v26初回FAIL/NO_GOと原ラベル/scorerは不変。属性・所有者・局所参照の共通処理を修正し、既知1400/1400・元監査14成功、歴史597全出力byte一致、全体1599成功/失敗0/既存ignore8・strict workspace/all-targets Clippy/fmt確認。初回全体のlocalhost模擬通信5拒否は原ログを残し、同一sourceで実行条件修正後に全成功。次の独立v27は候補/評価器を著作前に固定し、親が純fixtureと実コードをレビューしてから初回1回だけ観測する。まだ未著作・未実行。元v26の99ファイル＋旧730＝829参照を保持。実24問0/24・実回答F1未測定・Groq停止・provider/model/Oracle追加0、sourceマージ/Engine公開/CLI採用NO-GO。[判定](engine073-v26-decision.ja.md)、[固定修正の検証](../evaluation/engine073-launch-development-v26-local-references/README.md)。

[試作のRED/中間/最終](../evaluation/engine073-launch-development-v26-prototypes/README.md)も保存。private typed roleへ完全なforeign ownerを保持し、owned-by参照が同じownerに一致することを要求する。局所的な完全event/claim以外からは所有者を作らず、後続の確認は前の所有者を置き換えない。これは新規v27の合格ではない。
