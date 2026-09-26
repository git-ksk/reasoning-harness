# Evidence relevance calibration: operational prior art before v6

Status: design input for Issue #462 after frozen v5 failure.

The v5 failure is an operational serving problem, not evidence that the Harness semantic contract is wrong. Relevant prior art comes from distributed-systems tail latency, overload control, retry policy, and LLM-serving evaluation.

## Prior art

### The Tail at Scale — Dean and Barroso, CACM 2013

Tail latency is a first-class system property. A small fraction of slow component requests can dominate end-to-end completion. Hedged requests can reduce tail latency, but they create extra load and therefore must be delayed and bounded.

Application to #462:
- report tail latency, not only means;
- do not treat a long-tail provider call as a semantic failure;
- do not introduce hedged duplicate model calls while the provider is explicitly overloaded.

Reference: https://barroso.org/publications/TheTailAtScale.pdf

### Google SRE — Handling Overload / Addressing Cascading Failures

Retries can amplify overload. Google describes per-request and client-level retry budgets, and recommends that only the layer immediately above a failing dependency retry. Under broad overload, failures should bubble upward rather than create retry storms.

Application to #462:
- keep provider retry ownership in the provider layer;
- add a run/client-level retry budget or overload circuit rather than increasing every case deadline;
- after explicit overload evidence, slow or stop new attempts for a bounded recovery interval;
- never add another independent retry loop in the calibration runner.

References:
https://sre.google/sre-book/handling-overload/
https://sre.google/sre-book/addressing-cascading-failures/

### Gemini API guidance

Google documents 503 UNAVAILABLE as temporary overload/down and recommends exponential backoff, jitter, retryable-error filtering, and a finite retry limit. Official SDKs use bounded automatic retry for transient errors.

Application to #462:
- the current Google adapter is bounded, but its transient and rate-limit fallback schedules are deterministic;
- add jitter without changing semantic behavior;
- preserve 503/high-demand as provider failure telemetry rather than scoring it as irrelevance;
- do not retry permanent/client errors.

References:
https://ai.google.dev/gemini-api/docs/troubleshooting
https://ai.google.dev/gemini-api/docs/api-errors

### AWS Builders / Well-Architected retry guidance

AWS likewise recommends bounded retries, exponential backoff with jitter, explicit client timeouts, fail-fast behavior under stress, and avoiding retry amplification. Timeout values that are too high waste resources; values that are too low can create more retry traffic.

Application to #462:
- do not react to v5 by mechanically changing 60 seconds to 120 seconds;
- coordinate timeout and retry envelopes instead of stacking independent policies;
- introduce jitter and an explicit overload recovery policy;
- keep the total operation bounded.

References:
https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
https://docs.aws.amazon.com/wellarchitected/latest/framework/rel_mitigate_interaction_failure_limit_retries.html
https://docs.aws.amazon.com/wellarchitected/2022-03-31/framework/rel_mitigate_interaction_failure_client_timeouts.html

### HELM and LLM-serving evaluation literature

HELM separates model quality from efficiency rather than collapsing them into one score. Modern LLM-serving work commonly reports high-percentile latency because mean latency hides tail behavior.

Application to #462:
- keep semantic correctness/utility and provider operational reliability as separate dimensions;
- freeze raw observations and failure classes;
- report p50/p95/max latency for canonical arms;
- do not improve a semantic score by dropping or relabeling failed provider cases.

References:
https://arxiv.org/abs/2211.09110
https://arxiv.org/abs/2410.14257
https://arxiv.org/pdf/2407.00023

### Recent retry-amplification evidence

A 2026 preprint on retry amplification reports that naive retries can reduce success under correlated failures and argues for adaptive retry budgets. This recent preprint is supporting evidence, not the primary basis for the design; it is consistent with long-standing Google/AWS production guidance.

Reference: https://arxiv.org/abs/2608.25403

## v6 design constraints derived from prior art

Before a canonical v6 is frozen:

1. Keep binding proposal v2 and target-first materialization v3 unchanged.
2. Keep semantic fixture labels unchanged.
3. Do not use hedged duplicate model requests.
4. Do not add a second retry loop above the provider adapter.
5. Keep the 60-second per-case Harness deadline unless new measurements justify another value.
6. Add bounded jitter to provider retry delays.
7. Add a bounded run-level overload/retry budget or recovery circuit so one provider-capacity incident does not cause a full calibration to hammer the same dependency.
8. Preserve typed provider failures separately from semantic outcomes.
9. Improve operational telemetry so canonical results report p50/p95/max latency and provider attempt/failure information.
10. Run a small unchanged-surface Google recovery smoke diagnostic first. If it passes 6/6, run a fresh broader recovery diagnostic over all 15 v5 failures plus four controls before selecting the exact v6 operational policy. If the smoke still fails, skip the broader probe and harden overload/retry behavior first.

The recovery diagnostic is not a v5 rerun and cannot turn v5 into a pass.
