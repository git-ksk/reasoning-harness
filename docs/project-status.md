# Project status

## Current phase

The repository is an early research prototype with a maturing evidence-grounded runtime core. The authority boundary, native CLI, deterministic fixture benchmark, live Mistral/Google/NVIDIA candidate adapters, trusted verification receipts, causal/assumption/evidence-qualification diagnostics, metamorphic/repeated stability, corpus versioning, bounded resolution, and grounded finalization coverage are implemented.

The native core now implements the provider-neutral **evidence-grounded reasoning runtime** protocol from ADR-0002: unresolved verified state can produce bounded resolution requests, admitted evidence or revised candidates are re-verified, and final factual claims are coverage-checked. Concrete open-world resolver integrations and live resolution-quality evidence remain separate work.

This is not a claim that open-world reasoning is solved. Current correctness gains depend on deterministic structure and on trusted oracles where a hard answer exists. Future resolution work must preserve that authority boundary rather than turning retrieval or model self-correction into implicit truth.

## Implemented

- Rust-only core, CLI, eval, and provider adapter crates. Mistral, Google Gemini Interactions, and NVIDIA Hosted NIM candidate-generation adapters are implemented.
- Harness-owned evidence and untrusted `ReasoningCandidate` boundary.
- Deterministic structural/provenance validation.
- `accept | reject | unknown` policy.
- Trusted verification receipts that are never model-owned or model-visible.
- Receipt-backed support promotion and contradiction rejection.
- Narrow deterministic Five Whys lexical-restatement removal localized to the offending inference edge.
- Observational typed causal diagnostics with exact scoped support/refutation and conservative unknown handling for association, reverse direction, partial support, conflict, missing binding, and missing exact evidence.
- Harness-owned explicit assumptions and observational unsupported-premise diagnostics.
- Twenty committed claim-verdict regression fixtures (5 accept / 6 reject / 9 unknown) plus separate eight-case causal, five-case assumption, and eight-case evidence-qualification corpora.
- A six-family deterministic metamorphic regression layer with dedicated seed fixtures outside the ordinary correctness denominators.
- Repeated-trial diagnostic stability for adversarial, candidate-normalization, causal, assumption, and evidence-qualification signals, kept separate from correctness stability.
- Provider-neutral soft semantic-judge calibration with typed finding/no-finding/abstain observations, stable judge provenance, labelled positive/negative/ambiguous fixtures, precision/recall, pairwise agreement, and nominal Krippendorff alpha.
- Harness-owned temporal/scope/provenance evidence metadata and requirements with qualification-aware built-in structured verification.
- Versioned corpus v1 manifest covering 41 deterministic claim/causal/assumption/evidence-qualification cases with stable IDs, category/difficulty strata, score compatibility, provenance, contamination, redistribution, and lifecycle metadata.
- Provider-neutral bounded resolution requests/results, resolver and trusted-verifier adapter boundaries, explicit evidence admission, per-run/per-request budgets, mandatory re-verification, and terminal-state accounting.
- Composable `ReasoningPolicy` layers with policy identity, conservative authority/scope/resolver capability composition, immutable-snapshot invalidation, inference dependency propagation, finalization invalidation, and soft-finding escalation without truth authority.
- Grounded finalization with typed factual-claim coverage and re-routing of newly introduced factual propositions through hypothesis/resolution/verification.
- Nine deterministic controlled resolution scenarios plus `reason eval-resolution`, reported separately from corpus correctness and repeated diagnostic stability.
- Manual, secret-isolated live benchmark workflow spanning Mistral, Google-hosted Gemma/Gemini, and a narrowed routine NVIDIA Nemotron target.
- GitHub CI, Dependabot configuration, contribution/security guidance, issue and PR templates.

## Known gaps

### Reasoning control plane

ADR-0003 durable control-plane work is implemented without expanding core into a generic agent framework. #27 provides composable `ReasoningPolicy` and policy-change dependency invalidation; #28 provides append-oriented `ReasoningThread` events, deterministic checkpoints, interrupt/resume, non-destructive fork lineage, and replay-time policy verification. Resolver side effects are never replayed implicitly, persistence storage remains abstract, and skills/subagents remain deferred.


### Grounded runtime integration gaps

The provider-neutral bounded loop is implemented, but production acquisition integrations are intentionally not in core. Remaining product/research work includes:

