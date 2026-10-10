# Harness Engine 0.7.0 — 独立ソースリリース候補

**状態：レビュー中。正式な `engine-v0.7.0` タグ・GitHub ReleaseはCIとリリースゲートの最終確認後にのみ作成。**

## 配布座標

- Engine: `reasoning-harness-core 0.7.0`（ソースのみ、`publish = false`）
- 公開済みReason CLI: `reason-v0.5.4` / Engine 0.6.1 **不変**
- CLIの0.7.0採用、配布物更新、移行/ロールバックは別issue・別release
- 新機能は開発者が明示的に呼び出すオプトインの出典照合API。既存の通常CLIには自動適用しない。

## 今回採用する機能（#490）

同一targetの複数出典が意味的に互換であるとき、**別途検証されたhost側レビュー**に基づき、原文・原文のConflict・全引用をそのまま維持しつつ、target別に出典付きの互換表示を追加できる。矛盾する別targetは独立してConflictを保持する。モデルの一致判定だけではレビュー権限や事実の`Known`/`Supported`昇格は得られない。

- `harness-source-reconciliation-view-v1`
- `harness-source-reconciliation-target-answer-v1`
- `harness-source-compatibility-trusted-review-v1`

CLIの補助`reason-source-review-local`は開発用の別binaryであり、公開済み`reason` CLI 0.5.4に含まれていない。macOSの物理実機でOS keyring、署名、改ざん拒否、revocationをE2E確認したが、**Mac自動操作による試験は独立した人間の同意を証明しない**。同一OSユーザー権限の自動化による操作可能性を残存リスクとして明記する。

## 独立評価

- runner固定：`engine-0.7-independent-v1-runner-freeze` / `1124e5a7c34e42c088d81908c3cdc75aae2db309`
- 新規コーパス固定：`engine-0.7-independent-v1-corpus-freeze` / `78f99559bf044013e66025bde1a612895f2d41e6`
- 初回の正式実行：[GitHub Actions #38013006323](https://github.com/git-ksk/reasoning-harness/actions/runs/38013006323)
- 同一12ケース、15対象。Mistral +3、Google +6、Groq +6の出典付き互換表示の純増。3社ともAPI失敗0、hard gate違反0。Google/Groqの誤ったモデル助言はscope/versionガードで拒否。
- 初回結果を`evaluation/engine-0.7-independent-v1-first-observation/`に保存。13ファイルのSHA-256と事前固定採点の再計算一致をCIで検証。
- [詳細結果](engine-0.7-independent-v1-result.ja.md)。

**制約：** 評価は架空の出典を使った意味的整合性テストで、外部事実の正確性・実利用者の価値・独立した人間の承認は測っていない。

## 採用しない機能

- #488：一次情報の独立性/出典系譜の推定・追加スキーマは未採用。
- #489：改訂版の優先関係・最新版優先処理は未採用。
- #491：新しい回答充足性・自動再取得処理は未採用。

これらは具体的な実利用上の欠落と別のfreeze評価が得られるまでOPENで継続する。既存の保守的な0.6.1の時点・scope・検証・必要情報の判定は維持する。

## 互換性と最終ゲート

従来の`ReasoningArtifact`、`ReasoningThread`、出典v1契約、CLI main、管理セッションのソースは0.6.1と同一。新APIは追加のみ。Thread replay 11/11、CLI resume/fork 1/1、出典照合5/5、target表示1/1をローカルでPASS。no-Rustの4OS利用者チェックは先行PR #500でPASS。

**最終確認（未実施項目はPASSと主張しない）：** このrelease PRのexact-head workspace fmt/Clippy/tests、OS別CI、バージョン一致、独立初回結果アーカイブ検証、既存CLI公開物の不変性。すべてPASS後、merge-mainのcommitに対してのみ`engine-v0.7.0`を付け、GitHub source prereleaseとrelease証跡を記録する。Engine 0.6.1のタグ・配布物は変更しない。
