# Reference trusted command verifier

Issue #177 adds `trusted_command_verifier_v1` as a reference authority-bearing adapter on the existing `TrustedResolutionVerifier` boundary. It is deliberately separate from acquisition adapters such as `external_command_v1` and `mcp_readonly_v1`.

The external command does **not** return a `VerificationReceipt`. It receives the exact typed resolution request plus current Harness evidence/qualification policy and may return only `supported`, `contradicted`, or `no_result` with referenced evidence IDs. Reasoning Harness constructs the receipt itself, binds it to the exact requested proposition, and records the operator-configured verifier identity.

`resolution.trusted_command.trusted` must be explicitly `true`. This is an operator trust decision: configuring an LLM, retriever, arbitrary shell script, or unreviewed service as trusted would weaken correctness and is unsupported as a safe deployment pattern. Prefer deterministic compilers/tests/schema validators/signature verifiers/policy engines or another explicitly trusted oracle.

When a matching `EvidenceRequirement` exists, every evidence ID returned by the trusted command must already be `Qualified` under the current Harness temporal/scope/authority policy. Stale, wrong-scope, unknown-authority, or otherwise non-qualified evidence cannot be used to mint a receipt through this reference adapter. Without an applicable requirement, referenced evidence must still exist in the current artifact and IDs must be unique/non-empty.

The response schema is closed (`reason-trusted-verifier-response-v1`), so it cannot smuggle verifier identity, proposition bindings, receipt IDs, verdicts, or final prose. Operational failures remain typed resolution failures. Timeout and response-size limits are bounded, config identity is hashed, and ReasoningThread replay preserves the recorded attempt without re-executing the verifier.

Example config:

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "trusted_command": {
      "trusted": true,
      "verifier_id": "reference-policy-oracle",
      "program": "/usr/local/bin/policy-oracle",
      "args": ["--stdio"],
      "timeout_ms": 5000,
      "max_response_bytes": 65536
    }
  }
}
```

A supported response is shaped like:

```json
{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"]}}
```

Acquisition success and trusted verification success remain separate resolution attempts/metrics. Data returned by a resolver does not self-promote merely because a trusted verifier can later inspect it.
