# Engine 0.6 evidence relevance independent holdout v9 result

Status: immutable canonical FAIL.

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v9-freeze`
- Freeze commit: `1b50e5c54cdff4bd850bb0b1c58cfc66ce46d8bf`
- Canonical run: `37037497486`, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Google + Groq
- Effective qualification: v11
- Materialization: v23
- Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, or reinterpret the canonical run as PASS.

## Provider results

Mistral / `ministral-8b-latest`:
- operational: 26/26, provider failures 0
- effective authority qualification exact: 24/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- relation-scope misses: `v9h18_negative_distinct_availability_lookalike`, `v9h26_ambiguous_exact_target_numeric_observation`
- terminal materialization miss: `v9h26_ambiguous_exact_target_numeric_observation`, expected Ambiguous -> Irrelevant

Groq / `openai/gpt-oss-120b`:
- operational: 26/26, provider failures 0
- effective authority qualification exact: 24/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- same two relation-scope misses as Mistral
- same `v9h26` terminal materialization miss

Google / `gemini-3.5-flash-lite`:
- semantic observations completed: 25/26
- one operational failure: `v9h07_positive_alias_availability`
- failure class: timeout; local qualification exceeded the 60,000 ms semantic execution budget
- effective authority qualification exact: 24 recorded matches with one semantic relation miss plus one missing derivation
- materialized exact: 24/25 successful provider cases
- wrong-target Relevant: 0
- `v9h26` also materialized Irrelevant instead of expected Ambiguous

The Google timeout is operational evidence only and is not used as semantic evidence.

## Contract-adjudication finding

The semantic gate failure does not justify changing frozen v11/v23.

Two holdout-v9 expectations drifted from the already frozen pre-holdout relation contract:

1. `v9h18_negative_distinct_availability_lookalike` expected `unresolved`, but the pre-freeze successor-v9 development surface already classified the same non-frame family as `different_relation`:
   - `sv9d_10_availability_feature_support_non_frame`
   - expected relation scope: `different_relation`
   - expected disposition: Irrelevant

2. `v9h26_ambiguous_exact_target_numeric_observation` expected `unresolved` / Ambiguous, but the pre-freeze successor-v9 development surface already fixed the exact-target numeric-observation contract as `different_relation` / Irrelevant:
   - `sv9d_19_exact_target_model_only_limit`
   - expected relation scope: `different_relation`
   - expected disposition: Irrelevant

The related `sv9d_07_limit_numeric_observation` control also expected `different_relation`.

All three live providers that produced a semantic observation for `v9h26` returned the frozen-contract behavior. Mistral and Groq also returned the frozen-contract behavior for `v9h18`; Google had already abstained there under v11.

A candidate v12/v24 experiment that made negative relation authority fully symmetric was tested locally after the canonical result. It regressed numerous historical expected `different_relation` observations across holdouts and development surfaces, so it was rejected and not committed. Frozen v11/v23 remain byte-for-byte unchanged.

## Final gate interpretation

Official canonical result:
- operational completeness: FAIL because Google had one timeout
- correctness hard gate: FAIL under the canonical workflow because a required arm was incomplete
- qualification gate: FAIL
- materialization gate: FAIL
- utility gate: FAIL

Research interpretation:
- wrong-target Relevant remained 0 on every completed arm
- the repeated semantic misses are attributable to holdout expectation drift against the frozen pre-holdout contract, not evidence of a new v11/v23 defect
- the Google timeout is an independent operational failure
- therefore the next step is a fresh evaluation generation under unchanged v11/v23, not a semantic retune

## Successor evaluation direction

The next independent acceptance surface should be holdout v10 under unchanged frozen v11/v23.

Before its corpus is authored:
- freeze a dedicated holdout-v10 runner against the existing successor-v9 semantics tag;
- add an authoring review that explicitly checks holdout relation expectations against the frozen development contract so non-frame controls cannot silently switch from `different_relation` to `unresolved`;
- keep semantic and operational failures separate;
- treat the Google timeout as an operational robustness input, not a semantic label signal;
- preserve zero case/entity/task/signal/8-token overlap against holdout v1-v9 and all prior development surfaces.

Canonical holdout v9 remains immutable FAIL and is retained as evidence of the evaluation-contract gap.
