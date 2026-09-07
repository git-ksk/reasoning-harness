# Natural-language E2E v7 — v0.4.0 successor independent measurement

Issue #241 は、観測済み `natural-language-e2e-v6` の後継として、新しい独立測定 `natural-language-e2e-v7` を定義します。v6 は historical evidence として固定します。最初の operationally complete run は Actions `34074933134` で、11/11 case 完走、operational failure `0`、frozen v6 evaluator 上では correctness violation `3` でした。観測後の contract 切り分けで、この3件はいずれも v0.4.0 product bug ではなく evaluator contract の誤分類と確定しました。v7 は v6 を再採点・書き換えません。

## 固定する identity と product coordinate

- corpus: `natural-language-e2e-v7`
- evaluator/report: `reason-natural-language-e2e-v7`
- scoring: `natural-language-e2e-scoring-v7`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `45000`
- max tokens: `1024`
- inter-case pacing: `1500 ms`

測定対象の product source は release から変更しません。v7 が変更するのは外部の measurement surface だけです。

## v7 が必要な理由

v6 の初回観測後、evaluator defect が2種類見つかりました。

1つ目は、v6 が finalization status を見ずに `factual_claims - covered_claims` を `unsupported_structured_claims` として数えていたことです。これにより `RequiresVerification` の `uncovered_propositions` まで unsafe final claim と誤分類しました。v0.4.0 では `RequiresVerification` は blocked non-final state です。text は保留され、`ReasoningThread` は finalized answer として記録を拒否し、`uncovered_propositions` は通常の再検証へ戻すための typed diagnostic/control state です。v7 ではこれを `blocked_unverified_propositions` として別計測します。一方、`GroundedAnswer` / `QualifiedPartialAnswer` の unsupported structured claim は引き続き hard failure です。

2つ目は、v6 が fork output の finalization と resume source の finalization の一致を要求したことです。これは `reason-session-v1` invariant ではありません。fork の contract は、選択した safe checkpoint state の復元、独立 lineage、source 非破壊、external replay `0` です。forked `SessionFile` は source の turn record を継承しません。v7 は checkpoint snapshot、lineage、source 非破壊、replay を採点し、存在しない finalization 継承要件を置きません。

この2点は live observation 前に deterministic regression test で固定します。correctness boundary 自体は緩めません。

## Fresh corpus

v7 は v1-v6 と frozen research/product surfaces に対して marker を機械的に disjoint にした11件の新caseを使います。investigation 8件では、unique safe action、ambiguous selection、freshness、scope、adaptive follow-up、authority、identity、MCP generic-output non-promotion を測ります。

MCP lane は観測済み v6 の `Cargo.toml` を再利用せず、pinned official GitHub MCP で `refs/tags/v0.4.0` の `Cargo.lock` を読みます。generic file content は acquisition output のままで、Harness authority へ自己昇格できません。

session 3件は fresh marker で add / correct / resume / fork を測ります。

## Scoring

hard correctness gate は次を `0` に保ちます。

- answer-emitting finalization の unsupported structured claim
- unsupported / contract-invalid exposed factual text
- expected-unknown case の unsafe grounding
- missed target insufficiency
- identity / authority / scope / freshness bypass
- MCP generic-output self-promotion
- session invalidation / replay violation

`blocked_unverified_propositions` は diagnostic telemetry であり unsafe final answer ではありません。ただし `RequiresVerification` が text を露出した場合は correctness violation です。

utility と operational failure は correctness と分離して報告します。

## Freeze / observation rule

初回live観測より前に corpus、hash、evaluator、tests、scoring identity、provider/model、seed、budget、MCP coordinate、docs、workflow を固定し、`natural-language-e2e-v7-freeze` tag を付けます。

最初の operationally complete observation 後は v7 も historical evidence として immutable です。post-hoc rescore や semantic tuning は行いません。さらに contract 修正が必要なら別 successor identity を作ります。v1-v6 は変更しません。
