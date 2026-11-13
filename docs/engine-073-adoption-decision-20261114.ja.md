# Engine 0.7.3 後継候補 — 2026-11-14 採用判定

**判定: 正式採用 NO_GO / 開発継続 OK。** Issue [#531](https://github.com/git-ksk/reasoning-harness/issues/531) の採用条件を変更せず、候補の未公開Coreを通常Reason CLIへ昇格・公開・配布しない。これは欠測を失敗採点に変えるものではなく、必要な受入証拠が揃っていないための **gate not met** である。

## 判定対象・証拠の範囲

- 開発候補Core source pin: `ab3c6322921c8c738ec35bd018345917b5afa538`。比較時にbaseline `4abee90f0501f0d8fc09b453a0b45c942fbed69e` と同じ #552 source-resolution patch を適用。両armのCLI source pinは `988b69f086ac4a209fe93eb10cf1fa6786977567`。
- Core package manifest は **0.7.2**。**0.7.3 は未公開**であり、`engine-v0.7.3` タグ／対応する公開リリースは確認できない。比較用CLI package は **0.5.4**、公開Reason CLIの最新版は **0.5.5**。比較バイナリと正式配布物を混同しない。
- [#574 既存実測6件の読み取り専用再集計](https://github.com/git-ksk/reasoning-harness/issues/574#issuecomment-6857762530): v5の独立レビュー付き既消費 **positive 6件×2arm**。両armとも task success 6/6、claim TP 9 / FP 0 / FN 0、citation 旧18 / 候補20、重大安全違反0。候補のモデル総トークン **48,233** / baseline **49,043**、候補は **810 tokens (1.652%) 少ない**。ケースごとの増減は混在し、同一回答は3ケース。**これは開発診断のみ**で、新しい4ケースの実測ではない。
- 候補Coreの既知否定・撤回・所有者・対象スコープ回帰: 一時的な独立QAコピー上で `cargo test --offline --locked -p reasoning-harness-core --test launch_context_markers_520_521` を実行して **80 PASS / 0 FAIL**。ビルド用source exportには当時の `evaluation/engine073-launch-independent-*` 等のfixtureが含まれていなかったため、**候補commitから評価fixtureだけを検証用コピーに復元**した。既存source/binary/first-scoreやタグは変更していない。過去の既知ケースの回帰PASSは**未見の独立24問の精度PASSではない**。
- プロジェクトの既存採点器 `evaluation/engine07-quality-gates-v1/metrics.py` のプロトコル条件（24全行、明確なpositiveが8例以上、negativeが1例以上、F1 / Precision / Recall、欠測なし）に照らすと、positiveだけの6観測は `missing_or_extra_observations` と `insufficient_clear_class_samples` で拒否される。既存実測のF1を更新・再採点・合格化していない。
- モデルなしの回帰: `engine07-quality-gates-v1` **10/10 PASS**、`engine073-source-gate-v1` **10/10 PASS**、`dev-lite` **21/21 PASS**。現Macに `MISTRAL_API_KEY` は存在せず、Mistral Free / 追加課金不可の現在条件と残枠は未確認。**本判定によるモデル生成0件、料金発生0件**。

## 正式採用ゲート

| 判定条件 | 今回の状態 | 正式GOへの必要条件 |
|---|---|---|
| 候補Core既知ケース回帰 | **PASS (80/80)** | 未修正の重大欠陥0を維持 |
| 既知positiveの通常CLI品質 | **限定PASS (両arm 6/6)** | 既知ケースは診断に留め、混合ケースの回答と保留を測る |
| 新規独立24問の全件受入 | **未実施** | 候補固定後に独立に作成・ラベル固定した24問を完走 |
| 通常CLIの精度 | **全量F1未定義** | 必須provider／trackごと F1≥0.90、Precision≥0.95、Recall≥0.85。TP/FP/TN/FN、欠測0 |
| 安全性・引用 | **否定例の実モデル評価不足** | 重大違反0、引用対応・coverage・literal fidelity 1.0、保留判定とソース帰属の独立評価 |
| baseline非退行 | **6件のpositiveのみ比較** | 同一セットの全caseで有用回答と過剰保留を比較し、退行なし |
| 効率 | **診断差 -1.652%** | 24件の完全pairで平均logical calls差≤1、tokens比≤1.25、p95時間比≤1.5 |
| 0.7.3版確定・CLI統合・配布 | **未実施** | Core版固定、独立release/installation/rollback受入、各OSのCI |

必要な証拠が1つでも未定義のため、`quality_go=false`、`adoption_go=false`、公開/配布 **NO_GO** とする。正式な新しいF1を0・1・既存部分集合の100%として報告しない。

## 次の最短経路（独立署名を日常検証の必須条件に戻さない）

1. Mistralの現行無料・追加課金不可の状態、利用枠、実行資格情報を確認。その後Development Liteの**4問（回答＋未知/秘匿）×2arm**の実測を少量で実施し、private `review.py` で全文・出典と適切な保留を比較する。Development LiteのPASSは正式採用とは別。
2. 不合格の具体例だけ修正。Core候補と評価プロトコルを固定し、**未見の独立24問**（positive／negativeを含む）を別に用意する。レビューはソース内容を人が確認できる方法で十分であり、開発時のEd25519鍵・使い切りタグを必須にしない。
3. 必須providerの通常CLIで全件同一条件を比較し、既存の閾値・安全性・引用・非退行・効率を全て判定する。条件を満たしたときのみCore 0.7.3版確定とReason CLIへの採用PR・インストーラ／ロールバック受入へ進む。

過去v1–v6の原本、レビュー、初回採点、tag、失敗履歴を変更しない。No-Goは開発停止・失敗の取り繕いではなく、現時点での証拠に基づく採用判定である。
