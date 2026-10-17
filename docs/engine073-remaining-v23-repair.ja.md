# 次のv23修正計画

独立v23は01dc78b固定の初回FAIL40/60、安全性違反0・陽性20保留。同じ入力を今後は開発用として扱い、初回を変更しない。

1. 現コードの別コピーに一時的なカウンタを置き、20件の最初の実保留経路を確認。述語・引用・role/owner・各節・riskを別に記録し、外見だけで原因を確定しない。
2. 共通原因を構造の証明で修正し、対象/全体/未知owner/実欠落の対照を追加。語句だけの安全認定・role間のowner貸与はしない。
3. 同じ60件のP/R/F1/TP/FP/TN/FNとjoint成功を追う。不採用候補も保存。FP0必須。既知1200・元監査14・歴史597・全体locked/offline・Clippy/fmtを維持。
4. 既知不具合が残る間は新規v24を消費しない。修正後ソース・artifactを固定してから唯一の許可済み独立担当が新規著作/初回評価する。
5. #542のcase品質ゲートと主張F1を区別し、完全実資料scorer/host/数値予算/feature-active CLI経路をM0へ整える。新実24問著作・provider/model/Oracle追加0、Groq停止・NO-GOを維持。

[現候補の実経路診断](../evaluation/engine073-launch-trace-development-v6/README.md)では20件とも型付き参照での保留が最初の経路だった。全20effective tupleが元初回と一致。属性/引用/type/owner宣言と報告節を調べ、後続の保留やClipのriskも別に確認する。
