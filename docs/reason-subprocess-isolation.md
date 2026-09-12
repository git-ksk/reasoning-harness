# Local subprocess environment isolation

Reason CLI 0.5.0 treats local external acquisition processes as a separate OS-process trust boundary. This hardening does not change Harness Engine 0.4.2 evidence, verification, or authority semantics.

## Covered product lanes

The current product paths for external-command acquisition, MCP read-only v3 acquisition, and trusted-command verification start children from an explicit minimal environment. The v2 MCP compatibility adapter uses the same boundary. Historical frozen `mcp_readonly_v1` remains byte/semantic-frozen for research replay and is not the Reason CLI product MCP path.

## Environment boundary

Before spawning a covered child, Reason clears ambient environment inheritance and restores only a launch baseline:

- `PATH`;
- temporary-directory variables: `TMPDIR`, `TMP`, `TEMP`;
- locale/time variables when present: `LANG`, `LANGUAGE`, `LC_ALL`, `LC_CTYPE`, `LC_MESSAGES`, `TZ`;
- on Windows: `SystemRoot`, `WINDIR`, `ComSpec`, `PATHEXT`, and `SystemDrive`.

Provider credentials and unrelated developer variables such as `MISTRAL_API_KEY`, `GEMINI_API_KEY`, `AWS_*`, `GH_TOKEN`, `HOME`, and arbitrary project variables are not copied into covered children.

This is environment isolation, not a filesystem sandbox. A local executable still runs with the user's OS identity and filesystem permissions, so project trust and read-only/capability policy remain required.

## Integration-specific credentials

The provider layer has an explicit scoped-environment injection boundary for a selected integration, but Reason CLI 0.5.0 does not expose arbitrary ambient-variable inheritance or secret-valued project config. A future integration credential must come from a product-owned secure source and be injected only into that selected child. It must not become model context, evidence authority, diagnostics text, or persisted configuration.

## Compatibility and automation

JSON/pipe contracts are unchanged. Provider/model selection and Harness authority are unchanged. Missing integration-specific environment is expected to surface as the child integration's ordinary typed operational failure; Phase 4 `reason doctor` can add secret-free remediation without weakening this boundary.
