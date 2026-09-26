# Engine 0.6 candidate: evidence-target relevance calibration v5

Status: fresh unobserved successor after frozen v4 and the bounded Google model diagnostic. v1-v4 remain immutable historical evidence and must not be rerun or rescored.

## Why v5 exists

Frozen v4 run `35998574508` established the semantic contract on Mistral but left the Google arm operationally incomplete under the 30,000 ms Harness assessment deadline:

- Mistral: 26/26 operational, 26/26 exact materialized disposition, correctness 0, utility 0.
- Google: 23/26 operational; three cases ended at approximately 30,001 ms with typed `assessment_timeout`.
- The 23 completed Google cases had a 1,175 ms median latency and a 29,703 ms maximum latency.
- The subsequent five-case Google model diagnostic run `36000374933` kept the same v4 semantics and 30,000 ms budget. `gemini-3.5-flash-lite` completed 5/5 with no provider retry; its slowest case completed in 27,389 ms. `gemini-3.1-flash-lite` remained operationally weaker, so 3.5 remains canonical.

The observed tail is therefore too close to the 30-second outer Harness deadline. No evidence supports changing the semantic contract, materializer, canonical Google model, pacing, or adapter retry policy.

## v5 change

v5 makes one calibration-policy change only:

- assessment elapsed budget: **60,000 ms** per case.

Unchanged:

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`;
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`;
- max model calls: 2;
- max output tokens: 192;
- Google canonical model: `gemini-3.5-flash-lite`;
- Mistral canonical model: `ministral-8b-latest`;
- Google request-start pacing in Actions: 6,000 ms, with 6,100 ms inter-case delay;
- provider adapter bounded retry/backoff policy;
- all 26 semantic families and expected proposal/disposition labels.

The Google adapter itself permits at most four provider attempts. Short-window rate-limit retries are bounded by Retry-After or 10/20/40-second backoff; transient 5xx retries use 2/5/10 seconds. A 60-second Harness deadline deliberately does **not** absorb every possible adapter retry envelope. It allows measured latency headroom and limited retry recovery while still surfacing sustained provider instability as an operational failure.

## Fresh identity

- suite: `evidence-relevance-calibration-v5`
- issue: #462
- cases: 26
- seed: `4625605`
- status: `fresh_unobserved_calibration`
- production motivating incident: excluded from tuning

v5 copies the v4 semantic corpus solely to preserve calibration comparability. The only manifest-level policy delta is `max_elapsed_ms: 60000`; expected bindings and dispositions are unchanged.

## Acceptance

Both canonical arms must be operationally complete:

- successful provider cases: 26/26;
- failed provider cases: 0;
- wrong-target / unsafe relevance admission: 0;
- false relevance rejection: 0;
- expected-relevant left ambiguous: 0;
- utility miss: 0.

Proposal exact accuracy remains diagnostic. Final Harness-materialized disposition is the release-gating semantic result.

The frozen v5 workflow rejects reruns. If the first observation fails, v5 remains immutable failed evidence and the next attempt requires a new successor identity such as v6.

No independent holdout is authored until v5 passes and the #462 semantic implementation is frozen.
