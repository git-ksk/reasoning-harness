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

server processは通常のenvironmentを継承しますが、config schemaはcredential-likeな未知fieldを拒否します。resolver/admission telemetryには、literal command argumentsではなく安定したhash化config identityを記録します。

## 運用上の失敗と再実行

transport、authentication、permission、**negotiation**、**session**、protocol、tool-execution、timeout、policy-denialをtyped operational classとして区別します。unsupported/missing negotiated revisionは`negotiation`、initialized lifecycleの切断やpagination cycleは`session`、malformed JSON-RPCは`protocol`、tool resultの`isError: true`は`tool_execution`です。`timeout_ms`はprocess spawnからhandshake/list/callの全stdin write・bounded response read、termination、cleanup handoffまでを1つのwhole-invocation deadlineで覆います。semantic `unknown`へ変換しません。

各MCP requestはstableなrequest/attempt provenanceを持ち、`ResolutionAttempt`にはadapter/admission identityとcost telemetryを記録します。v3 config identityはrequested/supported protocol policyとpagination boundをbindし、acquired evidence IDには実際のnegotiated revisionも含めます。`ReasoningThread` replayは記録済みattemptを復元するだけで、MCP serverを再invokeしません。

deterministic fake-server testでallowlisted downlevel negotiation、unsupported revision rejection、initialized session切断、read-only annotation強制、protocol/tool failure分離、deadline、opaque non-promotion、explicit Harness acquisition envelopeを検証します。さらに#204で使ったpinned公式`ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33`に対するone-off acceptanceでもv3 sessionと`get_file_contents`が成功し、generic resultはfacts 0のopaqueのままでした。
