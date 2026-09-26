# Engine 0.6 candidate: evidence-target relevance calibration v13

Status: pre-freeze successor to immutable v12 canonical evidence. Historical v1-v12 fixtures, tags, observations, and scores remain immutable. Independent holdout authoring is prohibited until the first/only frozen v13 canonical calibration passes.

## Why v13 exists

v12 materially reduced the v11 open-world overblocking failure, but it did not pass. Required Mistral completed 73/73 with zero provider failures, zero qualification-risk misses, zero wrong-target relevance retention, and zero false relevance rejections, yet materialized only 55/73 exactly and recorded 18 utility misses. Seventeen misses were expected Irrelevant materialized Ambiguous; one was expected Relevant materialized Ambiguous. Sixteen of the eighteen misses also disagreed with the frozen expected relation binding.

A static post-observation audit found a separate specification problem in the frozen v12 corpus: some explicit-local-absence examples conflicted with the v12 prompt definition. Those labels remain immutable historical evidence and are not edited. v13 re-annotates a fresh calibration surface under an explicit pre-observation protocol.

v12 also exposed a distinct Groq operational failure. JsonSchema calls that failed with HTTP 400 could consume provider capacity without successful usage telemetry. The immediate strict-JSON Text fallback then received HTTP 429 responses whose retry-after exceeded the shared 60-second case deadline. v13 therefore uses strict raw-JSON Text as the Groq primary transport for both stages, while Mistral and Google retain JsonSchema primary plus one bounded strict Text transport fallback.

## Research anchors

The v13 design is driven by repository evidence first. The following papers are supporting anchors rather than normative authority:

