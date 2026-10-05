# Engine 0.6 evidence-target relevance calibration v7 result

Status: frozen FAIL / operationally incomplete. Do not rerun or rescore.

## Frozen identity

- issue: #462
- freeze tag: `engine-0.6-evidence-relevance-calibration-v7-freeze`
- freeze commit: `965582b244630200e5a01632eab2312e5fc0214c`
- first/only Actions run: `36026148264`
- run attempt: 1
- required arms: Mistral `ministral-8b-latest` and Groq `openai/gpt-oss-120b`
- semantic contract: binding proposal v2 + target-first materialization v3
- case budget: 2 model calls / 192 output tokens / 60,000 ms
- run-level circuit: abort after two consecutive operational provider failures

The workflow was cancelled during the Groq live arm. The frozen identity is not rerun. The Mistral arm had already completed and independently contained one utility miss, so v7 could not have passed even if Groq had subsequently completed.

## Mistral arm

- planned / completed: 26 / 26
- successful / failed provider cases: 26 / 0
- operational abort: none
- materialized exact: 25/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 1
- provider-attempt telemetry incomplete observations: 0
- latency p50 / p95 / max: 642 / 956 / 975 ms

The only disposition miss was `21_unknown_rename`:

- expected proposal: `target=unresolved`, `relation=exact`
- observed proposal: `target=different`, `relation=different`
- expected disposition: `ambiguous`
- materialized disposition: `irrelevant`
- assessment path: `model_assisted`
- reasons: `harness_canonical_name_anchor`, `model_irrelevant`

This is a utility failure / stochastic false-rejection risk, not an unsafe relevance admission. The candidate explicitly states that the supplied page does not establish whether the new name replaces the target, so the intended identity state remains unresolved.

The result also exposes a materialization-design risk: v3 gives an advisory `target_binding=different` enough authority to force `irrelevant` even when the same candidate contains a Harness target-name anchor and the identity relationship can be uncertain. That asymmetry must be reviewed before a successor canonical run rather than papered over with another seed.

## Groq arm

The Groq job was cancelled while the live observation was in progress. No complete result was produced, so Groq is non-scorable for v7.

The preserved checkpoint completed the first three cases before cancellation:

- `01_exact_name_availability`: exact / exact, 370 ms, HTTP 200
- `02_acronym_alias`: exact / exact, 2,736 ms, HTTP 200
- `03_expanded_alias`: exact / exact, 3,574 ms, HTTP 200

All three used one provider attempt and materialized exactly. The rate-limit telemetry showed request capacity remaining and no 429 during these observations. This partial evidence is not a Groq PASS and must not be extrapolated to the remaining 23 cases.

## Decision

v7 remains immutable failed/incomplete evidence. Do not rerun the v7 workflow or reuse its tag.

Before a fresh successor identity:

1. preserve the Google attempt-telemetry diagnostic result independently; it found no active quota/rate-limit signal in the later three-case probe;
2. treat `21_unknown_rename` as a general identity-uncertainty design problem, not as a one-case prompt-tuning target;
3. review whether negative target identity may become `irrelevant` solely from an advisory model `different` proposal when Harness-owned provenance does not establish distinct identity;
4. use fresh, independently authored identity-ambiguity probes before changing materialization policy;
5. keep v7 provider-role history explicit: the run does not establish a Groq failure.

Independent holdout authoring remains blocked.
