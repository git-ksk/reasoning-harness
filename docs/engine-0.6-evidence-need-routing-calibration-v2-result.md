# Engine 0.6 evidence-need routing calibration v2 result

Status: frozen v2 observation completed successfully; correctness hard gates passed on both provider arms, but one Mistral utility over-escalation remains, so semantic freeze and holdout authoring remain blocked.

## Frozen identity

- freeze tag: `engine-0.6-evidence-need-calibration-v2-freeze`
- candidate commit: `e4f57c347ef8bfc9c15d6dc19dcf7f88e70337d5`
- GitHub Actions run: `35956178160`
- corpus: unchanged `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

Preflight, both live jobs, and the combined final correctness gate all completed successfully.

## Raw v2 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 19/22 (86.36%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 20/22 (90.91%) | 22/22 (100%) |
| acquisition exact accuracy | 20/22 (90.91%) | 22/22 (100%) |
| correctness-boundary violations | 0 | 0 |
| utility misses | 1 | 0 |
| provider attempts | 22 | 22 |
| total tokens | 10,212 | 10,686 |
| model-call latency total | 15,968 ms | 23,770 ms |

## Confirmed v2 improvements

The v1 scorer false-positive is fixed. Mistral case `17_external_optional` again proposed `context_only`, but the runner now records its minimum Harness-permitted route as `context_only`; therefore the explicitly permitted downgrade is neither a correctness failure nor a utility failure.

The real v1 mixed-target over-routing is also fixed. Google no longer over-routes `09_mixed_summary_target`; its materialized-mode and acquisition exact accuracy are both 22/22 with zero utility misses.

## Remaining utility finding

Mistral case `15_followup_escalation` has:

- exact target: `Verify the current official status.`
- Harness flags: explicit verification = true, current state = true, trusted verification = false
- expected / minimum permitted mode: `external_required`
- model proposal: `trusted_verification_required`
- materialized mode: `trusted_verification_required`

This is correctness-safe but more expensive/authoritative than required. More importantly, `trusted_verification_required` represents a Harness-owned trusted-authority requirement. Allowing an untrusted model proposal to create that requirement conflicts with the architecture invariant that the model cannot create authority.

## v3 requirement

The successor must keep the calibration corpus unchanged and make trusted-verification proposal availability Harness-owned:

- when `trusted_verification_required == false`, the model-facing schema must not include `trusted_verification_required`;
- when `trusted_verification_required == true`, the mode remains available and the Harness hard floor remains trusted verification;
- model escalation up to ordinary `external_required` can remain advisory/safety-conservative;
- a model cannot introduce a stronger trusted authority class on its own.

No independent holdout will be authored until the versioned successor reaches zero correctness-boundary violations and zero utility misses on both frozen calibration arms.
