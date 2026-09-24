# Engine 0.6 evidence-need routing independent holdout v1

Status: PASS. The frozen first/only independent holdout observation is complete. See [holdout v1 result](engine-0.6-evidence-need-routing-holdout-v1-result.md).

## Independence boundary

The #461 candidate semantics were frozen in commit `38d5e58` after frozen calibration v3 passed both provider arms. This holdout was authored only after that freeze.

Holdout identity:

- suite: `evidence-need-routing-holdout-v1`
- issue: #461
- cases: 26
- corpus: `fixtures/evidence-need-routing-holdout-v1/manifest.json`
- planned freeze tag: `engine-0.6-evidence-need-holdout-v1-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-need-holdout-v1-live.yml`
- seed: `4611601`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credentials: GitHub repository secrets only

No task, target, or context string exactly duplicates the 22-case calibration corpus. The holdout uses different product/domain wording and includes four Japanese-language task formulations.

## Coverage

The 26 cases independently cover:

- non-factual formatting and translation;
- content-local summary, comparison, extraction, and conflict description;
- partial context that is sufficient for the exact target;
- truncated context that is insufficient for a full-source target;
- claims-about-content versus claims-about-world;
- current-state and explicit official verification;
- trusted exact verification;
- optional external corroboration;
- valid external/trusted evidence reuse;
- stale, scope-mismatched, and policy-mismatched evidence;
- mixed target-local versus current-state subrequests;
- follow-up mode changes;
- prompt injection embedded in supplied context;
- ambiguous account-specific implication;
- resolver unavailable while external evidence remains required;
- model inability to create trusted authority when Harness policy does not require it.

## Frozen scoring rules

The holdout uses the already-frozen v3 materialization and scoring semantics.

Correctness is measured against the minimum Harness-permitted route, not exact-route preference. Exact expected route remains diagnostic.

Utility fails only for avoidable stronger acquisition after correctness remains safe.

Provider/model operational failures are recorded separately from semantic failures.

Hard correctness gates remain zero for:

- unsafe skipped acquisition;
- context authority laundering;
- explicit verification downgrade;
- current-state downgrade;
- trusted-verification downgrade;
- model-created trusted authority;
- invalid existing-evidence reuse;
- mixed-target whole-turn over-routing that crosses the minimum safe route;
- replayed external side effects.

The utility gate requires zero avoidable stronger acquisition on both provider arms.

## Pre-observation validation

Before any provider credential is read:

- the exact holdout path, suite ID, issue binding, and `fresh_unobserved_holdout` status are validated;
- all 26 policies are materializable;
- every expected proposal materializes to its frozen expected mode/acquisition;
- exact calibration task/target/context reuse is absent;
- the frozen surface checksum is revalidated;
- formatter, clippy, holdout-runner tests, and core evidence-need tests must pass.

`--validate-only` performs the corpus/contract preflight without provider calls.

## Observation rule

The first observation is triggered only by pushing `engine-0.6-evidence-need-holdout-v1-freeze`.

Workflow reruns are rejected. If an operationally incomplete observation requires another attempt, a new explicit versioned holdout identity must be created rather than mutating or rerunning v1.

After the first model-backed observation, this corpus, expected labels, scorer, materialization semantics, prompt contract, and thresholds are immutable. A holdout failure may be analyzed, but it must not be repaired and rescored under the same identity.

A PASS independently accepts #461 only. It does not complete Engine 0.6.0; #462 and #463 remain separate semantic tracks.
