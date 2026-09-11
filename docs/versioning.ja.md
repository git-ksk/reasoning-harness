# 製品 / Engine / machine contract のバージョニング

Reasoning Harnessでは、製品UXを進めてもreasoning/correctness engineが変わったように見えないよう、version座標を分離します。

## 3つの座標

### 1. Reason CLIのSemVer

`reasoning-harness-cli`が、ユーザー向けの **Reason CLI** versionを所有します。

command、setup/config UX、credential管理、diagnostics、installer、packaging、supported output behaviorなど、製品としての変更で上げます。今後のCLI release tagは次の形式です。

```text
reason-vX.Y.Z
```

release workflowは、`reason-vX.Y.Z`と`reasoning-harness-cli` package versionが一致することを検証します。

### 2. Harness EngineのSemVer

`reasoning-harness-core`が **Harness Engine** versionを所有します。

authority、admission、verification、finalization、answer-safety semantics、deterministic investigation controlなど、reasoning/correctness実装の変更を表す座標です。Engine変更は独立したevidence/promotionを必要とし、CLIのUX releaseだけではEngine versionを上げません。

`reasoning-harness-providers`は内部crateとして独立versionを持ちますが、第3のユーザー向けproduct versionにはしません。

### 3. Machine contractの識別子

`reasoning-artifact-v1`、`reason-cli-output-v1`、`reason-config-v1`、`reason-session-v1`などのwire/schema identifierは、package SemVerとは独立したcompatibility座標として維持します。package versionを上げても、既存contract identityの意味を黙って変更しません。

## 移行境界

`v0.4.2`を**最後のunified historical release**として固定します。Reason CLI 0.4.2とHarness Engine 0.4.2は、同じ`v0.4.2` tagでreleaseされました。

以後は2本を独立して進めます。最初の一般向け製品lineは次の想定です。

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

CLI 0.5.0のproductization milestoneでは、Engine 0.4.2 semanticsを固定したままsetup、secure credential、diagnostics、distributionを改善します。別の **Harness Engine 0.5.0 — Verified Investigation Utility** milestoneが、engine semantics/utilityに踏み込む変更を所有し、その場合はfresh evaluationを必須にします。

過去のunified tag `v0.1.0`〜`v0.4.2`はimmutableのまま保持します。release workflowからこれらのhistorical tagを手動repackageすることはできますが、新しいCLIの自動releaseは`reason-v*` namespaceだけを使います。
