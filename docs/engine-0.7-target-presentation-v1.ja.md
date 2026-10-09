# Engine 0.7.0 — 対象別の出典付き回答表示（#490）

[English](engine-0.7-target-presentation-v1.md)

**状態：開発候補。事前固定14/14ケースPASS。独立評価とリリースは別判定。**

## 課題と改善内容

0.6.1の複数対象への回答では、対象Aの言い換えがレビュー済みでも、対象Bに矛盾があると回答全体が`Conflict`になります。旧本文には原文と出典が残りますが、対象ごとに何が解決したか判断しづらい課題がありました。

新しいオプトインAPI `reconcile_source_attributed_targets` は、明示的に承認された出典の互換性を元に、各targetについて`status`、従来どおりの引用文と出典（`original`）を返します。

- 例：対象Aは`reviewed_compatible`、対象Bは`conflict`。他対象の不一致を隠しません。
- 元の全体判定・本文・引用は変更しません。対象別の引用を結合した際に旧全体の引用と一致しない場合は拒否します。
- 独立契約ID：`harness-source-reconciliation-target-answer-v1`。旧v1 JSON/セッション形式は不変。
- 人間による承認済みレビューだけが互換判定に使用できます。モデルが同義と述べるだけでは権限になりません。
- 外部事実の`Known`/`Supported`昇格、引用の捏造、モデル呼出・外部取得はありません。replayで同じ結果を返します。
- CLI表示ではソース由来の制御文字・双方向テキスト操作文字をエスケープします。

## CLI（明示オプトイン）

レビューなしの保守的な表示は鍵登録なしで使えます。

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  show-targets --artifact /path/to/artifact.json \
  --target target-A --target target-B --json
```

署名付きの承認済みレビューがある場合は、それを指定できます。

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  show-targets --artifact /path/to/artifact.json \
  --target target-A --target target-B \
  --reviewer local-owner --approval /private/path/approval-A.json --json
```

公開済みの通常の`reason`コマンドは変更していません。新コマンドは開発版の明示的なオプトインです。

## 事前固定した開発評価

実装前にタグ`engine-0.7-target-presentation-dev-v1-spec-freeze`、commit `f5c098b2f5df2c93f5385a96bde1719c5e313a37`へ14ケースを固定。fixture SHA-256は`fe27ecc6704992109e8da100235bc4ba5ffbafec1073ebc4fa572993304b4214`。

- 正常に対象別表示できるケース：**11**
- 対象重複・対象外承認・出典変更を拒否：**3**
- 旧全体判定はConflictのまま、レビュー済み対象の互換状態を個別表示できた人工ケース：**6**
- 既存のv1本文/引用保持、全引用カバレッジ、replay一致。外部呼出・hard truth昇格は0。
- レポート改ざん検知・CIで固定ケースと再現結果を比較。

評価：`crates/reasoning-harness-core/tests/target_presentation_development_v1.rs`、`evaluation/engine-0.7-target-presentation-dev-v1-result.json`。

**注意：** 14ケースは合成の開発用データであり、独立Mistral・Google・Groq評価での実用価値の証明ではありません。実データでの回答価値、独立比較、リリース判定は[#492](https://github.com/git-ksk/reasoning-harness/issues/492)で進めます。