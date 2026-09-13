# Operational failure recovery

**Status:** implemented on the Reason CLI 0.5.0 development line. This is a product UX layer; Harness Engine 0.4.2 reasoning and authority semantics are unchanged.

Reason keeps operational failures separate from semantic outcomes such as `unknown`. A provider outage, invalid credential, broken MCP server, corrupt session, or update-integrity failure never becomes a semantic answer.

## Human errors

For product commands, recovery-oriented human errors answer four questions:

1. **What failed** — the product subsystem that prevented completion.
2. **Task execution** — whether execution did not start, may have started, or was interrupted/incomplete.
3. **Result trust** — whether any result from the failed operation can be treated as trustworthy.
4. **Next** — one safe command to inspect or recover without weakening security checks.

Example shape:

```text
Error: GROQ_API_KEY is set but empty; refusing OS-store fallback
What failed: provider credential or native credential-store readiness
Task execution: the task did not start
Result trust: no result was produced
Next: reason auth status
```

The remediation map covers credentials, model/capability compatibility, provider rate limits/quota/outages, provider protocol or malformed-output failures, configuration/project trust, MCP configuration/authentication/negotiation/read-only readiness, session state, usage budgets, cancellation, and update/version/distribution integrity.

Recovery commands intentionally use existing safe surfaces such as `reason auth status`, `reason models`, `reason config sources`, `reason trust status`, `reason mcp list`, `reason session list`, `reason update --check`, and `reason doctor [--live-check]`. Reason never recommends disabling TLS, bypassing provenance/integrity checks, weakening project trust, or silently switching provider/model identity.

## JSON compatibility

Existing machine-readable `failure.failure_class` values remain unchanged. The ordinary product failure envelope adds a `remediation` object:

```json
{
  "status": "failed",
  "failure": {
    "failure_class": "credentials",
    "message": "..."
  },
  "remediation": {
    "what_failed": "provider credential or native credential-store readiness",
    "task_execution": "not_started",
    "result_trust": "no_result",
    "next_command": "reason auth status"
  }
}
```

The remediation fields are operational metadata only. They do not alter evidence, verification, finalization, or semantic verdicts.
