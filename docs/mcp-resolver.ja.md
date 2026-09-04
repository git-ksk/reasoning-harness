# 読み取り専用MCPリゾルバー

Issue #176では、既存のbounded-resolution loop内の取得adapterとして`mcp_readonly_v1`を追加します。MCPはtransport/integrationに限られ、correctness boundaryではありません。

adapterはstdio上のMCP protocol `2026-07-28`を対象とします。各invocationでは、protocol version、client capabilities、client identity、Harness所有のrequest/attempt provenanceを含むJSON-RPC `tools/call` requestを送信し、`_meta`に格納します。initialize/session handshakeには依存しません。

## 安全性の境界

supported v0.3.0 surfaceは設定のみで、意図的に制限されています。

- `read_only`は`true`でなければならない。
- `resolver_class`は`evidence_acquisition`でなければならない。
- `server_id`、選択した`tool`、Harness所有の`source`を明示する。
- 選択したtoolは`allowed_tools`に含まれなければならない。
- 固定argumentはHarness configurationであり、modelが生成するargumentではない。
- 任意のprovenance argumentは明示的に設定した場合だけ注入でき、固定argumentを上書きできない。
- timeoutとresponse-size limitは正の値でなければならない。
- `mcp_readonly`、`external_command`、`--resolver-fact`は相互排他的なresolver laneである。

allowlistは、どのtoolをinvokeしてよいかに関するoperator policyの主張です。MCP tool annotationやtool callの成功はauthorityを作らず、任意のexternal serverがside-effect-freeであることも証明しません。したがってoperatorは、deployment contractがread-onlyであるtoolだけをallowlistに入れる必要があります。

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

transport、authentication、permission、protocol、tool-execution、timeout、policy-denialのfailureには#178のtyped operational resolution classを使用します。tool resultの`isError: true`は`tool_execution`であり、semantic evidenceではありません。これらのoutcomeはsemantic `unknown`とは区別されます。

各MCP requestはstableなrequest/attempt provenanceを持ち、生成された`ResolutionAttempt`にはadapter/admission identityとcost telemetryを記録します。`ReasoningThread` replayは記録済みattemptを復元するだけで、MCP serverを再invokeしません。

deterministic fake-server testでは、modern request metadata、allowlisting、timeout、typed tool error、opaque resultのnon-promotion、acquisition -> admission -> ordinary re-verificationという完全な経路をカバーします。deterministic CIにlive external MCP serverは必要ありません。
