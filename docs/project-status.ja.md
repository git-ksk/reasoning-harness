# プロジェクト状況

日本語 | [English](project-status.md)

これは**現在地だけを短く読むためのstatus**です。ユーザーやcontributorが、長いresearch chronologyを読まなくても「今何がreleaseされ、何を進めているか」を把握できるようにしています。

従来の詳細なprovenance台帳は[Project status history](project-status-history.ja.md)へ保存しました。product docsとresearch evidenceの入口は[Documentation index](README.ja.md)です。

## 現在のリリース

現在のtagged external previewは **Reasoning Harness v0.4.2** です。

user-facingなReason CLIとHarness Engineが同じSemVerを共有する最後のreleaseです。

```text
Reason CLI 0.4.2
Harness Engine 0.4.2
```

中心となるtrust boundaryは変わりません。model outputはuntrusted candidate / rendererであり、evidence admission、qualification、verification、bounded resolution、最終的に表示するfactual claimのauthorityはHarness側が持ちます。

最終freeze済みv0.4.2 natural-language release evaluationは、Mistral、Groq、Gemini 3.5 Flash-Lite、Gemma 4 31Bを独立にPASSしています。詳細は[v36 release acceptance](natural-language-e2e-v36-result.ja.md)を参照してください。

## 今できること

supportedなnative product surfaceは:

- `reason "TASK"` — natural-language verified path;
- `reason session ...` — reasoning stateの保存 / resume;
- `reason run` — structured candidate / application integration;
- `reason verify` — deterministic artifact validation;
- `reason semantic-check` — soft semantic diagnostics;
- `reason schema` — versioned machine contract;
- bounded external resolver acquisition;
- allowlist済みread-only MCP acquisition;
- 明示的にtrustedなdeterministic command verification;
- optional `reason-mcp` adapter。

Mistral、Google Gemini/AI Studio、NVIDIA Hosted NIM、Groqのprovider adapterを実装済みです。provider callが成功しただけで、そのmodel outputがverification authorityになることはありません。

## 現在の製品開発: Reason CLI 0.5.0

次の一般向けproduct lineは:

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

を予定しています。

目的は、**Engine 0.4.2のreasoning / correctness semanticsを変えずに、普通のterminalユーザーが使いやすいproductへ仕上げること**です。

主な0.5.0 productization:

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

## 主なユーザー向け課題

core correctness/runtimeは、現在のonboarding UXよりかなり先に成熟しています。一般利用へ向けた主なgapは基本機構不足ではなく、product UX / distributionです。

- installはrelease archiveまたはRust/Cargo workflowが中心;
- current releaseのcredential UXはenvironment variable中心;
- first-run setupがguidedではない;
- interactive / continue / resumeは成熟AI CLIより低レベル;
- config / MCP / provider discoveryとrecovery UXが未productized;
- progress、diagnostics、local-state management、update/uninstall lifecycleを0.5.0でhardening中。

これらは隠れた制約ではなく、次のReason CLI lineの明示的な対象です。

## 研究方針

Reasoning Harnessはopen-world reasoningを解決したとは主張しません。hard correctnessには、hard answerが存在する範囲でdeterministic structureとtrusted evidence / oracleが必要です。model-backed semantic mechanismは、別途authority-safeなpromotionを通さない限りadvisory / restrictiveです。

過去のsemantic study、holdout、RSD、provider investigation、product dogfood、natural-language E2Eの詳細は[Project status history](project-status-history.ja.md)と[Documentation index](README.ja.md)を参照してください。
