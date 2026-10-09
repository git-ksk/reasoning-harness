# Engine 0.7.0 — 出典互換性の開発候補v1（Issue #490）

日本語 | [English](engine-0.7-source-reconciliation-v1.md)

**状態：決定論的な開発用テストはPASS。独立live評価・リリースは未実施。** 公開済みEngineは0.6.1、Reason CLIは0.5.4のままです。

## 解決対象と既存機能の保護

[Issue #487の固定baseline](engine-0.7-baseline-v1-result.ja.md)は29/29の現行動作を再現し、同じ対象について意味は近いが文字列が違う2つの正確な引用が、保守的に`Conflict`へ分類される例を確認しました。これは外部事実を誤って`Known`/`Supported`にした不具合ではなく、**出典付き表示の有用性の改善候補**です。

今回の実装は、新しい**オプトインの照合結果**を返します。既存の`SourceAttributionState`、Conflict状態の更新・検証、従来のfinalizer、元の引用文・出典・hard verification、古いセッションを**変更しません**。

## 信頼境界と新API

- `capture_source_review_anchor`：対象方針、元のclaim、binding、原文とscope・時点を含む全admitted evidenceを正確に記録。
- `TrustedSourceReviewAuthority::new`：**信頼済みホストだけ**が、レビュー主体の真正性を確認して作成する権限オブジェクト。モデルや取得文書、MCP応答、ユーザー編集の保存済みレビューを信頼主体として採用してはいけません。
- `record_trusted_source_equivalence`：ホスト側が2つの原文を実際に調査して**意味が互換**と確認した後、その正確な対象・元文・bindingをレビュー記録に固定。**この関数自体は同義性を自動判定しません**。
- `reconcile_source_attributed_answer`：現在のartifactとの一致と、**別に渡されたホスト権限**を検査し、新契約の`SourceReconciliationView`を返す。保存したレビューだけでは自分の承認権限になりません。
- 新しい契約ID：`harness-source-compatibility-trusted-review-v1`と`harness-source-reconciliation-view-v1`。旧v1とは別物です。

レビューが承認された場合、追加viewの`status`だけが`reviewed_compatible`になります。**元の`view.original.status`は引き続き`conflict`**。文章は両ソースの**完全な引用をそれぞれの出典付きで維持**します。引用を1本化して別ソースがその文を言ったかのように表示することも、外部事実の真偽認定もありません。

## 安全性・拒否規則

- 不明な同義性、レビューなし、不完全な全ペア審査は`conflict`を維持。A≒B、A≒CだけでB≒Cを確定させません。
- 異なる対象、hard-verification必須の対象、出典・文面の改ざん、別のreviewer ID、重複レビュー、レビュー後に書き換わった証拠は拒否。
- 初期対象は**異なる正確な引用文**のみ。モデル生成paraphraseの自己申告は、信頼済みの同義性証明として採用しません。
- evidenceのscope、有効時点、権威、source versionが異なる場合は拒否。「後で取得したから新しい方が正しい」も禁止。
- 明白な否定、数値相違、可能性→確定、過去→現在、条件・全称・限定表現の違いも、レビュー主体が承認しようとしても拒否。一般的な意味解釈ではないため、その他の意味・適用時点は信頼済みreviewerの責務です。
- 1対象のレビューは最大16claim、依頼当たり最大1,024レビューに制限。限度超過時も保守的な判定とし、外部呼出はありません。

## 事前凍結した開発評価

実装前に`engine-0.7-reconciliation-dev-v1-spec-freeze`（commit `426b70195623d1b4d7196c5d6a1ab4a2272741c6`）を作成し、22件の期待動作を固定しました。fixtureのSHA-256は`8d50e04a24d09a2620827b15639a89c396b100dfa5d104baa69b3b9357190714`です。

| 結果 | 件数 |
| --- | ---: |
| 信頼済みレビューによる互換表示 | **4** |
| 従来どおりConflict | **9** |
| 改ざん・不当レビュー等を拒否 | **7** |
| 既存どおりQualified | **2** |
| **合計** | **22/22 PASS** |

さらに**凍結後の追加回帰テスト**で、過去・未来・可能性・範囲・数値、レビュー改ざん、replay、順序変化を検証しています。これらを凍結済み22件の母数へ後付けしていません。モデル・provider・外部取得呼出は**0件**、hard truth昇格は**0件**、旧artifactの状態変更も**0件**です。

- 実装：`crates/reasoning-harness-core/src/source_reconciliation.rs`
- 開発ケース：`fixtures/engine-0.7-source-reconciliation-development-v1/manifest.json`
- 測定：`crates/reasoning-harness-core/tests/source_reconciliation_development_v1.rs`
- 機械可読レポート：`evaluation/engine-0.7-reconciliation-dev-v1-result.json`
- 自動検証：`scripts/engine-0.7-reconciliation-dev-v1-report.py`、`scripts/test_engine_0_7_reconciliation_dev_v1_report.py`

## 残作業

**Issue #490はまだ完了ではありません。** 信頼済みレビューを実際に運用する方法、Reason CLIでの利用価値、候補コードとrunnerを凍結した後に作成する**新規の独立holdout**が必要です。Mistral・Google・Groqの各モデルで、元の0.6.1と同一入力を比較し、有用性がモデルごとに最低1件純増することと、安全性違反0を確認します。最初の独立FAILは書き換えません。

既存セッション・旧評価・Engine 0.6.1の配布物を変更せず、0.7.0 source releaseとCLI採用は[Issue #492](https://github.com/git-ksk/reasoning-harness/issues/492)で別途判定します。今回の新APIは**開発用・オプトイン**であり、通常の`reason`コマンドや自動モデル判定には組み込まれていません。
