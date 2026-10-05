# Engine 0.6 evidence relevance successor-v10 development v2 result

Status: immutable development FAIL.

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v2-freeze
- Freeze commit: 44796d470fdbb359ea1356dad121aa9417a98cb7
- Run: 37100490212
- Candidate: effective qualification v14 / materialization v26
- Surface: 18 fresh cases
- Required development providers: Mistral + Google
- Result: FAIL

The v2 tag and observation are immutable development evidence. Do not rerun, move the tag, relabel cases, or reinterpret the result as PASS.

## Provider result

Mistral / ministral-8b-latest:
- operational: 18/18
- provider failures: 0
- authority failures: 0
- identity/risk failures: 0
- materialized exact: 17/18
- total tokens: 30,796
- result: FAIL

Google / gemini-3.5-flash-lite:
- operational: 18/18
- provider failures: 0
- authority failures: 0
- identity/risk failures: 0
- materialized exact: 18/18
- total tokens: 31,696
- result: PASS

## Terminal miss

The only terminal miss was Mistral case sv10v2_18_requested_availability_with_feature.

Task: Where is Orchid Proxy deployable?

Candidate: Orchid Proxy deploys in Ridge zones and supports webhook retries.

Mistral returned:
- proposal: exact / exact
- raw local qualification: exact_target / different_relation / none
- v14 effective qualification: exact_target / requested_relation / none
- v26 materialized disposition: Irrelevant
- expected disposition: Relevant

The v14 authority correction was correct. The failure is in v26 composition.

v26 derives v14 correctly, but then delegates baseline materialization to v23 using the original raw local qualification. v23 therefore sees the raw DifferentRelation classification and can terminally materialize Irrelevant before v26 has an opportunity to use the corrected v14 RequestedRelation result.

This is not evidence that the Harness-owned v14 negative-relation cue is unsafe. The authority and identity/risk gates were clean for both providers. It is a composition defect between corrected effective qualification and final materialization.

## Next direction

Keep v14 unchanged.

A successor materializer should compose from the v14 effective qualification rather than re-materializing from the uncorrected raw qualification. The candidate must then replay:
- frozen successor-v9 expected contracts;
- frozen holdout-v10 expected contracts;
- all canonical holdout-v10 observations;
- all immutable v2 observations.

The successor must prove that:
- the known Groq v10h18 over-abstention becomes Irrelevant;
- Mistral sv10v2_18 becomes Relevant;
- Google v2 remains unchanged;
- no other historical terminal disposition changes unexpectedly.

Only after those deterministic replays pass should a fresh independent v3 development surface be observed.
