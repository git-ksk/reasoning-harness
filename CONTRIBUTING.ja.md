# コントリビューションガイド

reasoning-harness の改善にご協力いただきありがとうございます。

## 対象範囲

このプロジェクトは、確率的な候補出力を明示的な決定論的正しさ機構に通す、provider-neutral な Rust 研究ハーネスです。貢献では次の境界を維持してください。

- モデル出力は信頼されない候補データである。
- evidence と verification authority はハーネスが所有する。
- 決定論的 validator と外部 oracle はモデル自身の評価より優先される。
- `unknown` は有効な成功結果である。
- soft semantic judge は hard correctness gate を装ってはならない。
- first-party runtime component は Rust のみとする。

## 開発

Rust 1.88 以降を使い、`Cargo.lock` をコミット済みの状態に保ってください。

```bash
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run -q --locked -p reasoning-harness-cli -- eval fixtures --format human
```

reasoning semantics を変更する場合は fixture を追加または更新し、benchmark の期待値が変わった理由を説明してください。live provider の結果は研究上の evidence であり、決定論的な CI gate ではありません。

## プルリクエスト

変更範囲を絞ってください。影響を受ける correctness boundary、新たに対象とした failure mode、benchmark への影響を記載してください。provider API key、private dataset、credential、または機密入力を含む生成ログを決して含めないでください。
