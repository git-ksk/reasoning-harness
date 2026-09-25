# Engine 0.6 evidence-target relevance calibration v12 — immutable result

Status: FAIL. This result is immutable. Do not rerun, rescore, relabel, or retag v12.

Freeze:
- candidate/tag commit: 9534ae382640c154790abf1cc07c57d1065519b6
- tag: engine-0.6-evidence-relevance-calibration-v12-freeze
- canonical GitHub Actions run: 36148322898
- run attempt: 1
- final-gate conclusion: failure

## Required Mistral arm

Model: ministral-8b-latest.

Operationally complete:
- 73/73 successful provider cases
- provider failures: 0
- qualification invoked: 73/73
- provider-attempt telemetry complete

Safety remained conservative:
- qualification risk misses: 0
- wrong-target relevance retention: 0
- false relevance rejections: 0

Utility/materialization still failed:
- proposal exact: 35/73 (47.95%)
- local qualification exact: 28/73 (38.36%)
- qualification spurious risk blocks: 15
- materialized exact: 55/73 (75.34%)
- expected Relevant left Ambiguous: 1
- utility misses: 18

Case-level inspection showed that 17/18 disposition misses were expected Irrelevant materialized as Ambiguous and 1/18 was expected Relevant materialized as Ambiguous. Among the 18 misses, relation_binding differed from the frozen expectation in 16 cases. The dominant semantic residual is therefore not only residual overblocking: target identity, relation kind, and blocking-risk cues are still insufficiently orthogonal in the model contract.

A separate static specification audit also found frozen-label inconsistencies that must not be repaired in v12. For example, cases 28 and 29 expect explicit_local_absence=absent even though their candidate text explicitly states that no product-specific / target-specific information is present and the v12 prompt defines such local statements as explicit_local_absence=present. v12 scores remain frozen; the successor must establish a clearer annotation protocol before observation.

## Required Groq arm

Model: openai/gpt-oss-120b.

Operationally incomplete and non-scorable:
- planned/completed: 73/3
- successful provider cases: 1
- failed provider cases: 2
- abort: two consecutive operational failures after 03_expanded_alias; 70 cases remained
- provider-attempt telemetry incomplete on both failed cases
- latency p50/p95/max: 60,000 / 60,001 / 60,001 ms

The v12 transport change removed the v11 schema-generation retry loop, but it exposed a different pacing failure. Structured requests can consume provider capacity even when they return HTTP 400 without usage telemetry; the local token pacer records usage only for successful responses. The strict Text fallback then encountered HTTP 429 with retry-after values longer than the shared 60-second case deadline, so the outer assessment timeout expired before the bounded provider retry could recover.

This is an operational transport issue and does not justify weakening the semantic gate.

## Google replication

Model: gemini-3.5-flash-lite. This arm was non-gating.

Operationally incomplete:
- planned/completed: 73/37
- successful provider cases: 35
- failed provider cases: 2
- abort after two consecutive operational failures at 37_fresh_sibling_overlap; 36 cases remained

Observed semantic metrics before abort:
- proposal exact: 23/35 (65.71%)
- local qualification exact: 14/37 (37.84%)
- qualification risk misses: 1
- qualification spurious risk blocks: 5
- materialized exact: 27/35 (77.14%)
- wrong-target relevance retention: 1
- expected Relevant left Ambiguous: 1
- utility misses: 7

Because the arm was incomplete, these values are diagnostic only.

## Final decision

All required top-level gates were false:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification gate: false

v12 is therefore an immutable canonical FAIL. No v12 rerun, rescore, relabel, replacement tag, or post-hoc fixture repair is permitted. Independent holdout authoring remains prohibited.

## Successor requirements

v13 must address three independent residuals without relaxing the fail-closed Harness boundary:

1. **Annotation/protocol consistency.** Define each model field as an atomic proposition with a reviewable annotation rule before generating expected labels. Historical v12 labels remain immutable evidence and are not edited.
2. **Orthogonal semantics.** Target identity and relation kind must be judged independently. A clearly different sibling target can still express the exact requested relation, and relation mismatch must not be inferred merely from target mismatch.
3. **Observable blocking cues.** Because Present and Unresolved both block in materialization v7, the successor should avoid asking the model to distinguish two labels that have identical Harness consequences. Model output should identify whether a concrete local blocking cue exists; hypothetical open-world uncertainty must not create a blocker.
4. **Groq transport pacing.** A structured-output failure without usage telemetry must not immediately consume another request inside a 60-second case budget. The successor transport must precommit a provider-specific format/pacing strategy that avoids the observed 400 -> fallback -> 429 timeout chain.

Research anchors for the v13 design include:
- Weir et al., EMNLP 2024, *Enhancing Systematic Decompositional Natural Language Inference Using Informal Logic*: clear entailment annotation protocols improve consistency and downstream inference quality.
- Chen et al., Findings ACL 2023, *PropSegmEnt*: proposition-level decomposition avoids conflating multiple semantic claims in one coarse label.
- Srinivasan et al., Findings ACL 2024, *Selective “Selective Prediction”*: over-abstention is a distinct failure mode and can be reduced by identifying additional concrete evidence rather than treating generic uncertainty as sufficient reason to abstain.
- Xu et al., Findings ACL 2025, *Do Language Models Mirror Human Confidence?*: separating self-assessment from answer generation improves calibration, supporting independent guard semantics rather than action-like labels.
- Geng et al., 2025, *Generating Structured Outputs from Language Models: Benchmark and Studies*: constrained decoding capability and quality vary by schema/framework, supporting explicit provider transport handling rather than assuming identical structured-output behavior.
