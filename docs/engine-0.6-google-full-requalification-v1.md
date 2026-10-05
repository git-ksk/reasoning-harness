# Engine 0.6 Google full requalification v1

Status: fresh non-gating provider-operational diagnostic after frozen v4-v6 Google instability and the three-case attempt-telemetry recovery probe.

This study does not rerun or rescore calibration v7, does not change #462 relevance semantics, and does not make Google a required canonical provider by itself.

## Question

Can Google `gemini-3.5-flash-lite` complete the full 26-case evidence-relevance request-shape set under the already-frozen operational envelope without quota, provider-unavailable, timeout, or circuit-abort failures?

## Frozen operational envelope

- model: `gemini-3.5-flash-lite`
- case deadline: 60,000 ms
- maximum model calls per case: 2
- maximum output tokens: 192
- Google request-start minimum interval: 6,000 ms
- inter-case delay: 6,100 ms
- adapter retries remain bounded and jittered
- run-level circuit opens after two consecutive operational provider failures
- Google attempt telemetry records HTTP status/provider status/retry/cancellation metadata only; request prompts and response-body content are not logged

## Request-shape set

All 26 frozen v7 calibration fixtures are passed explicitly with `--fixture`. This intentionally makes the run non-canonical even though every request shape is covered:

- the run cannot become v7 acceptance evidence;
- semantic labels are not changed;
- semantic metrics are preserved only as diagnostics;
- the provider gate is operational only.

## Precommitted operational PASS

Google is operationally requalified by this study only if the first/only frozen run has all of the following:

- planned cases: 26
- completed cases: 26
- successful provider cases: 26
- failed provider cases: 0
- operational abort: none
- runner exit: 0

HTTP 429 / `RESOURCE_EXHAUSTED`, HTTP 503 / `UNAVAILABLE`, retries, in-flight cancellation, attempt counts, and latency p50/p95/max are recorded regardless of PASS/FAIL.

Materialized exactness, correctness, and utility metrics are reported but do not change the provider-operational verdict. Any future decision to restore Google to a required canonical provider is separate and must consider this result together with historical v4-v6 evidence.

The workflow rejects reruns. A failed or incomplete result remains frozen evidence.
