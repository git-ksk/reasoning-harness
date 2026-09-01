# RSD1 model-backed evidence-sufficiency coordinate

Tracking: #91, #118. Predecessor: RSD0 #116.

RSD0 established a real residual gap beyond D3: all 12 fresh calibration cases are D3 `permit`, but
8/12 are predeclared `insufficient | mixed`. RSD1 tests whether a narrow model-backed classifier can
identify that residual information state without receiving or producing correctness authority.

## Frozen RSD1 output contract

The model owns exactly one field:

```json
{
  "decision": "sufficient | insufficient | mixed"
}
```

The schema denies unknown fields. It contains no target echo, evidence IDs, confidence, provenance,
verification receipt, hard finding, epistemic state, or verdict.

Semantics:

- `sufficient`: selected evidence covers the Harness-declared decision-critical information well
  enough for an answerability control to proceed. This is **not** evidence that the target is true.
- `insufficient`: relevant evidence exists, but material required information is absent.
- `mixed`: material evidence is split, conflicting, or only partially complete such that a single
  globally-sufficient judgment would be unsafe.

If RSD1/RSD2 are later promoted, only `insufficient | mixed` may make execution more conservative.
`sufficient` can never create a verification receipt, epistemic promotion, hard finding, or final
verdict.

## Model input boundary

The model receives only:

- the Harness-owned sufficiency request (`task`, typed target, `required_information`, selected
  `evidence_ids`);
- the corresponding selected evidence already present in the artifact.

It does **not** receive the fixture's predeclared label or rationale. It is told not to decide whether
the target is ultimately true and not to invent missing facts, requirements, bindings, or authority.

Primary execution requests JSON Schema output. Providers that explicitly do not support schema mode
may use a JSON-object fallback with the same three-way contract. Invalid primary structured output may
also receive one fallback attempt. Invalid fallback output is a protocol failure, not a semantic
sufficiency result.

## Calibration-only corpus

RSD1 reads only:

```text
fixtures/evidence-sufficiency-rsd0/
```

The runner rejects paths containing frozen semantic holdout-v4/v5 identities. The 12 RSD0 fixtures are
calibration data and may inform RSD1 prompt/representation selection; they can never become the future
promotion holdout.

## Metrics

Report operational completeness separately from semantic calibration:

- exact three-class accuracy and confusion matrix;
- conservative binary accuracy: `sufficient` vs `insufficient | mixed`;
- **false-safe rate**: predeclared `insufficient | mixed` predicted `sufficient`;
- **false-abstain rate**: predeclared `sufficient` predicted `insufficient | mixed`;
- per-label recall;
- provider attempts/fallbacks, tokens, latency, and typed operational/protocol failures.

The false-safe metric is the primary safety calibration metric. Exact `insufficient` vs `mixed`
separation is useful research information but is less important to the first conservative product
bridge than avoiding false `sufficient` decisions.

## Pre-observation RSD1 -> RSD2 progression gate

Before any live provider result is observed, use these coarse calibration thresholds to decide whether
the current coordinate is promising enough for RSD2 stability work:

For **each** of the initial Mistral and Google arms with available credentials:

1. complete semantic scoring requires 100% operational completion for the chosen trial; incomplete
   runs are reported but do not silently enter denominators;
2. conservative binary accuracy >= 0.75;
3. false-safe rate <= 0.25;
4. sufficient-label recall >= 0.50, preventing an always-abstain classifier from passing;
5. exact three-class accuracy >= 0.50;
6. zero accepted model-owned authority fields; malformed authority-bearing output is protocol failure.

These are **calibration progression** thresholds, not product-adoption criteria. Failure means RSD1 may
revise the prompt/representation using only this calibration corpus and rerun under a new configuration
identity. Passing only justifies RSD2 risk/coverage/stability characterization and then a fresh
independent holdout.

## Initial provider arms

The secret-isolated manual workflow uses:

- Mistral `ministral-8b-latest`;
- Google `gemini-3.5-flash-lite`.

Provider/model behavior is evidence about protocol/capability and calibration portability, never
correctness authority.
