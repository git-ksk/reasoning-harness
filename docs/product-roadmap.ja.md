# 製品ロードマップ: Reason CLI / Harness Engine

日本語 | [English](product-roadmap.md)

これは**これから進む方向だけを読むためのroadmap**です。releaseごとの詳細な実装台帳は[Product roadmap history](product-roadmap-history.ja.md)へ保存しました。

現在は、CLI product UXとreasoning / correctness changeを分離しています。CLIが使いやすくなったことを、reasoning engineが変わったことと混同しないためです。

## 現在のバージョン座標

```text
公開済みsplit CLI:
  Reason CLI 0.5.3
  Harness Engine 0.5.0

最新の独立Engine source release:
  Harness Engine 0.5.0 (`engine-v0.5.0`)

直近完了した統合:
  Reason CLI 0.5.3 -> Harness Engine 0.5.0 (#455)

最後のunified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2
```

`v0.4.2`が最後のunified historical releaseです。split CLI lineは`reason-v0.5.3`まで進み、CLI / Engine / machine-contract identityは独立して進みます。`reason-v0.5.2`はEngine 0.4.2のままimmutableで、`reason-v0.5.3`が独立release済みEngine 0.5.0をadoptしました。[バージョニング](versioning.ja.md)を参照してください。

## 製品目標

Reasoning Harnessはgeneral-purpose agent frameworkより意図的に狭いproductです。

> ユーザー・開発者・automationが低摩擦でAIを使える一方、factual outputの境界はmodel confidenceではなく、Harness管理のevidence、verification、uncertainty、failure semanticsで決まる。

中心ルールは変わりません。

```text
model proposes
Harness verifies / qualifies / abstains
```

## トラックA — Reason CLI 0.5.x: 一般利用向け製品化

**現在の公開pairは`reason-v0.5.3` / Harness Engine 0.5.0。** 以前の`reason-v0.5.2`はEngine 0.4.2のままimmutableで、#455は新しいCLI coordinateでadoptionを完了しました。

base `reason-v0.5.0` と0.5.1 / 0.5.2 patch lineはEngine 0.4.2でrelease済みで、`reason-v0.5.3`はaccepted Engine 0.5.0をdistributionします。P0 release gateと#455 Engine adoptionは完了しました。残るP1はdistribution follow-up #375のみです。Homebrew physical acceptanceは完了、WinGet community manifestはvalidation / CLAを通過してcommunity moderator approval待ちです。

underlying authority semanticsを変えず、`reason`を成熟したterminal productとして使える状態にします。

### フェーズ1: インストール / trust / 最初の回答

- Rust/Cargo不要のnative installer;
- release integrity、必要なsigning / notarization;
- update / explicit rollback / uninstall lifecycle;
- executable / authority-bearing local configへのproject trust;
- OS-native secure credential storage;
- `reason auth`とguided `reason setup`;
- provider / model discoveryと明示default選択。

### フェーズ2: 日常的なターミナルUX

- 引数なしinteractive mode;
- simple continue / resume / session selection;
- crash / concurrency-safeなmanaged session store;
- verified / qualified / unresolvedが分かる表示;
- provider usageとenforceable budget visibility;
- discoverable config command;
- progress / retry / cancellation UX;
- shell help / completionとaccessible terminal rendering。

### フェーズ3: 外部情報取得UX

- unrelated ambient secretを渡さないsubprocess environment isolation;
- guided read-only MCP add/list/inspect/test/remove;
- local path安定後のsecure remote MCP / OAuth lifecycle。

MCP / resolver outputは引き続きacquired dataであり、correctness authorityではありません。

### フェーズ4: 診断 / 復旧

- `reason doctor`;
- credential / model / quota / network / config / trust / MCP / session / updateのactionable error;
- silent execution-identity switchをしないprovider/model retirement・fallback policy;
- insecure bypassを勧めないproxy / custom-CA / headless diagnostics。

### フェーズ5: 新規インストールでのリリース判定 — 完了

`reason-v0.5.0`は、supported platformでinstall、project trust、setup、secure credential、first answer、interactive follow-up、session resume、diagnostics、local privacy、subprocess secret isolation、update/rollback/uninstall、既存JSON automation compatibilityまでacceptanceした後にtagしました。このgateは0.5.x patch lineでもregression coverageとして維持します。

