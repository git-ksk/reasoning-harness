# 実資料24問の実API比較v2 — 2026-10-26

**新規実API比較と独立初回採点が完了（通常系列に欠測）。** [実run40248051474](https://github.com/git-ksk/reasoning-harness/actions/runs/40248051474)でMistral・Googleを測定している。Groqは保留。既知24問の開発測定であり、新規独立の精度受入・Engine公開・CLI採用は別判定。

## 固定した比較

| 項目 | 固定値 |
| --- | --- |
| baseline | Engine0.6.1、`4abee90f0501f0d8fc09b453a0b45c942fbed69e` |
| candidate | 未公開0.7.3修正候補、`ab3c6322921c8c738ec35bd018345917b5afa538`、package0.7.2 |
| 共通CLI | host `73bbd18173b4dfcd6664a26241de0385f2eb2e3e`、package0.5.4、両coreに同じresolution patch |
| 評価器 | source `4dbf8ba0409002eb45bb6b854a28a78f002fbbda` |
| 実行tag | `cli-repair-paired-real24-development-v2`、commit `76dde2887d5110d8606cfd449ededabe6258ed36` |
| runtime artifact | `12605803481`、build run40211547873 |
| 入力M1 | `597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3` |
| 実行receipt | SHA256 `2e964414858613223af8c26c3ddd28901b197d004b594b26dd73a04eb421cd1c` |
| モデル | Mistral `ministral-8b-2512`、Google `gemini-3.5-flash-lite` |

公開済みのCLI全体同士の比較ではなく、共通CLIと共通patchの下でのEngine対照である。通常CLIとopt-in補助APIを分け、各provider72jobs/96観測を予定する。主判定はcase/claimのF1≥0.90・Precision≥0.95・Recall≥0.85、安全性0、引用、有用性、退行、効率の固定条件。

## 初回v1停止と修正

run40209467266は両社の通常baseline1件で停止した。CLIは両方returncode0・現行v6出力で、CLI記録はMistral10attempts/11074tokens、Google5attempts/3555tokens。継承会計bridgeはv4のみを受け付け、UNKNOWN/usageNoneに落として後続を止めていた。親ledgerのknown subtotal0は未知扱いで、実使用量0ではない。各28096tokensの保守予約は維持する。

v2はstrict v5/v6会計adapterを実scheduler dispatchへ接続した。別担当が保存済みの両実出力を実AuditedRunnerへ流し、既知会計から次行spawnまで、quota・未知usage・異常終了・未知形式・上限超過の停止まで確認した。実compiled CLIの既存17経路も会計まで確認する。元のv1 raw/state/receipt/tagと初回結果を保持する。

## 実行前確認

別担当 `/root/real24_operational_review` が、新imageの79files/10binaries・実Linux80controls・provider18・CLI17・containment7・registryと、保存full-v6の実会計・次行継続を確認して正式判定を発行した。[判定とmanifest](../evaluation/cli-repair-paired-real24-development-v2/development-review/review.json)のbytesを固定している。

run40213628715の実metadata GETは両社HTTP200・モデル利用可能・各GET1/生成0。両報告はsource/proof/独立判定にbindし、24時間の有効期限とともにreceiptへ固定。全量quota headroomはUNKNOWN、完走保証はない。評価側自己上限Mistral USD2.5/Google USD7.5、事前予約・unknown usage/異常停止・モデル/tierの固定を維持する。

## 実行・観測の完了範囲

Googleは72/72jobsを実行完了し、320attempts/264844tokens、unknown usage0、latchなし。captureは96観測中94completeで、rw-20の通常両arm2行は`ambiguous_acquired_evidence`のためincomplete。モデル実行の完了と、採点できる観測の完備は別である。この2行は経路未検証として保持し、真偽・安全性の成功や失敗を推測せず、通常のfullF1は未確定のままとする。原本1257filesのSHA256 inventoryを保存した。Mistralも72/72jobsを実行完了し、384attempts/374908tokens、unknown usage0、latchなし。captureは96観測中76completeで、same-ID再取得14行・空の調査イベント6行がincomplete。原本1241filesのSHA256 inventoryも保存した。

[実行・capture記録](../evaluation/cli-repair-paired-real24-development-v2/execution-observation-summary.json)に、実行完了と採点前の欠測を分けて記録した。通常baseline/candidateのcapture完了はGoogle23/24・23/24、Mistral15/24・13/24。helperは両社・両armとも24/24である。

[実行後の構造診断と将来修正](../evaluation/cli-repair-paired-real24-development-v2/postrun-capture-diagnosis.json)は、異なるrequestの完全一致record再取得を許し、resolver未起動かつ承認IDs/事実keysが空のtraceを無加点で扱う。元資料一致、各response内重複・内容衝突・target/capability・時系列・実承認経路を検査する。保存96通常出力は修正案で構造確認でき、元から正常だった74行の取得結果は不変。85controls成功は計測器の修正検証であり、精度改善を意味しない。今回の凍結image/観測を変更せず、将来の実行には新runtimeレビュー・receiptが必要。[別担当の限定修正レビュー](../evaluation/cli-repair-paired-real24-development-v2/postrun-review/bounded-source-review.json)はblockingなし。通信禁止・read-only/nonrootの旧image＋2file overlayで85controls、保存96行・元complete74行の同一性、拒否境界16件を独立確認した。これは新runtimeの正式承認ではない。

## 独立初回採点

別担当 `/root/paired_evaluation_independent` が12原資料全文と新しい192観測を読み、全文UTF8区間・claim・引用edge・元資料/gold/観測hashを検査した。170行complete、経路未確認22行incompleteを保持し、[分析freeze](../evaluation/cli-repair-paired-real24-development-v2/first-score/analysis-freeze.json)を発行した。完全一致の共有は新v2 helper packet間だけで、各行を固有hashへ束縛した。完全blind・人間の署名・新規独立問題とは主張しない。[方法と限界](../evaluation/cli-repair-paired-real24-development-v2/first-score/semantic-review-method.json)。

元の凍結image・driverで一度だけ採点し、returncode0。実行後のcapture修正を適用せず、旧スコアや元観測も変更していない。[初回report](../evaluation/cli-repair-paired-real24-development-v2/first-score/report.json) SHA256 `be4f58e4436fcd282eb7cfd81c8b6c030cd769a9203fe7d4123c63604c8d797e`、分析freeze SHA256 `45ad679943fcc056052503d8a9583f17c6c8c524abb27ce8b7ec3712f97ddccc`。

| Provider / 系列 | 採点可能 baseline / candidate | case F1 baseline → candidate | candidate Precision / Recall | 全24問F1 |
| --- | --- | --- | --- | --- |
| Google 通常CLI | 23/24・23/24 | 0.933333 → 0.933333（参考） | 1.000 / 0.875（参考） | 欠測のためnull |
| Mistral 通常CLI | 15/24・13/24 | 0.428571 → 0.461538（参考・異なる集合） | 1.000 / 0.300（参考） | 欠測のためnull |
| Google helper opt-in | 24/24・24/24 | 0.222222 → 0.400000 | 1.000 / 0.250 | 0.400000 |
| Mistral helper opt-in | 24/24・24/24 | 0.222222 → 0.400000 | 1.000 / 0.250 | 0.400000 |

通常系列の参考F1は完全観測だけの記述値。Mistralは両armの欠測集合が違うため、0.428571→0.461538を改善量と見なせない。Googleは同じ23問で同点で、Engine差の実測改善はない。以前のGoogle通常0.583333との比較は共通CLIや実行条件も異なり、Engineだけの改善値にできない。

[採点後の独立解釈](../evaluation/cli-repair-paired-real24-development-v2/postrun-review/semantic-interpretation.json)でも結果とfreeze不変を確認した。helper candidateは両社ともTP2/FP0/FN6/TN16。TN16は誤って受理しなかったという判定で、有用な拒否説明が16件あったことを意味しない。baselineからの有用化はrw-09（CMake break）1件。FN6は、回答可能なのに保留したrw-01/14/21の3件と、同一資料・同一由来である説明を表示しないrw-05/08/15の3件。case F1は回答の成立・必要説明を含む。claim micro F1が1.000でも、正しい引用を表示したまま保留したり必要な注意書きを欠いたりすれば、case F1は合格しない。

通常claim micro F1はGoogle0.941176、Mistral candidate0.363636（いずれも完全観測部分のみ）。採点可能な引用edgeはすべて対応した。reportのhard violation件数0を、経路未確認22行・未実行Groqを含む全体の安全証明へ拡張しない。

Mistralのreport上のpaired utility `regressions=2/net_gain=0`は、片arm欠測を含む。rw-04/20はbaseline有用・candidate capture欠測であり、意味品質の退行2件ではない。両方completeの11ケースではrw-21が1件有用化し、観測された有用回答退行はなかった。ただし単回の選ばれた11件なので、core起因の全体改善とは断定しない。

効率は完全pairがGoogle23/Mistral11にとどまり、全24pair条件を満たさない。参考のcandidate/baseline tokens比は1.000/1.174、p95時間比は0.998/1.111、logical calls平均差は0/+0.364。欠測のため効率ゲートも未合格。

## 判定と次の作業

**CLI採用・Engine公開はNO_GO。** helper case F1/Recallは基準0.90/0.85未満で、通常全件F1と全pair効率は未確定。既知24問・12資料・6製品の開発診断であり、新規独立accuracyは0。Groqは未実行のままで、0点・成功・TNにはしない。

1. 今回の欠測原因であるcapture修正（限定独立PASS）をCI後に統合し、測定基盤の不具合を閉じる。現実験の凍結raw/観測/注釈/初回結果は保持する。
2. 通常CLI全件を比較できる次回計画と、新規独立資料の受入計画を用意する。実APIの再実行は新runtimeレビュー・receipt・provider条件の確認後に行う。
3. 実測した保留・同一由来の説明不足だけを改善候補にする。#488/#491はこの証拠の範囲で扱い、#489を含め未確認の新機能を一括実装しない。
4. 再評価で安全性・有用性・F1/P/R・効率が揃ってから、Engine版確定・CLI統合・各OS/配布/更新検証へ進む。

#531・#538・#540で追跡する。元のGoogle helper0.400000などの結果もそのまま保持し、今回同じ値が出たことを追加改善と呼ばない。
