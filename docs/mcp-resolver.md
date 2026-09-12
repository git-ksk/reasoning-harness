# Read-only MCP resolver

Issue #176 added the frozen `mcp_readonly_v1` acquisition adapter. #211 added the deadline-only stateless successor `mcp_readonly_v2`. Issue #204 now adds the supported product successor `mcp_readonly_v3`, while v1 remains byte-for-byte frozen and v2 remains available as its historical operational successor. MCP remains transport/integration only; it is not a correctness boundary.

`mcp_readonly_v3` keeps one stdio child alive for a bounded MCP session: `initialize` -> negotiated protocol validation -> `notifications/initialized` -> bounded `tools/list` read-only declaration check -> `tools/call`. The requested revision defaults to `2026-07-28`; the Harness-supported negotiation allowlist is restricted to `2026-07-28` and the explicitly accepted downlevel revision `2025-11-25`. Configuration may narrow that set but cannot add an unknown revision. The whole lifecycle uses the same absolute #211 deadline, not a fresh timeout per RPC.

## Safety boundary

The supported v0.4.0 surface is configuration-only and deliberately restrictive:

- `read_only` must be `true`;
- `resolver_class` must be `evidence_acquisition`;
- `server_id`, selected `tool`, and Harness-owned `source` are explicit;
- the selected tool must appear in `allowed_tools`;
- fixed arguments are Harness configuration, not model-generated arguments;
- an optional provenance argument may be injected only when explicitly configured and cannot overwrite a fixed argument;
- timeout and response-size limits must be positive;
- the selected tool must also appear in `tools/list`; v3 requires its server-reported `annotations.readOnlyHint` to be `true`;
- tool-list pagination is bounded (`max_tool_list_pages`, default 8, allowed 1..=32);
- protocol negotiation is fail-closed to the Harness-known revision allowlist;
- `mcp_readonly`, `external_command`, and `--resolver-fact` are mutually exclusive resolver lanes.

The operator allowlist plus server `readOnlyHint` declaration is a two-sided read-only gate for the selected invocation, but the annotation is still a server claim rather than correctness authority. A successful handshake, annotation, or tool call does not prove arbitrary external output correct and never promotes MCP data by itself.

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

## Guided CLI management

For the supported 0.5 product path, `reason mcp` manages one active user-scoped read-only acquisition source. Local stdio remains the simple path:

```text
reason mcp add inventory --program /path/to/mcp-server --arg=--stdio --tool lookup_item
reason mcp test inventory
reason mcp inspect inventory
reason mcp remove inventory
```

Remote Streamable HTTP uses the separate 2026-era transport and OAuth lifecycle:

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

`add` and `add-remote` persist only non-secret configuration. Replacing the active source requires `--replace`; local and remote acquisition transports are mutually exclusive. `list` and `inspect` never expose local executable argument values or OAuth token values. `test` never invokes the selected tool: local stdio stops after MCP negotiation and `tools/list`, while remote Streamable HTTP performs a stateless `tools/list`; both require the selected tool to declare `readOnlyHint=true`.

The remote adapter is pinned to MCP `2026-07-28`. It sends the protocol revision, client identity, and capabilities on every request and uses the required HTTP routing headers. It deliberately fails closed for selected tools using `x-mcp-header` until that parameter-to-header contract is implemented. Remote endpoints and OAuth metadata endpoints require HTTPS; loopback HTTP is accepted only for deterministic local tests.

`reason mcp login` uses authorization-code + PKCE with a loopback callback. `state` must match and the authorization response must carry the configured RFC 9207 `iss` value before Reason redeems the code. `--no-browser` prints the authorization URL for SSH/headless use instead of opening a browser. Access/refresh tokens are stored only in the native OS credential store and are bound to the MCP source name, authorization issuer, client ID, and exact resource endpoint. A changed issuer/client/resource therefore cannot reuse an old credential. Expired credentials refresh only against the configured issuer/token endpoint. `logout` deletes the stored credential and also works after `remove`, so orphaned credentials can be cleaned up explicitly.

Reason does not perform Dynamic Client Registration. Configure a pre-registered public client ID or a Client ID Metadata Document URL supported by the authorization server; no `client_secret` field exists in `reason-config-v1`. Project-level remote MCP configuration is high-risk network acquisition configuration and remains fail-closed until the project is approved with `reason trust add`.

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

Covered local MCP subprocesses start from the minimal isolated environment defined by #387 rather than inheriting the parent environment. Remote Bearer tokens are injected only into the HTTP request and are excluded from resolver config identity, telemetry, config files, sessions, evidence, stdout, and normal diagnostics. Config schemas reject credential-like unknown fields, including `access_token`, `refresh_token`, `client_secret`, and `api_key`. Resolver/admission telemetry records stable hashed non-secret config identities.

## Operational failures and replay

Transport, authentication, permission, **negotiation**, **session**, protocol, tool-execution, timeout, and policy-denial failures use typed operational resolution classes. Negotiation failure is used for an MCP handshake that returns an unsupported/missing negotiated revision; session failure is used for a broken initialized lifecycle such as an EOF/repeated pagination cursor. Malformed JSON-RPC remains `protocol`, and `isError: true` remains `tool_execution`. `timeout_ms` is one whole-invocation wall-clock deadline covering process spawn, every handshake/list/call stdin write and bounded response read, termination, and cleanup handoff. These outcomes remain distinct from semantic `unknown`.

Each MCP request carries stable request/attempt provenance and the resulting `ResolutionAttempt` records adapter/admission identities and cost telemetry. v3's config identity binds the requested/supported protocol policy and pagination bound; acquired evidence IDs also carry the negotiated revision so persisted replay provenance identifies the actual session revision. `ReasoningThread` replay restores recorded attempts and never invokes the MCP server again.

Deterministic fake-server tests cover allowlisted downlevel negotiation, unsupported-revision rejection, initialized-session breakage, read-only annotation enforcement, protocol/tool failure separation, bounded pagination/deadline behavior, opaque-result non-promotion, and the explicit Harness acquisition envelope. A one-off acceptance probe against the #204 pinned official `ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33` image also completed the v3 session and `get_file_contents` call successfully; its generic result remained opaque with zero fact candidates.
