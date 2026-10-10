# Harness Engine 0.7.0 — 初回独立3社評価 v1

**結果：固定した合成データによる出典照合の評価ゲートは3社ともPASS。** Engine 0.7.0の正式リリースを意味しません。

- 実行：[GitHub Actions #38013006323](https://github.com/git-ksk/reasoning-harness/actions/runs/38013006323) — preflight・3社の個別評価・最終採点の**5/5ジョブ成功**
- 比較候補：`4cae9326bcc3e210c3250f05772d84f00e41b645`（Engine 0.6.1の出典判定と並行比較）
- runner固定：`engine-0.7-independent-v1-runner-freeze`（`1124e5a7c34e42c088d81908c3cdc75aae2db309`）
- データ固定：`engine-0.7-independent-v1-corpus-freeze`（`78f99559bf044013e66025bde1a612895f2d41e6`）
- データSHA-256：`d3ce38ee2ef06310afb6a247d3762fc408b93cb49bd602a52d8ce1b8bcf7fa0b`
- 新規12ケース・15対象（互換6、否定/不明/文脈不一致8、完全同一1）

## 正式初回観測の数値

| 指標 | Mistral | Google | Groq |
| --- | ---: | ---: | ---: |
| 根拠付き互換対象の追加表示 | **3件** | **6件** | **6件** |
| 互換対象を見逃した判定 | 3件 | 0件 | 0件 |
| 不適切な互換アドバイス | 0件 | 1件 | 2件 |
| 実際に無根拠な互換性を承認した数 | **0件** | **0件** | **0件** |
| model呼出 | 14回 | 14回 | 14回 |
| provider試行 | 14回 | 14回 | 14回 |
| 入力token | 3,922 | 3,989 | 4,653 |
| 出力token | 84 | 70 | 786 |
| API運用失敗 | **0件** | **0件** | **0件** |
| 安全性hard gate違反 | **0件** | **0件** | **0件** |

GoogleとGroqの不適切な互換アドバイスは、出典のscopeまたはversionが異なる例に集中した。**モデルが同義と答えても、事前固定の評価用オラクルと出典境界が統合を許さなかった**ため、元の`Conflict`と出典が保持された。Mistralは互換6件中3件を保守的に保留した。

比較結果は実験的な`reviewed_compatible`という対象別表示の純改善である。**すべて架空の引用**であり、外部情報の真偽の検証、事実の`Known`/`Supported`への昇格、実ユーザーの有用性測定を意味しない。新機能は開発版の明示オプトインであり、公開済みCLI 0.5.4には組み込まれていない。

## データ保全・互換性

GitHub Actionsの生結果・case単位の初回checkpoint・モデル別検証結果・最終スコアを`evaluation/engine-0.7-independent-v1-first-observation/`に保存。各ファイルのSHA-256、GitHub Run ID、runner/corpusの固定commitを`archive_manifest.json`へ記録し、**凍結済み採点プログラムで再計算したJSONが公式の最終スコアと完全一致することを検証した**。

Engine 0.6.1からの従来の出典契約、Thread、Artifact、CLI main、管理セッションのソースは変更されていない。ローカルのThread replay 11/11、CLI session resume/fork replay 1/1、出典照合5/5、対象別開発ケース1/1をPASS。先行するPR #500ではRustを使わないバイナリ利用者の4OS向けチェックもすべてPASS。

## 未完了のリリース条件

#488（出典の独立性）、#489（改訂の親子関係）、#491（再取得の新たな制御）は0.6.1ベースラインに具体的な安全性破綻や必要な新実装が確認できていないため、この評価には**新機構として採用していない**。既存の保守的な動作を維持する。追加要求が明確になるまで別途保留とし、完了した新機能であると称さない。

[#490](https://github.com/git-ksk/reasoning-harness/issues/490)は出典の対象別表示が開発用独立ケースで改善したことを確認した段階。Engine-only source releaseの最終版管理、ソース互換性、公開手順と未解決項目は[#492](https://github.com/git-ksk/reasoning-harness/issues/492)で継続する。