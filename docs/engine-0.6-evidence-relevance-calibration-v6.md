# Engine 0.6 candidate: evidence-target relevance calibration v6

Status: fresh successor identity after frozen v5 operational failure and the failed six-case Google recovery smoke. v1-v5 and the recovery diagnostic remain immutable historical evidence and must not be rerun or rescored.

## Why v6 exists

Frozen v5 run `36008648993` showed no semantic regression but severe Google serving instability:

- Mistral completed 26/26 with exact materialized disposition and zero correctness/utility misses.
- Google completed only 11/26; 14 cases hit the 60,000 ms Harness assessment deadline and one returned explicit HTTP 503 high-demand after four provider attempts.
- all 11 completed Google cases materialized exactly.

The subsequent frozen recovery smoke run `36018360038` reused the unchanged v5 surface and still completed only 4/6 cases. `21_unknown_rename` and `26_url_only_identity` again reached 60,000 ms. This ruled out proceeding to v6 with only another retry or larger sample.

Prior-art review of *The Tail at Scale*, Google SRE overload/cascading-failure guidance, Gemini error guidance, AWS retry/jitter guidance, and LLM-serving evaluation practice supports separating semantic quality from provider reliability and bounding retry amplification.

## v6 operational hardening

v6 keeps relevance semantics unchanged and changes only provider/run reliability behavior and telemetry.

### Google provider retry jitter

The Google adapter keeps the existing bounded attempt policy:

- maximum provider attempts: 4;
- transient 5xx base schedule: 2 / 5 / 10 seconds;
- rate-limit fallback base schedule: 10 / 20 / 40 seconds;
- explicit `Retry-After` remains authoritative.

Deterministic fallback waits are replaced with bounded equal jitter in the 50%-100% range of each base delay. This reduces synchronized retry amplification without adding attempts or extending the retry cap.

### Run-level operational circuit

The calibration runner does not add any retry loop. Instead, it opens a run-level circuit after **two consecutive operational provider failures** and stops issuing further model requests.

Operational provider failure classes are:

- `assessment_timeout`;
- `credentials`;
- `provider`;
- `provider_unavailable`;
- `quota`;
- `rate_limit`;
- `timeout`;
- `transport`.

A successful/non-provider response resets the consecutive-failure streak. An aborted canonical run remains operationally incomplete and fails acceptance. This is load shedding, not a mechanism for hiding failed cases.

### Telemetry

v6 additionally records:

- planned vs completed cases;
- structured operational-abort reason and remaining cases;
- whether per-case provider-attempt telemetry is complete;
- count of observations with incomplete provider-attempt telemetry;
- total latency plus p50 / p95 / max latency;
- successful-case p50 / p95 / max latency.

An outer Harness timeout marks provider-attempt telemetry incomplete because cancellation can occur after an HTTP attempt started but before the adapter returns its attempt count.

### Visible workflow gating

The canonical workflow preserves raw arm artifacts but the final gate fails unless both canonical arms are 26/26 operational, have no run-level abort, and satisfy all semantic correctness/utility gates. A green artifact-preservation step is not treated as acceptance.

## Unchanged semantic/evaluation surface

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`;
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`;
- strict Harness-owned identity floor;
- assessment elapsed budget: 60,000 ms per case;
- max model calls: 2;
- max output tokens: 192;
- Google canonical model: `gemini-3.5-flash-lite`;
- Mistral canonical model: `ministral-8b-latest`;
- Google request-start pacing: 6,000 ms with 6,100 ms inter-case delay;
- all 26 semantic cases and all expected proposal/disposition labels.

There are no hedged duplicate requests, no second runner retry loop, and no blanket 90/120-second deadline increase.

## Fresh successor identity

- suite: `evidence-relevance-calibration-v6`
- issue: #462
- cases: 26
- seed: `4625606`
- status: `fresh_unobserved_calibration`
- production motivating incident: excluded from tuning

The corpus is reused for direct calibration comparability; this is not an independent holdout. Holdout authoring remains blocked until a canonical calibration passes.

## Acceptance

Both canonical arms must satisfy all of the following on the first/only frozen v6 observation:

- planned cases: 26;
- completed cases: 26;
- operational abort: none;
- successful provider cases: 26/26;
- failed provider cases: 0;
- wrong-target / unsafe relevance admission: 0;
- false relevance rejection: 0;
- expected-relevant left ambiguous: 0;
- utility miss: 0.

Proposal exact accuracy and latency statistics remain diagnostic. Harness-materialized disposition is the semantic release gate.

If the first frozen v6 observation fails, v6 remains immutable failed evidence and is not rerun.
