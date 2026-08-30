# Project status

## Current phase

The repository is an early research prototype with a maturing verification/diagnostic core. The core authority boundary, native CLI, deterministic fixture benchmark, live Mistral/Google/NVIDIA provider adapters, trusted verification receipts, edge-local Five Whys cleanup, observational evidence-aware causal diagnostics, assumption diagnostics, metamorphic robustness, and repeated diagnostic stability are implemented.

The product direction is broader than post-hoc diagnosis: the native runtime is intended to become an **evidence-grounded reasoning runtime** that can turn unresolved verified state into bounded resolution requests, re-verify repaired or newly grounded reasoning, and finalize answers only from adequately supported propositions. That end-to-end resolution/finalization loop is not yet implemented. See [ADR-0002](adr/0002-grounded-resolution-and-finalization.md).

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
- Twenty committed claim-verdict regression fixtures (5 accept / 6 reject / 9 unknown) plus a separate eight-case deterministic causal corpus and five-case assumption corpus.
- A six-family deterministic metamorphic regression layer with dedicated seed fixtures outside the ordinary correctness denominators.
- Repeated-trial diagnostic stability for adversarial, candidate-normalization, causal, and assumption signals, kept separate from correctness stability.
- Manual, secret-isolated live benchmark workflow spanning Mistral, Google-hosted Gemma/Gemini, and a narrowed routine NVIDIA Nemotron target.
- GitHub CI, Dependabot configuration, contribution/security guidance, issue and PR templates.

## Known gaps

### Grounded runtime closure

The largest product gap is no longer another diagnostic family. The runtime currently ends its implemented correctness path at `accept | reject | unknown`; it does not yet own an end-to-end bounded recovery loop from unresolved findings to additional evidence/verification, candidate repair, mandatory re-verification, and grounded final-answer construction.

Specifically not yet implemented:

- typed provider-neutral resolution requests;
- resolver attempt/history/budget state;
- adapters for acquiring new evidence or invoking external verifiers during a run;
- mandatory re-verification semantics after repair/regeneration;
- terminal resolution outcomes such as recovered, refuted, exhausted, or human-review-required;
- grounded finalization from verified artifact state;
- factual final-answer claim coverage checks that prevent a renderer from silently adding new unsupported propositions.

Retrieval, web search, databases, MCP tools, tests, compilers, and human review are expected to remain external adapters. Their output is not trusted merely because it was retrieved; it must cross the harness-owned evidence/verifier boundary.

### Existing research gaps

- Exact natural-language receipt binding was confirmed too brittle for live paraphrases. The current implementation now uses typed propositions and harness-owned structured facts for the built-in hard verifier; exact-string binding remains compatibility-only.
- Hard contradiction/counterexample discovery exists for structured harness-owned facts; semantic/model-backed discovery remains soft-only and is not yet implemented.
- Counterexample discovery coverage is still narrow outside explicit structured propositions.
- Five Whys lexical cleanup remains intentionally syntactic; evidence-aware causal inspection is observational and does not certify the whole artifact or change the final claim verdict.
- Candidate-supplied causal-evidence references remain deferred; the repeated-trial report can aggregate causal support/refutation/unknown assessments plus finding/reason observations without moving them into correctness authority. A live causal-generation/input contract remains deferred.
- Deterministic metamorphic robustness is implemented across six transform families. Repeated-trial diagnostic stability is also implemented: adversarial, candidate-normalization, causal, and assumption signals have complete-trial-only frequencies, count distributions, explicit operational exclusions, and Wilson intervals where the sample threshold is met.
- Assumption/unsupported-premise diagnostics are implemented with harness-owned explicit assumptions, deterministic typed premise checks, a separate five-case corpus, and repeated-trial diagnostic signals. Semantic extraction of untyped assumptions remains soft/deferred.
- Temporal validity, applicability scope, and provenance/authority qualification are not yet modeled on harness-owned evidence; #16 defines this provider-neutral evidence-quality boundary. Open-world retrieval and domain-specific source ranking remain out of core scope.
- The current claim, causal, and assumption corpora are not yet versioned/stratified as a public benchmark; #14 tracks corpus longevity and contamination/saturation policy.
- Semantic/model-backed discovery remains soft-only and is not yet calibrated; #13 is intentionally sequenced after deterministic measurement, evidence qualification, corpus discipline, and the grounded-runtime boundary are stable.
- Stable ranking claims require repeated trials. Issue #6 completed the 5-trial Mistral/Google matrix plus a targeted 10-trial follow-up for models tied on all primary correctness metrics; operational completeness is reported separately from correctness variance.

