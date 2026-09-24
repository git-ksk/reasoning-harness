# Engine 0.6 evidence-need routing calibration v3

Status: frozen v3 live calibration PASS; #461 candidate semantics are frozen. See [calibration v3 result](engine-0.6-evidence-need-routing-calibration-v3-result.md).

## Why v3 exists

Frozen v2 (GitHub Actions run `35956178160`) passed both provider correctness gates and eliminated the Google mixed-target over-routing found in v1. One Mistral utility miss remained: `15_followup_escalation` proposed `trusted_verification_required` even though the Harness-owned policy required only ordinary external/current-state verification.

That finding exposed an authority-boundary issue, not just prompt variance. Trusted verification is a Harness-owned authority class and must not be created by an untrusted model proposal.

See [v2 result](engine-0.6-evidence-need-routing-calibration-v2-result.md).

## v3 semantic change

The calibration corpus and expected labels remain unchanged.

- `trusted_verification_required` is included in the model-facing enum only when the Harness-owned target policy has `trusted_verification_required == true`;
- a manually supplied model proposal for trusted verification is ignored when that Harness flag is false;
- ordinary model escalation through `external_required` remains possible;
- Harness hard floors for explicit verification, current state, target kind, and trusted exact verification remain unchanged;
- runner identity is `evidence-need-routing-live-calibration-v3`.

This makes the existing invariant explicit: the model may advise routing, but it cannot create a trusted authority requirement.

## Frozen v3 surface

- freeze tag: `engine-0.6-evidence-need-calibration-v3-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-need-calibration-v3-live.yml`
- corpus: unchanged 22-case `evidence-need-routing-calibration-v1`
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credentials: GitHub repository secrets only
- checksum: `fixtures/evidence-need-routing-calibration-v1/surface-v3.sha256`

## Acceptance gate

Both provider arms must satisfy all of the following:

- 22/22 operationally complete;
- provider failures = 0;
- correctness-boundary violations = 0;
- utility misses = 0.

Exact proposal/mode/acquisition mismatches may remain as diagnostics only when they do not violate either correctness or utility gates.

If v3 passes, the #461 candidate semantics are frozen before a separately authored independent holdout is created. The v3 freeze is not rerun; any further tuning requires a new versioned identity.
