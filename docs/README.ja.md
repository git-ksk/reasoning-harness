# Reasoning Harness ドキュメント

日本語 | [English](README.md)

このrepositoryには、性質の違う2種類のドキュメントがあります。

1. **product documentation** — `reason`を使う人・組み込む人向け。
2. **research / evaluation evidence** — Harnessの変更を採用・棄却した根拠を保存する研究証跡。

普通に使うだけなら、研究履歴を先に読む必要はありません。目的に合う入口から入ってください。

## `reason`をまず試したい

この順がおすすめです。

1. [Getting Started](getting-started.ja.md) — current previewのinstallと、grounded / 根拠不足ケースを試す。
2. [ネイティブインストーラー](native-installers.ja.md) — Reason CLI 0.5.0向け1コマンド配布の契約。
3. [CLIガイド](cli.ja.md) — command、input、provider credential、config、stdin、JSON、exit semantics。
4. [Reasoning Harnessの仕組み](how-it-works.ja.md) — candidateとauthorityを分ける実行モデル。
5. [Product support / compatibility](support.ja.md) — supported platform、contract、provider、v0.x互換性。

インストール前に考え方だけ知りたい場合は、root [README](../README.ja.md) → [仕組み](how-it-works.ja.md)で十分です。

## アプリ / Agent / RAG / CIへ組み込みたい

おすすめ順:

- [CLIガイド](cli.ja.md) — `reason run`、`reason verify`、JSON envelope、stdin、exit semantics。
- [Reasoning Harnessの仕組み](how-it-works.ja.md) — materialization、verification、diagnostics、acceptance。
- [Grounded resolution](grounded-resolution.ja.md) — bounded acquisition → re-verification → finalization。
- [External resolver](external-resolvers.ja.md) — external acquisitionをauthorityへ直結させない境界。
- [Read-only MCP resolver](mcp-resolver.ja.md) — MCPをcorrectness authorityではなくbounded acquisitionとして使う。
- [Trusted verifier](trusted-verifier.ja.md) — 明示的にtrustedなdeterministic/oracle verification。
- [MCP product surface](mcp-product-surface.ja.md) — 外部MCP clientからnative runtimeを呼ぶ。
- [Resumable session](session.ja.md) — reasoning stateの保存、replay、訂正、fork。

## 信頼モデル / アーキテクチャを理解したい

まず:

- [Architecture](architecture.ja.md)
- [プロジェクトtrust](project-trust.ja.md) — executable / MCPを含むproject configの明示的activation boundary。
- [Reasoning policy](reasoning-policy.ja.md)
- [Evidence qualification](evidence-qualification.ja.md)
- [Exposed-text safety](exposed-text-safety.ja.md)
- [Bounded investigation](investigation.ja.md)
- [Terminology](terminology.ja.md)

設計判断の背景はADR:

- [ADR-0001: interface / packaging boundary](adr/0001-interface-and-packaging-boundaries.ja.md)
- [ADR-0002: grounded resolution / finalization](adr/0002-grounded-resolution-and-finalization.ja.md)
- [ADR-0003: reasoning control plane](adr/0003-reasoning-control-plane.ja.md)

## 現在の状況 / ロードマップだけ見たい

短い現行版を使ってください。

- [プロジェクト状況](project-status.ja.md) — release済み、active work、主要gap。
- [製品ロードマップ](product-roadmap.ja.md) — 現在のproduct / engine前進track。
- [Reason CLI 0.5.0 ロードマップ](reason-cli-0.5-roadmap.ja.md) — 一般向けterminal productization。
- [バージョニング](versioning.ja.md) — Reason CLI / Harness Engine / machine contractの分離。

従来の長い台帳は[project-status history](project-status-history.ja.md)と[product-roadmap history](product-roadmap-history.ja.md)として保存しています。

## 研究・評価をレビューしたい

まず現行release evidenceから入り、必要な場合だけ過去へ遡るのがおすすめです。

- [Harnessなし → ありの総合比較](product-external-info-v4-cross-model.ja.md) — 同じ入力条件でutility / safety / token / latencyを比較。
- [v36安全性補足](v36-raw-baseline-supplement.ja.md) — release surfaceの5つの安全境界でraw modelとHarnessを追試。
- [v0.4.2 final v36 release acceptance](natural-language-e2e-v36-result.ja.md) — current release evidence。
- [Product dogfood](product-dogfood.ja.md) — その他のproduct-oriented comparison。
- [ベンチマーク](benchmark.ja.md) — benchmark / evaluationの読み方。
- [研究計画](research-plan.ja.md) — 研究課題とpromotion discipline。
- [Corpus versioning](corpus-versioning.ja.md) — frozen case identityとscore compatibility。

過去のE2E、holdout、semantic judge、RSD、replication、provider studyはprovenanceのため意図的に残しています。ただし**onboarding用の読み物ではなく、ひとつの共通version列でもありません**。詳しくは[Terminology](terminology.ja.md)を参照してください。

## コントリビュートしたい

- [CONTRIBUTING.ja.md](../CONTRIBUTING.ja.md)
- [SECURITY.ja.md](../SECURITY.ja.md)
- [Architecture](architecture.ja.md)
- [プロジェクトtrust](project-trust.ja.md) — executable / MCPを含むproject configの明示的activation boundary。
- [プロジェクト状況](project-status.ja.md)

reasoning / correctness semanticsを変える変更では、fixture / evaluation evidenceを適切に追加し、untrusted model outputとHarness-owned authorityの境界を維持してください。

## 1分で分かる用語

| 用語 | 意味 |
| --- | --- |
| **Reason CLI** | ユーザー向け`reason` executableとterminal / distribution UX。 |
| **Harness Engine** | admission、verification、finalization、authority boundaryを持つreasoning/correctness runtime。 |
| **candidate** | model / agentが提案したreasoning / answer data。defaultではuntrusted。 |
| **evidence** | propositionを支える可能性のあるinput。authorityはadmission / qualification / verificationで決まる。 |
| **grounded** | factual claimがHarness-owned verified stateでcoverされている。 |
| **qualified** | 確認済みfactは出せるが、強い未証明結論はuncertainのまま。 |
| **unknown** | requested conclusionを出すだけのtrusted supportが現在は足りない。 |

machine identityやhistorical labelの正確な意味は[Terminology](terminology.ja.md)を参照してください。
