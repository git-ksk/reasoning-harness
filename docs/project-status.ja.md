# プロジェクト状況

日本語 | [English](project-status.md)

これは**現在地だけを短く読むためのstatus**です。ユーザーやcontributorが、長いresearch chronologyを読まなくても「今何がreleaseされ、何を進めているか」を把握できるようにしています。

従来の詳細なprovenance台帳は[Project status history](project-status-history.ja.md)へ保存しました。product docsとresearch evidenceの入口は[Documentation index](README.ja.md)です。

## 現在のリリース

現在公開済みのsplit CLIは **Reason CLI 0.5.5 / Harness Engine 0.6.1** です。従来の0.5.3 / Engine 0.5.0を変更しません。独立Engineソースの通常パッチは **Harness Engine 0.7.2**（`engine-v0.7.2`）です。CLI 0.5.5はEngine 0.6.1を継続します。[0.7.2リリースノート](engine-0.7.2-release.ja.md)を参照してください。

```text
公開済みCLI: Reason CLI 0.5.5 / Harness Engine 0.6.1
最新Engine source release: Harness Engine 0.7.2 (`engine-v0.7.2`)
```

`v0.4.2`はCLI / Engineが同じSemVerを共有したimmutableな最後のunified releaseです。`reason-v0.5.0`で最初のsplit general-use CLIをreleaseし、0.5.1 / 0.5.2 patch lineはEngine 0.4.2で進めました。`reason-v0.5.3`では独立acceptance済みのEngine 0.5.0を新しいCLI coordinateでadoptしました。

中心となるtrust boundaryは変わりません。model outputはuntrusted candidate / rendererであり、evidence admission、qualification、verification、bounded resolution、最終的に表示するfactual claimのauthorityはHarness側が持ちます。最終freeze済みEngine 0.4.2 natural-language release evaluationは、Mistral、Groq、Gemini 3.5 Flash-Lite、Gemma 4 31Bを独立にPASSしています。詳細は[v36 release acceptance](natural-language-e2e-v36-result.ja.md)を参照してください。

## 次の開発 — Engine0.7.3の修正と再評価

