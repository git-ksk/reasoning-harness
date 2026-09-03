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

## Active roadmap

Reasoning Harness now separates the active product/evaluation roadmap from the archived research chronology. Short research labels are retained only for provenance; see [Terminology and naming](terminology.md).

### Product

1. **Bounded resolver target closure (#159):** implemented in successor candidate `79ec3b44971c32f9a8847d8173672675947c7288`; exact Harness-owned unresolved targets are prioritized through the existing bounded acquisition/admission/re-verification boundary without model-owned authority.
2. **Renderer downgrade recovery (#160):** next: recover exact already-authorized requested targets from stochastic uncertainty downgrade while preserving exact proposition identity.
3. **Dependency-aware target-local recovery (#164):** allow exposure under artifact-global `Reject` only when the requested target is independently verified and rejected non-target state is demonstrably irrelevant; otherwise fail closed.
4. **Provider reliability / resumable evaluation (#126):** bounded provider-specific retries, rate-limit telemetry/pacing, and case-level checkpoint/resume without converting operational failure into semantic evidence.
5. **External CLI hardening (#90), model-specific UX (#139), and v1.0 readiness:** keep compatibility and real-workload usability moving after the successor candidate is frozen.

### Evaluation

1. **Closed current generation (#147):** preserve the historical six-case smoke set, frozen 24-case development matrix, five-seed Stage-B replication, and separately frozen 16-case Stage-C holdout as immutable evidence.
2. **Stage-C result:** Ministral 8B, Mistral Small, Gemma 4 31B, and Gemini 3.1 Flash-Lite each reached target coverage `1.00`; Ministral 14B reproduced `0.875` with one conservative `artifact_blocked_by_non_target_claims` miss. All completed arms retained unsupported grounded claims = `0` and missed target insufficiency = `0`.
3. **Successor evaluation:** #159 begins the successor line at candidate `79ec3b44971c32f9a8847d8173672675947c7288`; #160/#164 receive new identities when they change semantic behavior. The observed Stage-C holdout is not a calibration/tuning surface. After successor behavior is frozen, use fresh development/calibration evidence and a newly authored independent holdout before adoption.
4. **Operational completeness:** provider 429/5xx/quota/protocol failures remain separate from semantic scores; #126 may improve retry/resume mechanics without changing the semantic gate or historical outcomes.

### Research

The first semantic-decidability and residual evidence-sufficiency programs are complete. New research starts only from a measured product/research gap and receives a descriptive identity of its own. Historical labels such as `R1`–`R4`, `D1`–`D3`, and `RSD0`–`RSD4` remain in the chronology below because they are issue-scoped provenance, not product versions.

## Historical implementation and research chronology

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
- [calibration result #59] R3b Gemini 3.5 Flash-Lite + Ministral 8B completed 180/180 calls across five seeds; cross-model risk remained confined to four ambiguous fixtures, positive/negative disagreement stayed at zero, and the combined policy held precision/recall and ambiguous abstention at 1.0 with 0.6111 decision coverage
- [planned] investigate calibrated/selective-prediction methods only after these simple unanimity signals are characterized

#### R4 — independent successor evaluation
- [rejected #59] frozen run `33371523453` completed 280/280 calls with precision/recall 1.0, but fixture-collapsed ambiguous abstention was 0.8333 versus required >=0.85 and four of five per-trial values were below required >=0.80
- [rejected #59] labelled polarity stability failed on `v4h-03-contradiction-negative`: Gemini was consistently `no_finding`, Ministral consistently `finding`; the combined policy safely abstained but the frozen source/seed gate was violated
- [frozen diagnostic #59] holdout-v4 is now observed immutable evidence. A post-observation static audit found label/decision-rule conflicts in `v4h-13` and `v4h-20`; they must not be relabelled or used to rescue/re-score the candidate
- [baseline retained] `soft-semantic-v3` remains the runtime baseline and R3b is not adopted as an independently validated successor
- [next research] return to fresh calibration-only design for correlated/self-consistent over-assertion, add a pre-observation fixture-label/spec review gate, and require a newly frozen holdout-v5 for any future adoption attempt


### #73 Decidability/evidence-sufficiency gate — calibration research

Phase naming is issue-scoped: `R1`–`R4` are #59 semantic-successor research stages (`R4` = frozen
independent successor evaluation), while `D1`–`D3` are #73 decidability stages (`D1` = deterministic
contract, `D2` = provider calibration, `D3` = candidate freeze/adoption preparation). These are not
runtime version numbers.

R4 established that cross-model disagreement can expose uncertainty but agreement cannot certify correctness. The next calibration-only phase therefore separates a narrower harness-owned question from the semantic decision: whether explicit typed binding/evidence preconditions permit an assertive soft decision at all.

- [designed #73] deterministic `permit | force_abstain` gate; `permit` is only absence of a known blocker and never correctness evidence
- [designed #73] reuse claim/inference proposition binding, `EvidenceRequirement`, `EvidenceMetadata`, `EvidenceAuthorityPolicy`, and `EvidenceQualificationInspector` rather than asking a model to recreate owned metadata
- [designed #73] deterministic blockers are limited to explicit structural/qualification failures; absence of an evidence requirement and ordinary causal `Unknown` do not automatically force abstention
- [designed #73] composition is monotone: a gate may preserve a base soft decision or force `abstain`, never create/repair an assertive decision or operational failure
- [implemented #73] 14 deterministic calibration-only fixtures form seven control/mutation pairs covering binding, evidence presence, authority, scope, temporal validity, required metadata, and evidence conflict across contradiction/unsupported-premise plus structural counterexample binding; causal-gap remains permit-only until relation-level evidence requirements are typed
- [implemented #73] deterministic tests enforce 100% mutation monotonicity/control preservation, monotone decision composition, invalid-artifact separation, missing-target fail-closed behavior, and the rule that causal targets without explicit evidence requirements are not blocked by default
- [designed #73] D2 keeps `semantic_label` and `assertive_eligibility` as separate pre-observation axes so expected forced abstention cannot be miscounted as a semantic recall failure; eligible precision/recall/coverage and typed-insufficiency abstention are separate denominators
- [implemented #73] D2 v1 manifest has 15 calibration semantic cases across all four diagnostic kinds, 7 paired typed-insufficiency variants across three kinds, and four separate eligible ambiguity controls; causal-gap is deliberately permit-only, and checked-in semantic labels must match the existing calibration source fixtures before credentials are read
- [implemented #73] `reason-decidability-study` performs one unchanged R2 provider observation per semantic case/seed and applies all typed variants afterward; operational failure remains separate and exact-path guards reject non-D2 corpora before provider initialization
- [frozen #73] D2 v1 first-observation plan: full 15-case calibration corpus, Gemini 3.5 Flash-Lite and Ministral 8B reported separately, seeds 6000-6004, five trials, 512 output tokens, and predeclared operational/coverage/precision/recall/typed-insufficiency/stability gates; the workflow exposes no study-shaping inputs
- [calibration result #73] frozen D2 run `33377619803` completed 75/75 calls and 5/5 trials on each of Gemini 3.5 Flash-Lite and Ministral 8B; both retained eligible clear coverage/precision/recall 1.000, escalated all 35/35 typed-insufficiency variants from assertive base decisions to abstain, left zero composed unsafe assertions, and had zero clear-case seed disagreement
- [frozen #73] D3 candidate `semantic-decidability-d3-v1` = `soft-semantic-v3` + `materialization-r2-v1` + `deterministic-explicit-typed-preconditions-v1`, composed only by preserving or forcing abstain; it is not a runtime version
- [frozen #73] observation-free holdout-v5 now contains 24 fresh cases balanced across four diagnostic kinds and positive/negative/ambiguous labels, with 10 clear typed-insufficiency variants, no causal force variants, one inference-binding case, and SHA-256-frozen source/manifest payloads; `v5h05` and `v5h11` were clarified during static label/spec review before any provider observation
- [frozen #73] holdout-v5 execution is fixed to Gemini 3.5 Flash-Lite and Ministral 8B separately, seeds 7000-7004, five trials, 512 output tokens, exact full-corpus execution, and the predeclared D3 adoption gates; the workflow exposes no study-shaping inputs
- [pilot result #73] Ministral 8B completed the frozen holdout-v5 arm with 120/120 calls, 5/5 complete trials, eligible clear coverage/precision/recall 1.000, typed-insufficiency abstention 50/50, base unsafe assertions 50 -> 0, and zero clear-case seed disagreement
- [cross-family replication #73] Google-hosted Gemma 4 31B independently replayed R2, D2, and holdout-v5 without changing fixtures, labels, seeds, thresholds, or semantic contracts; its v5 arm also completed 120/120 with clear coverage/precision/recall 1.000 and unsafe assertions 50 -> 0, and its 120 base decisions matched Ministral 8B exactly
- [negative control #73] NVIDIA Nemotron 3.5 Lightning remains operationally/protocol incompatible with the current R2 materialized-decision contract: the bounded D2 probe succeeded on 7/15 calls and failed 8/15 with repeated forbidden `finding` fields, while the dependent v5 probe timed out after 18/24 attempted fixtures; this is compatibility evidence, not a semantic rejection of D3
- [completed #84] Gemini 3.5 Flash-Lite exact frozen holdout-v5 rerun passed in Actions run `33380880478` attempt 2 after quota reset: 120/120 calls, 5/5 complete trials, clear coverage/precision/recall 1.000, typed-insufficiency abstention 50/50, unsafe assertions 50 -> 0, zero permit-control escalations, zero clear-case seed disagreement, and zero provider/protocol failures; ambiguous abstention was 0.800 with disagreement confined to three ambiguous fixtures outside the frozen gate
- [implemented stabilization #73] D3 has a corpus-independent R2 capability preflight, typed materialization failure telemetry, atomic non-scorable partial checkpoints, frozen runtime/config identity, a provider-neutral baseline/D3 runtime API, and an explicit rollback profile to `soft-semantic-v3`
- [adopted #73] after the stabilization change passed CI, the separate runtime-adoption change switched `DEFAULT_SEMANTIC_RUNTIME_PROFILE` to `semantic-decidability-d3-v1`; `soft-semantic-v3` remains directly selectable as the rollback profile, and frozen D2/v5 semantic contracts/workflow plans remain unchanged
- [implemented #85] add a bounded synthetic live runtime smoke for Mistral/Gemma that validates the compiled D3 default, monotone permit/force-abstain behavior, explicit `soft-semantic-v3` rollback execution, and typed operational failures without reusing observed holdouts as calibration
- [runtime smoke result #85] Actions run `33408032079` passed 4/4 live calls on both Ministral 8B and Gemma 4 31B: both preserved base `finding` under `permit`, both produced `finding -> abstain` under the matched missing-binding D3 case, explicit v3 rollback remained executable and assertive, and no operational failures occurred
- [next research #73] after D3 stabilization/adoption, the first successor hypothesis is residual soft decidability for insufficiency not represented by current typed metadata; selective/conformal abstention is a later calibrated option, and causal relation-level sufficiency waits for explicit typed directional evidence binding
- [constraint #73] holdout-v4 remains immutable diagnostic history; holdout-v5 remains immutable after observation and must not be repaired, relabelled, or reused as calibration data

See [semantic decidability and evidence-sufficiency research](semantic-decidability.md).

This sequence changes the research question from “which schema makes models obey JSON?” to “how much semantic behavior survives representation changes, and how can the harness minimize representation-induced risk without granting the model more authority?”

Research anchors for this phase are evidence, not normative designs:

- Tam et al., [*Let Me Speak Freely? A Study On The Impact Of Format Restrictions On Large Language Model Performance*](https://aclanthology.org/2024.emnlp-industry.91/) (EMNLP Industry 2024): format restrictions can degrade reasoning performance and stricter restrictions can increase the effect.
- Schall and de Melo, [*The Hidden Cost of Structure: How Constrained Decoding Affects Language Model Performance*](https://aclanthology.org/2025.ranlp-1.124/) (RANLP 2025): constrained decoding can move instruction-tuned models away from preferred generations and affect task performance.
- Hamilton and Mimno, [*Lost in Space: Finding the Right Tokens for Structured Output*](https://aclanthology.org/2026.gem-main.18/) (GEM 2026): semantically similar output grammars/tokens can yield materially different downstream performance, especially for smaller models.
- Wang et al., [*SConU: Selective Conformal Uncertainty in Large Language Models*](https://aclanthology.org/2025.acl-long.934/) (ACL 2025): selective/conformal uncertainty is a later-stage candidate for risk-controlled abstention after simpler format/seed stability signals are characterized.

With #13, #27, #28, and the D3 pilot/replication evidence complete, the deterministic authority/control-plane roadmap is implemented through durable replay and the semantic-decidability line has a concrete stabilization candidate. D3 operational hardening and the separate reversible runtime-adoption step are now implemented; new semantic successors should wait for a measured residual gap or concrete consumer pressure rather than adding model breadth or generic agent orchestration by default.

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
