# RSD2 sufficiency risk / stability characterization

Tracking: #91, #121. Frozen predecessor configuration: `evidence-sufficiency-coordinate-rsd1-v1`.

RSD2 does not change the RSD1 prompt, three-way schema, fallback behavior, corpus, or model authority.
It repeats the same 12 calibration fixtures over five seeds for both initial provider/model arms.

## Primary distinction

The study reports exact three-class stability, but the conservative control boundary is:

```text
sufficient
vs
non_sufficient = insufficient | mixed
```

`mixed <-> insufficient` drift is still measured because it matters for diagnostics and future UX.
It does not by itself cross the first product-bridge safety boundary. Any
`sufficient <-> non_sufficient` drift is binary instability and is treated as materially higher risk.

No majority vote is used to choose a semantic result or create authority. Repeated decisions are only
observations used to characterize stability.

## Simulated risk interpretation

On this calibration corpus, a predeclared `insufficient | mixed` case represents a situation where an
assertive execution should not proceed without additional resolution. RSD2 therefore reports:

- `simulated_unsafe_proceed_before_gate`: count of successful non-sufficient cases if no residual gate
  existed after D3 `permit`;
- `simulated_unsafe_proceed_after_gate`: false-safe count after applying the advisory coordinate;
- `simulated_unsafe_proceed_prevented`: the difference.

These are **control simulations**, not observed product hallucination counts. Actual natural-language
unsafe assertion reduction remains an NL-5 product metric after a successor passes a fresh holdout and
is integrated.

## Frozen pre-observation gate to a fresh holdout

For each provider arm across five trials:

1. operational completion = 1.00;
2. conservative binary accuracy >= 0.95;
3. false-safe count = 0;
4. false-abstain rate <= 0.05;
5. sufficient recall >= 0.95;
6. binary fixture unanimity = 1.00;
7. authority-bearing output remains impossible/invalid by schema;
8. exact three-class accuracy and exact fixture unanimity are reported but do not override the binary
   safety gate.

Passing permits only the next research action: **freeze a fresh independent holdout**. It does not
permit product adoption or natural-language CLI integration.

## Cross-model replication after Gemini operational incompleteness

The first two unchanged Gemini 3.5 Flash-Lite RSD2 arms each produced 59/60 successful semantic
observations with zero observed false-safe/false-abstain decisions, but each missed the predeclared
100% operational-completion gate for a different provider-side failure. The gate is not relaxed and
the partial runs are not merged.

Before a fresh promotion holdout is frozen, run the exact frozen RSD1/RSD2 contract on an independent
second model arm. `gemma-4-31b-it` is predeclared as the next Google-hosted replication target because
it has prior semantic-runtime protocol evidence in this repository. The replication changes only the
model identifier: corpus, prompt/schema/fallback contract, seeds 5000–5004, max tokens, and RSD2
thresholds remain unchanged. Passing is cross-model replication evidence, not product authority.
