# 実API比較の実行前固定 — 2026-10-26 JST

既知の実資料24問を用いる新しい開発測定。新規独立の精度受入ではない。baseline Engine0.6.1 (`4abee90f`) と未公開0.7.3修正候補 (`ab3c6322`、package0.7.2) を、同じCLI host `73bbd181` と共通パッチ・同じモデル条件で比較する。

- 評価器source: `441ac12b36470896b1ae2a9d6e7366d6a2405a0c`、runtime artifact `12604860150`。
- `development-review/` は別担当 `/root/real24_operational_review` が発行した真正な最終運用レビュー。元の判定・manifestのbytesを保持する。モデルの精度合格や人間の承認証明ではない。
- `provider-readiness/` はGitHub run `40208555750` で実行したMistral・Google各1回のmetadata GET。両方HTTP200・利用モデル一致、モデル生成0。全量quota headroomはUNKNOWNのまま。
- `receipt.json` は上記証拠、77 runtime files、10 binaries、入力M1、採点driver、モデル/数値上限、reviewerと一度限りの実行tagを固定する。
- `runtime-proof.json` は独立レビューより前のオフライン構築証拠。元の非admission状態を変更しない。実行時には新receiptと実2報告を別途検証する。

各providerで通常48jobsと共通helper24jobs、計72jobs/96観測を予定。Groqは保留。費用予約上限はMistral USD2.5、Google USD7.5の既存値を維持し、quota・不明usage・異常終了で停止、無条件の再試行はしない。

実出力を別担当が全文・引用・範囲ごとに新規注釈し、凍結したdriverでF1/Precision/Recall、引用、有用性、安全性、効率を初回のみ採点する。元のraw/gold/注釈/スコアは保持。Engine公開・CLI採用・新規独立評価は別ゲート。
