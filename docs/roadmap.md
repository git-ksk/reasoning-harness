# Roadmap

## Project direction

Reasoning Harness is not trying to become a general-purpose model runner or a second Inspect/lm-eval. Its differentiator is a provider-neutral, authority-aware diagnostic layer for intermediate reasoning: deterministic structure and harness-owned evidence may create hard findings, while model-backed semantic discovery remains soft and observational until independently verified.

The next phase therefore prioritizes measurement quality before adding more named reasoning frameworks.

## v0.1 — trustworthy intermediate state and native CLI
- stabilize HarnessInput / ReasoningCandidate / ReasoningArtifact schemas
- JSON Schema export
- provenance coverage gates
- harness-owned evidence / untrusted candidate authority boundary
- verification receipts / oracle-backed promotion for safely upgrading supported claims **implemented**
- explicit unknown/assumption handling
- fixture-based eval runner
- native CLI for run / verify / eval workflows; explain remains deferred until renderer semantics are defined
- JSON output and CI-safe exit semantics
- first provider adapter experiment (Mistral HTTP adapter + manual live benchmark implemented)
- offline fixture regression separated from live provider benchmark runs
- explicit hard-validator vs soft-judge metric classification

## P0 completed — structured verifier binding
- [done] replace brittle exact-prose receipt matching with a typed `Proposition { key, value }` verification target
- [done] define harness-owned structured facts and provider-neutral verification boundaries
- [done] bind verifier results to structured propositions plus harness-owned structured facts, never model self-asserted authority
- [done] restore live accept/reject utility without increasing unsupported accepted claims
- [done] preserve exact-string receipt binding as a conservative compatibility mode
- [done] normalize malformed untrusted inference edges with explicit `candidate_diagnostics` rather than failing unrelated claims

## v0.2 — adversarial reasoning passes
- [done] provider-neutral `AdversarialDetector` contract with typed contradiction/counterexample findings
- [done] explicit `hard` vs `soft` finding strength; findings never own verdict authority
- [done] deterministic structured-fact contradiction/counterexample detector
- [done] counterexample detection metric and adversarial fixture coverage
- semantic/model-backed discovery remains soft until independently verified
- assumption pass moved to the research sequence below (#12)
- semantic-loss checks remain deferred until robustness/calibration foundations exist

## v0.3 — causal and framework diagnostics
- [done] extend the lexical Five Whys restatement pass with evidence-aware causal edge diagnostics; exact oracle-backed support/refutation is typed, unresolved semantic cases remain soft/unknown, and causal diagnostics stay outside final-verdict authority (#4 / PR #9)
- first-principles and Feynman/simplification work is deferred until the diagnostic contracts below demonstrate that another named framework adds measurable signal rather than presentation-only complexity
- a general framework plugin contract is likewise deferred until at least two independent semantic diagnostic families need the same extension boundary

## v0.4 — reproducible live research
- [done] cross-model benchmark matrix across Mistral, Google, and NVIDIA Hosted NIM
- [done] token/latency/cost accounting for live provider observations
- [done] fixture-level live concurrency with provider-owned pacing/retry semantics preserved
- [done] repeated-trial stability reporting with per-trial operational isolation and mean/min/max/stddev
- [done] 5-trial Mistral + Google stability matrix plus targeted 10-trial follow-up for tied models
- deterministic vs soft-verifier reporting remains explicit
- public benchmark corpus work moves to #14

### v0.4 research policy
- required CI remains deterministic and credential-free; live provider studies remain manual/secret-gated
- provider/model output remains an untrusted candidate and never owns verification or final-verdict authority
- operationally incomplete trials are reported explicitly and excluded from cross-trial correctness variance
- single live runs remain diagnostic observations and must not be presented as stable rankings
- NVIDIA routine coverage remains `nvidia/nemotron-3.5-lightning-30b-a3b`; other Hosted NIM model IDs are ad-hoc research inputs

## P0 next — robustness and diagnostic stability

### #10 Metamorphic reasoning robustness
Add deterministic semantics-preserving transformations and measure invariance of verdicts and hard findings separately from raw benchmark accuracy.

Priority transformations include ordering changes, irrelevant structured facts, stable-ID changes, and cause-set order. Free-form LLM paraphrase generation is not part of the initial hard benchmark.

### #11 Repeated-trial diagnostic stability
Extend the repeated-trial layer beyond final correctness to finding frequency and causal/adversarial diagnostic stability. Operational failures keep separate denominators. Add confidence intervals only with explicit methods and sample counts.

These two issues are the immediate next implementation sequence. They validate that the harness diagnoses the same reasoning problem reliably before semantic scope grows.

## P1 — broaden grounded reasoning signal conservatively

### #12 Assumption and unsupported-premise diagnostics
Add a narrow assumption pass that distinguishes explicit input assumptions from candidate-introduced unsupported premises. Hard status requires harness-owned structured support; semantic discovery remains soft.

### #16 Temporal, scope, and provenance evidence diagnostics
Extend harness-owned evidence with provider-neutral validity metadata so a proposition can be checked against the evidence's explicit time window, applicability scope, and configured provenance/authority requirement.

Hard findings require deterministic mismatch against explicit metadata. Missing metadata remains unknown; candidate-authored provenance cannot elevate authority; source-ranking policy stays outside domain-specific core logic. This is evidence qualification, not open-world retrieval or generic RAG orchestration.

## P2 — benchmark longevity and public research surface

### #14 Version and stratify the benchmark corpus
Version the current claim and causal suites, define category/difficulty strata, score-compatibility rules, contamination notes, change discipline, and saturation warnings. Benchmark composition changes must not silently redefine historical scores.

After #14, evaluate whether a larger public corpus, offline transcript scanning, or external eval-format export provides the highest research value. Avoid expanding provider count merely to increase matrix size.

## P3 — calibrated semantic expansion

### #13 Calibrated soft semantic diagnostic judges
Only after deterministic robustness, diagnostic stability, assumption/evidence-qualification diagnostics, and corpus discipline are established, define a calibration boundary for model-backed semantic discovery. Judge disagreement and abstention are data, not hard truth. No model judge gains verdict or verification authority.

## Decision gates for future features

A proposed feature should normally satisfy at least one of these before entering P0/P1:

1. exposes a failure mode that current verdict/diagnostic metrics cannot distinguish;
2. improves reproducibility, calibration, uncertainty reporting, or benchmark validity;
3. strengthens the harness-owned authority boundary;
4. is motivated by repeated failures observed in live model runs.

Features that primarily add UI, named reasoning styles, provider breadth, or generic agent orchestration remain deferred unless real consumer/research pressure appears.

## Deferred interfaces

These are intentional non-goals until the native runtime, CLI, and eval contracts mature:

- desktop UI: thin visualization/review client after artifact formats stabilize.
- public embedding API compatibility: after real consumer pressure validates the contract.
- MCP adapter: optional agent integration; never a required correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).

## Implementation constraint

All first-party components remain Rust-only. A future desktop application must use a Rust-capable native UI stack without requiring a JavaScript application runtime. Any future MCP adapter, if justified, is implemented in Rust and remains outside the core correctness boundary.
