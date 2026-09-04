# MCP product surface

Issue #180 implements the optional Rust-only `reason-mcp` adapter for external MCP clients. This is deliberately distinct from #176: `mcp_readonly_v1` lets Reasoning Harness call an MCP tool as an acquisition source, while `reason-mcp` lets an MCP client call selected Reasoning Harness product operations.

## Protocol and transport

`reason-mcp` serves newline-delimited JSON-RPC over stdio using MCP protocol revision `2026-07-28`. It is stateless: there is no `initialize`/`initialized` session. `server/discover` advertises the supported revision and tool capability. `tools/list` uses deterministic ordering and conservative `ttlMs: 0`, `cacheScope: "private"` cache hints. Responses stamp server identity in `_meta`.

Run it beside the native executable:

```bash
reason-mcp --reason-command /path/to/reason
```

When installed as sibling binaries, `--reason-command` can be omitted.

## Tools

- `reason_ask`: delegates to the native natural-language path, including bounded local resolution, finalization, and answer safety. The schema accepts a task, provider/model selection, untrusted context, explicit facts/hypotheses/local resolver facts, and bounded numeric controls. It does **not** accept arbitrary config, raw CLI arguments, external receipt injection, or arbitrary external resolver programs.
- `reason_run`: delegates structured `HarnessInput + ReasoningCandidate` to native `reason run --no-config --format json`. It cannot inject receipts, providers, config, or raw CLI flags.
- `reason_verify`: delegates a `reasoning-artifact-v1` to native `reason verify`.
- `reason_schema`: delegates supported native schema discovery.

For native invocations, `structuredContent` is the exact versioned `reason-cli-output-v1` JSON emitted by the native executable. MCP does not reinterpret `accept | reject | unknown`, finalization status, safety/runtime identity, verification receipts, or diagnostics. Native process/timeout/protocol failures are returned as `reason-mcp-operational-failure-v1`; they are not converted into semantic `unknown`.

## Authority boundary

MCP is an integration surface, not a correctness boundary. A successful `reason-mcp` call says only that the returned artifact/result was produced by that one native Harness invocation. It does not certify the external agent's planning, tool selection, memory, subsequent transformations, or overall loop.

The adapter intentionally does not introduce a new core API or MCP-specific verdict semantics. The native `reason` executable remains the product/runtime authority.
