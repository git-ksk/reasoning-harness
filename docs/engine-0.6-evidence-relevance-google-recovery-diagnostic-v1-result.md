# Engine 0.6 evidence relevance Google recovery diagnostic v1 result

Status: frozen operational FAIL. Do not rerun.

- tag: engine-0.6-evidence-relevance-google-recovery-diagnostic-v1
- commit: 9a3e5450058260e0bf01e85c3fda33ee087290e2
- Actions run: 36018360038
- run attempt: 1
- scope: six-case non-canonical recovery smoke over the unchanged v5 frozen surface

## Result

- operational: 4/6
- failed: 2/6
- failure class: assessment_timeout x2
- semantic materialization among completed cases: 4/4 exact
- semantic correctness/utility miss among completed cases: 0

Cases:

- 01_exact_name_availability: success, 35,157 ms
- 09_en_query_ja_alias: success, 27,105 ms
- 10_structured_metadata_plus_body: success, 50,383 ms
- 16_broad_landing_no_support: success, 20,800 ms
- 21_unknown_rename: assessment_timeout, 60,000 ms
- 26_url_only_identity: assessment_timeout, 60,000 ms

The GitHub Actions workflow conclusion is success because the diagnostic workflow intentionally preserved artifacts and exited zero after summarization. That green workflow status is not the diagnostic acceptance result. The authoritative summary field is operationally_complete=false. Future diagnostic workflows must make this distinction visible in their job conclusion or an explicit gate job.

## Decision

The smoke did not demonstrate provider recovery. Therefore do not launch the proposed 19-case recovery diagnostic v2 and do not proceed directly to canonical v6. Implement operational hardening first, while leaving relevance semantics unchanged.

Priority design work before v6:

1. bounded randomized jitter in the Google adapter retry schedule;
2. run/client-level retry or overload budget so retries cannot amplify provider overload;
3. explicit overload/cooldown or circuit behavior after provider-capacity evidence;
4. p50/p95/max latency and clearer provider-attempt/failure telemetry;
5. diagnostic workflows whose visible conclusion matches operational acceptance;
6. keep the current 60-second Harness case deadline unless new measurements justify changing it;
7. no hedged duplicate model calls and no second retry loop in the calibration runner.
