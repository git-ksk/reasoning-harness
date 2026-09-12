# Provider usage and budget guards

[日本語](reason-usage-budget.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.0 development line. Harness Engine 0.4.2 correctness and authority semantics are unchanged.

Reason surfaces provider/resolver consumption as operational telemetry. Usage and budget exhaustion never become epistemic evidence, never change a verdict into authority, and never mean semantic `unknown`. Provider quota/rate-limit failures remain their existing typed operational failure classes and are not reclassified as semantic uncertainty.

## What Reason reports

Natural-language JSON output contains an additive `usage` object. Human output ends with a concise `Usage` section. Interactive sessions also expose `/usage`.

The view distinguishes:

- **model calls** — logical Reason model operations such as candidate generation, investigation planning/action selection/regeneration, final rendering, and model-backed answer-safety checks;
- **provider attempts** — actual provider HTTP attempts reported by adapters, including bounded adapter retries;
- provider-reported input/output/total tokens, when available;
- resolver/MCP/trusted-verifier calls, added-token accounting, elapsed time, and adapter-reported external cost where available.

A failed provider operation that does not return usage makes affected token totals `unreported`; Reason does not pretend the previous partial total is complete.

## Hard guards

The natural-language path supports provider-neutral guards through CLI flags or `run` config:

```bash
reason "TASK" \
  --max-model-calls 6 \
  --max-output-tokens 4096 \
  --max-total-tokens 12000
```

Equivalent `reason-config-v1` fields are `run.max_model_calls`, `run.max_output_tokens`, and `run.max_total_tokens`.

`max_model_calls` is checked before another logical model operation. `max_output_tokens` also caps each next model request to the remaining output-token allowance. `max_total_tokens` is enforced after each measurable provider response because input-token totals are provider-reported rather than guessed. If a configured token ceiling cannot be verified because the provider omitted usage, Reason fails closed with `usage_budget_unmeasurable`. Exhaustion uses `usage_budget_exceeded`. Both are typed operational failures, not semantic uncertainty.

Existing resolution/investigation limits remain the hard bounds for resolver/action work (`max_resolution_attempts` and configured investigation target/round/action limits).

## Managed-session accounting

Managed interactive sessions carry cumulative usage across turns and resumes. `/usage` shows the current cumulative total, and a newly executed turn enforces the configured ceiling against prior tracked usage.

Usage is persisted in a private `reason-managed-usage-v1` sidecar under the managed session root rather than adding fields to `reason-managed-session-v1`. This preserves the #381 0.5.x rollback contract: older CLIs ignore the sidecar. If a pre-usage-tracking session or a crash leaves history without matching usage state, Reason marks the historical accounting incomplete instead of assuming zero; cumulative hard guards that need that history fail closed.

Deleting/purging a managed session removes its usage sidecar. `reason uninstall --purge-data` removes the managed session root, including usage sidecars. Explicit-path low-level session files remain outside this managed-data scope.

## Estimated currency cost

Reason ships no stale built-in price table. Currency estimation is available only when the operator supplies both token rates and a provenance label:

```bash
reason "TASK" \
  --input-cost-per-million 0.50 \
  --output-cost-per-million 1.50 \
  --pricing-source "provider-price-sheet-2026-09-12"
```

The result is labelled **estimated model cost**. Missing token reporting means no estimate. Pricing telemetry never affects correctness or authority.

## Setup live check

`reason setup --live-check` sends a real provider request and may consume quota or incur provider cost. Interactive setup asks before running it; non-interactive help/JSON output also carries this disclosure. The check remains bounded to a minimal response.
