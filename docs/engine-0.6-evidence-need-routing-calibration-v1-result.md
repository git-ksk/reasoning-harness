# Engine 0.6 evidence-need routing calibration v1 result

Status: first canonical live calibration observation recorded; v1 surface is frozen and will not be rerun or rescored.

## Frozen identity

- Issue: #461
- freeze tag: `engine-0.6-evidence-need-calibration-v1-freeze`
- candidate commit: `8b49adf731f3fe3290b3e904d56b8216954288d9`
- GitHub Actions run: `35954770046`
- corpus: `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral arm: `ministral-8b-latest`
- Google arm: `gemini-3.5-flash-lite`

The preflight passed before provider credentials were read. Both provider arms then passed credential checks from GitHub repository secrets and completed all 22 model-backed cases. The two live jobs succeeded. The combined final gate failed because the v1 scorer reported one Mistral correctness-boundary violation.

## Raw v1 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 17/22 (77.27%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 21/22 (95.45%) | 21/22 (95.45%) |
| acquisition exact accuracy | 21/22 (95.45%) | 21/22 (95.45%) |
| v1 reported correctness-boundary violations | 1 | 0 |
| v1 reported utility misses | 0 | 1 |
| provider attempts | 22 | 22 |
| total tokens | 7,383 | 7,826 |
| model-call latency total | 13,460 ms | 16,326 ms |

These are the values emitted by the frozen v1 runner. They are retained unchanged even where calibration analysis below identifies a scorer defect.

## Finding A — v1 scorer false-positive on permitted downgrade

Mistral case `17_external_optional` proposed `context_only` where the fixture expected `external_optional`.

The frozen Harness decision was:

- materialized mode: `context_only`
- acquisition: `context_only`
- reasons: `baseline`, `model_downgrade_applied`

The policy explicitly allowed the model to downgrade to `context_only`; supplied context was complete and target-local sufficient. This was therefore not an unsafe skipped acquisition. The v1 scorer nevertheless classified every materialized mode weaker than the fixture's exact expected mode as a correctness violation.

Calibration conclusion: correctness must be evaluated against the lowest Harness-permitted safety floor, not against exact-route expectation. Exact-route mismatch remains diagnostic. A cheaper route that is still above the policy safety floor is not a correctness failure.

The frozen v1 artifact is not rescored. A versioned successor runner must encode this distinction.

## Finding B — real mixed-target utility over-routing

Google case `09_mixed_summary_target` had a mixed user turn:

- summarize supplied article;
- separately verify whether the product is available now.

The exact target under classification was only `Summarize the supplied article.` with `content_local`, complete context, and sufficient target-local context. The model proposed `external_required`; the Harness therefore retained the conservative baseline and materialized external acquisition for the summary target.

This is correctness-safe but is a real utility miss. It reproduces the motivating class of over-routing: evidence need from a different subrequest leaked into a target-local summary decision.

Calibration conclusion: the proposal request must make the exact target the sole classification subject. The surrounding user turn may be present only for language/coreference context and must not transfer evidence requirements across targets. Harness-internal baseline/minimum/downgrade/reuse fields should not anchor the model proposal.

## v2 tuning requirements

The next calibration candidate must keep the 22-case v1 corpus unchanged and version only the runner/prompt/materialization evaluation surface.

Required changes before a new explicit freeze identity:

1. score correctness against the minimum Harness-permitted materialized decision rather than exact expected route;
2. score unnecessary stronger acquisition as utility, not correctness;
3. keep exact proposal/mode/acquisition accuracy as diagnostics;
4. strengthen target-local proposal instructions for mixed requests;
5. stop exposing Harness-internal baseline/minimum/downgrade/reuse values to the model-facing classification request;
6. preserve all v1 artifacts and do not rerun the v1 freeze tag.

No independent holdout is authored yet. Candidate semantics must first pass the versioned calibration successor and then be frozen before holdout authoring.
