# Provider / model retirement と fallback policy

日本語 | [English](model-retirement-policy.md)

**状態:** Reason CLI 0.5.0開発ラインで実装済み。Harness Engine 0.4.2のreasoning / authority semanticsは変更しません。

Reasonはmodel retirement、incompatibility、quota、outage、provider errorから復旧するために、provider/model execution identityを**黙って変更しません**。

## Catalog lifecycle state

`reason models`はcompatibility evidenceとは別に`availability`を表示します。

- `current` — general-use対象でもある場合、通常のconfigured useに利用可能。
- `deprecated` — 信頼できるlifecycle evidenceでretirement予定が確認されたidentity。product defaultには保存しない。
- `unavailable` — 信頼できるevidenceで利用不能が確認されたidentity。product defaultには保存しない。
- `known_incompatible` — 現行product protocol / roleとの非互換をReason側のevidenceで確認済み。
- `unlisted` — configured identityがcurated catalogにない。catalogに無いこと自体をprovider retirementとは扱わない。

`deprecated` / `unavailable`はrepositoryに信頼できるlifecycle evidenceがある場合だけ付与します。一時的なrate limit、quota error、outage、DNS failure、単発request failureからretirementを推測しません。

## Silent fallback禁止

`reason setup`、`reason model set`、doctorのlocal validationはnon-currentなcatalog identityをfail closedで拒否し、`reason models <provider>`から明示的にreplacementを選ぶよう案内します。通常のprovider failureはtyped operational failureのままで、別model/providerへhidden switchしません。

Reason CLI 0.5.0はautomatic provider/model fallback chainを実装しません。将来explicit fallbackを追加する場合は、実行前にcompatibilityを検証し、実際に実行したprovider/modelをhuman output、JSON、persisted session provenanceへ記録する必要があります。

## Persisted session

managed sessionは記録済みprovider/model runtime identityへpinされます。`--continue` / `--resume`はそのidentityを再利用し、明示指定が競合するprovider/modelは`session_incompatible`で拒否します。別identityを使う場合は新規session（または将来の明示migration/fork flow）を使い、過去sessionのidentityをReasonがin-placeで書き換えることはありません。

## Diagnostics

`reason doctor`はconfigured modelの`model_availability`を表示し、local compatibility/lifecycle validation失敗時は`reason models <provider>`を明示的なrecovery pathとして返します。live provider failureをmodel retirementの証拠として扱うことはありません。