[0.7.3ロードマップ](engine-0.7.3-roadmap.ja.md)／[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)／[評価 #531](https://github.com/git-ksk/reasoning-harness/issues/531)で、**#520修正 → 安全なrunner・通常経路の接続確認 → 小規模開発確認 → 新規独立の実資料受入 → ソース公開 → 専用CLI採用・配布**の順に進めます。0.7.3は未公開の計画です。0.7.2の公開物・比較結果は保持し、残存不具合があるため採用不可です。#488/#489/#491は実測で必要な場合だけ実装します。

[初回v2/v3の結果](engine07-realworld-development-results.ja.md)は不合格・不完全のままです。#535の診断保持・#537の停止制御・#541の共有runnerはmerge済みですが、live受入は未完了。Groqは記録済みの日次枯渇により追加起動を止めています。通常回答の改善と新規独立受入は未確認、採用1/6は開発比較・限界・NO-GO記録のみです。公開CLI0.5.5 / Engine0.6.1と#506の配布受入は別工程です。

候補`9505300`の既知v24修正は同じ60件の開発比較で両API F1=1.000、既知1264・元監査14成功、歴史597全出力byte一致、全体1582成功/失敗0/既存ignore8、strict Clippy/fmtを確認した。しかし、新しい独立v25の初回は **FAIL/NO_GO**。両APIともTP5/FP0/TN30/FN25、Precision1.000・Recall0.166667・F1 0.285714で、陽性30件中25件を保留した。否定誤採用・運用失敗・欠測は各0。#547の評価器/方針を著作前に固定し、Engine60件一process・scorer/helper各1回で観測、再実行/再採点/再ラベルなし。元v24と今回v25の初回は不変。修正に使った集合のF1=1だけでは受入できない。次は今回の25保留の実経路を計測し、役割・訂正先・ownerの共通処理を修正する。実24問0/24・実回答F1未測定、sourceマージ/公開/CLI採用NO-GO。[判定と次の順序](engine073-v25-decision.ja.md)を参照。

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

最初のsplit general-use release `reason-v0.5.0` はrelease済みで、現在のpatch coordinateは `reason-v0.5.5` / Harness Engine 0.6.1です。旧`reason-v0.5.3`はEngine 0.5.0、`reason-v0.5.2`はEngine 0.4.2のままimmutableです。

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

## 以前のEngineパッチ: Harness Engine 0.6.1

`engine-v0.6.1`はv17/v30のlaunch関連性判定で別対象・条件・否定・疑問・架空・未検証の記述からの誤昇格を防ぎます。歴史的なfreeze評価は変更しません。独立v2は30/30、Mistral・Google・Groqのlive acceptanceは各12/12 PASS（誤Relevant 0、utility miss 0、provider failure 0、[run 37935245178](https://github.com/git-ksk/reasoning-harness/actions/runs/37935245178)）。リリースPR #482は23/23 CI PASS。旧MCP v1のstdin-write制限は残り、v2/v3への移行が必要です。公開済みCLI 0.5.4はEngine 0.6.1を搭載し、旧0.5.3はEngine 0.5.0を保持します。[0.6.1リリースノート](engine-0.6.1-release.ja.md)を参照してください。

## 別系統のエンジン開発: Harness Engine 0.6.0

reasoning / correctnessを変える仕事は、CLI UXとは意図的に分離します。

Harness Engine 0.6.0はrelease-completeです。accepted deltaはtarget-local evidence-need routing（#461）、evidence-target relevance / relation qualification（#462/#468）、truth promotionを行わないsource-attributed qualified prose（#463）です。#460のbounded structured-output fallbackはtransport compatibilityのみを改善し、authorityを生成しません。

promotion前に独立freeze済みacceptanceを完了しています。#461 holdout run `35965160995`はMistral + Google 26/26・correctness/utility failure 0、#462/#468 holdout-v12 run `37218652869`はMistral / Google / Groq 26/26・authority / wrong-target / false-rejection / utility failure 0、#463 holdout-v2 run `37326666360`はMistral / Google / Groq 18/18・citation coverage 100%・全hard gate 0でした。historical failureはimmutableのまま保持します。

`engine-v0.6.0`は独立Engine source releaseです。公開済み`reason-v0.5.3` binaryはEngine 0.5.0のままimmutableで、その後CLI 0.5.4がEngine 0.6.1を別releaseで採用しました。[Engine 0.6.0 release notes](engine-0.6.0-release.ja.md)、[バージョニング](versioning.ja.md)、[製品ロードマップ](product-roadmap.ja.md)を参照してください。

## Engine 0.7ソースリリースと保留中の作業

[Engine 0.7.0](engine-0.7.0-release.ja.md)は#490の任意のホスト承認付き出典照合・対象別表示を、固定した独立合成評価とともに公開しました。ローカル承認CLIは開発用です。同じOSユーザーの自動入力によるkeyring E2Eは、独立した人間の承認証明ではありません。

[Engine 0.7.1](engine-0.7.1-release.ja.md)はマージ済み#512/#514/#515の後継v17/v30修正をまとめます。歴史的v16/v29と凍結済みの結果は維持します。パッチ固有の検証は決定論的・合成データによるもので、新しいlive-provider・実ユーザーacceptanceは主張しません。

**[Engine 0.7.2](https://github.com/git-ksk/reasoning-harness/releases/tag/engine-v0.7.2)をEngine単独の通常ソースパッチとして公開済みです。** [リリースPR #525](https://github.com/git-ksk/reasoning-harness/pull/525)の公開commitは`296690a86e049b75ee693ae5032a71504366137c`。#523で修正した[#520](https://github.com/git-ksk/reasoning-harness/issues/520)の架空・撤回・偽の上位文脈と、[#521](https://github.com/git-ksk/reasoning-harness/issues/521)の元の取り消し線・否定記号をv17/v30で保持します。最終候補はworkspace 1490成功・失敗0・ignore 8、strict Clippy・format・fixture20件・元archive13件を通過。最終PR head CI 23/23、マージ後CI 22/22も成功。受け入れ済みの新規独立v10 50/50、旧v16/v29の597入力byte一致を保持。[リリースノート](engine-0.7.2-release.ja.md)／[修正報告](launch-context-markers-520-521.md)／[0.7.2ロードマップ](engine-0.7.2-roadmap.ja.md)を参照。[#524](https://github.com/git-ksk/reasoning-harness/issues/524)と[Milestone #11](https://github.com/git-ksk/reasoning-harness/milestone/11)で公開完了を記録します。過去tag・初回失敗は不変。現在の公開CLIは0.5.5 / Engine 0.6.1で、合成・ローカル・CIの成功を新しいlive-providerの正しさ、外部truth、実ユーザーの有用性の実証とは扱いません。

[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10)には保留した#488の出典系譜、#489の信頼済み改訂関係、#491の追加回答可否・再取得を残し、親#486から実証待ちの研究と新しい修正パッチを案内します。3仮説の採用には必要性の実測と別acceptanceが必要です。新規2件が現時点で再現できた修正バックログであり、ほかの不具合が皆無とするものではありません。CLI 0.5.5 / Engine 0.6.1は公開済みで、#506のWinGetなどの配布受入は別工程です。

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

Harness Engine 0.6.0は#461/#462/#468/#463まで実装・acceptance・release完了で、`engine-v0.6.0`として独立公開します。現在配布中のCLIはReason CLI 0.5.5 / Engine 0.6.1です。旧CLI 0.5.3のEngine 0.5.0は保持します。#465はEngine correctness blockerではなく、non-semanticなCI fixture reliability follow-upとして継続します。

## 研究方針

Reasoning Harnessはopen-world reasoningを解決したとは主張しません。hard correctnessには、hard answerが存在する範囲でdeterministic structureとtrusted evidence / oracleが必要です。model-backed semantic mechanismは、別途authority-safeなpromotionを通さない限りadvisory / restrictiveです。

過去のsemantic study、holdout、RSD、provider investigation、product dogfood、natural-language E2Eの詳細は[Project status history](project-status-history.ja.md)と[Documentation index](README.ja.md)を参照してください。
