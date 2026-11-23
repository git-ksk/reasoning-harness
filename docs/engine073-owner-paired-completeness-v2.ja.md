# 回答完全性改善候補の再測定

前回の実測 run 45057660915 は owner-managed diagnostic で、0.6.1 と旧候補の暫定F1上限はいずれも0.8148。今回の候補は PR #622 の exact commit 770ac6b59b74b0889a9bbe2cc97b1675dc928819。Baseline は引き続き 4abee90f0501f0d8fc09b453a0b45c942fbed69e (Engine 0.6.1)。既存の共有CLI、共通 #552/#613 patch、Mistral、seed/予算/8シャードを維持する。

新規24問（16陽性・8陰性）は前回とは異なる不変commit上の12資料を使用。問題・Gold・採点契約を観測前にMac上で凍結。既存v1の測定実績・入力secret・タグ・claim・原本には触らない。runnerに渡す新規専用input secretはGoldを含まない。

実行スクリプトは `evaluation/owner-paired-v2/measure.py`、builderは `evaluation/engine073-531-independent24-20261116/build_completeness_pair.py`、workflowは `.github/workflows/engine073-owner-paired-v2.yml`。専用prefix `eval-owner-paired-v2-`、input secret `ENGINE073_OWNER_PAIRED_V2_INPUT`。既存MISTRAL_API_KEY、create-only cohort claim namespace、課金ゲート、暗号化受取証明書、復号鍵は従来どおり。優先順位・安全性・検証ゲートを緩めない。初回リクエスト一致が成立しない場合は実測前に停止。失敗または欠落を再試行で補完しない。

同じチャットでアーム非表示採点を実施可能。採点固定後にunblindし、F1>=0.90、Precision>=0.95、Recall>=0.85、安全違反/無効引用ゼロ、ペア回帰なしを確認。未確定陰性の扱いは従来契約どおり。新版の性能改善は実測後に判断する。正式独立評価ではない。今回の変更は生成時の正確な根拠参照・調査対象選択の改善であり、任意の自然言語の新規推論を安全に承認する機能や最終表示の全面改修はまだ含まない。別コホート間の点数差は直接的なpaired improvementとはみなさず、同一新規問題上のBaseline/Candidate比較を主に判断する。

詳しい運用は既存 engine073-owner-paired-measurement-runbook.ja.md を参照する。
