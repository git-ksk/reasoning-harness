# Semantic judge harness-owned materialization research

Issue #59 R2 evaluates whether the model-owned protocol surface can be reduced without granting the model additional authority. This is calibration-only research. Runtime `soft-semantic-v3` remains unchanged, historical holdouts v1/v2/v3 are not tuning data, and holdout-v4 remains blocked.

## R2 contract

The R2 model-facing output contains only:

- `decision`: `finding | no_finding | abstain`
- optional `advisory_note`

The schema contains no model-owned `finding`, `kind`, or `target` fields. The v3 kind-specific decision guidance is reused unchanged. R2 intentionally changes the output-ownership instructions so the model is told that the harness owns finding identity and binding.

When the parsed decision is `finding`, the harness constructs the soft finding by copying the request's existing `kind` and `target` exactly. The optional advisory note may be copied into the soft finding's note field. When the decision is `no_finding` or `abstain`, no finding is materialized even if an advisory note is present.

The resulting finding remains soft and advisory. Harness-owned materialization does not create trusted evidence, a hard finding, a verification receipt, epistemic promotion, or verdict authority.

## Normalization boundary

R2 permits syntax-only normalization equivalent to the existing structured-output policy. It must not:

- infer a different semantic decision;
- invent or change `kind` or `target`;
- turn `no_finding` or `abstain` into a finding;
- interpret malformed authority-like fields;
- repair multiple JSON values into one semantic answer.

Unknown fields fail closed through `deny_unknown_fields`. The study artifact records only whether an advisory note was present; it does not persist free-form advisory-note text for research scoring.

## Matched baseline comparison

The R2 study compares two arms within one provider/model:

1. exact v3 full-JSON primary representation;
2. harness-materialized decision protocol.

Cases are matched by `(fixture_id, trial, seed)`. Execution order alternates by fixture/trial so one arm is not always run first. Operational failures remain outside the decision-flip denominator.

The study reports protocol completion, precision, recall, decision coverage, ambiguous abstention, token usage, latency, advisory-note presence, matched decision transitions, and `decision_flip_rate`.

Disagreement is instability evidence only. The baseline does not become truth by majority vote, and repeated outputs cannot create authority.

## Calibration-only execution

The research binary canonicalizes the target and accepts only this checkout's exact `fixtures/semantic-judges` directory. A holdout directory, renamed copy, or symlink to a holdout is rejected before provider credentials are used.

```text
cargo run -p reasoning-harness-cli --bin reason-materialization-study -- \
  fixtures/semantic-judges \
  --provider google \
  --model gemini-3.5-flash-lite \
  --fixture 07_causal_positive \
  --fixture 08_causal_negative \
  --fixture 09_causal_ambiguous \
  --seed 2000 \
  --trials 1
```

The `semantic-materialization-study` GitHub Actions workflow defaults to the causal positive/negative/ambiguous triad, so the first live validation is six provider calls: three v3 baseline and three materialized-arm calls. Full calibration and repeated trials remain explicit later-stage choices.