- Weir et al., [Enhancing Systematic Decompositional Natural Language Inference Using Informal Logic](https://aclanthology.org/2024.emnlp-main.531/) (EMNLP 2024): clearer entailment annotation protocols improve internal consistency and downstream inference quality. v13 therefore freezes field-level annotation rules before observation.
- Chen et al., [PropSegmEnt: A Large-Scale Corpus for Proposition-Level Segmentation and Entailment Recognition](https://aclanthology.org/2023.findings-acl.565/) (Findings ACL 2023): sentence-level material can contain multiple propositions with different semantic status. v13 treats target identity and relation kind as separate atomic propositions.
- Srinivasan et al., [Selective “Selective Prediction”: Reducing Unnecessary Abstention in Vision-Language Reasoning](https://aclanthology.org/2024.findings-acl.767/) (Findings ACL 2024): selective systems can over-abstain, and concrete additional evidence can reduce unnecessary abstention without relaxing the error boundary. v13 therefore requires an observable local blocking cue rather than generic open-world uncertainty.
- Xu et al., [Do Language Models Mirror Human Confidence? Exploring Psychological Insights to Address Overconfidence in LLMs](https://aclanthology.org/2025.findings-acl.1316/) (Findings ACL 2025): separating self-assessment from answer production can improve calibration. v13 keeps the independent qualification guard fact-like and Harness-owned materialization separate.
- Geng et al., [Generating Structured Outputs from Language Models: Benchmark and Studies](https://arxiv.org/abs/2501.10868) (2025): structured-output behavior depends on schema and constrained-decoding implementation. v13 makes Groq transport an explicit provider-specific precommit rather than assuming identical JsonSchema behavior.

None of these papers establishes that v13 will pass. Their role is to sharpen the pre-observation hypothesis and reduce degrees of freedom.

## Atomic binding proposal v3

The primary model still returns only two advisory fields: target_binding and relation_binding, each exact | different | unresolved.

The fields are explicitly orthogonal. target_binding concerns which entity/product locally owns the substantive material. relation_binding concerns which relation kind the substantive proposition expresses, regardless of which entity owns it.

Therefore:
- sibling target + requested relation => different / exact;
- sibling target + another relation => different / different;
- exact target + another relation => exact / different;
- generic or explicit-local-absence copy with no substantive relation proposition => relation unresolved.

The model is prohibited from inferring relation difference merely from target difference.

## Local qualification v3

The independent guard returns six atomic fields:
- target_support: supported | not_supported | unresolved
- relation_support: supported | not_supported | unresolved
- identity_mapping_cue: absent | present
- ownership_scope_cue: absent | present
- context_gap_cue: absent | present
- explicit_local_absence: present | absent | unresolved

The three former Absent | Present | Unresolved risk fields become binary observable blocking cues. In materialization v7, both Present and Unresolved had the same fail-closed consequence, so asking the model to distinguish them increased label complexity without changing Harness authority. v13 instead asks whether a concrete local blocker is observable.

A cue may be present only when supplied local material itself contains the relevant signal: uncertain rename/successor/conflicting identity mapping; shared/unassigned row, section, or value ownership; or visibly clipped/truncated/omitted local context required for binding.

Generic open-world uncertainty is not a cue. A clearly generic passage or an explicit local statement that no target-specific information is present is usable local-absence evidence, not automatically a context gap.

## Materialization v8

Harness ownership remains fail-closed.

Hard Relevant requires primary exact/exact, guard target/relation supported/supported, all blocking cues absent, and the required Harness-owned identity anchor.

Hard Irrelevant requires one of:
- primary target different plus guard target not_supported, with all cues absent;
- primary target non-exact plus guard target not_supported plus explicit local absence present, with all cues absent;
- primary exact target + relation different, plus guard target supported + relation not_supported, with all cues absent.

Any blocking cue, CanonicalUrl-only identity floor, missing guard, missing proposal, strict identity failure, or two-key disagreement remains Ambiguous.

This changes the model-facing semantic decomposition, not the Harness authority boundary.

## Transport

Mistral and Google use JsonSchema primary and one strict raw-JSON Text fallback only on unsupported structured capability or malformed structured response. The whole response must parse directly into the typed contract.

Groq uses strict raw-JSON Text as the primary and only transport attempt for each semantic stage. There is no preceding JsonSchema request. Malformed Text is a typed protocol failure, not a semantic retry.

All providers prohibit JSON extraction, Markdown stripping, field synthesis, fuzzy repair, semantic retry, and a third model call. Primary and qualification share the same 60,000 ms case deadline. Two consecutive operational provider failures open the circuit.

## Calibration corpus

v13 contains 81 synthetic calibration cases. The first 73 are successor re-annotations of v12 calibration material under the v13 field protocol. Historical v12 expected labels are untouched. Two frozen v12 annotation contradictions around explicit local absence are corrected only in this new v13 corpus.

Eight fresh cases were authored before any v13 live observation: exact target + requested relation; sibling target + requested relation; sibling target + different relation; explicit generic local absence; uncertain rename + same relation; shared row + same relation; clipped local relation context; exact target + different relation.

Expected final dispositions: Relevant 24 / Irrelevant 29 / Ambiguous 28. Twenty-eight cases contain at least one expected blocking cue. Nine cases contain explicit local absence. Production motivating product content remains excluded.

## Canonical provider roles and acceptance gate

Required:
- Mistral ministral-8b-latest
- Groq openai/gpt-oss-120b

Full non-gating replication:
- Google gemini-3.5-flash-lite

Each required arm must independently satisfy 81/81 operational completion, provider failures 0, complete provider-attempt telemetry, local qualification expected/invoked 81/81, blocking-cue misses 0, wrong-target relevance retention 0, false relevance rejection 0, expected Relevant left Ambiguous 0, utility misses 0, and materialized exact 81/81.

Primary proposal exactness, full qualification-field exactness, and spurious cue blocks remain diagnostic. A spurious cue cannot silently pass the candidate because final disposition exactness and utility are hard gates.

The first/only frozen v13 canonical run is never rerun or rescored. Only canonical PASS permits authoring a fresh independent holdout. Runtime integration acceptance follows only after holdout PASS. PR #466 remains Draft through all stages.
