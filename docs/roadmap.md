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

### #14 Version and stratify the benchmark corpus — implemented
- [done] corpus v1 manifest covers 20 claim, 8 causal, 5 assumption, and 8 evidence-qualification cases with stable suite-prefixed IDs
- [done] category/difficulty/scoring/provenance/redistribution/contamination/lifecycle metadata is explicit and validated
- [done] `score_compatibility_id` defines direct score-comparison compatibility instead of inferring it from version strings
- [done] recorded claim eval reports category and difficulty slices alongside the unchanged historical aggregate
- [done] live eval records corpus identity but leaves repeated-trial stratification to future complete-trial-aware reporting
- [done] case add/change/deprecate/supersede discipline, contamination posture, and saturation warning policy are documented
- [done] public manifest coverage and obvious provider/credential coupling are deterministic CI checks

Corpus v1 now establishes the stable base-case identities needed for direct, diagnose-only, and bounded-resolution comparisons without changing denominators underneath recovery metrics.

## P3 — grounded resolution and finalization runtime — implemented

### #22 Bounded grounded resolution and finalization — implemented
- [done] typed provider-neutral requests for proposition, causal, evidence-qualification, revision, and human-review targets
- [done] generic resolver output is acquisition/revision only; trusted evidence metadata crosses `EvidenceAdmissionPolicy`, and trusted receipts use a separate `TrustedResolutionVerifier` boundary
- [done] per-run and per-request attempt/token/time budgets, resolver allowlists, required authority policy, attempt history, and explicit terminal states
- [done] admitted evidence and repaired/regenerated candidates re-enter the ordinary normalization/validation/verification/diagnostic/decision pipeline
- [done] grounded finalization consumes verified artifact state and machine-checks typed factual-claim coverage
- [done] renderer-introduced factual propositions are withheld, converted into new hypotheses, and routed through resolution/verification before grounded output
- [done] nine deterministic resolution variants cover support, refutation, stale/scope/authority mismatch, conflict, no-result, malformed output, and untrusted output
- [done] `reason eval-resolution` compares direct one-shot, diagnose-only, and bounded resolution on stable corpus-v1 base identity
- [done] recovery, unsafe-final-answer, final-claim-coverage, terminal, attempt, token, and elapsed-time metrics remain separate from ordinary correctness and diagnostic stability

The core now owns the bounded control protocol, not domain acquisition. Generic web/RAG/database/MCP/human-review implementations remain external adapters. Live resolution quality is not implied by the deterministic fixture-oracle baseline.

## P3.5 — reasoning control-plane architecture — designed

### #25 Mature harness control patterns — architecture complete
- [done] map execution sandbox to evidence/inference promotion policy rather than a new execution sandbox
- [done] define `ReasoningPolicy` as promotion/escalation policy that never owns truth authority
- [done] adopt durable `ReasoningThread`, typed append-oriented events, checkpoint/resume/fork, and explicit policy-change invalidation
- [done] reuse #22 resolver/admission/verifier boundaries instead of adding a competing evidence-provider abstraction
- [done] define proposition -> evidence -> edge -> artifact -> final-answer validation ladder and dependency invalidation
- [done] preserve repair as untrusted replacement + complete re-verification
- [done] defer skills/subagents and generic workflow orchestration until benchmark evidence justifies them

ADR-0003 control-plane implementation is complete across #27 policy/invalidation and #28 durable-thread replay.

### #27 Composable reasoning policy and dependency invalidation — implemented
- [done] typed global/domain/run `ReasoningPolicyLayer` composition with stable effective policy version identity
- [done] authority thresholds, scope, derived-support capability, and resolver-class permissions compose restrictively; contextual `as_of` changes force requalification
- [done] direct/deserialized policy input is validated fail-closed independently of the composition helper
- [done] policy changes create a new artifact snapshot; historical input is not mutated
- [done] supported/contradicted state requires reconstructable retained receipt authority, while known state must retain qualified direct evidence
- [done] invalidation propagates receipt -> claim -> inference edge -> downstream claim -> finalization
- [done] invalidated edges are removed from the new accepted snapshot and policy-sensitive qualification/assumption findings are recomputed
- [done] soft semantic findings may request evidence/verifier/human escalation but cannot create hard authority
- [done] #22 resolution policy can only be tightened by policy resolver/authority constraints
- [done] four deterministic policy fixtures cover authority, temporal, scope, and dependency invalidation outside existing score denominators

See [reasoning policy and dependency invalidation](reasoning-policy.md).

### #28 Durable reasoning threads and checkpoint replay — implemented
- [done] stable thread, checkpoint, event, candidate, and fork-lineage identities with schema/policy version binding
- [done] append-oriented task, candidate, artifact, soft-finding, resolution-attempt, policy, invalidation, checkpoint, interrupt/resume/fork, and finalization events
- [done] deterministic checkpoint/resume reconstruction of explicit harness-owned state
- [done] interrupted work is frozen and cannot be mistaken for verified/finalized state
- [done] fork creates a new lineage without rewriting source history; finalized source threads remain immutable
- [done] policy-change and invalidation events are replayed through deterministic #27 re-evaluation, preventing serialized authority injection
- [done] active policy is rechecked when accepted artifacts are recorded
- [done] recorded #22 resolution attempts are observations only; replay never re-executes resolver side effects
- [done] abstract `ReasoningThreadStore` boundary with no filesystem/database/cloud backend in core
- [done] credential-free replay/tamper tests and explicit no-hidden-chain-of-thought persistence contract

See [durable reasoning threads and deterministic replay](reasoning-thread.md). Concrete storage products, retention policy, UI/session surfaces, and content-addressed blob stores remain outside core.

## P4 — calibrated semantic expansion

### #13 Calibrated soft semantic diagnostic judges — implemented
- [done] provider-neutral async `SoftDiagnosticJudge` contract with harness/adapter-owned stable judge/model/configuration identity
- [done] typed soft contradiction/counterexample/unsupported-premise/causal-gap request and finding targets
- [done] `finding | no_finding | abstain` output with no API path to receipts, hard findings, epistemic promotion, or verdict authority
- [done] nine-case offline calibration corpus with positive, negative, and ambiguous labels and deliberate disagreement/abstention
- [done] per-judge confusion counts, precision, recall, decision coverage, and abstention metrics
- [done] pairwise categorical agreement plus nominal Krippendorff alpha with abstention treated as missing data
- [done] `reason eval-judges` keeps calibration metrics separate from final correctness, diagnostic stability, and resolution denominators
- [done] required CI remains deterministic and credential-free; recorded identities are synthetic calibration fixtures, not model-quality claims

Live semantic discovery is now permitted as optional/manual research under this boundary, but a judge remains soft even when its calibration metrics are strong. It may suggest a resolution target; hard authority must still come from evidence qualification or an explicit trusted verifier.

With #13, #27, and #28 complete, the deterministic authority/control-plane roadmap is implemented through durable replay. The next phase should be selected from live research or concrete consumer integration pressure rather than adding generic agent orchestration by default.

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
