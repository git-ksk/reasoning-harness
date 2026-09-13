# Reason doctor

**状態:** Reason CLI 0.5.0開発ラインで実装済み。Harness Engine 0.4.2のsemanticsは変更しません。

`reason doctor`は、install、config、credential、local product readiness、managed data path、project trust、MCP config、version identityを確認するread-only diagnostics surfaceです。

```text
reason doctor
reason doctor --format json
reason doctor --live-check
```

defaultではprovider request、remote MCP接続、GitHub release確認を行わず、local readinessだけを診断します。`--live-check`を付けると、bounded provider connectivity、設定済みuser MCP readiness、update availabilityも確認します。provider live checkは実requestを送るためquota消費や課金が発生する可能性があります。

## Stable diagnostics

human / JSONの両方で次を報告します。

- Reason CLI / Harness Engine versionを別々に表示;
- current executable pathと、安全に推定できる場合だけinstall method;
- user / project config path、存在、validity、effective config source;
- credential値を出さずprovider credentialのpresence/sourceだけを表示;
- native OS credential storeのavailability;
- configured provider/model compatibilityとlocal readiness;
- optionalなbounded provider live readiness;
- managed-session path health;
- trustを付与せずcurrent project trust stateを表示;
- configured user MCPのpresenceとoptionalなread-only readiness probe;
- optionalなupdate availability signal;
- typed / secret-freeなdiagnostic issueとsafe recovery command。

JSON resultは既存の`reason-cli-output-v1` product envelope内で`doctor_surface: "reason-doctor-v1"`を使います。

## Safety boundary

`reason doctor`はAPI key、OAuth token、refresh tokenなどcredential valueを表示しません。config、trust state、session、credential、MCP config、Engine stateを変更しません。通常実行ではconfigured MCP processを起動せず、provider/network requestも送りません。live MCP probeは`reason mcp test`と同じread-only readiness pathを使い、selected toolを実行しません。

operational diagnosticsはsemantic `unknown`とは別です。Doctor statusはproduct readinessを表すだけで、evidence、claim、final answerをcertifyしません。
