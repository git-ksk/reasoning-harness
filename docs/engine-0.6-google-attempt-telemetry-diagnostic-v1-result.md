# Engine 0.6 Google attempt telemetry diagnostic v1 result

Status: frozen PASS as a provider-observation diagnostic. This result does not change or rescore calibration v6.

## Frozen identity

- issue: #462
- branch: `diag/462-google-attempt-telemetry-v1`
- freeze tag: `engine-0.6-google-attempt-telemetry-diagnostic-v1`
- freeze commit: `a483bb09ae006b05dd1f56807f54c643c181575a`
- first/only Actions run: `36026307543`
- run attempt: 1
- provider/model: Google `gemini-3.5-flash-lite`
- probe cases: `01_exact_name_availability`, `02_acronym_alias`, `03_expanded_alias`
- Harness case deadline: 60,000 ms
- Google request-start pacing: 6,000 ms

## Result

All three probe cases were operationally successful.

- planned / completed: 3 / 3
- successful / failed provider cases: 3 / 0
- provider attempts: 3 total, one per call
- provider-attempt telemetry incomplete observations: 0
- retries: 0
- in-flight cancellations: 0
- HTTP statuses: six telemetry events at 200 (headers + completed response for each call)
- HTTP 429: 0
- HTTP 503: 0
- `RESOURCE_EXHAUSTED`: 0
- `UNAVAILABLE`: 0
- quota-window evidence: none
- typed provider error evidence: none

Header-response latency by call:

1. call 1: 35,871 ms
2. call 2: 28,397 ms
3. call 3: 33,530 ms

Runner case latency was 35,872 / 28,397 / 33,531 ms respectively. Materialized dispositions were exact for all three probes.

The frozen diagnostic summary classified the observation as `no_quota_or_capacity_signal_observed` and `provider_operational=true`.

## Interpretation

This observation provides no evidence that an active Google quota or rate limit caused the repeated v6 failures. The same early request shapes that produced two 60-second v6 assessment timeouts later completed on the first provider attempt with HTTP 200 and no retry.

The diagnostic cannot retroactively prove that no hidden transient quota event occurred during v6 because v6 did not yet expose per-attempt HTTP status before outer cancellation. However, the combined evidence now favors intermittent serving-tail / capacity variability over quota exhaustion:

- v5 captured an explicit HTTP 503 high-demand response after four provider attempts;
- v6 had one 47-second success followed by two requests that returned no completed adapter result before the 60-second Harness deadline;
- this diagnostic later completed those same early request shapes in roughly 28-36 seconds with HTTP 200, no retry, and no quota signal.

A three-case healthy probe is not enough to requalify Google as a required 26-case canonical provider. It only resolves the immediate quota question and confirms that the Google path can recover without configuration or semantic changes.

## Consequence

Do not increase the 60-second semantic-assessment deadline or remove the v6 operational hardening based on this result. Retain bounded jitter, fail-fast load shedding, attempt telemetry, and tail-latency reporting.

Google may remain useful for replication / operational requalification, but required-provider gating should be based on independent operational evidence rather than assuming a quota problem or tuning to #462 semantic outcomes.
