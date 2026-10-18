# 製品ロードマップ: Reason CLI / Harness Engine

**実資料評価の接続準備（2026-10-19）:** 通常CLI保存記録と全文資料・引用位置の照合に加え、既存Node開発入力で対象APIを呼ぶ実験hostを保存した。0.6.1/現在候補とも関連性は保留・引用0。別の手動binding対照では出典付き原文表示を確認し、関連性成功へ混ぜない。新規独立問題0・実回答F1未測定。次は意味/引用裁定・欠測会計・数値予算を完全M0へ結ぶ。[実験host](engine073-active-source-host-development.ja.md)、[通常CLI記録の照合](engine073-cli-trace-binding-development.ja.md)。

**評価運用の見直し（2026-10-19）:** 合否は事前固定した独立F1/P/Rと安全性で判断する。既知対照の件数・満点は完成度にしない。着手済み6件の修正後は合成60件の追加・修正ループを止め、実資料・通常CLIの経路/traceと完全採点器へ重点を戻す。既知回帰は再発防止対象を著作前に定義し、新holdoutの全FNを自動で100%必須へ追加しない。凍結初回は不変。[詳細](engine073-evaluation-focus.ja.md)。

日本語 | [English](product-roadmap.md)

これは**これから進む方向だけを読むためのroadmap**です。releaseごとの詳細な実装台帳は[Product roadmap history](product-roadmap-history.ja.md)へ保存しました。

現在は、CLI product UXとreasoning / correctness changeを分離しています。CLIが使いやすくなったことを、reasoning engineが変わったことと混同しないためです。

## 現在のバージョン座標

```text
公開済みsplit CLI:
  Reason CLI 0.5.5
  Harness Engine 0.6.1

最新の独立Engine source release:
  Harness Engine 0.7.2 (`engine-v0.7.2`)

直近完了した統合:
  Reason CLI 0.5.4 -> Harness Engine 0.6.1 (#484)

最後のunified historical release:
  Reason CLI 0.4.2
  Harness Engine 0.4.2
```

`v0.4.2`が最後のunified historical releaseです。split CLI lineは`reason-v0.5.5`（Harness Engine 0.6.1）まで進み、CLI / Engine / machine-contract identityは独立して進みます。`reason-v0.5.2`はEngine 0.4.2のままimmutableで、`reason-v0.5.3`が独立release済みEngine 0.5.0をadoptしました。[バージョニング](versioning.ja.md)を参照してください。

## 次の開発 — Engine0.7.3の修正と再評価

[0.7.3ロードマップ](engine-0.7.3-roadmap.ja.md)／[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)／[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)で、**#520修正 → 安全なrunner・通常経路の接続確認 → 小規模開発確認 → 新規独立の実資料受入 → ソース公開 → 専用CLI採用・配布**の順に進めます。0.7.3は未公開の計画です。0.7.2の公開物・比較結果は保持し、残存不具合があるため採用不可です。#488/#489/#491は実測で必要な場合だけ実装します。

[初回v2/v3の結果](engine07-realworld-development-results.ja.md)は不合格・不完全のままです。#535/#537/#541はmerge済み、共有runnerは47オフラインテストと限定独立再レビューを完了したが、live準備は未完了。Groqは日次枯渇のため追加起動を止めています。通常回答の改善と新規独立受入は未確認、採用1/6は開発比較・限界・NO-GO記録のみです。公開CLI0.5.5 / Engine0.6.1と#506の配布受入は別工程です。

修正候補 `ab3c632` は、同じ観測済みv27の開発評価で両API TP30/FP0/TN30/FN0、P/R/F1=1.000。元の独立v27初回F1 .888889/Recall .8/NO_GOは変更しない。完全な印字属性・同roleのowner/複数参照・括弧内の属性限定を共通処理で修正した。修正した不具合と安全性境界の退行を確認し、全体1605PASS/0FAIL/既存ignore8・strict Clippy/fmt、元14・歴史597byte互換を保存。過去1400は同じsourceの通常成果物で一度確認し、rootでは必要な60/14/597照合に絞った。件数・開発満点を完成度にせず、次の合成60件は作らない。次は実資料・通常CLIのfeature-active経路/traceと完全採点器・数値予算へ重点を戻す。主張算術/表示binding部品32検証は準備のみ、独立部品レビューは利用上限で未完了。全928原参照は不変。実24問0/24・実回答F1未測定・Groq停止、sourceマージ/Engine公開/CLI採用NO_GO。[方針](engine073-evaluation-focus.ja.md)、[保存済み修正](../evaluation/engine073-launch-development-v27-bounded-properties/README.md)、[原独立判定](engine073-v27-decision.ja.md)。

