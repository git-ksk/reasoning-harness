# Research plan

## Hypothesis

A deterministic reasoning harness can reduce unsupported claims and hidden assumptions even when candidate generation uses a smaller or cheaper model.

The next product-level hypothesis is stronger: when the harness can identify exactly why a candidate is unresolved, a bounded resolution and re-verification loop can recover more grounded answerable cases **without increasing unsupported final answers**.

The harness is therefore evaluated both as a diagnostic system and as a future evidence-grounded reasoning runtime. Diagnosis quality is necessary, but the eventual product test is whether verified intermediate state can safely control additional evidence acquisition, repair, and finalization.

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

## Next experiments

### E6 — bounded resolution recovery — deterministic baseline implemented
The initial controlled nine-scenario suite starts from fixtures that intentionally produce `unknown` because required evidence is absent or insufficiently qualified. Give the runtime access to a controlled resolver that can return the missing evidence, a refuting fact, no result, or malformed/untrusted data.

Compare:

- direct one-shot generation;
- diagnose-only harness execution;
- bounded resolution plus re-verification.

Primary metrics:

- initially-unknown case recovery rate;
- unsafe final answer rate;
- supported / refuted / exhausted terminal distribution;
- resolution attempts and added token/latency/tool cost;
- preservation of `unknown` when no trusted resolution exists.

A higher answer rate is not a success if unsupported final claims increase.

### E7 — grounded finalization coverage — core gate implemented
The finalization contract gives a renderer a verified artifact and tests whether the final prose remains within the artifact's supported proposition set. Include adversarial renderers that paraphrase correctly, omit important qualifications, introduce plausible new facts, or convert uncertainty into certainty.

Primary metrics:

- factual final-claim coverage by supported artifact propositions;
- unsupported additions reaching final output;
- uncertainty/qualification preservation;
- correct routing of newly introduced factual propositions back into verification.

### E8 — evidence qualification during resolution — deterministic baseline implemented
The controlled resolution suite includes resolvers that return evidence which is real but stale, wrong-scope, insufficient-authority, conflicting, or not-yet-valid.

Primary question: does the resolution loop reject false closure and preserve `unknown` unless newly acquired evidence actually satisfies the required qualification?

## Planned runtime capabilities

- [implemented] temporal, scope, and provenance evidence qualification (#16)
- [implemented] versioned/stratified benchmark corpus v1 with stable base-case identity (#14)
- [implemented] provider-neutral typed resolution requests
- [implemented] bounded per-run and per-request resolution attempt/token/time policies
- [implemented contract] resolver adapters remain outside the correctness authority boundary; concrete domain adapters are deferred
- [implemented] candidate repair/regeneration followed by mandatory re-verification
- [implemented] grounded finalization with factual claim coverage checks
- [implemented] composable reasoning policy/invalidation (#27)
- [implemented] durable typed ReasoningThread checkpoint/resume/fork replay (#28)
- [implemented] explicit resolution/finalization terminal states including grounded, qualified, refuted, exhausted, unavailable, human-review, unresolved, and abstain
- [implemented] composable reasoning policy with policy-change dependency/finalization invalidation (#27)
- durable reasoning threads, checkpoint/resume/fork, and deterministic replay (#28)
- [implemented] calibrated soft semantic-judge contract and offline reliability metrics (#13); live semantic discovery remains optional/manual

## Planned diagnostic/reasoning work

- live semantic contradiction/counterexample discovery experiments under the implemented calibrated soft-only boundary
- first-principles decomposition only if it adds measurable diagnostic signal
- semantic-loss verification after finalization/coverage semantics are defined
- generalized oracle adapter interface where concrete consumer needs justify it
- verification and resolution budget policies

## Evaluation principle

Do not optimize for a single judge-model score. Prefer measurable protocol properties, golden fixtures, adversarial fixtures, external oracles, and explicit authority boundaries. Model-judge metrics should be explicitly labeled as soft evidence.

For the grounded runtime, optimize neither for raw answer rate nor for maximum abstention. The core trade-off is grounded answerability versus unsafe final output. Every recovery metric must therefore be paired with an unsafe-final-answer or final-claim-coverage metric.

## Benchmark methodology

The first benchmark holds the generated candidate constant between a naive baseline and the harness arm so that measured differences come from the deterministic process rather than a different model sample. Recorded candidates are CI regression fixtures only; empirical claims require live repeated provider runs. See [benchmark design](benchmark.md).

The benchmark must penalize both false acceptance and trivial over-conservatism. Therefore verdict accuracy and per-class accept/reject/unknown recall are reported alongside unsupported accepted claims.

Future resolution-loop benchmarks must preserve the same corpus-v1 stable base-case identity across one-shot, diagnose-only, and bounded-resolution variants. A recovered case must not silently replace the original correctness denominator. Operational failures, resolution exhaustion, and missing resolver coverage are reported separately from correctness.

## Oracle-controlled regression vs open-world research

Fixture oracle receipts are used only where the expected hard result is deliberately known. They test whether the harness correctly consumes authority without granting that authority to the model. Open-world contradiction discovery, counterexample generation, semantic causal evaluation, and retrieval quality are separate research problems and must be measured separately.

A resolver finding a document is not equivalent to an oracle proving the requested proposition. Resolution experiments must explicitly distinguish acquisition success from verification success.

See [ADR-0002](adr/0002-grounded-resolution-and-finalization.md) for the runtime authority and finalization boundary that these experiments are intended to validate.
