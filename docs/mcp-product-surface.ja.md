# MCPプロダクトのサーフェス

#180では、外部MCP clientからReasoning Harnessを呼ぶoptional Rust-only adapter `reason-mcp`を実装します。#176とは向きが逆です。`mcp_readonly_v1`はHarnessからMCP toolをacquisition sourceとして呼び、`reason-mcp`はMCP clientからselected Harness product operationを呼びます。

## プロトコル / トランスポート

`reason-mcp`はMCP `2026-07-28`のnewline-delimited JSON-RPCをstdioで提供します。statelessなので`initialize`/`initialized` sessionはありません。`server/discover`でversion/tool capabilityを公開し、`tools/list`はdeterministic order、`ttlMs: 0`、`cacheScope: "private"`を返します。response `_meta`にはserver identityを記録します。

```bash
reason-mcp --reason-command /path/to/reason
```

sibling binaryとして配置した場合は`--reason-command`を省略できます。

## ツール

- `reason_ask`: native natural-language pathへ委譲し、bounded local resolution / finalization / answer safetyをそのまま使います。任意config、raw CLI args、外部receipt直接注入、任意external resolver programは受け付けません。
- `reason_run`: `HarnessInput + ReasoningCandidate`をnative `reason run --no-config --format json`へ委譲します。
- `reason_verify`: `reasoning-artifact-v1`をnative `reason verify`へ委譲します。
- `reason_schema`: native schema discoveryへ委譲します。

native invocation成功時の`structuredContent`はnative executableが出した`reason-cli-output-v1` JSONそのものです。MCP側で`accept | reject | unknown`、finalization、safety/runtime identity、receipt、diagnosticを書き換えません。native process/timeout/protocol failureは`reason-mcp-operational-failure-v1`として返し、semantic `unknown`へ変換しません。

## 権限の境界

MCPはintegration surfaceでありcorrectness boundaryではありません。`reason-mcp`の成功が保証するのは**その1回のnative Harness invocation**だけで、外部agentのplanning、tool selection、memory、後段変換、agent loop全体をverifiedとは扱いません。native `reason` runtimeが引き続きproduct/runtime authorityです。
