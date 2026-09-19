# プロジェクト状況

日本語 | [English](project-status.md)

これは**現在地だけを短く読むためのstatus**です。ユーザーやcontributorが、長いresearch chronologyを読まなくても「今何がreleaseされ、何を進めているか」を把握できるようにしています。

従来の詳細なprovenance台帳は[Project status history](project-status-history.ja.md)へ保存しました。product docsとresearch evidenceの入口は[Documentation index](README.ja.md)です。

## 現在のリリース

現在のtagged split external previewは **Reason CLI 0.5.2 / Harness Engine 0.4.2** です。

```text
Reason CLI 0.5.2
Harness Engine 0.4.2
```

`v0.4.2`はCLI / Engineが同じSemVerを共有したimmutableな最後のunified releaseです。`reason-v0.5.0`で最初のsplit general-use CLIをreleaseし、0.5.1 / 0.5.2 patch lineもEngine 0.4.2のreasoning / authority semanticsを変えずに進めています。

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

最初のsplit general-use release `reason-v0.5.0` はrelease済みで、現在のpatch coordinateは `reason-v0.5.2`、Harness Engineは引き続き0.4.2です。

0.5.x productizationでは、**Engine 0.4.2のreasoning / correctness semanticsを変えずに**次を提供しています。

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

## 別系統のエンジン開発: Harness Engine 0.5.0

reasoning / correctnessを変える仕事は、CLI UXとは意図的に分離します。

Harness Engine 0.5.0は、reasoning utilityやcorrectness behaviorへ影響しうるverified-investigation系の変更を担当します。この種の変更はfresh evaluationが必要で、CLI UX releaseへ紛れ込ませません。

[バージョニング](versioning.ja.md)と[製品ロードマップ](product-roadmap.ja.md)を参照してください。

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

product側で残るdistribution follow-upは#375です。Homebrew physical acceptanceは完了し、WinGet community manifestもMicrosoft validation / CLAを通過してcommunity moderator approval待ちです。この外部reviewはHarness Engine開発をblockしません。

Harness Engine 0.5.0のsemantic lineは#248 / #282 / #283の実装と、最終closeout #443まで完了しました。fresh `engine-0.5-final-v2-freeze` matrixではvalidated-required 4 rowがすべて独立にPASSし、observed correctness-boundary violationは0でした。observed / limited rowはrelease voteではなくcharacterization evidenceとして保持します。詳細は[Engine 0.5.0 final cross-model result](engine-0.5-final-v2-result.ja.md)。package上のEngine coordinateは、別途versioned releaseを行うまでは0.4.2のままです。

## 研究方針

Reasoning Harnessはopen-world reasoningを解決したとは主張しません。hard correctnessには、hard answerが存在する範囲でdeterministic structureとtrusted evidence / oracleが必要です。model-backed semantic mechanismは、別途authority-safeなpromotionを通さない限りadvisory / restrictiveです。

過去のsemantic study、holdout、RSD、provider investigation、product dogfood、natural-language E2Eの詳細は[Project status history](project-status-history.ja.md)と[Documentation index](README.ja.md)を参照してください。
