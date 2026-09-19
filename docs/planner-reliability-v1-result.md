# Planner reliability v1 — result

Issue #282 measured the remaining stochastic investigation planner/action-selection reliability on the released Reason CLI 0.5.2 / Harness Engine 0.4.2 runtime before Issue #283 changes executable-action ownership.

## Frozen coordinate

- freeze tag: `planner-reliability-v1-freeze`
- freeze commit: `ce0842dd6e6f27a2004659cc9f2a47483f8cd4b5`
- observed product runtime: `bcbd326e147fae21f06a601f988e6ab060ca41bb`
- corpus: `planner-reliability-v1`
- evaluator: `reason-planner-reliability-v1`
- scoring: `planner-reliability-scoring-v1`
- canonical Actions run: `35423433892`
- Mistral job: `105845143753`
- Google job: `105845143801`
- primary seeds: `86101`, `86102`, `86103`, `86104`, `86105`
- observed all-k group: `k=5`
- matched information-equivalent Surface A/B seed: `86105`

The surface, evaluator, scoring identity, provider/model coordinates, seeds, success predicate, and k group were frozen before the first provider call. The product runtime itself was unchanged from the recorded base commit.

## Result summary

| provider/model | complete primary trials | pass@1 | observed all-5 | correctness-boundary violations | matched Surface A/B |
| --- | ---: | ---: | --- | ---: | --- |
| Mistral `ministral-8b-latest` | 5/5 | 1.00 | true | 0 | complete; planner success on both |
| Google `gemini-3.5-flash-lite` | 5/5 | 0.80 | false | 0 | complete; planner success on both |

Both provider jobs passed the predeclared **measurement acceptance**: all five primary trials were operationally complete, the matched Surface A/B pair completed, and correctness-boundary violations were zero. Planner utility was deliberately not an acceptance gate.

No derived `p^5` value is reported. The observations are not assumed IID; the report preserves the predeclared observed all-five group instead.

## Mistral observation

Mistral completed all five primary trials with strict planner success.

Across the five complete trials:

- target recall: `3/3` in every trial;
- relevant tool selection: `3/3` in every trial;
- valid action shape: `3/3` in every trial;
- action-validation rejections: `0` in every trial;
- avoidable follow-up stalls: `0`;
- false abstentions: `0`;
- the no-result trigger was exposed in `5/5` follow-up cases;
- #249 exact-target follow-up conformance was `5/5`;
- correctness-boundary violations: `0`.

The matched Surface A/B seed-86105 pair was complete and produced zero delta across the predeclared planner metrics.

## Google observation

Google also completed all five primary trials operationally, with zero correctness-boundary violations, but one trial failed the strict planner-success predicate.

Seeds `86101`, `86102`, `86103`, and `86105` passed. Seed `86104` recorded one action-validation rejection in the stale-unknown case:

- the planner first selected both correct stale sources;
- both stale observations were rejected by the ordinary admission boundary as intended;
- the stochastic action selector then proposed `myrador-region-distractor` for target `check_window_one`;
- the Harness rejected that proposal as `unsupported_target_key`;
- the case remained safely ungrounded and stopped at `no_progress`.

Therefore:

- complete trials: `5/5`;
- planner success: `4/5 = 0.80`;
- observed all-five success: `false`;
- action-rejection count by trial: `[0, 0, 0, 1, 0]`;
- invalid-shape rejections: `0`;
- avoidable follow-up stalls: `0`;
- false abstentions: `0`;
- no-result trigger exposure: `5/5`;
- #249 conformance when exposed: `5/5`;
- correctness-boundary violations: `0`.

This is the intended diagnostic separation: stochastic executable-action selection showed a residual utility defect while Harness validation preserved the safety/correctness boundary.

The matched Surface A/B seed-86105 pair was complete and planner-successful on both surfaces with zero predeclared metric deltas.

## Interpretation

This result is a **baseline characterization**, not a model ranking, SLA, or population reliability guarantee.

The product finding is narrower:

1. the released Harness can complete this frozen planner surface safely and operationally on both routine provider targets;
2. stochastic action selection can still emit an inadmissible executable target/capability pairing even after useful evidence acquisition;
3. the Harness correctly rejects that proposal rather than turning it into authority or evidence;
4. Issue #283 now has a measured pre-change baseline against which deterministic Harness-owned action materialization can be evaluated.

The #282 surface is now consumed diagnostic evidence. Issue #283 must not tune and then claim adoption on this same surface; its adoption decision requires a separate fresh successor/holdout identity.

## Preserved machine reports

- [Mistral raw machine report](observations/planner-reliability-v1-mistral-run-35423433892-2026-09-19.json), SHA-256 `453012a07c8b2f8f807fa04b4f92d8e4012f9ce6f8f0681a1c81872916f82a3e`
- [Google raw machine report](observations/planner-reliability-v1-google-run-35423433892-2026-09-19.json), SHA-256 `b5b6ef82f8c2dbf019da95adb94ec13f3bece2e1282f42199fa3d688a6ffb533`

Frozen v17/v18 and the v0.4.2 release ruler were not modified, rescored, or repaired.
