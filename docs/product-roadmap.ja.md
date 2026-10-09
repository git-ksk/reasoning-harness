# 製品ロードマップ: Reason CLI / Harness Engine

日本語 | [English](product-roadmap.md)

これは**これから進む方向だけを読むためのroadmap**です。releaseごとの詳細な実装台帳は[Product roadmap history](product-roadmap-history.ja.md)へ保存しました。

現在は、CLI product UXとreasoning / correctness changeを分離しています。CLIが使いやすくなったことを、reasoning engineが変わったことと混同しないためです。

## 現在のバージョン座標

```text
公開済みsplit CLI:
  Reason CLI 0.5.4
  Harness Engine 0.6.1

最新の独立Engine source release:
  Harness Engine 0.6.1 (`engine-v0.6.1`)

直近完了した統合:
  Reason CLI 0.5.4 -> Harness Engine 0.6.1 (#484)

最後のunified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2
```

`v0.4.2`が最後のunified historical releaseです。split CLI lineは`reason-v0.5.4`（Harness Engine 0.6.1）まで進み、CLI / Engine / machine-contract identityは独立して進みます。`reason-v0.5.2`はEngine 0.4.2のままimmutableで、`reason-v0.5.3`が独立release済みEngine 0.5.0をadoptしました。[バージョニング](versioning.ja.md)を参照してください。

## 製品目標

Reasoning Harnessはgeneral-purpose agent frameworkより意図的に狭いproductです。

> ユーザー・開発者・automationが低摩擦でAIを使える一方、factual outputの境界はmodel confidenceではなく、Harness管理のevidence、verification、uncertainty、failure semanticsで決まる。

中心ルールは変わりません。

```text
model proposes
Harness verifies / qualifies / abstains
```

### Engine 0.6.1正式採用 — Reason CLI 0.5.4 (#484)

独立acceptance済みEngine 0.6.1を新しいCLI 0.5.4で配布します。署名provenance、更新とrollback時の明示Engine-change許可、既存CLI 0.5.3のimmutabilityを維持します。

## トラックA — Reason CLI 0.5.x: 一般利用向け製品化

**現在の公開pairは`reason-v0.5.4` / Harness Engine 0.6.1。** 以前の`reason-v0.5.3`はEngine 0.5.0、`reason-v0.5.2`はEngine 0.4.2のままimmutableで、#455・#484が別々のCLI coordinateでadoptionを完了しました。

base `reason-v0.5.0` と0.5.1 / 0.5.2 patch lineはEngine 0.4.2でrelease済みで、`reason-v0.5.3`はaccepted Engine 0.5.0を配布し、`reason-v0.5.4`ではEngine 0.6.1を配布しています。P0 release gateと#455 Engine adoptionは完了しました。残るP1はdistribution follow-up #375のみです。Homebrew 0.5.3 physical upgrade/test acceptanceは完了、WinGet community manifestはvalidation / CLAを通過してcommunity moderator approval待ちです。

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

## トラックC — Harness Engine 0.6.0: target-local evidence semantics — release完了

Engine 0.6.0は、Engine 0.5.0やfreeze済みhistorical observationを書き換えず、実測production-gap sequence #461 -> #462/#468 -> #463をpromotionします。

- **#460 structured-output transport helper:** bounded JSON-Schema -> JSON-object fallbackはtask/budget identityを維持し、evidence/authorityを生成しない。
- **#461 evidence need:** independent holdout run `35965160995`でMistral + Google 26/26、correctness violation 0、utility miss 0、provider failure 0。
- **#462/#468 relevance + relation:** final independent holdout-v12 run `37218652869`でMistral / Google / Groq 26/26、authority failure 0、wrong-target Relevant 0、false relevance rejection 0、utility miss 0。
- **#463 source attribution:** independent holdout-v2 run `37326666360`でMistral / Google / Groq 18/18、useful attribution 6/6、citation coverage 100%、provider failure 0、全hard gate 0。
- **#472 release closeout:** `reasoning-harness-core`を0.6.0へ進め`engine-v0.6.0`でreleaseする。release promotionではaccepted runtime semanticsを変更しない。公開済み`reason-v0.5.3`はEngine 0.5.0のまま保持し、その後別CLIの`reason-v0.5.4`がEngine 0.6.1を採用した。

## トラックD — Engine 0.6.0以降の保守（0.6.1公開済み）

Engine 0.6.0のmilestoneと`engine-v0.6.0` releaseは完了済みで、変更しません。そのtag以降にmainへmergeした変更は、**公開済みEngine 0.6.0のsource releaseには含まれません**。

