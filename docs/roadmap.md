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

Live semantic discovery remains soft even when calibration metrics are strong. #46 documents both the v3 holdout-v2 portability matrix and the independent v4/holdout-v3 successor test rather than ranking models. The v4 matrix failed its predeclared adoption gate with zero conformant and zero usable-with-limitations models: simplification weakened uncertainty behavior across Mistral and Google families, while the stricter discriminated schema improved Ministral 14B protocol completion without producing semantic portability and Nemotron remained protocol-incomplete/finding-collapsed. #55 therefore restores the exact previously characterized `soft-semantic-v3` runtime baseline while preserving v4 and holdout-v3 as immutable research history. Hard authority remains deterministic/trusted-verifier owned. See [cross-model semantic judge conformance](semantic-judge-conformance.md).

### #59 Next semantic research — representation robustness before another successor

The #57 calibration-only follow-up isolated the strict discriminated output schema from the v3 semantic wording. The result rejects the assumption that a model-facing schema is semantically neutral: Ministral 14B improved from 84/90 successful calls and 0/5 complete trials under the baseline representation to 90/90 and 5/5 under the strict representation, but the strict arm's ambiguous abstention rate was only 0.286. Ministral 8B remained protocol-complete while its ambiguous abstention rate fell from 0.943 to 0.714 when only the representation changed. Gemini 3.1 Flash-Lite was effectively invariant, while Nemotron remained protocol-incomplete. PR #58 was therefore closed without merge and `soft-semantic-v3` remains the runtime baseline.

The next semantic-judge research sequence is deliberately staged:

#### R1 — format-invariance characterization
- [calibration result #59] Gemini 3.5 Flash-Lite completed the counterbalanced five-trial v3-vs-`nested_result_object` study with 90/90 protocol-complete cases per representation and 2/90 matched format flips; both flips were the same ambiguous causal fixture, nested remained `abstain` across all five seeds, and the flips occurred under opposite execution orders
- [calibration result #59] the 18-fixture single-trial matrix showed protocol robustness is representation-sensitive even when successful pairs are stable: v3 18/18, nested 18/18, compact keys 17/18, tuple 7/18; Mistral full-corpus R1a remains blocked by provider structured-generation errors
- [implemented #59] regression tests prove the v3 baseline request is byte-for-byte unchanged, every R1a variant differs only in `output_format`, malformed representations fail closed, matched operational failures stay out of the semantic flip denominator, and multi-format execution is counterbalanced
- [implemented #59] `format_flip_rate`, format-conditioned semantic/operational metrics, provider enforcement fidelity, and calibration-only corpus guards are recorded without majority-vote truth or model-specific semantic branches

#### R2 — harness-owned semantic finding materialization
- [implemented infrastructure #59] the research arm exposes only model-owned `decision` plus optional `advisory_note`; when decision=`finding`, the harness copies request-known `kind` and `target` exactly, while non-finding decisions never materialize a finding
- [implemented #59] v3 kind-specific decision guidance and request controls are regression-locked while the ownership instructions/schema change intentionally under `materialization-r2-v1`
- [implemented #59] syntax-only normalization fails closed on unknown/authority-like fields or multiple semantic JSON values; advisory-note text is not persisted for research scoring
- [implemented #59] a counterbalanced calibration-only runner reports protocol completion, semantic metrics, matched decision flips, token/latency cost, and operational failure classes; exact-path guards reject holdout or symlink substitution before credentials
- [calibration result #59] causal-triad, 18-fixture single-trial, and five-trial R2 matrices are complete for Gemini 3.5 Flash-Lite and Ministral 8B; both R2 arms reached 90/90 protocol completion in repeated trials, while uncertainty behavior remained provider-dependent

#### R3 — selective abstention from instability
- [implemented #59] provider-neutral stability assessment separates decision disagreement, operational incompleteness, and no-success conditions; no vote count can become truth
- [implemented #59] two calibration-only selective candidates are explicit: disagreement-only and complete-unanimity, both of which may only preserve a unanimous soft decision or conservatively escalate to `abstain`
- [calibration result #59] cross-seed plus information-equivalent R2 representation stability is measured with decision-note, compact-key decision-note, and nested-decision-note surfaces under counterbalanced execution
- [implemented #59] report coverage, precision/recall, ambiguous abstention, risk-fixture count, and abstention escalation so always-abstain behavior cannot pass by construction
- [calibration result #59] R3 cross-representation stability detects two ambiguous Gemini 3.5 fixtures and safely escalates them to abstain, but Ministral 8B remains 18/18 protocol-complete and representation-stable while ambiguous abstention stays 0.5714; consistency alone is therefore insufficient
- [in progress #59] R3b adds optional N-source cross-model unanimity as an orthogonal risk signal for self-consistent errors; model/provider disagreement can only escalate to abstain and never becomes majority-vote truth
- [planned] investigate calibrated/selective-prediction methods only after these simple unanimity signals are characterized

#### R4 — independent successor evaluation
- [blocked] do not create or consume holdout-v4 until one provider-neutral candidate passes the R1-R3 calibration gates without tuning against holdout-v1/v2/v3
- [planned] freeze holdout-v4 before the first provider observation of that candidate
- [planned] assign a new configuration identity for any materially changed semantic/protocol contract
- [planned] require repeated cross-provider measurement with operational completeness separated from semantic denominators
- [planned] preserve `soft-semantic-v3` as the production/research baseline unless the predeclared independent adoption gate is met

This sequence changes the research question from “which schema makes models obey JSON?” to “how much semantic behavior survives representation changes, and how can the harness minimize representation-induced risk without granting the model more authority?”

Research anchors for this phase are evidence, not normative designs:

- Tam et al., [*Let Me Speak Freely? A Study On The Impact Of Format Restrictions On Large Language Model Performance*](https://aclanthology.org/2024.emnlp-industry.91/) (EMNLP Industry 2024): format restrictions can degrade reasoning performance and stricter restrictions can increase the effect.
- Schall and de Melo, [*The Hidden Cost of Structure: How Constrained Decoding Affects Language Model Performance*](https://aclanthology.org/2025.ranlp-1.124/) (RANLP 2025): constrained decoding can move instruction-tuned models away from preferred generations and affect task performance.
- Hamilton and Mimno, [*Lost in Space: Finding the Right Tokens for Structured Output*](https://aclanthology.org/2026.gem-main.18/) (GEM 2026): semantically similar output grammars/tokens can yield materially different downstream performance, especially for smaller models.
- Wang et al., [*SConU: Selective Conformal Uncertainty in Large Language Models*](https://aclanthology.org/2025.acl-long.934/) (ACL 2025): selective/conformal uncertainty is a later-stage candidate for risk-controlled abstention after simpler format/seed stability signals are characterized.

With #13, #27, and #28 complete, the deterministic authority/control-plane roadmap is implemented through durable replay. The next phase should be selected from the representation-robust semantic research above or concrete consumer integration pressure rather than adding generic agent orchestration by default.

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
