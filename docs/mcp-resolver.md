# Read-only MCP resolver

Issue #176 adds `mcp_readonly_v1` as an acquisition adapter inside the existing bounded-resolution loop. MCP is transport/integration only; it is not a correctness boundary.

The adapter targets MCP protocol `2026-07-28` over stdio. Each invocation sends a JSON-RPC `tools/call` request with the protocol version, client capabilities, client identity, and Harness-owned request/attempt provenance in `_meta`. It does not rely on an initialize/session handshake.

## Safety boundary

The supported v0.3.0 surface is configuration-only and deliberately restrictive:

- `read_only` must be `true`;
- `resolver_class` must be `evidence_acquisition`;
- `server_id`, selected `tool`, and Harness-owned `source` are explicit;
- the selected tool must appear in `allowed_tools`;
- fixed arguments are Harness configuration, not model-generated arguments;
- an optional provenance argument may be injected only when explicitly configured and cannot overwrite a fixed argument;
- timeout and response-size limits must be positive;
- `mcp_readonly`, `external_command`, and `--resolver-fact` are mutually exclusive resolver lanes.

The allowlist is an operator policy assertion about which tool may be invoked. MCP tool annotations or a successful tool call do not create authority and do not prove that an arbitrary external server is side-effect-free. Operators must therefore allowlist only tools whose deployment contract is read-only.

## Result handling

Generic MCP `content` or `structuredContent` is converted to opaque `AcquiredEvidence` with no facts and no trusted acquisition metadata. It cannot directly make a proposition `Supported`.

A cooperating read-only tool may return this optional structured payload:

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

Those fields are still resolver-supplied raw acquisition data. The Harness assigns the configured source identity, then `external_evidence_admission_v1` independently checks source allowlisting, freshness, scope, and authority policy before ordinary qualification and verification run again. The MCP tool cannot return trusted `EvidenceMetadata`, verification receipts, a verdict, or grounded final prose through this path.

## Configuration

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

The server process inherits the normal environment, but config schemas reject unknown credential-like fields. Resolver/admission telemetry records stable hashed config identities rather than literal command arguments.

## Operational failures and replay

Transport, authentication, permission, protocol, tool-execution, timeout, and policy-denial failures use the typed operational resolution classes from #178. Tool result `isError: true` is `tool_execution`, not semantic evidence. These outcomes remain distinct from semantic `unknown`.

Each MCP request carries stable request/attempt provenance and the resulting `ResolutionAttempt` records adapter/admission identities and cost telemetry. `ReasoningThread` replay restores those recorded attempts; it never invokes the MCP server again.

Deterministic fake-server tests cover modern request metadata, allowlisting, timeout, typed tool errors, opaque-result non-promotion, and the complete acquisition -> admission -> ordinary re-verification path. A live external MCP server is not required for deterministic CI.
