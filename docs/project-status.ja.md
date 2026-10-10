# プロジェクト状況

日本語 | [English](project-status.md)

これは**現在地だけを短く読むためのstatus**です。ユーザーやcontributorが、長いresearch chronologyを読まなくても「今何がreleaseされ、何を進めているか」を把握できるようにしています。

従来の詳細なprovenance台帳は[Project status history](project-status-history.ja.md)へ保存しました。product docsとresearch evidenceの入口は[Documentation index](README.ja.md)です。

## 現在のリリース

現在公開済みのsplit CLI previewは **Reason CLI 0.5.4 / Harness Engine 0.6.1** です。従来の0.5.3 / Engine 0.5.0を変更しません。**Harness Engine 0.6.1** を`engine-v0.6.1`として独立source releaseしました。CLI 0.5.4への採用は独立した別リリースとして完了済みです。

```text
公開済みCLI: Reason CLI 0.5.4 / Harness Engine 0.6.1
最新Engine source release: Harness Engine 0.6.1 (`engine-v0.6.1`)
```

`v0.4.2`はCLI / Engineが同じSemVerを共有したimmutableな最後のunified releaseです。`reason-v0.5.0`で最初のsplit general-use CLIをreleaseし、0.5.1 / 0.5.2 patch lineはEngine 0.4.2で進めました。`reason-v0.5.3`では独立acceptance済みのEngine 0.5.0を新しいCLI coordinateでadoptしました。

中心となるtrust boundaryは変わりません。model outputはuntrusted candidate / rendererであり、evidence admission、qualification、verification、bounded resolution、最終的に表示するfactual claimのauthorityはHarness側が持ちます。最終freeze済みEngine 0.4.2 natural-language release evaluationは、Mistral、Groq、Gemini 3.5 Flash-Lite、Gemma 4 31Bを独立にPASSしています。詳細は[v36 release acceptance](natural-language-e2e-v36-result.ja.md)を参照してください。

## 今できること

supportedなnative product surfaceは:

- 引数なし`reason` — managed interactive terminal、`reason "TASK"` — one-shot natural-language verified path;
- `reason setup` / `reason auth ...` / provider・model・config command — guided setupとsecureな日常利用;
- `reason session ...` / `-c` / `-r` — reasoning stateの保存 / continue / resume;
- `reason doctor` — local diagnosticsと明示的なbounded live readiness check;
- `reason mcp ...` — acquisition-only boundaryを維持したlocal / remote read-only MCP lifecycle;
- `reason update` / explicit rollback / `reason uninstall` — provenance検証済みCLI lifecycle;
- `reason run` — structured candidate / application integration;
- `reason verify` — deterministic artifact validation;
- `reason semantic-check` — soft semantic diagnostics;
- `reason schema` — versioned machine contract;
- bounded external resolver acquisition;
- allowlist済みread-only MCP acquisition;
- 明示的にtrustedなdeterministic command verification;
- optional `reason-mcp` adapter。

Mistral、Google Gemini/AI Studio、NVIDIA Hosted NIM、Groqのprovider adapterを実装済みです。provider callが成功しただけで、そのmodel outputがverification authorityになることはありません。

## 現在の製品ライン: Reason CLI 0.5.x

最初のsplit general-use release `reason-v0.5.0` はrelease済みで、現在のpatch coordinateは `reason-v0.5.4` / Harness Engine 0.6.1です。旧`reason-v0.5.3`はEngine 0.5.0、`reason-v0.5.2`はEngine 0.4.2のままimmutableです。

0.5.0〜0.5.2のproductization artifactでは、固定Engine 0.4.2 baseline上で一般利用向けterminal UXを提供しました。CLI 0.5.3がそのproduct surfaceを維持しつつEngine 0.5.0を採用し、CLI 0.5.4ではさらにEngine 0.6.1を採用済みです。

- Rust toolchain不要のnative install;
- verified distributionとupdate / rollback / uninstall lifecycle;
- OS-native secure credential storageと`reason auth`;
- provider / model discoveryを含む`reason setup`;
- executable / authority-bearing configに対する明示project trust;
- 引数なしinteractive modeと簡単なcontinue / resume;
- verified / qualified / unknownが分かるhuman output;
- provider usage / budgetの可視化;
- guided read-only MCP management;
- actionableな`reason doctor`とrecovery-oriented error;
- private local session/stateとsubprocess secret isolation;
- supported platformでのfresh-install acceptance。

詳細なacceptance planは[Reason CLI 0.5.0 ロードマップ](reason-cli-0.5-roadmap.ja.md)にあります。

## 最新product統合: Reason CLI 0.5.4

CLI 0.5.4は#484で独立release済みEngine 0.6.1を採用しました。各OSの署名付きnative配布とEngine-changeの明示許可を維持し、公開済み0.5.3のEngine 0.5.0は不変です。

## 最新Engineパッチ: Harness Engine 0.6.1

