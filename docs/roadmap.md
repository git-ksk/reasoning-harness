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

## Current product milestone

The current split external preview is **Reason CLI 0.5.3 on Harness Engine 0.5.0**. `v0.4.2` remains the immutable final unified release and historical Engine 0.4.2 baseline. **v0.4.2 — Investigation Utility & Provider Parity (milestone #5 / #260)** is complete and includes #261 deterministic safe acquisition precedence, #262 generic Groq provider parity, structured planner/action hardening through #281 and follow-ups, and the provider/evaluation resilience needed to complete metric-v13 acceptance without changing the v0.4.x authority/finalization boundary.

The release remained evidence-gated through final immutable v36: Mistral paired PASS, Groq generic candidate PASS, Gemini 3.5 Flash-Lite paired PASS, and Gemma 4 31B paired PASS. Gemini supplied strict frozen-row utility improvement while every required candidate row preserved zero operational/generation/correctness-boundary failures. Canonical reruns and post-freeze mutations were zero. See [v36 release acceptance](natural-language-e2e-v36-result.md).

Three independent lines now follow the final unified `v0.4.2` release:

1. **Reason CLI 0.5.x — General-use Productization** (milestone #6 / parent #359) shipped 0.5.0-0.5.2 on **Harness Engine 0.4.2**, then adopted released Engine 0.5.0 in `reason-v0.5.3` through completed #455. Only P1 #375 remains open: Homebrew 0.5.3 acceptance is complete, while WinGet is awaiting external community moderator approval after validation/CLA success. The detailed phase/P0/P1 plan is in [the CLI 0.5.0 productization roadmap](reason-cli-0.5-roadmap.md).
2. **Harness Engine 0.5.0 — Verified Investigation Utility** (milestone #4) is release-complete. In addition to #248/#247/#282/#283, final hardening #445/#446/#450 removed avoidable stochastic dependence in finalization scoring, explicit-fact correction continuity, and admitted exact-fact investigation materialization. `engine-0.5-final-v3-freeze` run `35457038163` passed all six independently required rows and 18/18 fresh cases with zero correctness-boundary violations and zero session external replay. See [Engine 0.5.0 final-v3 acceptance](engine-0.5-final-v3-result.md). The Engine source release is `engine-v0.5.0`; `reason-v0.5.3` now distributes it, while already-published Reason CLI 0.5.2 remains immutable on Engine 0.4.2.

3. **Harness Engine 0.6.0 candidate — Target-local Evidence Semantics** is the next measured semantic/correctness research line, not yet a release coordinate. It starts from production gaps #461/#462/#463 without rewriting released Engine 0.5.0: #461 owns target-local evidence-need routing before acquisition, #462 owns evidence-target semantic relevance after acquisition, and #463 owns source-attributed qualified prose after relevant material is admitted. The intended implementation/evaluation order is #461 -> #462 -> #463 unless dependency evidence requires otherwise. #461 froze candidate semantics after calibration v3 (run `35957170730`) and then passed the separately authored independent holdout v1 (run `35965160995`) on both Mistral and Google with 26/26 operational completion, 26/26 exact materialized mode/acquisition, zero correctness violations, and zero utility misses. It is independently accepted for the Engine 0.6 line. #462 is active on `feat/462-evidence-target-relevance`: the provider-neutral relevance contract, binding proposal v2, target-first materialization policy v3, and strict Harness-owned identity floor are implemented. Frozen v4 run `35998574508` passed Mistral 26/26 but failed overall because Google completed 23/26 with three assessment timeouts and two safe-ambiguity utility misses. The bounded Google diagnostic run `36000374933` then completed all five unresolved cases on `gemini-3.5-flash-lite` with exact materialized disposition, so 3.5 remains canonical rather than switching to 3.1. Frozen v5 run 36008648993 passed Mistral 26/26 but failed overall because Google was operational on only 11/26 cases: 14 assessment_timeout failures plus one explicit HTTP 503 high-demand failure after four provider attempts. All 11 completed Google cases materialized exactly with zero correctness or utility misses, so the evidence points to provider-capacity instability rather than a semantic regression. v5 remains immutable failed evidence and is not rerun. The six-case Google recovery smoke run 36018360038 was operational on only 4/6 cases: 21_unknown_rename and 26_url_only_identity again reached the 60,000 ms assessment timeout. All four completed cases materialized exactly with zero semantic misses. Provider recovery is therefore not established; skip the proposed 19-case recovery diagnostic v2 and harden operations before any canonical v6: bounded jitter, run/client retry or overload budgeting, cooldown/circuit behavior, tail-latency telemetry, and diagnostic gating whose visible conclusion matches operational acceptance. Keep the 60-second deadline, semantic contract, and expected labels unchanged for now. Frozen v6 run `36022978827` confirmed the hardening behavior but failed canonical acceptance: Mistral was 26/26 PASS with semantic gates at zero and p50/p95/max latency 554/863/944 ms; Google completed case 01 in 47,242 ms, then cases 02 and 03 each hit the 60,001 ms assessment timeout, causing the two-consecutive-failure circuit to open and suppress the remaining 23 requests. Completed Google semantics remained exact with zero correctness/utility misses. v6 is therefore immutable FAIL, while bounded jitter, fail-fast load shedding, attempt-telemetry completeness, tail-latency reporting, and visibly red operational gating are retained. For v7, the required-provider gate is precommitted before live observation to Mistral `ministral-8b-latest` plus Groq `openai/gpt-oss-120b`, based on provider evidence independent of #462 semantic outcomes: GPT-OSS 120B passed the earlier Engine 0.5 final-v2/final-v3 required/reference rows and remains a current Groq Production Model with JSON Schema support. Google 3.5 is removed from the required operational gate, not reclassified as a semantic failure; v4-v6 remain immutable Google operational/replication evidence and any future Google requalification requires a separately frozen study. Fresh v7 keeps all 26 cases, expected labels, relevance semantics, 60-second case deadline, and two-consecutive-operational-failure circuit unchanged. Frozen Google attempt-telemetry diagnostic v1 run `36026307543` subsequently completed the same early three Google request shapes 3/3 on first attempts with HTTP 200 only, no retry/cancellation, no 429/`RESOURCE_EXHAUSTED`, and no 503/`UNAVAILABLE`; headers still took roughly 28.4-35.9 seconds. This does not retroactively prove v6 had no hidden transient quota event, but together with v5 explicit 503 high-demand evidence it favors intermittent serving-tail/capacity variability over quota exhaustion, and the three-case probe is not enough to requalify Google as a required 26-case canonical provider. Frozen v7 run `36026148264` is also immutable FAIL/incomplete and is not rerun: Mistral completed 26/26 operationally but `21_unknown_rename` materialized `irrelevant` instead of expected `ambiguous`, producing one utility miss with no unsafe relevance admission; Groq completed only the first three cases (all HTTP 200, one attempt, exact materialization) before the workflow was cancelled and is therefore non-scorable. Before any fresh successor, treat that case-21 miss as a general identity-uncertainty / advisory-model-authority design problem and use independently authored identity-ambiguity probes rather than tuning a seed or prompt to this calibration case. Fresh identity ambiguity diagnostic v1 run `36029430165` then completed 36/36 observations on both Mistral and Groq. Mistral primary binding falsely emitted `different` on 11 expected-unresolved observations and scored baseline disposition 25/36, while the one-sided distinctness candidate had zero false confirmations, zero misses on explicit-different controls, and gated disposition 36/36. Groq had one primary false `different`, baseline 35/36, and the same candidate reached 36/36. This reproduces the v7 case-21 failure as a structural open-world identity risk rather than a seed-only accident and supports designing a fresh successor that requires affirmative distinctness confirmation before negative target identity can force `irrelevant`. Before choosing the next required-provider set, Google `gemini-3.5-flash-lite` is separately remeasured in a frozen 26-case full requalification v1 as a non-canonical, non-gating provider study. All 26 fixtures are explicitly selected while preserving the 60-second deadline, 6-second pacing, two-consecutive-operational-failure circuit, and attempt telemetry; only 26/26 operational completion determines provider requalification. Frozen full requalification v1 run `36078211994` then passed operationally at 26/26: every call completed on attempt 1 with HTTP 200, zero 429/503/retry/cancellation events, and p50/p95/max latency of 695/843/950 ms. Google is therefore operationally requalified, but v4-v6 historical instability remains relevant. For the next semantic successor, Mistral + Groq remain required arms while Google is included as a full 26-case non-gating replication arm; restoring Google to required status is a separate decision after the new successor semantics are independently demonstrated on Google and operational stability persists. The next fresh semantic successor is calibration v8 with materialization v4: primary `target=different` alone no longer forces `irrelevant`; a one-sided confirmation must establish either `confirmed_distinct_entity` or `confirmed_target_absent`, otherwise the Harness abstains to `ambiguous`. v8 expands the historical 26 cases with six fresh cases for a 32-case calibration, keeps Mistral + Groq required, and freezes Google as a full 32-case non-gating replication arm. Promotion to engine-v0.6.0 requires fresh calibration plus a separately frozen independent holdout with zero unsafe skipped acquisition, authority laundering, wrong-target relevance admission, truth promotion, renderer-only unsupported factual exposure, source-binding violation, paraphrase/translation strengthening, and replayed external side effects. Utility is scored separately so an always-external-required policy cannot pass merely by being conservative. Reason CLI 0.5.x distribution work (#375) remains a separate track; a CLI adoption coordinate is chosen only after the Engine candidate is accepted.

The v0.4.0 implementation order was **#210 exposed-text binding (P0) -> #211 full-lifecycle deadline -> #212 investigation planner -> #213 resumable sessions -> #214 fresh E2E evaluation**, with #204 negotiated/session `mcp_readonly_v3`, #232 dependency/freeze hygiene, and #233 unique-safe-action utility hardening completed before release. v4 cross-model replication (#208 / PR #209), including the #216 Groq operational extension, remains replication over frozen v4 evidence rather than a tuning surface for this line.

As an explicit measurement boundary, current `unsupported grounded claims = 0` demonstrates safety of structured `factual_claims`; it is not treated as equivalent proof for arbitrary free-form exposed prose. v0.4.0 promotes exposed-text consistency with verified propositions into an explicit correctness gate.

`reason-v0.5.3` is the current published split CLI preview on Harness Engine 0.5.0. `v0.4.2` remains the final unified historical release and immutable Engine 0.4.2 baseline; `engine-v0.5.0` is the current independently released Engine source coordinate; v0.4.1 remains the preceding Investigation Utility Hardening patch, and v0.4.0 remains the Grounded Investigation & Sessions product foundation. The preceding **v0.3.0 — External Evidence & Resolution** (milestone #1 / parent #173) remains complete historical provenance; follow-on work must start from a newly measured product or research gap rather than silently rewriting either released milestone.

The completed patch milestone is **v0.4.1 — Investigation Utility Hardening (milestone #3)**. Issue #249 addresses one post-release avoidable-abstention shape only: after typed `no_result`, continue the same exact target when one remaining explicit read-only fact-key-bound capability is mechanically unique. The patch does not merge targets or alter admission, authority, verification, finalization, answer safety, or frozen v1-v9 measurement semantics. The later #247 evaluator-contract work is complete via frozen v11, and #248 finalization/grounding is complete on the Engine 0.5.0 line; neither rewrites the historical v0.4.1 observations.

v0.3.0 does not add another reasoning mechanism by default. It connects the already-implemented bounded control loop to real external acquisition and trusted-verifier adapters through the existing `ResolutionResolver -> EvidenceAdmissionPolicy / TrustedResolutionVerifier -> re-verification` boundary.

Execution order:

1. #174 external resolver adapter and supported CLI/config wiring — **implemented** with `external_command_v1`;
2. #175 provenance/freshness/scope/authority admission hardening — **implemented** with `external_evidence_admission_v1`, exact-source allowlisting, normalized acquisition metadata, typed admission rejection, and mandatory ordinary re-verification;
3. #178 external-resolution budgets, telemetry, secret handling, and typed operational failures — **implemented** with typed operational terminals, call/latency/cost telemetry, stable hashed config identities, process timeout, and bounded response size;
4. #176 read-only MCP resolver adapter — **implemented** with `mcp_readonly_v1`, explicit server/tool allowlisting, read-only acquisition-only config, MCP 2026-07-28 stdio calls, typed tool failure, and ordinary admission/re-verification;
5. #177 reference trusted verifier/oracle integration — **implemented** with `trusted_command_verifier_v1`, Harness-constructed exact receipts, qualification-preserving evidence binding, and typed operational failure;
6. #179 non-frozen open-world product dogfood and v0.3.0 acceptance — **implemented/passed** with `external-resolution-acceptance-v1`; deterministic CI keeps unsupported grounded claims and missed target insufficiency at zero, plus a recorded live AWS public-feed recovery;
7. #180 optional full-runtime MCP product surface — **implemented** with Rust-only `reason-mcp`, MCP 2026-07-28 stateless discovery, closed native-operation schemas, and exact native product-output pass-through; still non-blocking for v0.3.0.

The release gate requires at least one safe real external-evidence recovery while preserving unsupported grounded claims = `0` and missed target insufficiency = `0` on the declared acceptance set. External acquisition success and hard verification success are separate observations. Frozen Stage-C/RSD2 and other historical holdouts remain immutable and are not used for product tuning.

New reasoning research still starts only from a newly measured gap and receives a fresh research/evaluation identity. v0.3.0 is therefore a product/distribution milestone, not a semantic-generation bump.

## Completed product line through v0.3.0

Reasoning Harness separates the product/evaluation roadmap from the archived research chronology. Short research labels are retained only for provenance; see [Terminology and naming](terminology.md).

### Product

1. **Bounded resolver target closure (#159):** implemented in successor candidate `79ec3b44971c32f9a8847d8173672675947c7288`; exact Harness-owned unresolved targets are prioritized through the existing bounded acquisition/admission/re-verification boundary without model-owned authority.
2. **Renderer downgrade recovery (#160):** implemented in successor candidate `a020b5925497ff3fdf200a9622270fa1889a6aa1`; exact requested authorized targets may recover from renderer-only `uncertain` downgrade without treating renderer output as authority.
3. **Dependency-aware target-local recovery (#164):** implemented in successor candidate `993874fa0051d06a02c8db8f7a220a2ac7773c17`; global `Reject` is preserved and exact directly verified targets receive target-only qualified exposure only under strict typed structural isolation from rejected non-target state.
4. **Provider reliability / resumable evaluation (#126):** implemented without a semantic identity change: bounded provider-specific retries and actual attempt telemetry remain operational; product dogfood v10 adds exact-identity completed-case checkpoint/resume and preserves interrupted provider/protocol failures separately from semantic evidence.
5. **External CLI hardening (#90), model-specific UX (#139), and v1.0 readiness:** closeout complete on current main. Four-platform process compatibility, deterministic CI, current live runtime smoke, and two-class real-workload acceptance are green; Ministral 8B Harness target coverage is 1.00 with zero unsupported grounded claims/missed target insufficiency. The readiness gate is complete, while an actual v1.0 tag/release remains a separate explicit decision.
6. **Trusted-context entity identity adoption (#197):** after the independent #193/#195/#196 research line, candidate v12 `d1db067e6efe6033656b8e7c3315a9fe322c015d` recorded 16/16 on a new one-shot holdout with zero safety/infrastructure violations. #197 moves the unchanged semantics into materialized stable source plus deterministic-equivalence CI. A main merge remains a separate reviewed decision.

### Evaluation

1. **Closed current generation (#147):** preserve the historical six-case smoke set, frozen 24-case development matrix, five-seed Stage-B replication, and separately frozen 16-case Stage-C holdout as immutable evidence.
2. **Stage-C result:** Ministral 8B, Mistral Small, Gemma 4 31B, and Gemini 3.1 Flash-Lite each reached target coverage `1.00`; Ministral 14B reproduced `0.875` with one conservative `artifact_blocked_by_non_target_claims` miss. All completed arms retained unsupported grounded claims = `0` and missed target insufficiency = `0`.
3. **Successor evaluation:** #159 began the successor line at `79ec3b44971c32f9a8847d8173672675947c7288`; #160 advanced it to `a020b5925497ff3fdf200a9622270fa1889a6aa1`; #164 advances it to `993874fa0051d06a02c8db8f7a220a2ac7773c17`. The observed Stage-C holdout is not a calibration/tuning surface. After this successor behavior is frozen, use fresh development/calibration evidence and a newly authored independent holdout before adoption.
4. **Operational completeness:** #126 keeps provider 429/5xx/quota/protocol failures separate from semantic scores while adding bounded retry/attempt telemetry and exact-identity product-dogfood checkpoint/resume. Historical outcomes and the semantic gate remain unchanged.

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

## Historical research phase v0.2 — adversarial reasoning passes (not CLI v0.2.0)
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

### #252/#254 v0.4.1 successor measurement

- [observed #252] frozen v10 canonical Mistral run `34125135760` completed 11/11 with correctness-boundary violations `0` and operational failures `0`, but measurement validity failed.
- [censored #252] the dedicated #249 lane recalled the exact target but executed no cache action, so typed `no_result` was never reached; v10 remains inconclusive for the post-trigger #249 effect and is not rerun or tuned.
- [completed #254] frozen v11 canonical Mistral run `34129798774` completed 13/13 with correctness/operations/measurement/report gates passing. Trigger reachability was 1/3. The single trigger-exposed case was conditionally conformant 1/1 (`cache no_result -> registry`, Harness follow-up telemetry 1) and produced verification progress, while the other two cases were planner trigger misses and follow-up target grounding remained 0/3.
- [routed] planner/action-selection residual is assigned to v0.4.2 #261 under a narrow deterministic read-only selection invariant; downstream grounding/finalization remains #248 in Harness Engine 0.5.0. Frozen v9/v10/v11 and #256 observations are not rerun or tuned.
- [provider parity] v0.4.2 #262 exposes the existing GroqAdapter through the generic natural-language `reason` surface; #263 must preflight exact provider support before any live canonical launch so unsupported combinations cannot become 13 process failures again.
- [v18 product residual] #281 keeps the patch-line fix narrow: encode acquire/stop shape requirements structurally in the model-facing schema after v18 observed repeated missing-`capability_id` proposals. Runtime validation stays authoritative/fail-closed; no missing ID is inferred or repaired.
- [routed to Harness Engine 0.5.0] #282 owns distributional planner reliability (`pass^k`/repeated-trial characterization) and #283 owns the broader question of moving mechanically safe action materialization into deterministic Harness control flow. Neither changes the frozen v0.4.2 ruler or retroactively rescues v18.

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
- MCP full-runtime product surface (#180): **implemented** as optional `reason-mcp` agent integration over selected native operations; never a correctness boundary. Read-only MCP acquisition remains separately implemented in #176 as `mcp_readonly_v1`.

See [ADR-0001](adr/0001-interface-and-packaging-boundaries.md).

## Implementation constraint

All first-party components remain Rust-only. A future desktop application must use a Rust-capable native UI stack without requiring a JavaScript application runtime. Any future resolver adapter, MCP adapter, or embedding API must preserve the same core authority boundary rather than owning a competing reasoning loop.

### Engine 0.6 #462 v9 successor note

Frozen v8 run `36090688415` remains immutable FAIL. Its misses established two successor requirements: negative confirmation must cover both `different` and `unresolved` target proposals, and every `exact/exact` path to `relevant` must pass an independent positive target-local confirmation. Calibration v9 uses materialization v5, strict enum-only Text confirmation with no fuzzy/JSON repair, one bounded confirmation call inside the unchanged shared 60-second case deadline, and 47 cases (32 regression plus 15 independently authored fresh identity/ownership cases). Mistral + Groq remain required and Google remains full non-gating replication. Confirmation subtype accuracy is diagnostic; final materialized disposition and false safe/positive confirmation counts are gated. Independent holdout authoring remains blocked until the first/only frozen v9 canonical observation passes.

### Engine 0.6 #462 v10 successor note

Frozen v9 run `36106913331` is immutable FAIL and is not rerun or rescored. Mistral completed 47/47 with all hard safety counts at zero but materialized only 38/47 exactly, with 9 utility misses and 2 relevant cases left ambiguous. Groq aborted after two positive-confirmation `protocol` failures caused by the 24-token raw-Text contract returning no model text with `finish_reason=length`; Google replication aborted after two HTTP 400 failures because the derived confirmation seed exceeded the provider's signed-32-bit range. v10 therefore keeps the safety boundary but replaces explanation-subtype Text confirmation with candidate-local action-safety decisions (`safe_to_reject|abstain`, `safe_to_accept|abstain`) over JsonSchema with one bounded JSON-object transport fallback. Google seed normalization moves into the provider adapter. Materialization v6 keeps strict Harness-owned identity floors and the independent positive boundary. Calibration v10 contains 56 cases (47 regression plus 9 fresh action-safety cases), with Mistral + Groq required and Google full non-gating replication. Holdout authoring remains blocked until first/only frozen v10 canonical PASS.

### Engine 0.6 #462 v11 successor note

Frozen v10 run `36135286772` is immutable FAIL. v10 fixed all operational transport/provider failures (all three arms 56/56, provider failures 0) but exposed a semantic architecture issue: a primary binding could bypass safety review, and a secondary `safe_to_reject|safe_to_accept` action could directly authorize a hard disposition. v11 therefore moves to a two-key boundary: primary binding remains advisory, an independent local-qualification guard reports support/risk facts rather than actions, and hard Relevant/Irrelevant requires agreement plus Harness-owned anchor/risk rules. Exact-target relation mismatch is no longer an unconditional rejection. Holdout remains blocked until a fresh canonical successor passes.

### Engine 0.6 #462 v12 successor note

Frozen v11 run 36142920405 is immutable FAIL and must not be rerun or rescored. Mistral completed 65/65 with zero provider failures, zero qualification risk misses, zero wrong-target relevance retention, and zero false relevance rejections, but the v11 guard contract over-produced open-world risk: local qualification exact 0/65, spurious risk blocks 43, materialized exact 22/65, 19 expected Relevant cases left Ambiguous, and utility misses 43. Groq was separately operationally incomplete at 22/65 after repeated structured-generation capability failures and two timeouts opened the two-consecutive-failure circuit. Google non-gating replication observed the same conservative-overblocking direction with risk misses 0, spurious risk blocks 25, materialized exact 37/65, and utility misses 26, plus two assessment timeouts.

v12 preserves the two-key architecture and materialization v7. Local qualification v2 redefines risk as a concrete local ambiguity signal rather than proof that every hypothetical external risk is impossible; unresolved is reserved for a specific risk-relevant cue that local material cannot resolve. Harness-owned canonical names and aliases are authoritative local identity anchors. The calibration runner also changes its single bounded structured-output fallback from provider JSON-object mode to strict raw-JSON Text with whole-body typed parsing, preserving the two-call stage ceiling and prohibiting extraction, repair, or semantic retry. v12 contains 73 cases: all 65 v11 regressions plus 8 fresh paired no-risk/concrete-risk cases. Mistral + Groq remain required, Google remains full non-gating replication, and holdout authoring remains blocked until first/only frozen v12 canonical PASS.

### Engine 0.6 #462 v13 successor note

Frozen v12 run 36148322898 is immutable FAIL and is not rerun, rescored, relabelled, or retagged. Mistral completed 73/73 with zero provider failures and zero missed qualification risk but materialized 55/73 exactly with 18 utility misses; 17 misses were expected Irrelevant -> Ambiguous, and 16/18 misses disagreed with the frozen relation-binding expectation. A static post-observation audit also found annotation-protocol contradictions in frozen v12 explicit-local-absence labels. Groq separately aborted at 3/73 after the JsonSchema -> strict-Text fallback sequence consumed capacity and hit retry-after values longer than the shared 60-second case deadline. Google replication was incomplete at 37/73.

v13 precommits atomic target/relation semantics, binary observable blocking cues, materialization v8 with the same fail-closed Harness authority boundary, a pre-observation label-review protocol, and provider-specific Groq strict-JSON Text primary transport. The 81-case corpus is 73 successor re-annotations plus 8 fresh orthogonality/cue controls; production motivating content remains excluded. Research on decompositional NLI annotation, proposition-level segmentation, unnecessary selective abstention, separated self-assessment, and structured-output variability is used as supporting evidence, not as a substitute for observed calibration results. Mistral + Groq remain required, Google remains full non-gating replication, and holdout authoring remains blocked until the first/only frozen v13 canonical PASS.

### Engine 0.6 #462 v14 successor note

Frozen v13 run `36154375809` is immutable FAIL. Mistral and Google both completed 81/81, but materialized only 61/81 and 65/81 respectively; Mistral also exposed a wrong-target/false Relevant regression and both providers missed required blocking cues. Required Groq stopped at 3/81 because the strict-Text local qualification response was truncated at the 192-token completion budget. The separate noncanonical Groq postmortem `36156756468` is diagnostic only and must complete before v14 freeze.

The calibration line stops growing here. `evidence-relevance-fixed-core-v1` contains exactly 48 cases selected by semantic coverage before any v14 live observation, with `fixed_no_new_cases` policy. v14 removes redundant target/relation support votes from the independent guard, leaving only compact `blocking_reason` plus `explicit_local_absence`, and materialization v9 consumes primary target/relation binding plus that compact guard under the same Harness-owned strict identity floor. Binding proposal v4 explicitly supports policy-authorized semantic equivalents and preserves explicit rename/alias/successor uncertainty as unresolved. A replay over already-observed v13 outputs improved fixed-core materialization from 32/48 to 36/48 on Mistral and 37/48 to 40/48 on Google with no newly regressed previously-correct cases; this is design evidence, not a v14 score. Groq strict-Text transport gets a 512-token floor. Canonical arms continue through all 48 cases after operational failures for diagnostics, while any provider failure still fails acceptance. Holdout remains blocked until first/only frozen v14 canonical PASS.

### Engine 0.6 #462 v15 successor note

Frozen v14 run 36174639970 is immutable FAIL and is not rerun. Required Mistral completed 48/48 operationally but materialized only 35/48 exactly, dominated by false explicit_local_absence=present on six clear positive cases and primary positive-target under-binding. Required Groq attempted all 48 cases but only 6 succeeded; the remaining 42 were explicit tokens-per-day quota failures at the 200K TPD limit. The v14 quota hardening behaved correctly: those calls failed fast as typed Quota with no transient rate-limit retry and did not become assessment timeouts. Google replication attempted all 48 cases but only 13 succeeded; all 13 successful cases materialized exactly, while the remaining 35 reached the outer 60-second deadline during provider HTTP 429 Retry-After windows. Attempt telemetry shows repeated 20–59 second retry delays, confirming that bounded retry itself works but provider wait/retry time is incorrectly charged against the same semantic case budget.

The v15 successor keeps evidence-relevance-fixed-core-v1 unchanged at 48 cases. It must (1) remove or deterministically constrain model authority over explicit local absence, (2) address positive-target under-binding without relaxing the Harness-owned identity floor, (3) separate provider throttle/retry wait accounting from semantic execution while retaining a finite absolute operational deadline and bounded retries, and (4) fail closed at run time when a required provider reaches a daily token quota, immediately latching that provider arm instead of requiring a manual pre-run TPD attestation. Holdout authoring remains blocked until a first/only frozen successor canonical passes.

#### v15 pre-implementation audit constraints

A post-v14 audit adds the following constraints before implementation starts:

- **Do not promote raw Harness-anchor substring matches into positive authority.** `anchor_match()` currently performs normalized substring matching over candidate text, including short aliases. A target mention in comparison/negative text is still an anchor match, and short aliases can collide inside unrelated tokens. If v15 relies more heavily on Harness-owned identity, matching must become boundary-aware and an anchor must remain an identity floor/check, not independent proof that the requested relation belongs to the target.
- **Restore one-sided confirmation before negative identity can force Irrelevant.** v14 case 45 showed that an erroneous primary `target=different` can still collapse an expected Ambiguous case to Irrelevant. A primary negative binding alone must not regain final rejection authority; affirmative local distinctness or explicit local-absence confirmation is required, while uncertainty stays Ambiguous.
- **Do not replace model-authored explicit absence with naive lexical rules.** Candidate text is untrusted and the current signal kinds do not themselves confer trust. Deterministic negative evidence must be grounded in Harness-owned structure or an explicitly bounded confirmation contract, not phrase matching such as `no X information`.
- **Separate three operational budgets, not only two.** Keep a semantic execution budget, a bounded cumulative provider wait/retry budget, and a finite absolute case wall-clock deadline. Cap a single accepted `Retry-After` as well as cumulative retry sleep. Removing provider wait from the semantic budget must not permit multi-minute or unbounded cases.
- **Add run-level overload/quota protection.** A confirmed daily-quota failure should latch the provider arm and suppress further external calls rather than send dozens of guaranteed failures. Repeated correlated 429/5xx events must consume a run-level retry budget/circuit so `--continue-after-operational-failures` cannot become a retry storm. Suppressed cases remain operationally incomplete and cannot pass acceptance.
- **Make cancellation telemetry authoritative.** Outer-deadline cancellation currently leaves runner observations with `provider_attempts=0` even when provider attempt telemetry recorded HTTP 429 attempts. v15 must preserve completed/started attempt counts and expose pacing wait, retry sleep, provider HTTP time, semantic execution time, and absolute case time separately.
- **Prevent calibration overfitting and telemetry leakage.** The fixed 48 cases are now observed calibration data. v15 implementation must not branch on case IDs, synthetic entity names, or exact fixture phrases; structural/property/metamorphic tests may be added without expanding the scored calibration core. Provider error artifacts must also sanitize organization/project/account identifiers, billing URLs, and other unnecessary provider payload before persistence in this public repository.

Groq standard response headers expose RPD request remaining and TPM token remaining, not TPD token remaining, so the canonical does not pretend to infer daily headroom from a tiny probe. No manual TPD attestation is required. Instead, typed daily-quota exhaustion is an authoritative run-time stop signal: the Groq arm latches immediately, remaining guaranteed-failure calls are suppressed, and the required arm is operationally incomplete.

### Engine 0.6 #462 v15 candidate status

v15 semantics are implemented in e760939 and the operational budget/circuit hardening is implemented in 769866f. The fixed 48-case core remains unchanged. The v15 verifier replaces model-authored explicit local absence with one-sided local binding confirmation, materialization v10 restores confirmation before negative target rejection and permits a confirmed positive rescue only under the existing Harness identity floor, and ASCII alias anchoring is boundary-aware. Required-provider retry ownership remains in the adapters.

Operationally, Mistral/Groq/Google now separate active execution from provider pacing/retry wait. The canonical budget is 60s active + 45s cumulative provider wait, with a 30s single-wait cap and 120s absolute case deadline. Typed quota latches the provider arm after one failure; correlated capacity failures latch after two, while suppressed cases remain non-scorable. Public runner failures are sanitized before persistence.

The first/only v15 canonical has no manual Groq TPD headroom start gate. If Groq reports typed daily quota exhaustion during the run, the arm latches immediately and v15 becomes an immutable operational FAIL with no rerun/rescore/relabel/retag; any fresh canonical attempt moves to a successor version. Mistral + Groq remain required; Google remains full non-gating replication.

### Engine 0.6 #462 v15 immutable result / v16 successor

Frozen v15 run `36217988625` at freeze commit `80aba233aa375fa88d4515d68c586f9f0c5f02bb` is immutable FAIL and is not rerun, rescored, relabeled, or retagged. Required Mistral completed 48/48 operationally with complete 96/96 attempt telemetry, but materialized only 33/48 exactly, including one wrong-target Relevant retention (`59_fresh_shared_owner_positive_looking`), three Relevant -> Ambiguous misses, and 14 utility misses. The v15 verifier produced 13 spurious blockers, 14 binding-confirmation misses, and one spurious positive confirmation. Required Groq completed 43 provider-success cases before a typed daily-quota failure on `45_url_only_identity_fresh`; the run-level quota latch then suppressed the final four cases exactly as designed. Groq already had seven disposition misses among successful observations, so the successor is required independently of quota. Google replication hit two early typed rate-limit failures and the correlated-capacity latch suppressed the remaining 46 cases; it is non-scorable and non-gating.

v16 keeps `evidence-relevance-fixed-core-v1` at 48 cases and does not alter v15 labels. The successor removes unresolved-primary positive rescue, replaces the free-form `binding_confirmation` with orthogonal verifier identity-scope / relation-scope / scope-risk fields, strengthens generic primary binding instructions, and derives positive terminal disposition only from full primary/verifier agreement under the Harness identity floor. Negative terminal disposition additionally permits explicit local absence when neither primary axis claims exact support and the verifier independently reports target/relation absence. Operational budgets, adapter-owned retries, telemetry separation, sanitization, one-quota latch, two-capacity-failure latch, and the no-manual-TPD-start-gate policy are retained.

The v16 semantics are implemented at `566a4b5a39ad937d9f43d006550ad7a84436a0f8`. Pre-freeze validation is green: v16 routing 9/9, evidence-relevance runner 22/22, full workspace tests PASS, provider library final row 153 passed / 0 failed / 1 ignored, full workspace Clippy with `-D warnings` PASS, fmt/diff checks PASS, workflow YAML parse PASS, and validate-only reports 48 planned / 0 observed with no abort or provider latch. Detailed immutable v15 results and the v16 design are recorded in `docs/engine-0.6-evidence-relevance-calibration-v15-result.md` and `docs/engine-0.6-evidence-relevance-calibration-v16.md`. Holdout authoring remains blocked until the first/only frozen v16 canonical passes.
