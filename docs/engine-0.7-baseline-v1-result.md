# Engine 0.7.0 development baseline v1 — frozen Engine 0.6.1 residual measurement

[日本語](engine-0.7-baseline-v1-result.ja.md) | English

**Status: deterministic development BASELINE PASS, not Engine 0.7.0 semantic acceptance.**

Tracking: [#487](https://github.com/git-ksk/reasoning-harness/issues/487) (baseline), [#486](https://github.com/git-ksk/reasoning-harness/issues/486) (Engine 0.7.0 planning). This report records observations on existing 0.6.1 semantics **before** choosing or implementing new mechanisms.

## Provenance and reproduction

| Identity | Value |
| --- | --- |
| Published Engine baseline | \`engine-v0.6.1\` → \`4abee90f0501f0d8fc09b453a0b45c942fbed69e\` |
| Read-only measurement | \`reasoning-harness-core\` 0.6.1; core source and its manifest have **zero differences** from \`engine-v0.6.1\` |
| Pre-observation fixture freeze | \`engine-0.7-baseline-v1-spec-freeze\` → \`d874bd3a98618a1d61ed3de50e9376d79d07dfa8\` |
| Frozen fixture SHA-256 | \`3bf35129ab786e7b069c6b8805b92c42a9e1b9cd10e3752c0b60f8f7fe66b268\` |
| Inputs | \`fixtures/engine-0.7-baseline-v1/manifest.json\` (29 frozen synthetic **development** cases) |
| Measurement runner | \`crates/reasoning-harness-core/tests/engine_0_7_dev_baseline_v1.rs\` |
| Machine-readable result | \`evaluation/engine-0.7-baseline-v1-result.json\` |
| Parser and integrity verifier | \`scripts/engine-0.7-baseline-v1-report.py\` |

This work **did not** change any Engine semantic source, provider adapter, release coordinate, schema contract, or historical frozen evaluation. The original source-tag diff was checked with \`git diff engine-v0.6.1 -- crates/reasoning-harness-core/src crates/reasoning-harness-core/Cargo.toml\` (empty).

Run from repository root:

\`\`\`bash
shasum -a 256 fixtures/engine-0.7-baseline-v1/manifest.json
cargo test --locked -p reasoning-harness-core --test engine_0_7_dev_baseline_v1 \
  -- --test-threads=1 --nocapture > /tmp/engine-0.7-baseline-v1.log 2>&1
python3 scripts/engine-0.7-baseline-v1-report.py \
  --log /tmp/engine-0.7-baseline-v1.log \
  --output /tmp/engine-0.7-baseline-v1-result.json
\`\`\`

The report generator pins the **literal** frozen SHA-256 and baseline source commit, rejects duplicate/missing observations, and verifies that all predictions and target-specific safety observations match. The separate order-invariance regression permutes evidence/source insertion order. The pre-observation fixture freeze tag must not be moved, rebased, edited, retagged, or re-scored.

## Actual measurement

| Family | Cases | Precommitted observable Engine 0.6.1 behavior |
| --- | ---: | ---: |
| Source-attributed exact quotes / source conflict | 10 | **10/10 matched** |
| Qualified structured evidence / hard receipt policy | 11 | **11/11 matched** |
| Target-local evidence need and reusable evidence | 8 | **8/8 matched** |
| **Total** | **29** | **29/29 matched** |

Separate test: source order / evidence order invariance **PASS**. Both Rust integration tests passed.

The fixture includes **24 baseline-safe/negative-control cases** with no proposed gap, plus five **labeled gap hypotheses**, deliberately kept apart from safety error counts. All source quotes remained **source-local** (hard-verification receipts created: **0**) and all required qualified structured-fact receipts/withholdings matched their precommitted observations. Model calls **0**, provider attempts **0**, external acquisition **0**, provider failures **0**. No latency/token or live-model quality comparison is claimed.

## Feature gap matrix — exact code boundaries

| Proposed track | Engine 0.6.1 source evidence | Observed outcome | Finding |
| --- | --- | --- | --- |
| **#488 origin lineage** | \`source_attribution.rs\`: \`SourceAttributionBinding\` stores source ID/URL, retrieval time and source version; \`finalize_source_attributed_answer\` preserves citations | \`src-02\`, \`src-03\`, \`src-10\` expose two source citations, while fixture-only oracle says shared or unknown origin. No oracle origin metadata is passed into the Engine. \`src-01\` is an independent-origins positive control. | **Representation not supported**; **not** a demonstrated false independent-verification promotion. Citations are **not** independent-source votes. Implementation is conditional on an actual product demand for corroboration. |
| **#489 version supersession** | \`types.rs\`: \`EvidenceMetadata.temporal\` and \`EvidenceRequirement.as_of\`; \`evidence_qualification.rs\`: stale/future guards; \`source_attribution.rs\`: retrieval timestamp and source version | \`src-06\` keeps different versioned source statements in Conflict; \`src-07\` correctly keeps disagreement where only later retrieval is known. \`qual-03/04/05\` correctly respect explicit effective intervals. | **No intrinsic source-supersession contract**, but **no correctness defect established**. Same oracle origin and different version strings alone **do not prove** a trusted parent/child revision relationship. Defer until explicit verified lineage is available. |
| **#490 compatible paraphrases** | \`source_attribution.rs\`: \`refresh_conflict_states\` classifies distinct \`statement.trim()\` values for the same target as conflict; \`finalize_source_attributed_answer\` retains separate citations | \`src-04\` (pre-labeled human semantic equivalence) becomes source-local \`Conflict\` with **2 citations**. \`src-05\` genuinely opposed wording also remains \`Conflict\` with **2 citations**. | **Observed conservative false-conflict marker in this exact-quote development scenario** (1/1). **No fabricated truth or unsafe hard verdict.** Highest-priority targeted product-utility follow-up. |
| **#491 target answerability** | \`evidence_need.rs\`: typed floor/reuse dispositions; \`semantic_sufficiency.rs\` + \`answer_safety.rs\`: advisory requirement sufficiency and answer safety | \`need-01..08\` behave as predicted, including stale/ambiguous evidence reacquisition, trusted-verification floor and sufficient local context. | **Existing safe behavior demonstrated**; no complete-task end-to-end required-information defect measured here. **Defer** new answerability control-plane implementation until specific complete-task counterexample is independently documented. |
| Shared structured-fact qualification | \`evidence_qualification.rs\`, \`verification.rs\` | \`qual-01..11\` cover conflict withholding, as-of transitions, unknown provenance/time, scope mismatch, authority floors, wrong-key controls | **Already implemented**, do not rebuild. |

**Critical distinction:** \`src-04\` is a conservatively displayed discrepancy, not an *unsafe factual contradiction verdict*; \`src-02\` does not imply that the Engine has ever counted two citations as two independent proofs. \`src-06\` does not prove which version supersedes the other. A release-blocking finding would require an actual undesired claim/decision in a supported end-to-end path, not simply a missing optional field.

## Precommitted candidate gates — set after baseline, before independent holdout

These are **development-gate decisions**, not 0.7.0 success claims. They must be retained or explicitly revised **before** any independent holdout is authored; no adjustment after holdout observation.

1. **Safety floor (all adopted candidates):** 0 unsupported \`Known\`/\`Supported\` promotions; 0 missing/invented target, source, origin or citation bindings; 0 stale/scope/authority bypasses; 0 erased actual conflicts; 0 model-authority or session-replay side effects. Unchanged 0.6.1 controls: **24/24** behaviorally safe, including source opposition and trusted-verification withholding. Deviations must be separately justified as a *safe* versioned semantic contract, never silently accepted.
2. **#490 first candidate:** reduce demonstrably avoidable conflict marking on \`src-04\` from **1/1 to 0/1**, preserve \`src-05\` genuine conflicting-source markers at **1/1**, retain **2/2** citations in each, and create **0** hard truth receipts. Fresh independent held-out content must show at least **one net useful source-grounded answer improvement** per required model arm (Mistral, Google, Groq), without new hard safety violations or negative-control regressions.
3. **#488 conditional:** only after a concrete user-facing corroboration need is reproduced; a future provenance contract must distinguish verified distinct origins, shared originals and unknown origin **4/4** on \`src-01/02/03/10\`, with **0** fabricated independent-origin credits and **0** extra model calls to *certify* provenance.
4. **#489 conditional:** require a newly authored, separately frozen development case with **explicit verified** revision/supersession ancestry and as-of policy before changing source behavior. Maintain **0** newest-wins promotions absent that provenance and add **0** model calls to decide hard source-version authority. The current \`src-06\` does **not** meet that proof prerequisite.
5. **#491 conditional:** require a measured **complete-task** required-information/partial-answer defect, not a failure inferred from 8 successful evidence-need controls. Preserve **8/8** existing disposition guards and enforce 0 budget bypasses, external replay calls or silent provider switching.
6. **Efficiency:** deterministic provenance/version work adds **0** model calls. If semantic reconciliation is adopted, allow at most **one incremental advisory semantic call per exact target** against a paired 0.6.1 development run, capped at **two provider attempts per such new call**; any bounded acquisition cannot exceed existing per-run budgets, and replay performs **0** calls. Measure absolute token/latency/provider attempts on each matching development arm before freezing an independent holdout; do not fabricate an LLM latency or dollar baseline from this zero-model-call suite.
7. **Independent promotion:** freeze code/runner/scoring, then author new disjoint holdout entities, passages and exact quote windows. Require safety **0** failures across all deterministic cases and each Mistral/Google/Groq arm, no new negative-control losses, and **at least one net improvement** on newly authored independent examples for **each adopted mechanism and required model arm**, compared against a frozen identical-input 0.6.1 baseline. Provider failure/quotas remain operational and cannot be scored as epistemic unknown. Historical first-observation FAIL stays immutable.

## Recommended next engineering work

- **Begin #490's narrow *development candidate*** on semantically equivalent source-local paraphrases, with explicit negative controls for genuine negation, wrong target, scope and time. Do not provide semantic model output with authority to approve exact source evidence, dismiss source conflicts, or publish hard truth. A failure/uncertainty should remain a separately attributed conflict or conservative withholding.
- **Keep #488 in design/reproduction** until real corroboration requirements require stronger data semantics. Only trusted provenance providers may assert origin independence.
- **Defer #489** pending independently trusted supersession data, and **defer #491** until an actual target-required-information completeness miss is measured end-to-end.

No Engine 0.7.0 version bump, tag or CLI-adoption PR follows from this baseline. The source-only release is still subject to [#492](https://github.com/git-ksk/reasoning-harness/issues/492).
