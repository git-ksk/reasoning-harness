# Planner reliability v1

Issue #282 defines this lane as a diagnostic baseline for the stochastic investigation planner/action selector before Issue #283 changes executable-action ownership.

## Coordinate

- corpus: `planner-reliability-v1`
- evaluator: `reason-planner-reliability-v1`
- scoring: `planner-reliability-scoring-v1`
- product under observation: Reason CLI 0.5.2 / Harness Engine 0.4.2 at `bcbd326e147fae21f06a601f988e6ab060ca41bb`
- providers are reported separately:
  - Mistral / `ministral-8b-latest`
  - Google / `gemini-3.5-flash-lite`

This is not a retrofit or rescore of v17/v18 or any frozen v0.4.2 release evidence.

## Frozen trial plan

The primary surface is Surface A. It runs five predeclared trials with seeds:

    86101, 86102, 86103, 86104, 86105

Each surface has three fresh investigation roles:

1. direct grounded acquisition with two exact-key read-only candidates plus one nonmatching distractor;
2. no-result / registry follow-up with two exact-key read-only candidates plus one nonmatching distractor;
3. stale-evidence safe stop with two exact-key read-only candidates plus one nonmatching distractor.

No capability has `selection_priority`. At least two exact compatible read-only capabilities remain for every target, so the existing globally unique-pair and precedence selectors cannot trivially consume the whole action-selection problem. The surface therefore retains the stochastic action selector that #283 is intended to study.

Surface B is information-equivalent but uses fresh names, keys, values, capability IDs, and sources. It runs once at seed 86105, matching Surface A trial 5. That matched pair is reported only as descriptive surface sensitivity; it is not a second holdout and is not averaged into the primary five-trial result.

## Metrics and denominators

Every attempted trial remains in the raw report. A trial is operationally complete only when all three cases complete without process, typed action, or generation failure.

Semantic and utility distributions use complete primary trials only. Operationally incomplete trials are excluded from those distributions and reported separately with exact failure classes and trial IDs.

The report includes:

- trial completion count/rate;
- pass@1 as strict planner-success trials divided by complete primary trials;
- observed all-k reliability for the one predeclared k=5 group;
- exact per-case frequencies for target recall, valid action shape, relevant tool selection, trigger exposure, conditional #249 conformance, avoidable stalls, false abstention, action rejection reason, and stop reason;
- mean/min/max plus raw values for trial-level planner metrics;
- observed provider/model execution identities and provider usage telemetry;
- the matched Surface A/B seed-86105 deltas.

The strict planner-success predicate requires, for an operationally complete trial: zero correctness-boundary violations, every target recalled, a relevant tool selected in every case, zero action-validation rejections, zero irrelevant acquisitions, zero avoidable follow-up stalls, zero false abstentions, grounded output for grounded cases, safe non-grounding for the stale case, and #249 conformance whenever its no-result trigger is actually exposed.

Trigger exposure itself is not required for success. Selecting the registry directly can still be useful; #249 conformance is scored only when the cache/no-result predecessor is actually observed.

## pass^k interpretation

The report uses observed all-five success for the predeclared five-trial group. It deliberately does not emit a derived p^5 estimate because these observations are not assumed independent and identically distributed.

Five trials are a descriptive characterization for this frozen provider/model/surface slice. They are not a production SLA, population-level reliability guarantee, or model ranking.

## Measurement acceptance versus utility

The live lane is accepted when:

- all five primary trials are operationally complete;
- the matched Surface A/B pair is complete;
- correctness-boundary violations are zero.

Planner pass@1 or all-five success may be low and the measurement can still be valid. That result is the baseline #283 needs; utility failure must not be hidden by turning it into evaluation invalidity.

Repeated observations never create evidence, authority, truth by voting, or majority repair. Provider reports remain separate and are never cross-model averaged.

## Freeze discipline

The surface, evaluator, scoring identity, provider/model coordinates, seeds, k, success predicate, and matched pair are frozen before the first provider call. The first live launch per provider is canonical. A post-launch semantic/evaluator/surface change requires a successor identity. Pre-live infrastructure failures may be retried only before the evaluator records its live-boundary marker.

The final #283 adoption evaluation must use a separate fresh successor/holdout identity; this baseline is consumed diagnostic evidence.
