# Engine 0.6 evidence-target relevance calibration v6 result

Status: frozen FAIL. Do not rerun or rescore.

## Frozen identity

- issue: #462
- branch: `feat/462-evidence-target-relevance`
- freeze tag: `engine-0.6-evidence-relevance-calibration-v6-freeze`
- freeze commit: `c4c16dd4968b9418f0105fa42bf11895ef26bb1b`
- first/only Actions run: `36022978827`
- run attempt: 1
- suite: `evidence-relevance-calibration-v6`
- semantic contract: binding proposal v2 + target-first materialization v3
- per-case budget: 2 model calls / 192 output tokens / 60,000 ms elapsed
- run-level operational circuit: abort after two consecutive operational provider failures

The run is immutable historical evidence. The v6 result remains FAIL even though the operational hardening behaved as designed.

## Mistral canonical arm

Model: `ministral-8b-latest`

- planned / completed: 26 / 26
- operational abort: none
- successful / failed provider cases: 26 / 0
- materialized exact: 26/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0
- provider-attempt telemetry incomplete observations: 0
- latency p50 / p95 / max: 554 / 863 / 944 ms

Result: PASS.

## Google canonical arm

Model: `gemini-3.5-flash-lite`

- planned cases: 26
- completed cases before circuit open: 3
- successful / failed provider cases: 1 / 2
- failures: `assessment_timeout` x2
- operational abort: `consecutive_operational_failure_budget_exhausted`
- circuit opened after `03_expanded_alias`
- next unsent case: `04_semantic_paraphrase`
- remaining requests suppressed: 23
- provider-attempt telemetry incomplete observations: 2

Observed cases:

1. `01_exact_name_availability`: success, 47,242 ms, materialized exact.
2. `02_acronym_alias`: assessment timeout, 60,001 ms; provider-attempt count incomplete because the outer Harness deadline cancelled the in-flight adapter future.
3. `03_expanded_alias`: assessment timeout, 60,001 ms; provider-attempt count likewise incomplete.

Among completed semantic observations:

- proposal exact accuracy: 1.0
- materialized exact accuracy: 1.0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

Result: operational FAIL. No semantic regression was observed in the completed case.

## Hardening outcome

The operational hardening introduced for v6 behaved as intended:

- the Google provider retry policy remained bounded and now uses bounded equal jitter instead of deterministic fallback delays;
- the runner did not add a second retry layer;
- after two consecutive operational failures, the run-level circuit stopped the arm rather than sending the remaining 23 requests;
- outer-deadline cancellation is explicitly marked as incomplete provider-attempt telemetry;
- p50 / p95 / max latency telemetry is emitted;
- the canonical workflow final gate is visibly red on operational incompleteness rather than relying on a green artifact-preservation job.

Therefore v6 does not justify reverting the hardening or increasing the 60-second deadline. It establishes that Google 3.5 serving instability persisted even after bounded retry jitter and load shedding.

## Gate-design implication before v7

Google must not be removed because of a semantic score: completed Google observations remain semantically exact and safe. The question is whether a provider that repeatedly cannot produce a valid canonical run should remain a required operational gate.

Before v7, choose any replacement required provider/model from evidence independent of the #462 calibration outcomes, to avoid tuning the provider choice to this semantic corpus. Existing repository history already provides such evidence for Groq `openai/gpt-oss-120b`, while NVIDIA's routine candidate has documented protocol/timeout incompleteness. Current provider capability and availability must also be revalidated before freezing v7.

Google's v4/v5/v6 results remain explicit replication/operational evidence; changing its gating role must not rewrite or discard them.

Independent holdout authoring remains blocked until a fresh canonical calibration passes.
