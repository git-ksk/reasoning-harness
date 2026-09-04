# Read-only MCP resolver

#176では`mcp_readonly_v1`を既存bounded-resolution loopの**取得adapter**として追加します。MCP自体はcorrectness boundaryではありません。

対象protocolはMCP `2026-07-28` / stdioです。1回のJSON-RPC `tools/call`ごとにprotocol version、client capabilities、client identity、Harness-owned request/attempt provenanceを`_meta`へ入れます。initialize/session handshakeには依存しません。

v0.3.0のsupported surfaceは意図的に制限しています。`read_only:true`、`resolver_class:"evidence_acquisition"`、server/tool/sourceの明示、selected toolを含む`allowed_tools`が必須です。tool argumentsはHarness configで固定し、modelに任意tool callを組み立てさせません。`mcp_readonly` / `external_command` / `--resolver-fact`は同時利用できません。

allowlistはoperator側の実行policyです。MCP annotationやtool成功だけでside-effect freedomやauthorityを証明したことにはしません。read-only contractを持つtoolだけをoperatorがallowlistしてください。

通常のMCP `content` / `structuredContent`はfacts無し・trusted metadata無しのopaque `AcquiredEvidence`になります。cooperating toolが`structuredContent.reasoning_harness`に`observation` / `facts` / `acquisition_metadata`を返すことはできますが、それもraw acquisition dataです。Harness-owned source allowlist / freshness / scope / authority admissionを通り、その後に通常のqualification / verificationを再実行して初めてsupportへ進めます。

transport / authentication / permission / protocol / tool execution / timeout / policy denialは#178のtyped operational stateとして扱い、semantic `unknown`へ混同しません。`isError:true`もevidenceではありません。resolution attemptにはhashed adapter/admission identityとcost telemetryを残し、`ReasoningThread` replayでは記録だけを復元してMCP toolを再実行しません。

設定例とwire contractの詳細は英語版[`mcp-resolver.md`](mcp-resolver.md)を参照してください。
