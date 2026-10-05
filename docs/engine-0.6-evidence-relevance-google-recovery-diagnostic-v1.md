# Engine 0.6 evidence relevance: Google recovery diagnostic v1

Status: fresh non-canonical operational diagnostic after frozen v5 FAIL.

This diagnostic does not change or rescore v5. It cannot make v5 pass. It is a six-case recovery smoke probe only and is not sufficient by itself to authorize v6.

## Question

Was the Google failure in canonical v5 primarily a transient provider-capacity incident, or is gemini-3.5-flash-lite still operationally unstable under the unchanged v5 envelope?

## Frozen inputs

- exact v5 semantic/evaluation surface
- binding proposal v2
- target-first materialization v3
- 2 model calls
- 192 output tokens
- 60,000 ms elapsed budget
- Google model gemini-3.5-flash-lite
- Google request-start pacing 6,000 ms
- inter-case delay 6,100 ms

The workflow verifies fixtures/evidence-relevance-calibration-v5/surface-v5.sha256 before credentials are used.

## Diagnostic subset

- 01_exact_name_availability — v5 success baseline
- 09_en_query_ja_alias — v5 assessment timeout
- 10_structured_metadata_plus_body — v5 explicit 503 high-demand failure
- 16_broad_landing_no_support — v5 timeout; completed in the earlier five-case diagnostic
- 21_unknown_rename — v5 timeout; completed in the earlier five-case diagnostic
- 26_url_only_identity — v5 late-run timeout

This is a recovery probe over existing calibration material, not a new semantic corpus.

## Reading the result

- 6/6 operational completion with no provider failure supports a transient-capacity interpretation, but does not erase v5 FAIL and does not authorize v6.
- if v1 is 6/6, author a fresh recovery diagnostic v2 covering all 15 v5-failed cases plus four control successes spanning early and late portions of the run before selecting the v6 operational design.
- if v1 still shows timeout / provider-unavailable, skip the broad v2 probe and implement overload/retry hardening before any v6 canonical run.
- semantic disposition is diagnostic here; expected labels are unchanged.

The diagnostic is first/only and rejects workflow reruns.
