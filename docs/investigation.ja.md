# 制約付き調査プランニング

Issue #212 では、自然文の `reason "TASK"` に opt-in の investigation lane を追加する。これは、最初の回答がまだ `Accept` でないときに **次に何を調べるか** を決めるための product control layer である。planner、model、acquisition tool に correctness authority を与える機能ではない。

machine identity:

- runtime: `bounded-investigation-v1`
- plan proposal: `reason-investigation-plan-v1`
- action proposal: `reason-investigation-action-v1`
- investigation external-command adapter: `investigation_external_command_v1`
- investigation external-command request: `reason-investigation-external-resolver-request-v1`
- MCP acquisition: `mcp_readonly_v3`

historical freeze 対象の `mcp_readonly_v1` と、既存 static `external_command_v1` の request protocol は変更しない。

## 処理の流れ

`resolution.investigation` が設定され、最初の natural-language run がまだ `Accept` でない場合、Harness は次の bounded loop を実行する。

1. model は closed な `reason-investigation-plan-v1` JSON schema を通じて、少数の investigation question を提案する。生成された target は明示的に `model_proposed_untrusted` として扱う。
2. Harness が target 数、ID、shape を検証し、通過した proposal を canonical investigation target にする。これは planning object の受け入れであって、提案された答えを真と認定する処理ではない。
3. 各 round で model が選べるのは、既存 target ID と既存 read-only capability ID の組み合わせ1つ、または stop だけである。action schema には自由な query text、tool argument、authority class、evidence、receipt、verdict の field を持たせない。
4. Harness は未知 capability、`read_only` でない capability、selector/key の不一致、同じ target/capability pair の再実行、budget 超過を拒否する。
5. 選ばれた acquisition adapter を1回だけ実行する。investigation external command は `external_command_v1` を暗黙拡張せず専用 request identity を使う。MCP は v0.4 product 向けの `mcp_readonly_v3` と、Harness-owned の fixed argument / tool allowlist を使う。
6. 取得データは引き続き untrusted である。admission policy がある場合だけ、通常の source allowlist、freshness、scope、authority policy を適用する。admission がなければ external data は trusted evidence に昇格できない。
7. evidence が admit された後、更新済み Harness input から natural-language candidate を再生成し、通常の validation、qualification、verification、diagnostics、verdict、finalization、answer-safety をもう一度通す。
8. `no_result`、`rejected_evidence`、`ambiguous`、`verification_progress`、`operational_failure` などの typed outcome を記録し、次 round の planner はその結果を見て別の未試行 capability を選択できる。
9. resolved、planner stop、target exhaustion、action/round budget、no-progress 上限、operational terminal のいずれかで停止する。

つまり **acquisition success と verification success は別物** である。grounded output を許可できるのは、従来どおり Harness の authority path だけである。

## 設定

`resolution.investigation` は `--resolver-fact`、`resolution.external_command`、`resolution.mcp_readonly` と排他的な acquisition lane である。この4種類を同時には使えない。一方 `resolution.trusted_command` は後段の独立 verifier なので併用できる。

read-only capability を2つ設定する例:

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "investigation": {
      "max_targets": 4,
      "max_rounds": 4,
      "max_actions": 6,
      "max_no_progress_rounds": 2,
      "planner_max_tokens": 256,
      "capabilities": [
        {
          "kind": "external_command",
          "id": "deployment-reference",
          "read_only": true,
          "supported_fact_keys": ["service.region"],
          "program": "deployment-reference-resolver",
          "args": ["--stdio"],
          "timeout_ms": 5000,
          "max_response_bytes": 262144,
          "admission": {
            "evaluation_time_unix_seconds": 1788652800,
            "authority_ranks": {"primary": 20},
            "minimum_authority_class": "primary",
            "sources": {
              "deployment:reference": {
                "authority_class": "primary",
                "max_age_seconds": 300
              }
            }
          }
        },
        {
          "kind": "mcp_readonly",
          "id": "inventory-lookup",
          "read_only": true,
          "supported_fact_keys": ["inventory.count"],
          "server_id": "inventory",
          "program": "inventory-mcp",
          "args": ["--stdio"],
          "allowed_tools": ["lookup"],
          "tool": "lookup",
          "fixed_arguments": {"board": "primary"},
          "source": "mcp:inventory:lookup",
          "timeout_ms": 5000,
          "max_response_bytes": 262144,
          "admission": {
            "evaluation_time_unix_seconds": 1788652800,
            "authority_ranks": {"primary": 20},
            "minimum_authority_class": "primary",
            "sources": {
              "mcp:inventory:lookup": {
                "authority_class": "primary",
                "max_age_seconds": 300
              }
            }
          }
        }
      ]
    }
  }
}
```

capability ID はすべて一意でなければならず、`read_only` は必ず `true`。複数 capability が admission policy を持つ場合、round ごとに authority system をすり替えられないよう authority-rank policy を一致させる。MCP tool argument は Harness-owned fixed config のままで、model は生成できない。

investigation external command が受け取る request は `reason-investigation-external-resolver-request-v1` であり、既存 `reason-external-resolver-request-v1` ではない。response は closed な `reason-external-resolver-response-v1` envelope を再利用するが、investigation runtime は acquired-evidence / no-result 以外を authority として利用しない。candidate revision や human-review contribution は acquisition-only boundary で拒否する。

## boundedness と telemetry

runtime は `max_targets`、`max_rounds`、`max_actions`、`max_no_progress_rounds` で構造的に上限を持つ。planner call には `planner_max_tokens`、candidate regeneration には既存 natural-language generation の max-token 上限があり、各 acquisition adapter には whole-invocation timeout と response-size 上限がある。invalid/repeated action も round budget は消費するため、無限 repair loop にはならない。

自然文 JSON output では investigation を通常の resolution round と分けて記録する。accepted target / capability descriptor、planner call 数、typed action rejection、action record、admitted evidence 数、verification progress、stop reason、plan/action/candidate-regeneration の provider observation を保持する。provider/protocol failure は operational evidence のままで、semantic fact や `unknown` の根拠へ変換しない。

## static path との互換性

`resolution.investigation` を設定しなければ、従来の natural-language resolver behavior は変わらない。`--resolver-fact`、static `external_command_v1`、static `mcp_readonly_v3`、`trusted_command_verifier_v1` は既存 path を維持する。frozen research/evaluation surface は変更せず、`mcp_readonly_v1` は freeze workflow で byte-for-byte 保護したままである。
