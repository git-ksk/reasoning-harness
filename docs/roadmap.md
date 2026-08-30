# Roadmap

## Project direction

Reasoning Harness is not trying to become a general-purpose model runner or a second Inspect/lm-eval. Its core differentiator is provider-neutral, authority-aware control of intermediate reasoning: deterministic structure and harness-owned evidence may create hard findings, while model-backed semantic discovery remains soft and observational until independently verified.

That diagnostic layer is a foundation, not the final product boundary.

The long-term product direction is an **evidence-grounded reasoning runtime** that owns the loop around stochastic candidate generation:

```text
generate
  -> ground / verify / diagnose
  -> resolve missing support or revise refuted reasoning
  -> re-verify under the same authority boundary
  -> finalize only from sufficiently grounded propositions
```

The runtime must also be allowed to stop with `unknown`, a qualified partial answer, or abstention. Improving answerability must never require silently promoting retrieved data, model repairs, or fluent final prose into correctness authority.

See [ADR-0002](adr/0002-grounded-resolution-and-finalization.md).

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

## P0 completed — robustness and diagnostic stability

### #10 Metamorphic reasoning robustness — implemented
- [done] provider-neutral typed transform contract
- [done] six deterministic transform families covering evidence order, independent inference order, stable-ID remapping, irrelevant evidence, causal cause-set order, and causal evidence order
- [done] final-verdict, hard-finding, soft-finding, and typed diagnostic-status invariance reporting
- [done] raw diagnostic-ID/reason delta reporting without treating referential IDs as semantic truth
- [done] dedicated reproducible metamorphic seed fixtures kept outside the 20-case and eight-case correctness denominators

Free-form LLM paraphrase generation remains outside the hard benchmark.

### #11 Repeated-trial diagnostic stability — implemented
- [done] typed diagnostic signal/report contract independent from final correctness
- [done] per-fixture complete-trial finding frequencies and count distributions
- [done] adversarial, candidate-normalization, causal, assumption, and evidence-qualification signal types
- [done] operationally incomplete trials excluded from diagnostic denominators and reported explicitly
- [done] 95% Wilson score intervals with exact denominator and minimum-observation policy
- [done] live CLI JSON exposes `stability.diagnostics` alongside unchanged `stability.correctness`

## P1 — broaden grounded reasoning signal conservatively

### #12 Assumption and unsupported-premise diagnostics — implemented
- [done] harness-owned explicit assumptions are a distinct input contract from hypotheses
- [done] typed premise assessments distinguish supported, explicit input assumption, unsupported, and unbound
- [done] typed unsupported premises are hard process findings relative to supplied context; missing proposition binding remains soft
- [done] repeated premise reuse is deduplicated semantically while preserving all claim/inference references
- [done] candidate-authored `inferred` state is not trusted as support; derived support requires a chain from trusted supported/known claims or explicit input assumptions
- [done] five-case deterministic assumption corpus and separate detection/recognition metrics remain outside final correctness denominators
- [done] assumption findings feed the #11 provider-neutral repeated diagnostic report without gaining verdict authority

### #16 Temporal, scope, and provenance evidence diagnostics — implemented
- [done] harness-owned `EvidenceMetadata` for validity windows, applicability scope, and opaque provenance classes
- [done] one provider-neutral `EvidenceRequirement` per proposition key plus harness-owned authority-rank policy
- [done] hard stale/not-yet-valid/scope-mismatch/scope-expansion/insufficient-authority/conflict findings and soft missing-metadata findings
- [done] qualification-aware structured-fact verification; unqualified or conflicting qualified evidence cannot create a hard receipt
- [done] candidate schema cannot create evidence metadata, requirements, authority policy, or qualification findings
- [done] eight-case deterministic qualification corpus and separate reason-detection metric outside final correctness/causal denominators
- [done] evidence-qualification findings feed the #11 repeated diagnostic report without gaining verdict authority

Open-world retrieval, domain-specific source rankings, and generic RAG orchestration remain out of core scope. This work is now an implemented prerequisite for the future resolution loop because newly acquired evidence must be qualified for time, applicability, and authority before it can safely resolve an unknown.

