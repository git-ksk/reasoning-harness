# Reason doctor

**Status:** implemented on the Reason CLI 0.5.0 development line. Harness Engine 0.4.2 semantics are unchanged.

`reason doctor` is the read-only diagnostics surface for installation, configuration, credentials, local product readiness, managed data paths, project trust, MCP configuration, and version identity.

```text
reason doctor
reason doctor --format json
reason doctor --live-check
```

The default command does not make provider requests, contact remote MCP servers, or check GitHub releases. It reports local readiness only. `--live-check` additionally performs bounded provider connectivity, configured user-MCP readiness, and update-availability checks. A provider live check sends a real request and may consume quota or incur provider cost.

## Stable diagnostics

Human and JSON output report:

- Reason CLI and Harness Engine versions separately;
- current executable path and an installation method only when it can be inferred safely;
- user/project config paths, presence, validity, and effective config sources;
- provider credential presence/source without credential values;
- native OS credential-store availability;
- configured provider/model compatibility and local readiness;
- optional bounded provider live readiness;
- managed-session path health;
- current project trust state without granting trust;
- configured user-MCP presence and optional read-only readiness probe;
- optional update-availability signal;
- typed, secret-free diagnostic issues and a safe recovery command.

The JSON result uses `doctor_surface: "reason-doctor-v1"` inside the existing `reason-cli-output-v1` product envelope.

## Safety boundary

`reason doctor` never prints API keys, OAuth tokens, refresh tokens, or other credential values. It does not mutate config, trust state, sessions, credentials, MCP configuration, or Engine state. A normal run does not activate configured MCP processes or make provider/network requests. Live MCP probing uses the same read-only readiness path as `reason mcp test` and never invokes the selected tool.

Operational diagnostics remain separate from semantic `unknown`. Doctor status describes product readiness; it does not certify evidence, claims, or final answers.
