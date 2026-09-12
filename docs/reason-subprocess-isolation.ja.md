# Local subprocess environment isolation

Reason CLI 0.5.0では、local external acquisition processを独立したOS-process trust boundaryとして扱います。このhardeningでHarness Engine 0.4.2のevidence / verification / authority semanticsは変更しません。

## 対象product lane

現行product pathのexternal-command acquisition、MCP read-only v3 acquisition、trusted-command verificationは、明示的なminimal environmentからchild processを起動します。MCP v2 compatibility adapterも同じ境界を使います。historical freeze対象の`mcp_readonly_v1`はresearch replay用にbyte/semantic freezeを維持し、Reason CLIのproduct MCP pathではありません。

## Environment boundary

対象childをspawnする前にambient environment inheritanceをclearし、起動に必要なbaselineだけを戻します。

- `PATH`;
- temporary directory: `TMPDIR`, `TMP`, `TEMP`;
- locale/time: `LANG`, `LANGUAGE`, `LC_ALL`, `LC_CTYPE`, `LC_MESSAGES`, `TZ`（存在する場合）;
- Windows: `SystemRoot`, `WINDIR`, `ComSpec`, `PATHEXT`, `SystemDrive`。

`MISTRAL_API_KEY`、`GEMINI_API_KEY`、`AWS_*`、`GH_TOKEN`、`HOME`、任意project variableなどのprovider credential / unrelated developer environmentは対象childへコピーしません。

これはenvironment isolationでありfilesystem sandboxではありません。local executableは引き続きuserのOS identity / filesystem permissionで実行されるため、project trustとread-only/capability policyは必要です。

## Integration固有credential

provider layerには選択されたintegrationだけへ明示的に値を渡すscoped-environment injection boundaryを用意しますが、Reason CLI 0.5.0では任意ambient variable inheritanceやsecret-valued project configを公開しません。将来integration credentialを追加する場合はproduct-owned secure sourceから取得し、選択されたchildだけへ注入します。model context、evidence authority、diagnostics text、persisted configへは流しません。

## Compatibility / automation

JSON / pipe contractは変更しません。provider/model selectionとHarness authorityも変更しません。integration固有environment不足はchild integration側の通常のtyped operational failureとして扱い、Phase 4の`reason doctor`でsecret-freeなremediationを追加できますが、このboundaryを弱めません。