## P2 — benchmark contract before end-to-end product claims

### #14 Version and stratify the benchmark corpus
Version the current claim, causal, assumption, and evidence-qualification suites; define category/difficulty strata, score-compatibility rules, contamination notes, change discipline, and saturation warnings. Benchmark composition changes must not silently redefine historical scores.

The corpus should also establish stable baselines for future resolution-loop research: diagnose-only, one-shot generation, and bounded-resolution variants must be comparable without changing denominators underneath the result.

## P3 — grounded resolution and finalization runtime

This is the main step from research harness toward a general product runtime. Diagnostics become control signals for a bounded recovery loop rather than only observations.

### Bounded resolution loop

Add a provider-neutral runtime contract that can turn unresolved verified state into typed requests for additional evidence, deterministic verification, candidate revision, or explicit human review.

Required properties:

- resolution requests identify missing support without inventing the missing answer;
- resolver output is acquired data, not trusted authority by default;
- evidence and verifier results re-enter through existing harness-owned authority boundaries;
- regenerated/repaired candidates are fully untrusted and re-run through normalization, validation, verification, diagnostics, and policy;
- attempt, token, time, and resolver-class budgets are explicit;
- exhaustion preserves `unknown`/abstain instead of forcing an answer;
- domain-specific web/RAG/tool logic remains outside core behind adapters.

### Grounded finalization

Make final answer construction a first-class correctness boundary rather than a presentation afterthought.

Required properties:

- a finalizer consumes verified `ReasoningArtifact` state;
- a model may render prose but cannot upgrade epistemic state;
- factual final-answer claims must be covered by supported artifact propositions or explicitly represented as uncertainty according to policy;
- newly introduced factual claims are routed back through the reasoning/verification loop;
- finalization can emit grounded answer, qualified partial answer, or abstention;
- `reason explain` can later reuse the same renderer/coverage primitives without becoming a second correctness implementation.

### Resolution research metrics

Report separately from ordinary correctness and diagnostic stability:

- initially-unknown case recovery rate;
- unsafe final answer rate;
- final factual-claim coverage;
- resolution attempts to convergence/exhaustion;
- added token/latency/tool cost;
- supported/refuted/exhausted terminal distribution;
- regression against direct-generation and diagnose-only baselines.

The primary success criterion is **more grounded answerable cases without increasing unsafe final answers**.

## P4 — calibrated semantic expansion

### #13 Calibrated soft semantic diagnostic judges
Only after deterministic robustness, diagnostic stability, assumption/evidence-qualification diagnostics, corpus discipline, and the grounded runtime boundary are established, define a calibration boundary for model-backed semantic discovery. Judge disagreement and abstention are data, not hard truth. No model judge gains verdict or verification authority.

Semantic judges may eventually help propose resolution targets or identify missing semantic links, but they remain soft inputs to the same runtime rather than becoming the runtime's source of truth.

## Decision gates for future features

A proposed feature should normally satisfy at least one of these before entering a near-term phase:

1. exposes a failure mode that current verdict/diagnostic metrics cannot distinguish;
2. improves reproducibility, calibration, uncertainty reporting, or benchmark validity;
3. strengthens the harness-owned authority boundary;
4. increases grounded answerability without increasing unsafe final output;
5. is motivated by repeated failures observed in live model runs.

Features that primarily add UI, named reasoning styles, provider breadth, or generic agent orchestration remain deferred unless real consumer/research pressure appears.

## Deferred interfaces

These are intentional non-goals until the native runtime, artifact, resolution, and finalization contracts mature:

- desktop UI: thin visualization/review client after artifact formats stabilize.
- public embedding API compatibility: after real consumer pressure validates the runtime contract.
- MCP adapter: optional agent integration; never a required correctness boundary.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).

## Implementation constraint

All first-party components remain Rust-only. A future desktop application must use a Rust-capable native UI stack without requiring a JavaScript application runtime. Any future resolver adapter, MCP adapter, or embedding API must preserve the same core authority boundary rather than owning a competing reasoning loop.