## Release posture

No stable API guarantee is made yet. Breaking schema/runtime changes are acceptable while the research contracts are still being validated by fixtures and live experiments.

The project should not claim end-to-end grounded answer generation until the ADR-0002 resolution and finalization contracts are implemented and measured. Today the implemented strength is an authority-aware reasoning verification and diagnostic runtime.

- Live Mistral testing exposed malformed inference suggestions as a separate provider-quality issue. The runtime now isolates structurally invalid inference edges and records them in `candidate_diagnostics` instead of failing unrelated claims.

## Latest live verification result

After migrating the built-in hard verifier to typed propositions, canonical verified rendering, and explicit normalization of malformed untrusted inference edges, the 2026-08-30 Mistral live benchmark completed 7/7 runs with zero deterministic verifier failures. The harness arm reached 6/7 verdict accuracy (85.7%), kept unsupported accepted claims at 0, achieved 100% accept recall and 100% unknown recall, and reached 50% reject recall. The remaining miss is now tracked as generic contradiction/counterexample discovery rather than verifier binding.

### Adversarial discovery

The core now has a provider-neutral `AdversarialDetector` contract and typed `AdversarialFinding` records. Structured harness-owned fact conflicts are classified deterministically as hard contradictions or counterexamples. Findings themselves remain observational; only the verifier boundary can change epistemic state or force rejection. The 20-case recorded corpus reaches contradiction detection 1.0 and counterexample detection 1.0 under deterministic structured-fact coverage.

### Evidence-aware causal diagnostics

Issue #4 adds typed `CausalRelation`, harness-owned `CausalEvidence`, per-edge assessments, and typed hard/soft findings. Exact scoped support can mark an edge supported; exact explicit refutation can mark it refuted. Association-only evidence, reverse-direction support, partial support, conflicting evidence, missing exact evidence, and incomplete proposition binding remain unknown. The inspector cannot mutate claim state, create verification receipts, or directly decide `accept | reject | unknown`. Its eight-case deterministic corpus is reported separately from the 20-case claim benchmark and from Issue #6 correctness denominators.

### Assumption and unsupported-premise diagnostics

Issue #12 adds harness-owned explicit `assumptions` separately from task `hypotheses`, plus an observational `AssumptionDiscoveryPass`. Premises with trusted supported/known state or a derivation from trusted support are classified `supported`; propositions explicitly supplied as assumptions are `explicit_input_assumption`; typed premises with neither are `unsupported`; untyped premises are `unbound`. Unsupported typed premises produce hard process findings relative to the supplied context, while unbound premises remain soft because semantic identity is unavailable. Findings do not mutate claim state or final verdict. The five-case assumption corpus is reported separately from the 20-case correctness and eight-case causal corpora, and its signals participate in the repeated diagnostic report.

### Benchmark hardening

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

Issue #11 adds a provider-neutral diagnostic observation/report contract. Live claim benchmarks now emit `stability.diagnostics` as a sibling of `stability.correctness`, so finding frequency cannot alter Issue #6 correctness or operational denominators. Per-fixture diagnostic signals report exact complete-trial occurrences/denominators, family-level count mean/min/max/population-stddev, and a 95% Wilson score interval when at least five complete observations exist. Operationally incomplete trials are excluded from diagnostic distributions and counted explicitly. The same core report type accepts causal support/refutation/unknown assessments plus finding/reason observations and assumption signals; live causal candidate generation remains a separate deferred input contract.
