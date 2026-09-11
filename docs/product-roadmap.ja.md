# Product roadmap: Reason CLI / Harness Engine

日本語 | [English](product-roadmap.md)

これは**これから進む方向だけを読むためのroadmap**です。releaseごとの詳細な実装台帳は[Product roadmap history](product-roadmap-history.ja.md)へ保存しました。

現在は、CLI product UXとreasoning / correctness changeを分離しています。CLIが使いやすくなったことを、reasoning engineが変わったことと混同しないためです。

## Current coordinates

```text
現在のrelease:
  Reason CLI 0.4.2
  Harness Engine 0.4.2

次のgeneral-use product line:
  Reason CLI 0.5.0
  Harness Engine 0.4.2
```

`v0.4.2`が最後のunified historical releaseです。今後のCLI tagは`reason-vX.Y.Z`を使い、Engine versionとmachine-contract identityは独立して進みます。[Versioning](versioning.md)を参照してください。

## Product goal

Reasoning Harnessはgeneral-purpose agent frameworkより意図的に狭いproductです。

> ユーザー・開発者・automationが低摩擦でAIを使える一方、factual outputの境界はmodel confidenceではなく、Harness管理のevidence、verification、uncertainty、failure semanticsで決まる。

中心ルールは変わりません。

```text
model proposes
Harness verifies / qualifies / abstains
```

## Track A — Reason CLI 0.5.0: general-use productization

**Engine baselineは0.4.2に固定。**

underlying authority semanticsを変えず、`reason`を成熟したterminal productとして使える状態にします。

### Phase 1: install / trust / first answer

- Rust/Cargo不要のnative installer;
- release integrity、必要なsigning / notarization;
- update / explicit rollback / uninstall lifecycle;
- executable / authority-bearing local configへのproject trust;
- OS-native secure credential storage;
- `reason auth`とguided `reason setup`;
- provider / model discoveryと明示default選択。

### Phase 2: daily terminal UX

- 引数なしinteractive mode;
- simple continue / resume / session selection;
- crash / concurrency-safeなmanaged session store;
- verified / qualified / unresolvedが分かる表示;
- provider usageとenforceable budget visibility;
- discoverable config command;
- progress / retry / cancellation UX;
- shell help / completionとaccessible terminal rendering。

### Phase 3: external acquisition UX

- unrelated ambient secretを渡さないsubprocess environment isolation;
- guided read-only MCP add/list/inspect/test/remove;
- local path安定後のsecure remote MCP / OAuth lifecycle。

MCP / resolver outputは引き続きacquired dataであり、correctness authorityではありません。

### Phase 4: diagnostics / recovery

- `reason doctor`;
- credential / model / quota / network / config / trust / MCP / session / updateのactionable error;
- silent execution-identity switchをしないprovider/model retirement・fallback policy;
- insecure bypassを勧めないproxy / custom-CA / headless diagnostics。

### Phase 5: fresh-install release gate

`reason-v0.5.0`は、supported platformでinstall、project trust、setup、secure credential、first answer、interactive follow-up、session resume、diagnostics、local privacy、subprocess secret isolation、update/rollback/uninstall、既存JSON automation compatibilityまでacceptanceできるまでtagしません。

完全なP0/P1 issueとacceptance matrixは[Reason CLI 0.5.0 roadmap](reason-cli-0.5-roadmap.md)を参照してください。

## Track B — Harness Engine 0.5.0: verified investigation utility

reasoning / correctness behaviorへ影響しうる変更はこのtrackで扱い、fresh evidenceを要求します。

対象にはたとえば:

- verified targetのexposureへ影響するfinalization / grounding改善;
- one-shot成功ではなくrepeated-trialで測るplanner reliability;
- model variabilityが不要な箇所のdeterministic Harness-owned action materialization。

正確なscopeはengine milestone / issueをsource of truthとします。Engine 0.5.0のsemantic / utility changeを、CLI-only productizationとして混ぜません。

## Promotion rule

新しいreasoning mechanismは、1回良いexperiment結果が出ただけではsupported product behaviorへ昇格しません。

必要に応じて:

1. authority boundaryを明示;
2. deterministic regression coverage;
3. calibration / development evidenceとindependent evaluationの分離;
4. frozenまたはprovenance-stableなevaluation identity;
5. operational stabilizationとsemantic scoringの分離;
6. runtime / contract identityと必要なrollback;
7. fail-closed behaviorを弱めないCLI/API integration。

historical FAIL / INCONCLUSIVEは、後続fix後に書き換えてPASS扱いしません。

## このroadmapでやらないこと

現在のgeneral-use productizationではReasonを以下へ変えません。

- write-capable coding agent;
- background autonomous agent platform;
- correctness core内の汎用browser / RAG crawler;
- task完了のためsilentにmodel/providerを切り替える仕組み;
- retrieval / tool / model textを便利だからtrusted扱いするproduct。

必要なら別のproduct / authority designとして扱います。

## Historical releases / research

v0.1.0〜v0.4.2のimplementation chronology、completed milestone、旧evaluation coordinate、research provenanceは[Product roadmap history](product-roadmap-history.ja.md)を参照してください。

現在release済みの内容とuser-facing gapは[Project status](project-status.ja.md)、全docsの入口は[Documentation index](README.ja.md)です。
