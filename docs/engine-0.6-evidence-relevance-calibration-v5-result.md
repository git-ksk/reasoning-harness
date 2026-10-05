# Engine 0.6 evidence-target relevance calibration v5 result

Status: frozen FAIL. Do not rerun or rescore.

## Frozen identity

- issue: #462
- branch: feat/462-evidence-target-relevance
- freeze tag: engine-0.6-evidence-relevance-calibration-v5-freeze
- freeze commit: a6cdb5f6a6c67e7f1c7a0f514e649a461b2c0030
- first/only Actions run: 36008648993
- run attempt: 1
- suite: evidence-relevance-calibration-v5
- semantic contract: binding proposal v2 + target-first materialization v3
- per-case budget: 2 model calls / 192 output tokens / 60,000 ms elapsed

The run was not rerun. This result remains immutable historical evidence even if a successor later passes.

## Mistral canonical arm

Model: ministral-8b-latest

- operational: 26/26
- provider failures: 0
- materialized exact: 26/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0
- provider attempts: 26
- total latency: 20,327 ms

Proposal exact was 17/26. Proposal exactness is diagnostic; Harness-materialized disposition is the semantic gate.

## Google canonical arm

Model: gemini-3.5-flash-lite

- operational: 11/26
- provider failures: 15
- materialized exact among completed cases: 11/11
- wrong-target / unsafe relevance admission among completed cases: 0
- false relevance rejection among completed cases: 0
- expected-relevant left ambiguous among completed cases: 0
- utility miss among completed cases: 0
- failure classes: assessment_timeout 14, provider_unavailable 1
- total observed latency: 1,261,376 ms
- successful-case median latency: 30,628 ms
- successful-case max latency: 49,319 ms

The typed provider_unavailable occurred on 10_structured_metadata_plus_body after four Google provider attempts and 51,181 ms. The provider returned HTTP 503 with an explicit high-demand message. Fourteen other cases hit the 60,000 ms Harness deadline before the adapter returned a completed result, so their outer observation records provider_attempts = 0; that does not imply no HTTP request had started.

The failure pattern is operational rather than semantic. Every completed Google case materialized to the expected disposition.

## Interpretation

v5 shows that Google 3.5 tail latency can exceed the v4 30-second envelope, because successful v5 observations reached 49.3 seconds. It also shows that simply extending the deadline further is not justified by this run: the provider explicitly reported high demand and remained intermittently unavailable.

Do not respond to v5 by blindly increasing the Harness deadline to 90/120 seconds, by adding aggressive semantic materialization, by switching models, or by adding unbounded retries.

Before authoring v6, use the unchanged v5 frozen semantic surface in a small Google recovery diagnostic to distinguish a transient provider-capacity incident from continuing operational instability.

Independent holdout authoring remains blocked because v5 did not pass.
