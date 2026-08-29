# Research plan

## Hypothesis

A deterministic reasoning harness can reduce unsupported claims and hidden assumptions even when candidate generation uses a smaller or cheaper model.

## Initial experiments

### E1 — provenance discipline
Compare direct model answers with harnessed answers on fixtures containing intentionally missing evidence.

Primary metric: unsupported accepted claims.

### E2 — explicit uncertainty
Remove a required fact from otherwise answerable fixtures.

Primary metric: correct `unknown` classification instead of fabricated completion.

### E3 — framework structure
Compare prose-only 5 Whys with typed causal links that require evidence references.

Primary metrics: unsupported causal edges, restated symptoms, root-cause mismatch.

### E4 — model substitution
Run identical fixtures through multiple model adapters, including a low-cost/free candidate generator.

Primary question: how much output quality variance is absorbed by the harness?

### E5 — semantic preservation
Transform a verified artifact into progressively simpler explanations and detect dropped invariants or unsupported additions.

## Planned passes

- contradiction detection
- counterexample generation + deterministic recording
- assumption extraction
- first-principles decomposition
- semantic-loss verification
- generalized oracle adapter interface (typed trusted receipts are implemented; adapter contract remains)
- verification-budget policies

## Evaluation principle

Do not optimize for a single judge-model score. Prefer measurable protocol properties, golden fixtures, adversarial fixtures, and external oracles. Model-judge metrics should be explicitly labeled as soft evidence.

## Benchmark methodology

The first benchmark holds the generated candidate constant between a naive baseline and the harness arm so that measured differences come from the deterministic process rather than a different model sample. Recorded candidates are CI regression fixtures only; empirical claims require live repeated provider runs. See [benchmark design](benchmark.md).

The benchmark must penalize both false acceptance and trivial over-conservatism. Therefore verdict accuracy and per-class accept/reject/unknown recall are reported alongside unsupported accepted claims.

## Oracle-controlled regression vs open-world research

Fixture oracle receipts are used only where the expected hard result is deliberately known. They test whether the harness correctly consumes authority without granting that authority to the model. Open-world contradiction discovery, counterexample generation, and semantic causal evaluation are separate research problems and must be measured separately.
