# Engine 0.6 evidence-target relevance calibration v13 — immutable result

Status: FAIL. This canonical result is immutable. Do not rerun, rescore, relabel, or retag v13.

Freeze:
- commit: dc8ed61031a28bc156d4fa34dac0e2a9f87115fe
- tag: engine-0.6-evidence-relevance-calibration-v13-freeze
- canonical run: 36154375809
- run attempt: 1
- final gate: failure

## Required Mistral arm

Model: ministral-8b-latest.

Operational:
- completed 81/81
- successful provider cases 81
- provider failures 0
- provider-attempt telemetry complete

Semantic / materialization:
- proposal exact 42/81 (51.85%)
- local qualification exact 28/81 (34.57%)
- blocking-cue misses 7
- spurious cue blocks 5
- materialized exact 61/81 (75.31%)
- wrong-target / false Relevant retention 1
- false relevance rejections 2
- expected Relevant left Ambiguous 6
- utility misses 19

This is a safety regression relative to v12. The binary observable-cue simplification reduced some over-abstention but also failed to retain required ambiguity and allowed one non-Relevant case to materialize Relevant.

On the fixed 48-case successor core selected after v13 failure, Mistral was 32/48 exact (66.67%), with 1 wrong Relevant, 2 false rejects, 4 Relevant -> Ambiguous cases, 5 expected blocking-cue misses, and 4 spurious cue blocks. The compaction therefore does not hide the v13 failure.

## Required Groq arm

Model: openai/gpt-oss-120b.

Canonical arm was operationally incomplete:
- completed 3/81
- successful provider cases 1
- failed provider cases 2
- circuit opened after case 03 with 78 cases remaining
- provider-attempt telemetry was complete for the observed cases

The v13 transport change removed the previous JsonSchema -> Text double-request failure mode. The new failures were strict-JSON Text local-qualification truncations:
- case 02: JSON ended mid-string; finish_reason=length
- case 03: no model text output; finish_reason=length; completion_tokens=192

This identifies a different operational defect: the 192-token local qualification completion budget is too small for the Groq gpt-oss-120b strict-Text path. The successor diagnostic raises the Groq transport-only completion budget to 512 and disables the consecutive-failure circuit so every case can be attempted. That diagnostic is noncanonical and cannot change the v13 result.

## Google replication

Model: gemini-3.5-flash-lite.

Operational:
- completed 81/81
- successful provider cases 81
- provider failures 0
- provider-attempt telemetry complete

Semantic / materialization:
- proposal exact 57/81 (70.37%)
- local qualification exact 38/81 (46.91%)
- blocking-cue misses 3
- spurious cue blocks 4
- materialized exact 65/81 (80.25%)
- wrong-target relevance retention 0
- false relevance rejections 0
- expected Relevant left Ambiguous 3
- utility misses 16

Google independently confirms that v13's semantic contract still over-abstains and still misses some required blocking cues, despite better exactness than Mistral.

On the fixed 48-case successor core, Google was 37/48 exact (77.08%), with 0 wrong Relevant, 0 false rejects, and 2 Relevant -> Ambiguous cases.

## Cross-model residual

Five fixed-core cases missed under both Mistral and Google:
- 04_semantic_paraphrase
- 15_navigation_only_match
- 16_broad_landing_no_support
- 21_unknown_rename
- 32_separate_product_table

These are treated as model-independent design residuals rather than provider-specific noise.

The dominant failure pattern is that the model-facing guard is still allowed to relitigate facts the Harness already owns, while the six-field qualification task entangles target support, relation support, explicit absence, and blocking cues. In particular:
- Harness-owned canonical/alias anchors can be contradicted by model target_support;
- uncertain rename/successor mappings are sometimes hardened into different/not_supported rather than preserving ambiguity;
- navigation/generic pages are sometimes converted into context-gap blockers instead of ordinary negative evidence;
- explicit distinct structured evidence can spuriously trigger identity-mapping risk;
- freshness/truth concerns sometimes leak into target-support classification.

## Final decision

All required top-level gates are false:
- operational completeness: false
- correctness: false
- utility: false
- materialization: false
- qualification safety: false

v13 is an immutable canonical FAIL. Independent holdout authoring remains prohibited.

## Successor evaluation hygiene

The calibration line had grown 65 -> 73 -> 81 cases by appending hypothesis-specific fresh cases. That growth stops here.

A fixed successor calibration core has been selected:
- identity: evidence-relevance-fixed-core-v1
- 48 cases
- 14 Relevant / 18 Irrelevant / 16 Ambiguous
- fixed_no_new_cases policy
- preserves identity-mapping, ownership-scope, context-gap, explicit-local-absence, target/relation orthogonality, alias, cross-language, URL-only, injection, freshness, and relation-mismatch coverage

Future semantic work may change the successor contract and annotations before live observation, but it must not append cases in-place merely because a model missed them. A genuinely new semantic dimension requires an explicit new core version.

A separate noncanonical Groq postmortem run may be used to characterize the v13 operational failure distribution. It is diagnostic only and never changes the immutable v13 score.
