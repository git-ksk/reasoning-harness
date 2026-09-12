# 読み取り専用MCPリゾルバー

Issue #176でfrozen `mcp_readonly_v1`を追加し、#211でdeadline-onlyのstateless successor `mcp_readonly_v2`を追加しました。Issue #204ではsupported product successorとして`mcp_readonly_v3`を追加します。v1はbyte-for-byte freezeを維持し、v2もhistorical operational successorとして残します。MCPは引き続きtransport/integrationであり、correctness boundaryではありません。

`mcp_readonly_v3`は1つのstdio child processをbounded sessionとして維持し、`initialize` -> negotiated protocol検証 -> `notifications/initialized` -> bounded `tools/list` read-only declaration検証 -> `tools/call`の順で実行します。requested revisionのdefaultは`2026-07-28`、Harnessが受け入れるrevisionは`2026-07-28`と明示downlevelの`2025-11-25`だけです。configはこの集合を狭められますが未知revisionを追加できません。全lifecycleはRPCごとの新しいtimeoutではなく、#211と同じ1つのabsolute deadlineを共有します。

## 安全性の境界

supported v0.4.0 surfaceは設定のみで、意図的に制限されています。

- `read_only`は`true`でなければならない。
- `resolver_class`は`evidence_acquisition`でなければならない。
- `server_id`、選択した`tool`、Harness所有の`source`を明示する。
- 選択したtoolは`allowed_tools`に含まれなければならない。
- 固定argumentはHarness configurationであり、modelが生成するargumentではない。
- 任意のprovenance argumentは明示的に設定した場合だけ注入でき、固定argumentを上書きできない。
- timeoutとresponse-size limitは正の値でなければならない。
- 選択toolは`tools/list`にも存在し、v3ではserver側`annotations.readOnlyHint=true`を必須とする。
- tool-list paginationは`max_tool_list_pages`（default 8、1..=32）でboundedにする。
- protocol negotiationはHarness既知revision allowlistに対してfail-closedとする。
- `mcp_readonly`、`external_command`、`--resolver-fact`は相互排他的なresolver laneである。

operator allowlistとserverの`readOnlyHint` declarationを両方要求しますが、annotation自体はserver claimでありcorrectness authorityではありません。handshake成功、annotation、tool call成功のどれも外部outputを正しいものとして昇格させません。

## 結果の処理

汎用MCPの`content`または`structuredContent`は、factsもtrusted acquisition metadataも持たないopaqueな`AcquiredEvidence`へ変換されます。これだけでpropositionを`Supported`にすることはできません。

read-only toolが協調する場合、次のstructured payloadを任意で返せます。

```json
{
  "structuredContent": {
    "reasoning_harness": {
      "observation": "service.region=eu-west-1",
      "facts": {"service.region": "eu-west-1"},
      "acquisition_metadata": {
        "observed_at_unix_seconds": 1000,
        "retrieved_at_unix_seconds": 1001,
        "claimed_authority_class": "primary"
      }
    }
  }
}
```

これらのfieldもresolverが供給するraw acquisition dataにすぎません。Harnessは設定済みのsource identityを割り当て、その後`external_evidence_admission_v1`がsource allowlisting、freshness、scope、authority policyを独立に検査してから、通常のqualificationとverificationを再実行します。MCP toolはこの経路でtrusted `EvidenceMetadata`、verification receipt、verdict、grounded final proseを返せません。

## Guided CLI管理

0.5のsupported product pathでは、`reason mcp`がuser-scopedなread-only acquisition sourceを1件activeとして管理します。local stdioは従来どおりです。

```text
reason mcp add inventory --program /path/to/mcp-server --arg=--stdio --tool lookup_item
reason mcp test inventory
reason mcp inspect inventory
reason mcp remove inventory
```

remote Streamable HTTPは2026-era専用transportとOAuth lifecycleを使います。

```text
reason mcp add-remote docs \
  --endpoint https://mcp.example.com/mcp \
  --tool search \
  --issuer https://auth.example.com \
  --authorization-endpoint https://auth.example.com/authorize \
  --token-endpoint https://auth.example.com/token \
  --client-id https://client.example.com/reason.json \
  --scope mcp:read
reason mcp login docs
reason mcp login docs --no-browser
reason mcp status docs
reason mcp test docs
reason mcp logout docs
reason mcp remove docs
```

`add` / `add-remote`が保存するのはnon-secret configだけです。active sourceの置換には`--replace`が必要で、local / remote acquisition transportはmutually exclusiveです。`list` / `inspect`はlocal executable argument valueやOAuth token valueを表示しません。`test`はselected toolを実行しません。localはnegotiation + `tools/list`、remoteはstateless `tools/list`までで停止し、どちらもselected toolの`readOnlyHint=true`を必須にします。