完全なP0/P1 issueとacceptance matrixは[Reason CLI 0.5.0 ロードマップ](reason-cli-0.5-roadmap.ja.md)を参照してください。

### Harness Engine 0.5.0 adoption — 完了（#455）

`reason-v0.5.3`を **Reason CLI 0.5.3 / Harness Engine 0.5.0** として公開済みです。adoptionは`reason-v0.5.2`のimmutabilityを維持し、Engine semantic workを再開していません。

- release manifest / provenanceはCLI 0.5.3、Engine 0.5.0、merge commit `e9148c737c6f9bf29ce7c9258d549f5c526dfb4a`をbinding;
- supported-platform package candidate / installer / no-Rust consumer / lifecycle / credential-store / CLI smoke gateはPASS;
- live `reason-v0.5.2` -> `reason-v0.5.3` updateと`reason-v0.5.3` -> `reason-v0.5.2` rollbackはいずれもEngine transitionを表示し、`--allow-engine-change`なしではfail closed;
- explicit consent付きupdate / rollbackは公開済みprovenance-verified release間で成功;
- Engine 0.5.0 frozen evidenceは変更なし。

## トラックB — Harness Engine 0.5.0: 検証付き調査の有用性 — 完了

reasoning / correctness behaviorへ影響しうる変更はこのtrackで扱い、fresh evidenceを要求します。

現在地と順序:

- **#248 finalization / grounding — 完了:** PR #435をmergeし、fresh frozen `issue-248-finalization-e2e-v2`で3/3 case、correctness-boundary violation 0、session external-call replay 0を確認;
- **#247 evaluator semantics — 完了:** frozen v11でpath observability / product utility / hard correctness / operational completenessを分離し、frozen v9はrescoreしていない;
- **#282 repeated-trial planner reliability — 完了:** frozen `planner-reliability-v1`は両routine providerでprimary 5/5 trialを完了。Mistral strict planner successは5/5、Googleはinadmissible action proposal 1仰が安全にrejectされ4/5、correctness-boundary violationは0;
- **#283 deterministic Harness-owned action materialization — 完了:** accepted candidateでmechanically safeなexact-key read-only action materializationをHarness control flowへ移し、`reason-investigation-intent-v1` / `target-intent-materialization-v1`を採用。frozen v3 adoption evidenceでarchitecture-path gateをPASSし、authority / finalization semanticsは変更していない。
- **#443 initial Engine 0.5 final cross-model acceptance — 完了:** fresh `engine-0.5-final-v2-freeze`でaccepted baselineを確立;
- **#445/#446/#450 final hardening — 完了:** finalization correctnessとplanner target-recall utilityを分離し、explicit-fact session correction continuityとmechanically uniqueなadmitted exact-fact materializationをHarness controlへ決定論化。verification authorityは弱めていない;
- **#452 final-v3 / versioned release closeout — 完了:** `engine-0.5-final-v3-freeze` canonical run `35457038163`で独立required 6 row・fresh 18/18 case PASS、correctness-boundary violation 0、session external replay 0;
- **Versioned Engine release — 完了:** `engine-v0.5.0`は`4fd6acc85511f286fd6b2f7c9439665b7819206a`を指し、`reasoning-harness-core`は0.5.0をreport。milestone #4はopen issue 0でclosed。

Engine 0.5.0はclosed release baselineで、Reason CLI 0.5.3からdistributionされています。今後のsemantic Engine workは新たに測定されたgapと新しいEngine identityを要求します。完了済み#455 adoptionは0.5.0 evidenceをreopen / rewriteしていません。

## 昇格ルール

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

## このロードマップでやらないこと

現在のgeneral-use productizationではReasonを以下へ変えません。

- write-capable coding agent;
- background autonomous agent platform;
- correctness core内の汎用browser / RAG crawler;
- task完了のためsilentにmodel/providerを切り替える仕組み;
- retrieval / tool / model textを便利だからtrusted扱いするproduct。

必要なら別のproduct / authority designとして扱います。

## 過去のリリース / 研究

v0.1.0〜v0.4.2のimplementation chronology、completed milestone、旧evaluation coordinate、research provenanceは[Product roadmap history](product-roadmap-history.ja.md)を参照してください。

現在release済みの内容とuser-facing gapは[Project status](project-status.ja.md)、全docsの入口は[Documentation index](README.ja.md)です。
