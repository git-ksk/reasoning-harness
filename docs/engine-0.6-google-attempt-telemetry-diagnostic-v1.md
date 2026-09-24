# Engine 0.6 Google attempt telemetry diagnostic v1

Status: fresh provider-operational diagnostic after frozen evidence-relevance calibration v6 FAIL.

This diagnostic does not rerun, rescore, or modify v6. It exists only to distinguish explicit Google quota/rate-limit responses from provider-capacity responses and requests that do not return HTTP headers before the Harness deadline.

## Frozen question

For `gemini-3.5-flash-lite`, when the same early v6 request shape is exercised under the unchanged 60,000 ms Harness case deadline, does Google return:

- HTTP 429 / `RESOURCE_EXHAUSTED` quota or short-window rate-limit evidence;
- HTTP 503 / `UNAVAILABLE` provider-capacity evidence;
- or no HTTP response headers before the Harness deadline cancels the in-flight adapter future?

## Diagnostic scope

The diagnostic reuses three already-observed v6 calibration cases solely as request-shape probes:

- `01_exact_name_availability`
- `02_acronym_alias`
- `03_expanded_alias`

The relevance semantics, expected labels, binding proposal v2, target-first materialization v3, 60-second case deadline, Google model, and 6-second request pacing are unchanged. This is not a semantic calibration and cannot make v6 pass.

## Attempt telemetry

The Google adapter writes opt-in JSONL telemetry only when `REASON_GOOGLE_ATTEMPT_TELEMETRY_PATH` is configured. Events contain safe operational metadata only:

- generated `call_id` and provider attempt number;
- model identifier;
- attempt start;
- HTTP status as soon as response headers arrive;
- classified error (`quota`, `rate_limit`, `provider_unavailable`, and so on);
- structured quota window when Google supplies one;
- bounded retry delay;
- safe rate-limit response headers;
- bounded provider error detail;
- in-flight cancellation before a completed response classification.

API keys, request prompts, system instructions, candidate evidence text, and response body content are not written to telemetry.

## Acceptance reading

- Any 429 classified as `quota` or `rate_limit` is explicit quota/rate-limit evidence.
- Any 503 classified as `provider_unavailable` is explicit provider-capacity evidence.
- `cancelled_in_flight` without any prior HTTP headers for that `call_id` means the Harness deadline expired before Google returned response headers; it is not evidence of a quota response.
- Mixed signals remain mixed; the diagnostic must not collapse them into a single semantic failure class.

The workflow is first/only and rejects reruns. A provider-operational failure is intentionally visible as a red gate after artifacts and diagnostic classification have been preserved.
