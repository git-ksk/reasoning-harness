# プロジェクト状況

日本語 | [English](project-status.md)

これは**現在地だけを短く読むためのstatus**です。ユーザーやcontributorが、長いresearch chronologyを読まなくても「今何がreleaseされ、何を進めているか」を把握できるようにしています。

従来の詳細なprovenance台帳は[Project status history](project-status-history.ja.md)へ保存しました。product docsとresearch evidenceの入口は[Documentation index](README.ja.md)です。

## 現在のリリース

現在公開済みのtagged split CLI previewは **Reason CLI 0.5.3 / Harness Engine 0.5.0** のままです。**Harness Engine 0.6.0** はaccepted semantic lineから`engine-v0.6.0`として独立releaseし、Reason CLIによるEngine 0.6.0 adoptionは別の将来releaseとして扱います。

```text
公開済みCLI: Reason CLI 0.5.3 / Harness Engine 0.5.0
最新Engine source release: Harness Engine 0.6.0 (`engine-v0.6.0`)
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

最初のsplit general-use release `reason-v0.5.0` はrelease済みで、現在のpatch coordinateは `reason-v0.5.3`、Harness Engineは0.5.0です。`reason-v0.5.2`はEngine 0.4.2のままimmutableで、新しいEngineは0.5.3という別releaseでのみadoptしています。

0.5.0〜0.5.2のproductization artifactでは、固定Engine 0.4.2 baseline上で一般利用向けterminal UXを提供しました。Reason CLI 0.5.3はそのproduct surfaceを維持しつつ、別trackでaccept済みのEngine 0.5.0をadoptしています。

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

## 別系統のエンジン開発: Harness Engine 0.6.0

reasoning / correctnessを変える仕事は、CLI UXとは意図的に分離します。

Harness Engine 0.6.0はrelease-completeです。accepted deltaはtarget-local evidence-need routing（#461）、evidence-target relevance / relation qualification（#462/#468）、truth promotionを行わないsource-attributed qualified prose（#463）です。#460のbounded structured-output fallbackはtransport compatibilityのみを改善し、authorityを生成しません。

promotion前に独立freeze済みacceptanceを完了しています。#461 holdout run `35965160995`はMistral + Google 26/26・correctness/utility failure 0、#462/#468 holdout-v12 run `37218652869`はMistral / Google / Groq 26/26・authority / wrong-target / false-rejection / utility failure 0、#463 holdout-v2 run `37326666360`はMistral / Google / Groq 18/18・citation coverage 100%・全hard gate 0でした。historical failureはimmutableのまま保持します。

`engine-v0.6.0`は独立Engine source releaseです。公開済み`reason-v0.5.3` binaryは別のCLI adoption coordinateをreleaseするまでEngine 0.5.0のままimmutableです。[Engine 0.6.0 release notes](engine-0.6.0-release.ja.md)、[バージョニング](versioning.ja.md)、[製品ロードマップ](product-roadmap.ja.md)を参照してください。

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

product側で残るdistribution follow-upは#375です。Homebrew 0.5.3 physical upgrade/test acceptanceは完了し、WinGet community manifestもMicrosoft validation / CLAを通過してcommunity moderator approval待ちです。この外部reviewはHarness Engine開発をblockしません。

Harness Engine 0.6.0は#461/#462/#468/#463まで実装・acceptance・release完了で、`engine-v0.6.0`として独立公開します。現在distribution済みCLIはReason CLI 0.5.3 / Engine 0.5.0のままで、Engine 0.6.0 adoptionは意図的に別product releaseとします。#465はEngine correctness blockerではなく、non-semanticなCI fixture reliability follow-upとして継続します。

## 研究方針

Reasoning Harnessはopen-world reasoningを解決したとは主張しません。hard correctnessには、hard answerが存在する範囲でdeterministic structureとtrusted evidence / oracleが必要です。model-backed semantic mechanismは、別途authority-safeなpromotionを通さない限りadvisory / restrictiveです。

過去のsemantic study、holdout、RSD、provider investigation、product dogfood、natural-language E2Eの詳細は[Project status history](project-status-history.ja.md)と[Documentation index](README.ja.md)を参照してください。
