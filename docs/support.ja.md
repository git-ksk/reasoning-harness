# プロダクトのサポートと互換性

`reason`はv0.x期間中の外部プレビューCLIです。正しさの責任主体は引き続きnative runtimeにあり、product supportはhuman-readable outputやproviderのマーケティング上の保証ではなく、明示的なCLI/data contractを基準に定義します。

## サポート対象のプロダクトサーフェス

互換性を追跡するプロダクトコマンドは次のとおりです。

- `reason run`
- `reason verify`
- `reason semantic-check`
- `reason schema`

`reason eval`、`reason eval-resolution`、`reason eval-judges`、および専用の研究用バイナリはresearch/evaluation surfaceです。これらはより速いペースで変更される可能性があり、v0.1 product compatibility promiseには含まれません。

## サポート対象のリリースプラットフォーム

すべてのproduct pull requestでは、次の環境でcredential-freeの`reason` smoke suiteを実行します。

- Linux x86_64（`ubuntu-24.04` runner class）;
- macOS arm64 / Apple Silicon（`macos-15` runner class）;
- macOS x86_64 / Intel（`macos-15-intel` runner class）;
- Windows x86_64（`windows-2025` runner class）。

タグ付きリリースでは、これらの各platform class向けにnativeの`reason` executableを1つずつpackageします。他のtargetもcompileできる場合がありますが、matrixに追加されるまではrelease-supportedではありません。

## マシン契約の方針

executableのsemverとmachine contract identityは別々の座標です。

現在のproduct identityには次が含まれます。

- `reason-cli-output-v1`
- `reasoning-artifact-v1`
- `reasoning-candidate-v1`
- `reason-config-v1`
- `semantic-check-input-v1`
- semantic runtime identity `semantic-runtime-identity-v1`

既存のoutput-contract identityの範囲では、consumerは追加フィールドを許容する必要があります。フィールドの削除、フィールドの意味の変更、authority/exit semanticsの変更には、黙って変更するのではなく、該当する新しいcontract identityが必要です。Config schemaは設計上unknown fieldに対してfail closedするため、新しく追加されたfieldを使うconfigには、それに対応する新しいCLIが必要になる場合があります。

人間向けテキストはpresentationであり、compatibilityやcorrectnessのcontractではありません。

## v0.xの破壊的変更

v1.0以前は、command flagやproduct schemaが引き続き変更される可能性があります。意図的な非互換変更では、必ず次を行います。

1. `CHANGELOG.md`に明記する。
2. wire meaningが変わる場合は、該当するmachine contract identityを更新する。
3. 既存の外部workflowに影響が出る場合は、migration guidanceを含める。
4. merge前にcross-platform product smokeを通過させる。

## プロバイダーのサポート方針

providerに依存しないruntimeがproduct boundaryです。Provider adapterはtransport/APIの挙動をnormalizeしますが、verification authorityになることはありません。

- Mistral、Google Gemini/AI Studio、NVIDIA Hosted NIMのadapterは、live candidate generation用に実装されています。
- MistralとGoogle-hosted Gemmaは、サポート対象のcurrent/rollback `semantic-check` product pathでlive smokeを実施しています。記録されたworkloadでは、Ministral 3B/8B/14B、Mistral Small、Gemma 4 31B、Gemini 3.1/3.5 Flash-Liteについてproduct dogfoodを完了しています。完了したことはutilityが同等であることを意味しません。記録されたtarget-coverage matrixの範囲は0.00から1.00です。
- model/providerは、特定のstructured-output protocolと互換性がない場合もあります。Gemma 4 26B A4BとNemotron 3.5 Lightningが記録されている例です。いずれもproduct dogfood runではfallback後のinvalid structured outputで失敗しており、semantic scoreや作為的なabstentionではなく、operational/protocol evidenceとして扱います。
- Provider quota、service availability、rate limit、model retirement、model固有のoutput qualityは外部のoperational dependencyであり、harness correctnessとは分けて報告します。

Provider credentialはenvironment variableのまま保持し、`reason-config-v1`では受け付けません。

## 安定性ステータス

`v0.3.0`は現在の外部プレビューリリースです。bounded external acquisition、Harnessが所有するprovenance/freshness/scope/authority admission、typed external-resolution budgets/telemetry、read-only MCP acquisition、独立したtrusted verifier lane、既存のresearch/authority foundation上でのoptionalな`reason-mcp` integrationを追加しています。文書化されたv1.0 readiness gateは満たされていますが、`v0.3.0`は意図的にstable v1.0の主張ではなく、prerelease/v0.x compatibility promiseのままです。将来のv1.0には、通常のprovenance workflowを通じた明示的なversion/tag/release decisionがなお必要です。


product term、machine/runtime identifier、過去のresearch labelの区別については、[用語と命名ルール](terminology.ja.md)を参照してください。