[実回答の採点準備](engine073-realworld-metrics-preparation.ja.md)で主張grounding Precision、必要主張Recall/F1、引用、正しい/過剰保留、有用性、運用欠測、provider別会計を定義した。PREPARATION_ONLY、M0/M1未固定・実回答未採点。新scorer、実host callback/dependency closure・数値予算/headroom・金額自己上限、featureが働く通常CLI経路が残る。[過去候補01dcの同一host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md)は固定Node1経路で両Engineの同じgrounded回答・取得1・関連性API呼出し0、直接v17/v30対照ではcounter動作を確認した。全経路の不活性や今回修正の実回答効果を確定した結果ではない。#541共有runnerはmerge済みだが#540のlive準備はOPEN。#531新規実24問0/24・採用1/6、Groq追加起動禁止、provider/model/Oracle追加0を維持する。

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

**現在の公開pairは`reason-v0.5.5` / Harness Engine 0.6.1。** 以前の`reason-v0.5.3`はEngine 0.5.0、`reason-v0.5.2`はEngine 0.4.2のままimmutableで、#455・#484が別々のCLI coordinateでadoptionを完了しました。

base `reason-v0.5.0` と0.5.1 / 0.5.2 patch lineはEngine 0.4.2でrelease済みで、`reason-v0.5.3`はaccepted Engine 0.5.0を配布し、`reason-v0.5.4`ではEngine 0.6.1を配布しています。P0 release gateと#455 Engine adoptionは完了しました。#375 P1 distribution acceptance条件も検証済みです。Homebrew 0.5.3の実機testはPASSし、MicrosoftはWinGet package `git-ksk.Reason`（最新0.5.4）を公開、Windows実機でのfresh installもPASSしました。`doctor`のlink検出補正は別途#504で追跡します。

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
- **Engine 0.7ソース系列:** 0.7.0で出典照合の採用範囲を公開し、0.7.1でマージ済み#512/#514/#515修正をまとめます。配布CLIへの採用は別リリースです。

## トラックE — Harness Engine 0.7: 修正パッチと保留中の仮説

[Engine 0.7.0](engine-0.7.0-release.ja.md)は、#487 baseline・#496の承認ホスト制御・#492の固定した独立合成評価後、#490の任意のホスト承認付き出典照合・対象別表示を公開しました。元のConflict・claim・引用は維持し、モデルの助言はレビュー許可や外部truthを作りません。同じOSユーザーの自動keyring E2Eは、独立した人間の承認証明ではありません。

[Engine 0.7.1](engine-0.7.1-release.ja.md)は、マージ済み#512の日英準備記述・#514/#515の発売記述修正をまとめます。歴史的v16/v29、凍結観測、machine契約を維持します。パッチ固有の検証と限界は0.7.0のprovider観測と分けて記録しています。

**[Engine 0.7.2](https://github.com/git-ksk/reasoning-harness/releases/tag/engine-v0.7.2)をEngine単独の通常ソースパッチとして公開済みです。** [リリースPR #525](https://github.com/git-ksk/reasoning-harness/pull/525)の公開commitは`296690a86e049b75ee693ae5032a71504366137c`。#523で修正した[#520](https://github.com/git-ksk/reasoning-harness/issues/520)の架空・撤回・偽の上位文脈と、[#521](https://github.com/git-ksk/reasoning-harness/issues/521)の元の取り消し線・否定記号をv17/v30で保持します。最終候補はworkspace 1490成功・失敗0・ignore 8、strict Clippy・format・fixture20件・元archive13件を通過。最終PR head CI 23/23、マージ後CI 22/22も成功。受け入れ済みの新規独立v10 50/50、旧v16/v29の597入力byte一致を保持。[リリースノート](engine-0.7.2-release.ja.md)／[修正報告](launch-context-markers-520-521.md)／[0.7.2ロードマップ](engine-0.7.2-roadmap.ja.md)を参照。[#524](https://github.com/git-ksk/reasoning-harness/issues/524)と[Milestone #11](https://github.com/git-ksk/reasoning-harness/milestone/11)で公開完了を記録します。過去tag・初回失敗は不変。現在の公開CLIは0.5.5 / Engine 0.6.1で、合成・ローカル・CIの成功を新しいlive-providerの正しさ、外部truth、実ユーザーの有用性の実証とは扱いません。

[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10)／[親#486](https://github.com/git-ksk/reasoning-harness/issues/486)は、保留した[#488の出典系譜](https://github.com/git-ksk/reasoning-harness/issues/488)、[#489の信頼済み改訂関係](https://github.com/git-ksk/reasoning-harness/issues/489)、[#491の追加回答可否・再取得](https://github.com/git-ksk/reasoning-harness/issues/491)のためOPENです。新機構を必要とする具体的な製品上の失敗は未実証で、実装前に必要性の実測と別acceptanceの固定が必要です。限定的なソースパッチの公開は妨げません。

修正した2件が0.7.2の修正範囲です。保留3件は実験が必要な改善案で、追加の既知不具合3件ではありません。ほかの不具合が皆無とするものでもありません。#465の旧MCP CI安定性と#506のCLI配布検証は別工程です。

公開済み製品はCLI 0.5.5 / Engine 0.6.1です。#506にはWinGetなどの別配布受入を残します。Engine 0.7のCLI採用は#531の評価後に判断します。

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