remote adapterはMCP `2026-07-28`へpinします。各requestにprotocol revision / client identity / capabilityを載せ、HTTP routing headerも送信します。`x-mcp-header`付きselected toolはparameter-to-header contractをまだ公開していないためfail closedです。remote MCP endpointとOAuth metadata endpointはHTTPS必須で、loopback HTTPはdeterministic local testだけ許可します。

`reason mcp login`はauthorization code + PKCEとloopback callbackを使います。Reasonがcodeをredeemする前に`state`一致とRFC 9207 `iss`のconfigured issuer一致を必須にします。`--no-browser`ではbrowserを開かずauthorization URLを表示するため、SSH/headless環境でも利用できます。access/refresh tokenはnative OS credential storeだけに保存し、MCP source名 / authorization issuer / client ID / exact resource endpointへbindします。issuer/client/resourceが変わった場合、古いcredentialを再利用しません。期限切れtokenのrefreshもconfigured issuer/token endpointだけへ送ります。`logout`はconfigの`remove`後でも実行できるため、orphan credentialを明示削除できます。

ReasonはDynamic Client Registrationを実行しません。authorization serverで利用可能なpre-registered public client IDまたはClient ID Metadata Document URLを設定し、`reason-config-v1`に`client_secret` surfaceは持たせません。MCP `fixed_arguments`はnested object / arrayも含めてcredential-bearing keyをrecursiveに検査するため、runtime credentialはconfigではなくscoped secure credential injectionまたはnative OS credential storeを使います。project-level remote MCP configはhigh-risk network acquisition configとして扱い、`reason trust add`でprojectを承認するまでfail closedです。

## 設定

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "mcp_readonly": {
      "server_id": "inventory-prod",
      "program": "/path/to/mcp-server",
      "args": ["--stdio"],
      "allowed_tools": ["lookup_item"],
      "tool": "lookup_item",
      "read_only": true,
      "resolver_class": "evidence_acquisition",
      "fixed_arguments": {"board": "primary"},
      "provenance_argument": "reason_provenance",
      "source": "mcp:inventory-prod:lookup_item",
      "requested_protocol_version": "2026-07-28",
      "supported_protocol_versions": ["2026-07-28", "2025-11-25"],
      "max_tool_list_pages": 8,
      "timeout_ms": 5000,
      "max_response_bytes": 262144,
      "admission": {
        "authority_ranks": {"primary": 20},
        "minimum_authority_class": "primary",
        "sources": {
          "mcp:inventory-prod:lookup_item": {
            "authority_class": "primary",
            "max_age_seconds": 300
          }
        }
      }
    }
  }
}
```

対象local MCP subprocessは#387のminimal isolated environmentから起動し、親environment全体をinheritしません。remote Bearer tokenはHTTP requestへだけinjectし、resolver config identity / telemetry / config file / session / evidence / stdout / normal diagnosticsから除外します。config schemaは`access_token`、`refresh_token`、`client_secret`、`api_key`などcredential-like unknown fieldをrejectします。resolver/admission telemetryにはnon-secret configだけから作るstable hash identityを記録します。

## 運用上の失敗と再実行

transport、authentication、permission、**negotiation**、**session**、protocol、tool-execution、timeout、policy-denialをtyped operational classとして区別します。unsupported/missing negotiated revisionは`negotiation`、initialized lifecycleの切断やpagination cycleは`session`、malformed JSON-RPCは`protocol`、tool resultの`isError: true`は`tool_execution`です。`timeout_ms`はprocess spawnからhandshake/list/callの全stdin write・bounded response read、termination、cleanup handoffまでを1つのwhole-invocation deadlineで覆います。semantic `unknown`へ変換しません。

各MCP requestはstableなrequest/attempt provenanceを持ち、`ResolutionAttempt`にはadapter/admission identityとcost telemetryを記録します。v3 config identityはrequested/supported protocol policyとpagination boundをbindし、acquired evidence IDには実際のnegotiated revisionも含めます。`ReasoningThread` replayは記録済みattemptを復元するだけで、MCP serverを再invokeしません。

deterministic fake-server testでallowlisted downlevel negotiation、unsupported revision rejection、initialized session切断、read-only annotation強制、protocol/tool failure分離、deadline、opaque non-promotion、explicit Harness acquisition envelopeを検証します。さらに#204で使ったpinned公式`ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33`に対するone-off acceptanceでもv3 sessionと`get_file_contents`が成功し、generic resultはfacts 0のopaqueのままでした。