- **#474 launch表現の限定的な修正:** 肯定形`has made … generally available`を順序・局所性がある場合にのみ認識し、否定形は引き続きfail closed。肯定形と複数の否定形を回帰テストで保護します。過去の0.6.0 acceptanceを遡及変更しません。
- **#475 / milestone #9 Engine 0.6.1 closeout:** `engine-v0.6.1`を独立source releaseしました。PR #480/#481/#482のexact-head CIはPASS、リリース候補は23/23。独立v2は30/30、Mistral/Google/Groqのliveは計36/36 PASS（run 37935245178）。以前の独立v1 FAILは変更せず保持します。
- **#476 wrong-target launch binding:** `has made … generally available`を対象自身に結び付け、他対象・節またぎ・否定の表現からadvisory model判断による誤昇格を防ぎます。
- **#479 launch根拠の事実性:** 最新v17/v30は対象自身の肯定的事実だけを正のrelationとして受け入れ、条件・否定・疑問・架空・未検証の主張ではfail closedにします。歴史的な評価版は変更しません。
- **#477 legacy MCP timeout:** 歴史的な評価に必要な凍結済みv1実装を変更せず、期限管理された後継v2/v3への移行を推奨します。v2のblocked-stdin回帰テストを追加し、旧v1の期限超過リスクは明示して管理します。この対応は0.6.1 **source release**の対象ですが、Reason CLIリリースは別です。
- **#465 CI fixture安定化:** legacy MCP subprocess transport testの断続的な失敗は、semantic correctnessやfreeze済みevaluationと分けて追跡します。
- **#467 / #470 依存関係保守:** 通常のCargo依存更新はEngineのsemantic acceptance記録とは別に管理します。
- **次のEngine source release（0.7.0計画）:** Track Eで#487のbaselineと#492の独立acceptanceを要求し、過去のfreeze tag・結果・release artifactを保持します。CLI adoptionは別releaseで行います。

## トラックE — Harness Engine 0.7.0: 複数根拠の整合（計画段階・未リリース）

**[マイルストーン #10](https://github.com/git-ksk/reasoning-harness/milestone/10) · [親Issue #486](https://github.com/git-ksk/reasoning-harness/issues/486) · [詳細ロードマップ](engine-0.7.0-roadmap.ja.md)。** 現在公開済みのEngineは引き続き **0.6.1** です。

Engine 0.6.1には、時点・scope・権威による適合判定、構造化事実の矛盾時のhard receipt保留、target別の根拠取得要否、厳密な出典付き説明、advisoryな充足性判定があります。0.7.0では**出典の系譜、明示的な改訂履歴、矛盾を維持する統合、target別の回答可否**が有用性を高めるかを検証します。現時点では実証済みの欠落ではありません。

1. **P0 [#487](https://github.com/git-ksk/reasoning-harness/issues/487)：** 0.6.1のbaseline・不足の事例・採点ルールを、新semantic実装の**前**に固定。
2. **P1 [#488](https://github.com/git-ksk/reasoning-harness/issues/488)：** 必要性が確認できた場合、転載元・独立した出所・由来不明を区別。出典の独立性を捏造しない。
3. **P1 [#489](https://github.com/git-ksk/reasoning-harness/issues/489)：** 必要性が確認できた場合、明示的な改訂関係とas-of・有効時刻を突き合わせる。「新しい方が正しい」は禁止。
4. **P1 [#490](https://github.com/git-ksk/reasoning-harness/issues/490)：** 必要性が確認できた場合、互換的な説明と不一致を区別し、衝突・引用・authority ceilingを保持。
5. **P1 [#491](https://github.com/git-ksk/reasoning-harness/issues/491)：** 必要性が確認できた場合、target別の必要情報で限定回答・予算内再取得・保留を制御。
6. **P0 [#492](https://github.com/git-ksk/reasoning-harness/issues/492)：** candidate/runnerを固定してから新規独立holdoutを作り、deterministicとMistral/Google/Groqの個別評価、exact-head CIをリリース条件にする。

**重大違反の許容数は0：** 根拠なしの`Known`/`Supported`、独立ソース・引用の捏造、対象・scope・時点の混線、未証明の新版優先、衝突の隠蔽、replayの外部副作用。実用性・コストの数値基準は#487後、独立評価の**前**に固定します。非互換なmachine/session contractは別IDと互換テストを必要とします。

これは**Engineの計画のみ**で、CLI 0.5.4の公開済みbinaryや過去のEngine評価は変更しません。実測で有用性を示せない機能候補は見送ります。

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
