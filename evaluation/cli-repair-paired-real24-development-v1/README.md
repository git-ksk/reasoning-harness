# 実API比較の実行前固定 — 2026-10-26 JST

既知の実資料24問を用いる新しい開発測定。新規独立の精度受入ではない。baseline Engine0.6.1 (`4abee90f`) と未公開0.7.3修正候補 (`ab3c6322`、package0.7.2) を、同じCLI host `73bbd181` と共通パッチ・同じモデル条件で比較する。

- 評価器source: `441ac12b36470896b1ae2a9d6e7366d6a2405a0c`、runtime artifact `12604860150`。
- `development-review/` は別担当 `/root/real24_operational_review` が発行した真正な最終運用レビュー。元の判定・manifestのbytesを保持する。モデルの精度合格や人間の承認証明ではない。
- `provider-readiness/` はGitHub run `40208555750` で実行したMistral・Google各1回のmetadata GET。両方HTTP200・利用モデル一致、モデル生成0。全量quota headroomはUNKNOWNのまま。
- `receipt.json` は上記証拠、77 runtime files、10 binaries、入力M1、採点driver、モデル/数値上限、reviewerと一度限りの実行tagを固定する。
- `runtime-proof.json` は独立レビューより前のオフライン構築証拠。元の非admission状態を変更しない。実行時には新receiptと実2報告を別途検証する。

各providerで通常48jobsと共通helper24jobs、計72jobs/96観測を予定。Groqは保留。費用予約上限はMistral USD2.5、Google USD7.5の既存値を維持し、quota・不明usage・異常終了で停止、無条件の再試行はしない。

実出力を別担当が全文・引用・範囲ごとに新規注釈し、凍結したdriverでF1/Precision/Recall、引用、有用性、安全性、効率を初回のみ採点する。元のraw/gold/注釈/スコアは保持。Engine公開・CLI採用・新規独立評価は別ゲート。

## 初回の実行結果 — run40209467266

Mistral/Googleとも最初の通常baseline1件を実行した後、実行器の会計bridgeが現行v6出力を認識せず `unknown_failure_class` で停止した。CLI自体は両方returncode0。MistralのCLI記録は10 provider attempts/11074 tokens、Googleは5 attempts/3555 tokens。親ledgerのknown subtotal0は未知扱いによる値で、実使用量0を意味しない。両側の保守予約28096 tokensは維持し、返金扱いにしない。

各72jobの完走は0件、candidate/helperは未実行。raw/state/receipt/初回tagを保持し、新しいF1は算出しない。保存済みの実出力だけで会計と引用捕捉を診断し、後継v2に現行契約adapterを追加する。元runを再開・成功へ変更せず、新しいsource/image/独立判定/metadata/receiptを固定してから再測定する。
