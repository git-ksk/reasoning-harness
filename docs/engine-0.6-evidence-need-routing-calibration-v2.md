# Engine 0.6 evidence-need routing calibration v2

Status: frozen v2 observation completed. See [calibration v2 result](engine-0.6-evidence-need-routing-calibration-v2-result.md).

## Why v2 exists

Frozen calibration v1 (GitHub Actions run `35954770046`) produced two actionable findings without any provider/runtime incompleteness:

1. the v1 scorer incorrectly treated an explicitly permitted cheaper downgrade as a correctness failure;
2. Google over-routed one mixed-request summary target because the proposal request allowed surrounding subrequest requirements and Harness-internal routing controls to influence the model.

The raw v1 artifacts remain immutable and are not rescored. See [calibration v1 result](engine-0.6-evidence-need-routing-calibration-v1-result.md).

## What changed

The calibration corpus, expected labels, and case ordering remain exactly `evidence-need-routing-calibration-v1`.

Only the versioned candidate/evaluation surface changes:

- runner configuration becomes `evidence-need-routing-live-calibration-v2`;
- correctness is scored against the minimum Harness-permitted materialized route, not exact expected route;
- a permitted cheaper route remains an exact-route mismatch diagnostic but is not a correctness or utility failure;
- stronger-than-expected mode/acquisition remains a utility miss;
- the model-facing request makes the exact target the sole classification subject;
- surrounding user-turn text is explicitly limited to language/coreference context and must not transfer evidence requirements between targets;
- Harness-internal `baseline_mode`, `minimum_mode`, `model_downgrade_floor`, and `existing_evidence` are no longer exposed to the classifier.

## Frozen v2 live surface

- freeze tag: `engine-0.6-evidence-need-calibration-v2-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-need-calibration-v2-live.yml`
- corpus: unchanged 22-case `evidence-need-routing-calibration-v1`
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credentials: GitHub repository secrets only
- checksum: `fixtures/evidence-need-routing-calibration-v1/surface-v2.sha256`

The workflow rejects reruns of the same frozen observation. Any further tuning requires another explicit versioned freeze identity.

## Acceptance interpretation

The v2 calibration is acceptable for semantic freeze only when:

- both provider arms are operationally complete: 22/22 cases, zero provider failures;
- correctness-boundary violations are zero on both arms;
- mixed-target unnecessary external acquisition is eliminated;
- exact-route mismatch remains visible as diagnostic data;
- utility and correctness remain separate metrics.

Independent holdout authoring remains blocked until v2 is observed and candidate semantics are frozen.
