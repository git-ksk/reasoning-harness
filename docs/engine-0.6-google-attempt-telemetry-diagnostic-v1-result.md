# Engine 0.6 Google attempt telemetry diagnostic v1 result

Status: frozen diagnostic PASS. This is provider-operational evidence, not a semantic calibration pass.

## Frozen identity

- issue: #462
- freeze tag: `engine-0.6-google-attempt-telemetry-diagnostic-v1`
- freeze commit: `a483bb09ae006b05dd1f56807f54c643c181575a`
- first/only Actions run: `36026307543`
- run attempt: 1
- model: `gemini-3.5-flash-lite`
- request pacing: 6,000 ms
- inter-case delay: 6,100 ms
- Harness case deadline: 60,000 ms
- cases: `01_exact_name_availability`, `02_acronym_alias`, `03_expanded_alias`

The diagnostic does not rerun, rescore, or modify frozen calibration v6.

## Provider observation

All three cases completed on the first provider attempt with HTTP 200.

| case | provider attempts | latency | HTTP result |
| --- | ---: | ---: | --- |
| `01_exact_name_availability` | 1 | 35,872 ms | 200 |
| `02_acronym_alias` | 1 | 28,397 ms | 200 |
| `03_expanded_alias` | 1 | 33,531 ms | 200 |

Aggregate provider metrics:

- planned / completed: 3 / 3
- successful / failed provider cases: 3 / 0
- provider attempts: 3
- provider-attempt telemetry incomplete observations: 0
- retries: 0
- `cancelled_in_flight`: 0
- observed 429 responses: 0
- observed 503 responses: 0
- typed quota / rate-limit errors: 0
- typed provider-unavailable errors: 0
- provider status values such as `RESOURCE_EXHAUSTED` / `UNAVAILABLE`: none
- latency p50 / p95 / max: 33,531 / 35,872 / 35,872 ms

The workflow classifier emitted:

`no_quota_or_capacity_signal_observed`

## Semantic observation

The diagnostic was non-canonical and is not used to re-score v6. For completeness, all three completed cases materialized to their expected relevance disposition with zero semantic misses.

## Interpretation

This run provides no evidence that a persistent project quota or rate limit is currently blocking these requests. If Gemini had returned a quota/rate-limit response before the Harness deadline, the attempt telemetry would have preserved HTTP 429 plus its typed `quota` / `rate_limit` classification and structured quota window when present. None was observed.

It also provides no 503 / `UNAVAILABLE` evidence during this specific run. Instead, all requests returned HTTP 200, but with high first-attempt latency of roughly 28-36 seconds.

This does **not** retroactively prove that calibration v6 could not have encountered an unobserved quota response: v6 did not yet have attempt-level HTTP telemetry and its two timed-out adapter futures were cancelled before returning a completed provider result. However, the combined evidence now favors variable Google serving latency/capacity over a persistent quota explanation:

- v5 directly observed HTTP 503 high-demand after bounded provider retries;
- v6 observed one 47.2-second success followed by two 60-second outer timeouts;
- this diagnostic observed the same early request shapes as three first-attempt HTTP 200 responses, still taking 28-36 seconds.

The provider is therefore reachable and not currently quota-blocked, but its latency remains materially variable for this evaluation path.

## Premature v7 attempt

A v7 calibration freeze was started before this diagnostic completed. Run `36026148264` was cancelled once the sequencing violation was detected. Its preflight completed, while both live provider arms were cancelled. The v7 run is not acceptance evidence and must not be rerun or rescored under the same identity.

Any successor canonical calibration must use a fresh identity.

## Gate implication

Do not increase the 60-second deadline merely to absorb provider tail latency, and do not reinterpret provider failures as semantic failures.

For the next fresh canonical successor, required provider selection should continue to use provider-operational evidence independent of #462 semantic scores. Google remains valuable replication/diagnostic evidence, but repeated v4-v6 operational instability means it should not be the sole reason a semantically stable candidate cannot be evaluated. A stable alternate required provider may be used if justified independently; Google can remain as non-gating replication evidence.

Independent holdout authoring remains blocked until a fresh canonical calibration passes.
