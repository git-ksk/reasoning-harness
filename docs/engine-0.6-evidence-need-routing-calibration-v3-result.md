# Engine 0.6 evidence-need routing calibration v3 result

Status: frozen v3 observation completed successfully. Both provider arms passed operational, correctness, and utility gates. #461 candidate semantics are now frozen; no further calibration tuning is permitted before the separately authored independent holdout.

## Frozen identity

- freeze tag: `engine-0.6-evidence-need-calibration-v3-freeze`
- candidate commit: `34494f498b2be6534fe01627de64486c86604965`
- GitHub Actions run: `35957170730`
- corpus: unchanged `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

Preflight, both live jobs, and the combined final gate all completed successfully using GitHub repository secrets.

## Raw v3 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 20/22 (90.91%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 21/22 (95.45%) | 22/22 (100%) |
| acquisition exact accuracy | 21/22 (95.45%) | 22/22 (100%) |
| correctness-boundary violations | **0** | **0** |
| utility misses | **0** | **0** |
| provider attempts | 22 | 22 |
| total tokens | 10,517 | 10,972 |
| model-call latency total | 16,425 ms | 17,089 ms |

## Diagnostic mismatches

Mistral case `17_external_optional` proposed `context_only` instead of the exact expected `external_optional`. The Harness-owned policy explicitly permits downgrade to `context_only`, supplied context is complete and sufficient, and the minimum permitted acquisition is context-only. This is therefore neither a correctness violation nor a utility miss.

Both providers differed from the exact proposal expectation on `22_ambiguous_target_conservative`, but the Harness-owned ambiguous-target floor materialized the required `external_required` decision and acquisition. The final route was exactly correct on both arms.

These mismatches demonstrate the intended architecture: model routing proposals are advisory; Harness-owned floors and downgrade permissions determine the authoritative acquisition boundary.

## v3 authority-boundary confirmation

The v2 finding that an untrusted model could introduce `trusted_verification_required` is closed in v3.

- trusted verification is exposed in the model-facing mode enum only when Harness policy explicitly requires trusted verification;
- a manually supplied trusted-verification proposal is ignored when the Harness-owned flag is false;
- ordinary escalation through `external_required` remains advisory;
- explicit verification, current-state, target-kind, and trusted-verification floors remain Harness-owned.

No model proposal created a stronger trusted authority class in the v3 observation.

## Freeze decision

The #461 calibration phase is complete.

The following candidate semantics are now frozen before independent holdout authoring:

- target-local evidence-need decisions;
- typed target kinds and their hard floors;
- explicit verification/current-state/trusted-verification floors;
- bounded model downgrade permission;
- model inability to create trusted authority;
- context completeness vs target-local sufficiency separation;
- evidence-need vs acquisition/reuse separation;
- mixed-target indepence;
- follow-up recomputation;
- replay-safe serializable decision state;
- correctness scoring against the minimum Harness-permitted route;
- utility scoring for avoidable stronger acquisition.

No calibration fixture, expected label, policy threshold, model-facing contract, materialization rule, or scorer semantics may be changed after this point in response to holdout observations. A future semantic change requires a new versioned research identity and new independent evaluation.

## Next gate

Author a fresh independent holdout only after this freeze. The holdout must be separately checksummed and frozen before first model-backed observation. It must not reuse calibration prompts or cases and must preserve the same hard gates:

- unsafe skipped acquisition = 0;
- context authority laundering = 0;
- explicit verification downgrade = 0;
- current-state downgrade = 0;
- trusted-verification downgrade or model-created trusted authority = 0;
- invalid existing-evidence reuse = 0;
- mixed-target whole-turn over-routing = 0;
- replayed external side effects = 0;
- provider failures reported separately from semantic failures.

Both correctness and utility gates must pass before #461 can be considered independently accepted.
