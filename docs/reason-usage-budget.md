# Provider usage and budget guards

[日本語](reason-usage-budget.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.x product line. Harness Engine 0.4.2 correctness and authority semantics are unchanged.

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

`max_model_calls` is checked before another logical model operation. With `max_output_tokens`, Reason caps each provider request, including structured-output fallbacks, to the remaining cumulative output allowance. If a configured token ceiling cannot be verified because usage is missing, Reason stops before a further request with `usage_budget_unmeasurable`; an exhausted allowance yields `usage_budget_exceeded`. Both are typed operational failures, not semantic uncertainty.

When either token ceiling is configured, Reason disables adapter-internal HTTP retries for that run, so a rate limit or transient error can end the operation sooner. The existing schema compatibility fallbacks remain possible when usage is known and allowance remains. Model clients also disable implicit HTTP redirects and transport-level retries; their explicit adapter retry policies still apply when no token ceiling is configured. The output cap relies on the provider honoring the request limit. `max_total_tokens` is checked after each measurable response: provider input tokens are unknown before dispatch, so a single response can exceed the total ceiling. This is not a pre-dispatch hard bound on total usage or currency cost.

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

## Usage on failed runs

After #535, source builds preserve observed usage in a failed natural invocation's optional `result.partial_diagnostics.usage`; `complete` remains false. Budget enforcement and the original `usage_budget_exceeded` / `usage_budget_unmeasurable` failure stay unchanged. Missing token totals remain unavailable. On cancellation, completed-operation counters are lower bounds and total-token/currency estimates are omitted because an in-flight request may have been billed. This diagnostic field does not create a successful answer or a managed-session checkpoint. Published CLI0.5.5 artifacts remain unchanged.
