# Research plan

## Hypothesis

A deterministic reasoning harness can reduce unsupported claims and hidden assumptions even when candidate generation uses a smaller or cheaper model.

The next product-level hypothesis is stronger: when the harness can identify exactly why a candidate is unresolved, a bounded resolution and re-verification loop can recover more grounded answerable cases **without increasing unsupported final answers**.

The harness is therefore evaluated both as a diagnostic system and as a future evidence-grounded reasoning runtime. Diagnosis quality is necessary, but the eventual product test is whether verified intermediate state can safely control additional evidence acquisition, repair, and finalization.

## Product/research split

The adopted D3 runtime remains the stable semantic baseline. The first residual-sufficiency successor
program (#91) is complete: RSD0/RSD1/RSD2, independent frozen holdout, operational stabilization,
versioned product wiring, and NL-5 all finished without granting model output authority.

Current work is split between:

- **Product (#90):** harden the native `reason` CLI as the first supported external interface, with
  versioned machine-readable contracts, installation/release compatibility, observability, and
  repeated real-workload acceptance evidence.
- **Product utility (#139):** improve low-coverage / over-abstaining model behavior without weakening
  the authority boundary or tuning against frozen research holdouts.
- **Follow-on research:** selective/conformal abstention or relation-level causal sufficiency only if a
  newly specified gap justifies another research identity and fresh evaluation sequence.

Research is never shipped directly. Any future candidate must again pass fresh calibration, an
independently frozen holdout for adoption, operational stabilization, explicit profile/rollback, and
CLI compatibility coverage before a separate reversible product-adoption change.

See [native CLI product roadmap](product-roadmap.md).

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

- [implemented #33] repeated live semantic contradiction/unsupported-premise/causal-gap discovery under the calibrated soft-only boundary, including explicit ambiguity-abstention measurement
- [implemented #36] frozen independent semantic-judge holdout v1 with expanded ambiguity/counterexample coverage; first five-trial Mistral study completed without changing the frozen corpus
- [implemented #38] `soft-semantic-v3` calibration and independent holdout-v2 study completed; five-trial holdout precision/recall 1.000, mean coverage 0.700, and ambiguous abstention 0.933
- [implemented #46/#53/#55] cross-model semantic conformance: v4 was independently measured on frozen holdout-v3, failed the predeclared portability gate with zero conformant/usable models, and the runtime baseline was restored to `soft-semantic-v3`
- [calibration result #57] strict discriminated output was isolated from v3 semantic wording; it fixed Ministral 14B's repeated non-finding/finding-object protocol failures but changed uncertainty behavior on Mistral models, demonstrating that model-facing structured output cannot be assumed semantically neutral; PR #58 was closed without merge
- [calibration result #59] R1a format invariance is characterized on Gemini 3.5 Flash-Lite: the counterbalanced five-trial v3-vs-nested run completed 90/90 pairs per representation with 2/90 format flips, both isolated to one ambiguous causal fixture; the nested form stayed stable across all five seeds, while tuple/key variants showed that protocol robustness can change independently of successful-pair semantics. Mistral full-corpus R1a remains operationally blocked.
- [calibration result #59] R2 materialization exposes only model-owned `decision` plus optional `advisory_note`, deterministically copies request-known `kind`/`target` only for `finding`, and is 90/90 protocol-complete across five trials on both Gemini 3.5 Flash-Lite and Ministral 8B; Gemini preserves high uncertainty abstention while Ministral remains stably assertive on three of seven ambiguous fixtures
- [calibration result #59] R3b Gemini 3.5 Flash-Lite + Ministral 8B completed 180/180 calibration calls across five seeds; disagreement was confined to four ambiguous fixtures, clear positive/negative cases never disagreed, and the combined disagreement-only policy retained precision/recall 1.0, ambiguous abstention 1.0, and clear-case coverage 1.0
- [rejected #59] independent holdout-v4 run `33371523453` completed 280/280 calls but failed the frozen uncertainty gate (fixture-collapsed ambiguous abstention 0.8333 < 0.85; four of five trials < 0.80) and source/seed labelled-polarity stability; R3b is not adopted
- [frozen diagnostic #59] post-observation audit found label/spec conflicts in `v4h-13` and `v4h-20`; holdout-v4 remains unchanged and cannot be tuning data
- [designed #73] fresh calibration now separates deterministic semantic decidability/evidence-sufficiency from the final soft decision: explicit missing binding or unsatisfied typed evidence requirements may only force `abstain`; `permit` is not correctness evidence
- [implemented #73] deterministic calibration has 14 fixtures in seven matched control/mutation pairs spanning target binding, evidence presence, temporal/scope/authority qualification, required metadata, and qualified-evidence conflict; v1 forces only contradiction/unsupported-premise plus structural counterexample binding, while causal-gap remains permit-only until relation-level sufficiency is typed; at the D1/D2 calibration freeze no holdout-v5 yet existed
- [designed #73] D2 separates semantic polarity from assertive eligibility: semantic precision/recall are computed only on eligible clear cases, while typed-insufficiency abstention and unsafe-assertion reduction use a separate denominator; this prevents correct forced abstention from looking like a recall regression
- [implemented #73] D2 v1 now resolves 15 existing calibration semantic cases across all four kinds into a separate typed-eligibility manifest and runner; seven clear cases carry one paired typed-insufficiency mutation across three kinds, causal-gap remains permit-only, and four ambiguous cases remain eligible controls outside the insufficiency denominator
- [implemented #73] the D2 runner validates source labels and deterministic gate expectations before provider initialization, reuses one unchanged R2 observation across matched typed variants, and preserves provider/protocol failure as operationally incomplete rather than semantic abstention
- [frozen #73] D2 v1 first-observation plan fixes the full 15-case calibration corpus, separate Gemini 3.5 Flash-Lite / Ministral 8B studies, seeds 6000-6004, five trials, 512 output tokens, and predeclared operational/coverage/precision/recall/typed-insufficiency/stability gates before any D2 provider call
- [calibration result #73] frozen D2 run `33377619803` passed every predeclared candidate gate on both Gemini 3.5 Flash-Lite and Ministral 8B: 75/75 calls, 5/5 complete trials, eligible clear coverage/precision/recall 1.000, 35/35 typed insufficiency abstentions, base unsafe assertions 35 -> composed unsafe 0, and no clear seed disagreement per provider
- [frozen #73] D3 candidate `semantic-decidability-d3-v1` keeps `soft-semantic-v3` and R2 model semantics unchanged and composes only the deterministic explicit-typed-preconditions gate; no runtime adoption occurs yet
- [frozen #73] fresh holdout-v5 is observation-free and statically reviewed: 24 new semantic cases (6 per diagnostic kind; 8 positive / 8 negative / 8 ambiguous), 10 clear typed-insufficiency variants across contradiction/unsupported-premise/counterexample only, one inference-binding case, and SHA-256-frozen source/manifest payloads
- [frozen #73] the independent execution plan fixes Gemini 3.5 Flash-Lite and Ministral 8B separately, seeds 7000-7004, five trials, 512 output tokens, full-corpus execution, and the already predeclared D3 adoption gates with no workflow inputs that can reshape the study
- [replication result #73] fixed `gemma-4-31b-it` cross-family replay completed R2, D2, and holdout-v5 in run `33384957101`; D2 and v5 each retained clear coverage/precision/recall 1.000, typed insufficiency abstention 1.000, and zero composed unsafe assertions, while Gemma 4 and Ministral 8B produced identical base decisions across all 120 matched v5 case/seed observations; this supports cross-family generalization without retroactively changing the original D3 provider set
- [pilot result #73] frozen holdout-v5 completed on Ministral 8B with 120/120 calls, 5/5 complete trials, eligible clear coverage/precision/recall 1.000, typed-insufficiency abstention 50/50, base unsafe assertions 50 -> 0, and zero clear-case seed disagreement
- [cross-family replication #73] fixed `gemma-4-31b-it` independently reproduced the D3 pattern on holdout-v5: 120/120 calls, 5/5 complete trials, clear coverage/precision/recall 1.000, typed-insufficiency abstention 50/50, base unsafe assertions 50 -> 0, and the same 120 base decisions as Ministral 8B
- [negative control #73] `nvidia/nemotron-3.5-lightning-30b-a3b` did not satisfy the current R2 protocol capability boundary: D2 completed only 7/15 provider observations and the v5 probe timed out after 18/24 fixtures, with repeated materialization-protocol failures caused by forbidden model-owned `finding` fields
- [completion #84] Gemini 3.5 Flash-Lite exact frozen holdout-v5 rerun completed in Actions run `33380880478` attempt 2 after quota reset: 120/120 calls, 5/5 complete trials, eligible clear coverage/precision/recall 1.000, typed-insufficiency abstention 50/50, base unsafe assertions 50 -> 0, zero permit-control escalations, zero clear-case seed disagreement, and zero provider/protocol failures; ambiguous abstention was 32/40 with disagreement confined to three ambiguous fixtures outside the adoption threshold
- [implemented stabilization #73] `semantic-decidability-d3-v1` has frozen runtime/config identity, corpus-independent R2 capability preflight, typed quota/rate-limit/timeout/provider/protocol telemetry, atomic partial-result checkpoints marked non-scorable, and an explicit `soft-semantic-v3` rollback profile
- [adopted #73] a separate reversible runtime PR switches the compiled default to D3 after stabilization CI; the frozen semantic contract and historical D2/v5 study workflows remain unchanged, and `soft-semantic-v3` stays available as the rollback profile
- [next semantic research #73] with D3 stabilization/adoption complete, test residual soft decidability (`sufficient | insufficient | mixed`) only for a measured gap left by deterministic typed metadata; keep selective/conformal abstention as a later calibrated option and causal relation-level sufficiency deferred until directional relation evidence is explicitly typed
- [implemented #39] classify structured-output fallback causes with provider-neutral typed telemetry before broader model comparison
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
