# External resolver adapters

Reasoning Harness v0.3.0 keeps external acquisition outside the correctness core. A resolver may fetch or compute data, but it does not gain authority merely because it ran successfully.

Issue #174 adds the first supported external acquisition adapter: `external_command_v1`. It implements the existing `ResolutionResolver` contract by launching one explicitly configured executable and exchanging one JSON request/response over stdio. No shell is inserted between `reason` and the configured program; every `--resolver-arg` / configured argument is passed literally.

The adapter remains inside the existing control path:

```text
ResolutionRequest
  -> external_command_v1
  -> AcquiredEvidence | CandidateRevision | NoResult | HumanReviewRequired
  -> EvidenceAdmissionPolicy
  -> ordinary validation / verification / diagnostics / decision / finalization
```

## CLI and config

The natural-language path accepts one external command resolver:

```bash
reason "Determine the failover region" \
  --hypothesis service.failover_region=eu-west-1 \
  --resolver-command /path/to/resolver \
  --resolver-arg --json \
  --max-resolution-attempts 1
```

The equivalent `reason-config-v1` fragment is:

```json
{
  "schema_version": "reason-config-v1",
  "resolution": {
    "external_command": {
      "program": "/path/to/resolver",
      "args": ["--json"],
      "admission": {
        "authority_ranks": {"secondary": 10, "primary": 20},
        "minimum_authority_class": "primary",
        "required_scope": {
          "region": {"kind": "values", "values": ["eu-west-1"]}
        },
        "sources": {
          "example:external-source": {
            "authority_class": "primary",
            "max_age_seconds": 300,
            "scope": {
              "region": {"kind": "values", "values": ["eu-west-1", "eu-west-2"]}
            }
          }
        }
      }
    }
  }
}
```

`--resolver-command` has CLI precedence over configured `resolution.external_command`. `--resolver-arg` is valid only with an explicit `--resolver-command`. The external command path and `--resolver-fact` are intentionally mutually exclusive for this first product lane because the current bounded runtime selects one resolver per resolver class.

Environment variables are inherited by the child process in the ordinary operating-system sense, so an integration may obtain credentials from its environment. Credentials are not copied into `ResolutionRequest`, acquired evidence, trusted metadata, receipts, or final output by the harness. Explicit secret transport and redaction/telemetry hardening are tracked separately in #178.

## Stdio protocol

`reason` writes one UTF-8 JSON object to resolver stdin using schema `reason-external-resolver-request-v1`:

```json
{
  "schema_version": "reason-external-resolver-request-v1",
  "adapter_id": "external_command_v1",
  "attempt_index": 0,
  "request": {
    "id": "resolution:proposition:service.failover_region=eu-west-1",
    "reason": "missing_support",
    "target": {
      "kind": "proposition",
      "proposition": {
        "key": "service.failover_region",
        "value": "eu-west-1"
      }
    },
    "resolver_class": "evidence_acquisition",
    "budget": {}
  }
}
```

The resolver writes exactly one JSON response to stdout using schema `reason-external-resolver-response-v1`. The acquisition form is:

```json
{
  "schema_version": "reason-external-resolver-response-v1",
  "contribution": {
    "kind": "acquired_evidence",
    "evidence": [
      {
        "id": "resolver-result-1",
        "source": "example:external-source",
        "observation": "service.failover_region=eu-west-1",
        "facts": {
          "service.failover_region": "eu-west-1"
        },
        "acquisition_metadata": {
          "observed_at_unix_seconds": 1788487200,
          "retrieved_at_unix_seconds": 1788487210,
          "scope": {
            "region": {"kind": "values", "values": ["eu-west-1"]}
          },
          "claimed_authority_class": "primary"
        }
      }
    ]
  },
  "cost": {
    "added_tokens": 0,
    "elapsed_ms": 0
  }
}
```

Other allowed contribution kinds are `candidate_revision`, `no_result`, and `human_review_required` because those already exist in `ResolutionResolverContribution`.

The response wire format is deliberately closed. Resolver responses cannot include `EvidenceMetadata`, verification receipts, a verdict, or final prose. Unknown response fields are rejected as malformed output instead of being silently interpreted as authority.

## Trust behavior after #175

External command acquisition remains **untrusted by default**. If `resolution.external_command.admission` is absent, the CLI still pairs `external_command_v1` with `RejectAllEvidenceAdmission`; a successful external call therefore cannot turn an unsupported target into `Supported`.

When admission is configured, the resolver may report only normalized **acquisition metadata**: exact `source` identity, observation time, retrieval time, applicability scope, and a claimed authority class. Those fields are not `EvidenceMetadata`. `external_evidence_admission_v1` checks them against Harness-owned configuration before any `Evidence` is created:

- `source` must exactly match an allowlisted source entry;
- `observed_at_unix_seconds` and `retrieved_at_unix_seconds` are required, retrieval cannot precede observation, and freshness is bounded by the configured per-source `max_age_seconds`;
- acquired scope cannot exceed the source's configured maximum and must cover any request/config-required scope;
- the resolver's `claimed_authority_class` must match the authority class assigned to that source by Harness configuration;
- the assigned source authority must meet the strongest configured/requested minimum under Harness-owned `authority_ranks`.

The resolver's authority claim is therefore only an assertion to check. The trusted `EvidenceMetadata.provenance_class` is copied from the Harness-owned source policy, never promoted from resolver output. Missing, stale, future, wrong-scope, unknown-authority, insufficient-authority, or mismatched-authority data fails closed.

Admission rejection is machine-observable on `ResolutionAttempt.admission_rejection` (for example `stale`, `scope_mismatch`, or `authority_claim_mismatch`). Rejection remains a resolution observation, not semantic evidence.

For admitted evidence, the Harness also installs the configured authority policy and qualification requirements into the ordinary input before re-running validation, evidence qualification, verification, diagnostics, decision, and finalization. A deterministic end-to-end regression proves that fresh/in-scope/allowed external evidence can recover an initially unknown target only after this re-verification, while stale evidence remains `unknown` with no verification receipt.

Candidate revisions still do not gain authority. A revised candidate is simply re-run through the same normal pipeline against harness-owned evidence and policy.

## Failure and budget behavior

The adapter maps executable-not-found to `unavailable`, invalid JSON/schema to `malformed_output`, and other process failures to the existing adapter `failed` class. Those remain operational outcomes, not semantic evidence.

The existing `GroundedResolutionPolicy` still owns resolver-class allowlisting and per-run/per-request attempt/token/time accounting. #174 does not add a second budget system. Stronger timeout/retry enforcement, detailed telemetry, redaction, and replay-safe external-call semantics are the scope of #178.

## Reference smoke path

The deterministic adapter test launches a temporary executable, sends a real typed request over stdin, receives acquired evidence over stdout, and separately verifies that attempts to smuggle trusted metadata or receipts fail schema parsing. This smoke path exercises actual process I/O without requiring a network service or changing frozen research fixtures.

For a live integration, the configured executable may itself call a web API, database, compiler/test tool, or other read-only source. Its returned data still enters Reasoning Harness only as the contribution types above. MCP-specific acquisition is intentionally deferred to #176 rather than being special-cased here.
