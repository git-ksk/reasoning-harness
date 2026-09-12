# Reason CLI 0.5.0 一般利用向け製品化ロードマップ

Reason CLI 0.5.0は、Harness Engineと独立してversioningする最初のproduct lineです。受け入れ済みの **Harness Engine 0.4.2** を一般のterminal userが使いやすい製品へ仕上げますが、v0.4.2のreasoning / authority / admission / verification / finalization / answer-safety / MCP non-promotion / session replay boundaryは変更しません。

`v0.4.2`は最後のunified historical releaseとして固定します。最初のsplit release予定は次の形です。

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

目的はcoding agentをコピーすることではありません。成熟したAI CLIで期待される低摩擦UXを、Reason固有の厳しいboundaryを保ったまま実現します。toolchain不要install、明示的project trust、guided secure auth、引数なしinteractive mode、理解しやすいverified evidence、continue/resume、provider usage可視化、provider/model/config/MCPのdiscoverability、actionable diagnostics、private local state、検証可能で可逆なupdate/uninstallを対象にします。

Tracking: milestone **Reason CLI 0.5.0 — General-use Productization** (#6)、parent Issue #359。

## 利用者から見た流れ

### 初回利用

```text
検証済みnative Reason binaryをinstall
        |
        v
projectへ入る -> trust前はexecutable/authority configを有効化しない
        |
        v
reason setup
        |
        +--> provider/modelを選択
        +--> credentialをOS-native secret storageへ保存
        +--> bounded connectivity/readiness check
        |
        v
reason "TASK"
```

初回userにRust、Cargo、shell profile編集、平文credential file、内部Harness JSON contractの知識、unsigned/tampered binaryを通すためのOS security bypassを要求しません。

### 日常利用

```text
reason
  -> interactive session
  -> follow-up / untrusted file context追加
  -> verified fact / unresolved item / sourceを確認
  -> Ctrl+Cで現在の処理を安全にcancel
  -> exit

reason -c
  -> 最新compatible sessionをcontinue

reason -r <session>
  -> 選択したsessionをresume
```

terminalには`Planning`、`Acquiring`、`Verifying`、`Finalizing`のようなHarness-owned lifecycle stateを表示できます。これは進捗表示であり、hidden chain-of-thoughtではありません。通常のhuman outputでは、model proseをauthorityへ昇格させず、verified fact / uncertainty / admitted source provenanceを理解できる形で示します。

### 復旧

```text
reason doctor
reason auth status
reason config sources
reason mcp test <name>
reason update --check
```

user-facing operational errorは「何が失敗したか」「taskが実行されたか」「得られた結果を信用してよいか」「次に実行すべき安全なcommand」を説明します。

## フェーズ0 — 製品 / バージョン境界

- **#355 — 完了:** Reason CLIとHarness EngineのSemVer座標を分離。
- `v0.4.2`はimmutableな最後のunified tag。
- 今後のCLI releaseは`reason-vX.Y.Z`。
- machine contract identityは独立したcompatibility座標として維持。

## フェーズ1 — インストール、trust、認証、最初の回答まで

### 配布まわり — #358

- **#371 P0 — 完了:** macOS / Linux / Windows向けone-command native installer。
- **#382 P0 — 完了:** release provenance、必要に応じたcode signing/notarization、trusted installer/updater verification。SHA-256は維持するが唯一のtrust rootにはしない。
- **#372 P0 — 完了:** provenance検証済みupdate、明示的rollback、保持defaultのuninstall lifecycle。
- **#375 P1:** canonical installer/update contract安定後のHomebrew / winget channel。

### プロジェクトtrust — #377（完了）

project `.reason/config.json`にはexecutable/authority-bearing acquisition設定を置けるため、untrusted cloneのcwdへ移動しただけで有効化してはいけません。trustは明示的・inspectable・revocableで、canonical path identityを考慮し、non-interactiveでもfail-closedに扱います。trust後にexecutable/authority-bearing configが変わった場合は永久blanket approvalを引き継がず、関連trust fingerprintを失効または再承認します。`reason trust status/add/list/revoke`、canonical project identity、直接executableのSHA-256 binding、project `trusted_command`禁止まで実装済みです。

### セットアップ / 認証 — #356

- **#361 P0 — 完了:** macOS Keychain / Windows Credential Manager / Linux Secret Service・keyringのOS-native secure credential backend。平文へのsilent fallbackは禁止し、secret入力をargv/shell historyへ残さない。
- **#362 P0 — 完了:** `reason auth login/list/status/logout`、secure credential replacement/rotation、将来のwork/personal named accountを阻害しないstorage identity。
- **#367 P0 — 完了:** provider/model discoveryとdefault切替。silent model fallbackは禁止。
- **#363 P0 — 完了:** provider選択、secure auth、model選択、non-secret default、bounded readiness check、billable check時の明示、最初のcommandまでをまとめる`reason setup` wizard。

CI、container、remote shell、server用途ではenvironment variableも引き続きサポートし、OS-stored credentialとのprecedenceをdeterministicに定義・文書化します。

## フェーズ2 — 日常的な対話型ターミナルUX ✅ 完了

**2026-09-12 完了。** Phase 2のP0/P1項目はすべて実装・merge済みで、`main` commit `c0233728cedea4f0d5cc558e6bff220df8d937d0` まで反映済み。merge後の `ci` / `cli-platform-smoke` / `credential-store-smoke` / `lifecycle-smoke` もすべてpass。

- **#364 P0 — implemented:** human TTYでbare `reason`を実行するとin-memory interactive REPLを起動し、JSON/non-TTY/piped automationはnon-interactiveのまま維持する。`/add <path>`は既存untrusted context pathを再利用し、末尾`\`でmultiline prompt、過去のexposed exchangeはuntrustedなin-memory conversation contextとしてのみ保持し、shell-style history fileは書かない。managed persistenceは#365/#381で扱う。
- **#365 P0 — implemented:** `-c/--continue`、`-r/--resume [id]`、TTY picker、managed `reason session list`、stable id/title、successful-turn checkpoint persistenceを既存typed `SessionFile` / `ReasoningThread` runtime上に実装。Coreのimmutable task identityを保つproduct-layer conversation wrapperを使い、過去のexposed turnは`untrusted_context`のまま扱う。store concurrency / recovery / migrationは#381で扱う。
- **#381 P0 — implemented:** managed session store lock、in-memory digestによるoptimistic concurrency、crash-safe atomic replacement/temp recovery、corrupt/incompatible listing、0.5.xのno-destructive-migration rollback policyを実装。
- **#379 P0 — implemented:** private managed-state permission/ownership check、明示的`--ephemeral`、managed session export/delete/scoped purge、deterministicなuninstall retention/purge boundary、provider/MCP/resolverへのoutbound-data disclosure、default first-party telemetryなしを実装。
- **#378 P0 — implemented:** finalized answer、canonical verified fact、unresolved/qualified proposition、supporting evidence/source provenance、untrusted context、typed acquisition/rejection noteを分離したhuman outputを実装。interactive `/status` / `/evidence`もtyped checkpointから同じviewを再構成し、JSONは変更せず、hidden reasoningやunsupported model proseへauthorityを与えない。
- **#380 P0 — implemented:** additiveなhuman/JSON provider/resolver usage、interactive `/usage`、rollback-safeなmanaged-session累積accounting、hardな`max_model_calls` / reported output-token / measurable total-token guardとtyped operational exhaustion、明示operator pricing provenanceがある場合のみcurrency estimateを実装。
- **#366 P1 — implemented:** `reason config list/get/set/unset/path/sources`でsafeなrun defaultのeffective value・per-key provenance・precedenceを確認し、編集はuser layerだけに限定。secretとauthority-bearingなresolver/MCP/trusted-verifier設定はこのsurface外で、`reason-config-v1`でも引き続きreject。
- **#369 P1 — implemented:** human full-TTYではHarness-owned `Planning` / `Acquiring` / `Verifying` / `Finalizing` phase、bounded provider retry/elapsed status、secret/CoT-freeな任意`--verbose` operational detailを表示。JSON/piped/non-TTYは静かなまま、Ctrl+Cはincompleteな通常turnをsuccessful checkpoint化せずtyped `cancelled`で終了。
- **#373 P1 — implemented:** self-teaching help、copy-paste examples、stdout-onlyのzsh/bash/fish/PowerShell completion。commonな自然言語workflowを優先しつつ、advanced/research surfaceは明示的にdiscoverableなまま維持。
- **#383 P1 — implemented:** explicit `--plain`、`NO_COLOR`、`TERM=dumb`、non-TTYを統合したterminal presentation policyを実装。plain modeではdecorative progressを抑えつつprompt/Ctrl+Cを維持し、JSON/pipe outputはdecoration-freeのまま、human renderingはwidth由来のbyte truncationを行わずUnicodeを保持。

既存のone-shot `reason "TASK"`、repeatable `--file`、piped stdin context、JSON automation surfaceは維持します。

## フェーズ3 — 外部情報取得UXとprocess isolation

- **#387 P0:** local external-command / MCP / trusted-verifier subprocessは、親processのprovider/developer secretを丸ごとinheritせず、minimal scoped environmentから起動する。
- **#368 P1:** guided read-only MCP management: `reason mcp add/list/inspect/test/remove`。
- **#386 P1:** local read-only management後のsecure remote MCP / Streamable HTTP OAuth lifecycle。可能な範囲でheadless/no-browser authも扱う。

これはconfiguration / transport / isolation / visibilityの改善です。MCP outputはauthorityではなくacquisition dataのまま、write-capable/ambiguous capabilityはfail closedを維持し、Harness Engine 0.4.2のMCP correctness boundaryを変更しません。

## フェーズ4 — 診断と運用復旧

- **#357 P0:** `reason doctor`でReason CLI / Harness Engine versionを別々に表示し、install/config source、credential presence（値は非表示）、provider/model readiness、OS credential store、managed session path、project trust、設定済みMCP readinessを確認。human/JSON diagnosticsを提供。
- **#370 P0:** credential、model/protocol、quota/rate limit/outage、structured output、config/trust、MCP、session、update/version、distribution-integrity failureをrecovery-orientedなhuman errorへ整備。
- **#385 P1:** explicit model retirement/fallback policy。defaultでprovider/model execution identityをsilent変更しない。将来fallback chainを入れる場合も実使用provider/modelを記録する。
- **#384 P1:** proxy/custom CA/headless network diagnostics。insecure TLS bypassを推奨しない。

Typed machine failureとepistemic `unknown`は分離したままです。friendly remediationのためにoperational failureをsemantic uncertaintyへ潰してはいけません。

## フェーズ5 — 新規インストールでのリリース判定

**#374 P0** をCLI 0.5.0のacceptance gateにします。supported platformで次を検証します。

1. Rustなしでpublished native artifactからinstallし、trusted release identityを検証;
2. untrusted project configがexplicit trust前にexecutable/MCP/trusted-verifierをactivateできない;
3. empty user config/homeからsetup;
4. secure credential storage、またはheadless unsupported時の明示的typed path。secret-valued argvは不要;
5. configured defaultを使ったone-shot execution;
6. interactive execution + follow-up + 通常のfile/context追加;
7. human outputでverified fact / unresolved・qualified state / admitted source provenanceを理解できる;
8. persisted sessionのcontinue/resume;
9. concurrent/crash-interrupted sessionでmanaged stateをsilent破損・overwriteしない;
10. provider/model/configのinspect/switch;
11. provider usage可視化とbudget exhaustionの安全なtyped behavior;
12. `reason doctor`とCLI/Engine別version表示;
13. expected operational failureからactionable recovery;
14. private local data permission、ephemeral/no-persist、scoped purge/retain-data;
15. local MCP/resolver/verifier subprocessが親processのunrelated secret sentinelを観測できない;
16. artifact verification付きupdate/check、explicit rollback、uninstall/retain-data;
17. 既存JSON/non-interactive contract smoke;
18. stdout / stderr / diagnostics / config / session/history / subprocess environmentへのcredential/secret leakが0。

このgateがgreenで、unresolved P0 product blockerが0になるまで`reason-v0.5.0`はtagしません。

## 優先度

### P0 — CLI 0.5.0必須

#357、#361、#362、#363、#364、#365、#367、#370、#371、#372、#374、#377、#378、#379、#380、#381、#382、#387。

P0は、replacement acceptance pathを明示してscope変更しない限りgeneral-use release blockerです。

### P1 — 製品としての完成度 / polish

#366、#368、#369、#373、#375、#383、#384、#385、#386。

P1はpolished product lineとして進めますが、最初の安全な0.5.0 releaseを自動的にはblockしません。ただし実装中にP0級のusability/safety/supportability gapが判明した場合は昇格します。

## このマイルストーンで明示的にやらないこと

- Harness Engine 0.4.2のreasoning/authority semantics変更;
- Reasonをwrite-capable coding agentやbackground-agent platformへ変えること;
- supported product boundaryに存在しないdestructive tool向けapproval systemの追加;
- availability/retirement回避のためprovider/modelをsilent切替すること;
- provider/integration secretを`reason-config-v1`、project config、session/history artifact、evidence、通常CLI argvへ保存すること;
- local MCP/resolver/verifierへ親process environment全体をinheritさせること;
- convenienceのためにMCP/read-only fail-closed checkを弱めること;
- text file/stdin + 明示的acquisition/MCPで0.5.0 product pathを満たせる段階でrich PDF/image/browser ingestionをrelease blockerにすること;
- cloud account/session syncやbuilt-in hosted telemetryをlocal useの前提にすること;
- v0.4.2やfreeze済みevaluation evidenceのretroactive変更。

semantic/utility変更は別の **Harness Engine 0.5.0 — Verified Investigation Utility** milestone (#4)で扱い、adoption前にfresh evidenceを必須にします。
