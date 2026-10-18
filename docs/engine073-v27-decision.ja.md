# Engine0.7.3 独立v27初回の判定

2026-10-19 JST。Source26c98adbec19f9e8ea0d46f260f080d4319a8e60 / SHA2564daf64b918532a9004dea9807b8001ec743244b20b5cd1892c7f3a7ce316d1ef. Package0.7.2 unchanged.

候補 `26c98ad` の新規独立v27初回は **FAIL / NO_GO**。両API別 TP24 / FP0 / TN30 / FN6、Precision1.000 / Recall0.800 / F1 **0.888889**。安全性誤採用・運用失敗・欠測は各0だが、固定F1>=.90・Recall>=.85未達。joint54/60は診断。新99ファイル＋旧829＝928参照をbyte一致保存し、再実行/再採点/再ラベル0。原v26 F1 .846154/NO_GOも不変。候補の既存1400対照・元14・歴史597・全体1599PASS/0FAIL/既存ignore8、同ソース証拠head826c98dのCI19/19成功は別に保持。v27の6保留を含めた既知集合は1454/1460。5件typed_reference_not_excluded、025はunresolved_adverse_scopeと実経路で確認した。次は完全な印字属性・同じ局所role/owner・括弧内の限定訂正を共通処理し、未知/競合owner・対象/全体否定を維持して1460回帰を確認する。実資料向けには主張F1算術＋表示hash/UTF-8 span照合部品32検証を追加したが、完全意味scorer/M0ではない。実24問0/24・実回答F1未測定、Groq停止、sourceマージ/Engine公開/CLI採用NO-GO。[判定](engine073-v27-decision.ja.md)、[原初回](../evaluation/engine073-launch-independent-v27/README-results.md)、[主張採点準備](engine073-answer-metrics-development.ja.md)。

M0 aaa4b0d2494c3a89d87b3c0714847ee8d468f84295e1f868e7518398e41635bb; M1 fb8a350de68ee6dfc61e4d440411a96eb217b54a4fdec7bfe5a6ebf02709d82c; final anchor abaae2385e6c17783cc7093c10e15424b6e4ce01080e8fcdea5eab3a9e5bb439. Parent59 pure fixtures,13 actual first provenance checks,99new/928total byte hashes and saved-first arithmetic verified. Engine/scorer/helper1 each;driver1 is semanticNO_GO,not operational failure. Closure-only quoted-string error and parent pre-copy manifest-field adaptation remain preserved;neither reruns Engine/scorer/helper.

[Parent audit](../evaluation/engine073-launch-v27-parent-verification/final-copy-verification.json), [saved-first arithmetic](../evaluation/engine073-launch-first-metrics-v27-report.json), [actual consumed trace](../evaluation/engine073-launch-trace-development-v11/README.md).

6件は005印字watermark属性、025括弧内clip-finish限定、029quoted wordのlettering属性、045ownerless quoteと同roleのforeign owner、049複数roleへの複数形assertions参照、053冠詞なし印字heading。完全な外側の型とowner/参照の共通規則で直し、列挙した文字列だけの特例にしない。修正後の同じ60件は開発評価であり、新規独立受入にはしない。
