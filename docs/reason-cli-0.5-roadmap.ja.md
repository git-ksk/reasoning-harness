# Reason CLI 0.5.0 一般利用向け製品化ロードマップ

Reason CLI 0.5.0は、Harness Engineと独立してversioningする最初のproduct lineです。受け入れ済みの **Harness Engine 0.4.2** を一般のterminal userが使いやすい製品へ仕上げますが、v0.4.2のreasoning / authority / admission / verification / finalization / answer-safety / MCP non-promotion / session replay boundaryは変更しません。

`v0.4.2`は最後のunified historical releaseとして固定します。最初のsplit release予定は次の形です。

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

目的はcoding agentをコピーすることではありません。成熟したAI CLIで期待される、toolchain不要のinstall、guided auth、引数なしで使えるinteractive mode、continue/resume、provider/model/configのdiscoverability、actionable diagnostics、可逆なupdate/uninstallといった低摩擦のterminal UXをReasonへ持ち込みます。

Tracking: milestone **Reason CLI 0.5.0 — General-use Productization** (#6)、parent Issue #359。

## Product journey

### 初回利用

```text
native Reason binaryをinstall
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

初回userにRust、Cargo、shell profile編集、平文credential file、内部Harness JSON contractの知識を要求しません。

### 日常利用

```text
reason
  -> interactive session
  -> follow-up
  -> Ctrl+Cで現在の処理を安全にcancel
  -> exit

reason -c
  -> 最新compatible sessionをcontinue

reason -r <session>
  -> 選択したsessionをresume
```

terminalには`Planning`、`Acquiring`、`Verifying`、`Finalizing`のようなHarness-owned lifecycle stateを表示できます。これは進捗表示であり、hidden chain-of-thoughtではありません。

### 復旧

```text
reason doctor
reason auth status
reason config sources
reason mcp test <name>
reason update --check
```

user-facing operational errorは「何が失敗したか」「得られた結果を信用してよいか」「次に実行すべき安全なcommand」を説明します。

## Phase 0 — Product/version boundary

- **#355 — 完了:** Reason CLIとHarness EngineのSemVer座標を分離。
- `v0.4.2`はimmutableな最後のunified tag。
- 今後のCLI releaseは`reason-vX.Y.Z`。
- machine contract identityは独立したcompatibility座標として維持。

## Phase 1 — Install、認証、最初の回答まで

### Distribution umbrella — #358

- **#371 P0:** macOS / Linux / Windows向けone-command native installer。checksum検証を必須化。
- **#372 P0:** update、明示的rollback、uninstall lifecycle。
- **#375 P1:** canonical installer/update contract安定後のHomebrew / winget channel。

### Setup/auth umbrella — #356

- **#361 P0:** macOS Keychain / Windows Credential Manager / Linux Secret Service・keyringのOS-native secure credential backend。平文へのsilent fallbackは禁止。
- **#362 P0:** `reason auth login/list/status/logout`。
- **#367 P0:** provider/model discoveryとdefault切替。
- **#363 P0:** provider選択、secure auth、model選択、non-secret default、readiness check、最初のcommandまでをまとめる`reason setup` wizard。

CI、container、remote shell、server用途ではenvironment variableも引き続きサポートし、OS-stored credentialとのprecedenceをdeterministicに定義・文書化します。

## Phase 2 — 日常的なinteractive terminal UX

- **#364 P0:** 引数なし`reason`でusage errorではなくinteractive REPLを起動。
- **#365 P0:** `-c/--continue`、`-r/--resume`、session list/picker、安全なcheckpoint persistence。既存typed session runtimeを利用。
- **#366 P1:** `reason config list/get/set/unset/path/sources`。secretは`reason-config-v1`から引き続きreject。
- **#369 P1:** high-level progress/retry statusとdeterministicな安全cancel。JSON/piped modeは明示指定なしでは静かに保つ。
- **#373 P1:** self-teaching help、examples、zsh/bash/fish/PowerShell completion。

既存のone-shot `reason "TASK"` とJSON automation surfaceは維持します。

## Phase 3 — External acquisition UX

- **#368 P1:** guided read-only MCP management: `reason mcp add/list/inspect/test/remove`。

これはconfiguration/visibilityの改善だけです。MCP outputはauthorityではなくacquisition dataのまま、write-capable/ambiguous capabilityはfail closedを維持し、Harness Engine 0.4.2のMCP correctness boundaryを変更しません。

## Phase 4 — Diagnosticsと復旧

- **#357 P0:** `reason doctor`でReason CLI / Harness Engine versionを別々に表示し、install/config source、credential presence（値は非表示）、provider/model readiness、OS credential store、session path、設定済みMCP readinessを確認。human/JSON diagnosticsを提供。
- **#370 P0:** credential、model/protocol、quota/rate limit/outage、structured output、config、MCP、session、update/version failureをrecovery-orientedなhuman errorへ整備。

Typed machine failureとepistemic `unknown`は分離したままです。friendly remediationのためにoperational failureをsemantic uncertaintyへ潰してはいけません。

## Phase 5 — Fresh-install release gate

**#374 P0** をCLI 0.5.0のacceptance gateにします。supported platformで次を検証します。

1. Rustなしでpublished native artifactからinstall;
2. empty user config/homeからsetup;
3. secure credential storage、またはheadless unsupported時の明示的typed path;
4. configured defaultを使ったone-shot execution;
5. interactive execution + follow-up;
6. persisted sessionのcontinue/resume;
7. provider/model/configのinspect/switch;
8. `reason doctor`とCLI/Engine別version表示;
9. expected operational failureからactionable recovery;
10. update/checkとuninstall/retain-data;
11. 既存JSON/non-interactive contract smoke;
12. stdout / stderr / diagnostics / config / session artifactへのcredential leakが0。

このgateがgreenで、unresolved P0 product blockerが0になるまで`reason-v0.5.0`はtagしません。

## Priority model

### P0 — CLI 0.5.0必須

#361、#362、#363、#364、#365、#367、#370、#371、#372、#357、#374。

P0は、replacement acceptance pathを明示してscope変更しない限りgeneral-use release blockerです。

### P1 — Product parity / polish

#366、#368、#369、#373、#375。

P1はpolished product lineとして進めますが、最初の安全な0.5.0 releaseを自動的にはblockしません。ただし実装中にP0級のusability/safety/supportability gapが判明した場合は昇格します。

## このmilestoneで明示的にやらないこと

- Harness Engine 0.4.2のreasoning/authority semantics変更;
- Reasonをwrite-capable coding agentへ変えること;
- supported product boundaryに存在しないdestructive tool向けapproval systemの追加;
- provider secretを`reason-config-v1`、project config、session artifact、evidenceへ保存すること;
- convenienceのためにMCP/read-only fail-closed checkを弱めること;
- v0.4.2やfreeze済みevaluation evidenceのretroactive変更。

semantic/utility変更は別の **Harness Engine 0.5.0 — Verified Investigation Utility** milestone (#4)で扱い、adoption前にfresh evidenceを必須にします。
