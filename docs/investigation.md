# Bounded investigation planning

Issue #212 adds an opt-in investigation lane to the natural-language `reason "TASK"` path. It is a product-control layer for deciding **what to investigate next** when an answer is not already accepted. It does not give the planner, model, or acquisition tool correctness authority.

Machine identities:

- runtime: `bounded-investigation-v1`
- plan proposal: `reason-investigation-plan-v1`
- action proposal: `reason-investigation-action-v1`
- investigation external-command adapter: `investigation_external_command_v1`
- investigation external-command request: `reason-investigation-external-resolver-request-v1`
- MCP acquisition: `mcp_readonly_v2`

The frozen historical `mcp_readonly_v1` implementation and the existing static `external_command_v1` request protocol remain unchanged.

## Control flow

When `resolution.investigation` is configured and the initial natural-language run is not already `Accept`, the Harness runs this bounded loop:

1. The model proposes a small list of investigation questions through the closed `reason-investigation-plan-v1` JSON schema. These targets are explicitly marked `model_proposed_untrusted`.
2. The Harness validates target count, identity, and shape and turns accepted proposal objects into canonical investigation targets. This is planning admission only; it does not make any proposed answer true.
3. For each round, the model may select one existing target ID and one existing configured read-only capability ID, or stop. The action schema contains no arbitrary query text, tool arguments, authority class, evidence, receipt, or verdict fields.
4. The Harness rejects unknown capabilities, capabilities not declared read-only, selector/key mismatches, duplicate target/capability pairs, and actions beyond the configured budgets.
5. The selected acquisition adapter runs once. Investigation external commands use their own request identity rather than extending `external_command_v1`. MCP uses the v0.4 `mcp_readonly_v2` operational successor and Harness-owned fixed arguments/tool allowlists.
6. Acquired evidence is still untrusted. If an admission policy is configured, normal source allowlisting, freshness, scope, and authority rules are applied. Without admission, external data cannot become trusted evidence.
7. After admitted evidence is added, the natural-language candidate is regenerated against the updated Harness input and passes through the ordinary validation, qualification, verification, diagnostics, verdict, finalization, and answer-safety path again.
8. Typed outcomes such as `no_result`, `rejected_evidence`, `ambiguous`, `verification_progress`, or `operational_failure` are recorded. A later planner round can select another untried capability based on those outcomes.
9. The loop stops on resolution, explicit planner stop, target exhaustion, action/round budget exhaustion, repeated no progress, or an operational terminal.

A successful acquisition is therefore **not** a successful verification. Only the ordinary Harness authority path can make a factual claim eligible for grounded exposure.

## Configuration

`resolution.investigation` is an alternative acquisition lane to `--resolver-fact`, `resolution.external_command`, or `resolution.mcp_readonly`. Those four acquisition lanes are mutually exclusive. `resolution.trusted_command` remains a separate downstream verifier and may still be configured.

Example with two independently configured read-only capabilities:

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

All capability IDs must be unique. `read_only` must be `true`. When multiple capability admission policies are present, their authority-rank policy must match so the investigation cannot silently switch authority systems between rounds. MCP tool arguments remain Harness-owned fixed configuration; the model does not generate them.

An investigation external command receives `reason-investigation-external-resolver-request-v1`, not `reason-external-resolver-request-v1`. The response remains the closed `reason-external-resolver-response-v1` envelope, but the investigation runtime accepts only acquired-evidence/no-result behavior; candidate revision and human-review contributions are rejected by the acquisition-only boundary.

## Boundedness and telemetry

The runtime is bounded structurally by `max_targets`, `max_rounds`, `max_actions`, and `max_no_progress_rounds`. Every planner call also has `planner_max_tokens`, candidate regeneration uses the existing bounded natural-language generation limit, and each acquisition adapter retains its configured whole-invocation timeout/response-size cap. Invalid/repeated actions consume bounded rounds even when they do not consume an acquisition action.

Natural JSON output reports the investigation object separately from ordinary resolution rounds. It contains the accepted targets/capability descriptors, planner-call count, typed action rejections, action records, admitted-evidence counts, verification-progress flags, stop reason, and provider observations for plan/action/candidate-regeneration calls. Provider or protocol failure remains operational evidence; it is never converted into a semantic fact or `unknown` proof.

## Static-path compatibility

If `resolution.investigation` is absent, the existing natural-language resolver behavior is unchanged. `--resolver-fact`, static `external_command_v1`, static `mcp_readonly_v2`, and `trusted_command_verifier_v1` continue to use their existing paths. Frozen research/evaluation surfaces remain untouched and `mcp_readonly_v1` remains byte-for-byte protected by its freeze workflow.