`engine-v0.6.1`はv17/v30のlaunch関連性判定で別対象・条件・否定・疑問・架空・未検証の記述からの誤昇格を防ぎます。歴史的なfreeze評価は変更しません。独立v2は30/30、Mistral・Google・Groqのlive acceptanceは各12/12 PASS（誤Relevant 0、utility miss 0、provider failure 0、[run 37935245178](https://github.com/git-ksk/reasoning-harness/actions/runs/37935245178)）。リリースPR #482は23/23 CI PASS。旧MCP v1のstdin-write制限は残り、v2/v3への移行が必要です。公開済みCLI 0.5.4はEngine 0.6.1を搭載し、旧0.5.3はEngine 0.5.0を保持します。[0.6.1リリースノート](engine-0.6.1-release.ja.md)を参照してください。

## 別系統のエンジン開発: Harness Engine 0.6.0

reasoning / correctnessを変える仕事は、CLI UXとは意図的に分離します。

Harness Engine 0.6.0はrelease-completeです。accepted deltaはtarget-local evidence-need routing（#461）、evidence-target relevance / relation qualification（#462/#468）、truth promotionを行わないsource-attributed qualified prose（#463）です。#460のbounded structured-output fallbackはtransport compatibilityのみを改善し、authorityを生成しません。

promotion前に独立freeze済みacceptanceを完了しています。#461 holdout run `35965160995`はMistral + Google 26/26・correctness/utility failure 0、#462/#468 holdout-v12 run `37218652869`はMistral / Google / Groq 26/26・authority / wrong-target / false-rejection / utility failure 0、#463 holdout-v2 run `37326666360`はMistral / Google / Groq 18/18・citation coverage 100%・全hard gate 0でした。historical failureはimmutableのまま保持します。

`engine-v0.6.0`は独立Engine source releaseです。公開済み`reason-v0.5.3` binaryはEngine 0.5.0のままimmutableで、その後CLI 0.5.4がEngine 0.6.1を別releaseで採用しました。[Engine 0.6.0 release notes](engine-0.6.0-release.ja.md)、[バージョニング](versioning.ja.md)、[製品ロードマップ](product-roadmap.ja.md)を参照してください。

## 次期Engine 0.7.0計画 — 未リリース

[マイルストーン #10](https://github.com/git-ksk/reasoning-harness/milestone/10) / [詳細ロードマップ](engine-0.7.0-roadmap.ja.md)では、まずEngine 0.6.1の複数ソースに関する実測baselineを固定します。[開発用baseline v1結果](engine-0.7-baseline-v1-result.ja.md)では29/29の事前予測一致と、同義表現が保守的にConflict扱いされる1例を確認していますが、hardな誤判定を発見したとは主張しません。その結果に応じて、出典の系譜、明示改訂時点、矛盾を保つ統合、target別回答充足性を段階的に検証します。改善効果・独立評価・exact-head release gateを満たすまでは未実装候補を公開済みEngineに昇格しません。

**#496レビュー主体候補：** [ローカルOSキーチェーンと手動承認](engine-0.7-local-reviewer-v1.ja.md)を開発用コードとして追加、固定18件の決定論的テストはPASS。本人が行う物理動作と独立live評価が残り、公開済みCLI・Engineは不変です。

**#490の開発候補：** [信頼済みホスト照合view](engine-0.7-source-reconciliation-v1.ja.md)を別契約で追加。従来の保存済みConflict・引用・検証は不変。独立provider評価とCLI連携は未完了です。

## 現在の信頼境界

現在のproduct-levelな約束は次の通りです。

1. **Model outputはuntrusted。** modelが自分でclaimを`supported`と書いてもevidence / receipt / final authorityは作れない。
2. **Acquisitionとverificationは別。** retriever / resolver / MCP outputはadmit・verifyされるまで取得データ。
3. **Unknownは正常な結果。** support不足を埋めるためにconfident answerを捏造しない。
4. **Operational failureとepistemic uncertaintyを分離。** provider / quota / transport / protocol failureをsemantic `unknown`へ変換しない。
5. **最終factual textもauthorityにbinding。** rendererの流暢さで強いunsupported factを混ぜない。
6. **Historical evaluationは書き換えない。** freeze / observe済みstudyを後から修正して都合の良い結果にしない。

## 現在の主なgap

general-use CLI productizationはrelease済みです。native install、guided setup/auth、interactive / continue / resume、config/model/MCP discovery、progress/cancellation、diagnostics、private local state、lifecycle managementは現在の0.5.x product lineに含まれます。

package-manager distribution #375は実機acceptanceを完了しました。Homebrew 0.5.3のupgrade / testはPASSし、WinGet community manifestもReason CLI 0.5.2（初回）と0.5.4（更新）がmoderator承認・merge・publish済みです。Windows実機ではdefault WinGet indexから0.5.4を取得でき、fresh install・SHA-256照合・`reason --version`・provenance付き非破壊update checkをPASSしました。`doctor`のalias path診断修正は別途#504で追跡し、既存0.5.4 binaryには未反映です。

Harness Engine 0.6.0は#461/#462/#468/#463まで実装・acceptance・release完了で、`engine-v0.6.0`として独立公開します。現在配布中のCLIはReason CLI 0.5.4 / Engine 0.6.1です。旧CLI 0.5.3のEngine 0.5.0は保持します。#465はEngine correctness blockerではなく、non-semanticなCI fixture reliability follow-upとして継続します。

## 研究方針

Reasoning Harnessはopen-world reasoningを解決したとは主張しません。hard correctnessには、hard answerが存在する範囲でdeterministic structureとtrusted evidence / oracleが必要です。model-backed semantic mechanismは、別途authority-safeなpromotionを通さない限りadvisory / restrictiveです。

過去のsemantic study、holdout、RSD、provider investigation、product dogfood、natural-language E2Eの詳細は[Project status history](project-status-history.ja.md)と[Documentation index](README.ja.md)を参照してください。
