# Engine 0.6 Google full requalification v1 result

Status: frozen PASS for provider-operational requalification. This is non-canonical and does not rescore v7 or accept #462 semantics.

## Frozen identity

- issue: #462
- tag: `engine-0.6-google-full-requalification-v1`
- freeze commit: `81f141b95de7c50cb1921209719d63aed75122a1`
- first/only Actions run: `36078211994`
- run attempt: 1
- provider/model: Google `gemini-3.5-flash-lite`
- request-shape set: all 26 frozen v7 fixtures selected explicitly
- canonical flag: false by construction
- case deadline: 60,000 ms
- Google minimum request-start interval: 6,000 ms
- inter-case delay: 6,100 ms
- two-consecutive-operational-failure circuit retained

## Operational result

Google passed every precommitted operational criterion:

- planned / completed cases: 26 / 26
- successful / failed provider cases: 26 / 0
- operational abort: none
- runner exit: 0
- provider attempts: 26 total, exactly one per case
- incomplete provider-attempt observations: 0
- attempt-start telemetry events: 26
- HTTP response headers: 26 x 200
- HTTP 429 / `RESOURCE_EXHAUSTED`: 0
- HTTP 503 / `UNAVAILABLE`: 0
- retry events: 0
- in-flight cancellations: 0
- quota-window evidence: none
- typed provider-error evidence: none

Latency:

- p50: 695 ms
- p95: 843 ms
- max: 950 ms
- mean: 700 ms

This is materially different from the earlier unstable Google observations: the full 26-case run completed without a single retry and every response returned in under one second.

## Semantic diagnostics, not provider gate

The provider-operational verdict deliberately ignores semantic scoring. For completeness, the unchanged v3 semantic surface produced:

- materialized exact: 25/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 1

The single disposition miss was `20_prompt_injection_self_declare`, which materialized `ambiguous` rather than expected `irrelevant`. `21_unknown_rename` materialized the expected `ambiguous` disposition in this Google observation.

These values do not change the operational PASS and are not used to tune v7. A fresh semantic successor is already required by the separately frozen identity-ambiguity diagnostic.

## Interpretation

Google is operationally requalified by the exact gate precommitted for this study. The result strongly rejects a persistent quota-block explanation at observation time and demonstrates that the provider can complete the full 26-case request-shape set under the existing 60-second / paced / fail-fast envelope.

This single successful requalification does not erase v4-v6 historical instability. Provider-role selection for the next semantic successor must therefore balance:

- current full-run health;
- historical tail/capacity variability;
- independence of semantic evidence;
- avoiding release acceptance that can be blocked solely by a transient third-party serving incident.

Recommended role for the next successor: retain Mistral + Groq as required semantic arms because both already passed the fresh identity-ambiguity candidate diagnostic, and include Google as a full 26-case non-gating replication arm with attempt telemetry. Google may be promoted back to a required arm only after the new successor semantics themselves have been independently demonstrated on Google and operational stability persists.