- real web/database/MCP/human-review resolver adapters owned outside core;
- live repeated resolution studies against stochastic model/resolver combinations;
- automatic causal-evidence acquisition/ingestion for the typed `CausalRelation` resolution target;
- model-backed final renderers evaluated against the implemented claim-coverage gate;
- concrete persistence/storage and product-level pause orchestration when a consumer requires it; core thread/checkpoint replay itself is implemented.

Resolver success must continue to be distinguished from verification success. The deterministic nine-scenario suite proves control-flow and authority invariants, not open-world answer quality.

### Existing research gaps

- Exact natural-language receipt binding was confirmed too brittle for live paraphrases. The current implementation now uses typed propositions and harness-owned structured facts for the built-in hard verifier; exact-string binding remains compatibility-only.
- Hard contradiction/counterexample discovery exists for structured harness-owned facts. Model-backed semantic discovery is now implemented only through the calibrated soft-judge boundary; it cannot create hard findings, verification receipts, epistemic promotion, or verdict authority. Live quality remains an empirical/manual research question.
- Counterexample discovery coverage is still narrow outside explicit structured propositions.
- Five Whys lexical cleanup remains intentionally syntactic; evidence-aware causal inspection is observational and does not certify the whole artifact or change the final claim verdict.
- Candidate-supplied causal-evidence references remain deferred; the repeated-trial report can aggregate causal support/refutation/unknown assessments plus finding/reason observations without moving them into correctness authority. A live causal-generation/input contract remains deferred.
- Deterministic metamorphic robustness is implemented across six transform families. Repeated-trial diagnostic stability is also implemented: adversarial, candidate-normalization, causal, assumption, and evidence-qualification signals have complete-trial-only frequencies, count distributions, explicit operational exclusions, and Wilson intervals where the sample threshold is met.
- Assumption/unsupported-premise diagnostics are implemented with harness-owned explicit assumptions, deterministic typed premise checks, a separate five-case corpus, and repeated-trial diagnostic signals. Semantic extraction of untyped assumptions remains soft/deferred.
- Temporal validity, applicability scope, and provenance/authority qualification are implemented for generic harness-owned evidence. Domain-specific source taxonomy, open-world retrieval, and automatic qualification of the separate `CausalEvidence` contract remain out of core scope.
- Corpus v1 now versions and stratifies all 41 primary deterministic cases. Future version changes must preserve stable IDs and score-compatibility rules; metamorphic seeds remain unscored controls.
- Cross-model semantic-judge portability research (#46/#53/#55) is complete for the v3 -> v4 experiment. The frozen v4/holdout-v3 matrix produced zero conformant and zero usable-with-limitations models: Ministral 8B lost labelled precision and uncertainty abstention, Gemini preserved labelled precision/recall but fell below the ambiguous-abstention gate, Mistral Small again collapsed abstention, Ministral 14B became protocol-complete but semantically over-assertive, and Nemotron remained protocol-incomplete with every successful call returning `finding`. The predeclared v4 adoption gate therefore failed. Runtime defaults are restored to the previously characterized `soft-semantic-v3` contract; v4, holdout-v3, and run results remain immutable research history. See `docs/semantic-judge-conformance.md`.
- The calibration-only #57 isolation study then held v3 semantic wording fixed and changed only the model-facing output representation. Strict discriminated output made Ministral 14B protocol-complete (84/90 -> 90/90; 0/5 -> 5/5 complete trials) but produced only 0.286 ambiguous abstention; Ministral 8B stayed 90/90 while ambiguous abstention fell 0.943 -> 0.714; Gemini was effectively invariant; Nemotron remained incomplete. This demonstrates representation-induced semantic instability, so PR #58 was closed without merge and no holdout-v4 was consumed. The next research gate is format invariance, minimal harness-owned materialization, and selective abstention on calibration data before any new independent holdout.
- Issue #59 R1a calibration characterization is now measured on Gemini 3.5 Flash-Lite with counterbalanced execution. In the 18-fixture single-trial matrix, v3 and `nested_result_object` were 18/18 complete with zero flips, compact keys were 17/18 complete, and the tuple form was only 7/18 complete because non-finding decisions frequently carried invalid finding payloads. The five-trial v3-vs-nested gate then completed 90/90 pairs per representation with 2/90 matched flips, both on `15_causal_incomplete_scope_ambiguous`; nested was stable `abstain` across all five seeds while v3 changed to `finding` for two seeds. Mistral full-corpus R1a remains blocked by provider structured-generation failures, no historical holdout was consumed, and R2 harness-owned materialization is the next calibration-only step. See `docs/semantic-format-invariance.md`.
- Stable ranking claims require repeated trials. Issue #6 completed the 5-trial Mistral/Google matrix plus a targeted 10-trial follow-up for models tied on all primary correctness metrics; operational completeness is reported separately from correctness variance.

## Release posture

No stable API guarantee is made yet. Breaking schema/runtime changes are acceptable while the research contracts are still being validated by fixtures and live experiments.

The project may claim an implemented provider-neutral bounded resolution/finalization protocol, but not generic open-world grounded-answer quality. That stronger claim requires real resolver integrations and live measurement beyond fixture oracles.

- Live Mistral testing exposed malformed inference suggestions as a separate provider-quality issue. The runtime now isolates structurally invalid inference edges and records them in `candidate_diagnostics` instead of failing unrelated claims.

### Bounded grounded resolution and finalization

Issue #22 adds typed resolution requests for proposition, causal, evidence-qualification, revision, and human-review targets; separate untrusted resolver and trusted verifier boundaries; harness-owned evidence admission; per-run and per-request budgets; mandatory re-verification; explicit terminal states; and typed final factual-claim coverage. The initial nine deterministic resolution variants reuse corpus-v1 base case `claim:missing-evidence`: one recovers unknown to supported, one resolves to refuted, and seven preserve unknown under stale/scope/authority/conflict/no-result/malformed/untrusted resolver conditions. The aggregate records zero unsafe final answers and 1.0 typed final-claim coverage. These are regression fixtures, not model/resolver quality claims.

## Latest live verification result

After migrating the built-in hard verifier to typed propositions, canonical verified rendering, and explicit normalization of malformed untrusted inference edges, the 2026-08-30 Mistral live benchmark completed 7/7 runs with zero deterministic verifier failures. The harness arm reached 6/7 verdict accuracy (85.7%), kept unsupported accepted claims at 0, achieved 100% accept recall and 100% unknown recall, and reached 50% reject recall. The remaining miss is now tracked as generic contradiction/counterexample discovery rather than verifier binding.

### Adversarial discovery

The core now has a provider-neutral `AdversarialDetector` contract and typed `AdversarialFinding` records. Structured harness-owned fact conflicts are classified deterministically as hard contradictions or counterexamples. Findings themselves remain observational; only the verifier boundary can change epistemic state or force rejection. The 20-case recorded corpus reaches contradiction detection 1.0 and counterexample detection 1.0 under deterministic structured-fact coverage.

### Evidence-aware causal diagnostics

Issue #4 adds typed `CausalRelation`, harness-owned `CausalEvidence`, per-edge assessments, and typed hard/soft findings. Exact scoped support can mark an edge supported; exact explicit refutation can mark it refuted. Association-only evidence, reverse-direction support, partial support, conflicting evidence, missing exact evidence, and incomplete proposition binding remain unknown. The inspector cannot mutate claim state, create verification receipts, or directly decide `accept | reject | unknown`. Its eight-case deterministic corpus is reported separately from the 20-case claim benchmark and from Issue #6 correctness denominators.

### Assumption and unsupported-premise diagnostics

Issue #12 adds harness-owned explicit `assumptions` separately from task `hypotheses`, plus an observational `AssumptionDiscoveryPass`. Premises with trusted supported/known state or a derivation from trusted support are classified `supported`; propositions explicitly supplied as assumptions are `explicit_input_assumption`; typed premises with neither are `unsupported`; untyped premises are `unbound`. Unsupported typed premises produce hard process findings relative to the supplied context, while unbound premises remain soft because semantic identity is unavailable. Findings do not mutate claim state or final verdict. The five-case assumption corpus is reported separately from the 20-case correctness and eight-case causal corpora, and its signals participate in the repeated diagnostic report.

### Temporal, scope, and provenance evidence qualification

Issue #16 adds harness-owned `EvidenceMetadata`, proposition-key qualification requirements, and a domain-neutral authority-rank policy. Exact metadata coverage qualifies evidence; stale/not-yet-valid records, disjoint or expanded scope, insufficient authority, and conflicts among otherwise qualified structured values produce hard findings. Missing temporal/scope/provenance bindings remain soft/unknown. When requirements exist, the built-in structured verifier uses only qualified evidence and withholds hard receipts on missing qualification or qualified-value conflict; old inputs without requirements keep historical verifier behavior. The eight-case qualification corpus and repeated diagnostic signals are separate from final correctness and causal-edge metrics. Explicit external trusted receipts remain an independent oracle boundary whose caller owns qualification policy.

### Versioned corpus and benchmark hardening

Issue #14 defines `fixtures/corpus/v1.json` as corpus `1.0.0` with score-compatibility ID `corpus-v1`. It covers all 41 active primary deterministic cases: 20 claim, 8 causal, 5 assumption, and 8 evidence-qualification cases. Recorded claim eval adds category/difficulty slices without changing the historical overall comparison, while live runs preserve corpus identity without creating pooled stratum scores. Manifest validation fails closed on duplicate/missing active case identity and committed public metadata is provider-neutral/secret-free. Change discipline, contamination limitations, and saturation warnings are documented separately.

The 20-case benchmark now uses typed proposition labels instead of provider-generated claim IDs. Harness-owned hypotheses formalize task propositions independently of model output, and `unsafe_accept_cases` distinguishes true final unsafe acceptance from strong intermediate claims inside an overall `Unknown` result. The manual Mistral workflow now compares Ministral 3B/8B/14B and Mistral Small on the same corpus.

### Cross-model observation

The first hardened 20-case Mistral matrix completed successfully for Ministral 3B/8B/14B and Mistral Small. Harness accuracy was 0.80 for 3B and 1.00 for 8B, 14B, and Small. Every harness arm recorded zero unsafe final accepts, 1.00 contradiction detection, 1.00 counterexample detection, and zero deterministic verifier failures. Mistral Small achieved the 20/20 result with substantially fewer tokens and lower latency than the 8B and 14B runs in this single trial; repeated trials are required before drawing a model-ranking conclusion.

- Gemma 4 support uses the current Google Gemini Interactions API and remains outside the correctness authority boundary. Live Gemma acceptance is pending a configured `GEMINI_API_KEY`.

### Gemma 4 provider validation

The Rust provider boundary now includes Google-hosted Gemma and Gemini text models through the Gemini Interactions API. The live diagnostic matrix includes Gemma 4 26B/31B plus Gemini 3.1 Flash-Lite and Gemini 3.5 Flash-Lite; managed agents such as Antigravity are intentionally excluded. A live `gemma-4-31b-it` run completed all 20 benchmark cases: baseline accuracy 0.85, harness accuracy 0.95, unsafe final accepts 0, reject/unknown recall 1.00, contradiction and counterexample detection 1.00, and deterministic verifier failures 0. This is the first cross-family live validation beyond Mistral. `gemma-4-26b-a4b-it` remains experimental: the Issue #6 five-trial study generated 98/100 cases, with two provider-side HTTP 400 copyright/recitation blocks producing 3 complete and 2 incomplete trials.

### NVIDIA Hosted NIM research

NVIDIA Hosted NIM support is implemented through the OpenAI-compatible Chat Completions endpoint with model IDs treated as data. Nemotron Lightning is the only routine NVIDIA matrix target after the 20-case research sweep; GPT-OSS 20B, Gemma-through-NVIDIA, and DeepSeek V4 Flash remain ad-hoc because of observed protocol/timeout instability. NVIDIA request-start pacing is a client-side 1.6-second minimum interval (37.5 starts/minute), not a claimed provider quota.

### Repeated-trial stability phase

Issue #6 now provides explicit per-trial operational completeness, correctness denominators, complete-trial-only mean/min/max/population-stddev, and separate token/latency distributions. The five-trial matrix found perfect complete-trial harness correctness for Ministral 8B/14B and Gemini 3.1, 0.99 for Mistral Small, 0.98 for Gemini 3.5, 0.95 for Gemma 31B, 0.867 across three complete Gemma 26B trials, and a consistently over-conservative 0.75 for Ministral 3B. The targeted 10-trial follow-up left 8B/14B tied at 10/10 perfect complete trials; Gemini 3.1 remained correctness-perfect across 9 complete trials but had one protocol failure in 200 attempted generations. Required deterministic CI remains credential-free and live studies remain diagnostic.

### Repeated-trial diagnostic stability

Issue #11 adds a provider-neutral diagnostic observation/report contract. Live claim benchmarks now emit `stability.diagnostics` as a sibling of `stability.correctness`, so finding frequency cannot alter Issue #6 correctness or operational denominators. Per-fixture diagnostic signals report exact complete-trial occurrences/denominators, family-level count mean/min/max/population-stddev, and a 95% Wilson score interval when at least five complete observations exist. Operationally incomplete trials are excluded from diagnostic distributions and counted explicitly. The same core report type accepts causal support/refutation/unknown assessments plus finding/reason observations, assumption signals, and evidence-qualification findings; live causal candidate generation remains a separate deferred input contract.
