# Engine 0.6 evidence-target relevance calibration v11 — immutable result

Status: FAIL. This result is immutable. Do not rerun, rescore, or retag v11.

Freeze:
- candidate/tag commit: 6eebdd4c90cbb646cb67b06b0345a3c9a51d2208
- tag: engine-0.6-evidence-relevance-calibration-v11-freeze
- canonical GitHub Actions run: 36142920405
- run attempt: 1
- final-gate conclusion: failure

## Required Mistral arm

Model: ministral-8b-latest.

Operationally complete: 65/65 successful provider cases, provider failures 0, qualification invoked 65/65, attempt telemetry complete.

Safety remained conservative:
- qualification risk misses: 0
- wrong-target relevance retention: 0
- false relevance rejections: 0

The acceptance failure was utility/materialization:
- local qualification exact: 0/65
- qualification spurious risk blocks: 43
- materialized exact: 22/65
- expected Relevant left Ambiguous: 19
- utility misses: 43

The root cause is a contract mismatch rather than a need to weaken the materializer. v11 required a risk to be marked unresolved whenever it could not be globally ruled out from supplied local material. That turns ordinary open-world uncertainty into a local blocker. The contract also conflicted with its own Harness-owned alias expectation: known Harness aliases should be authoritative local anchors, while the prompt could classify stated/plausible alias equivalence itself as identity risk.

## Required Groq arm

Model: openai/gpt-oss-120b.

The arm was operationally incomplete and non-scorable:
- planned/completed: 65/22
- successful provider cases: 10
- failed provider cases: 12
- failure classes among observed failures: 10 unsupported_capability, 2 assessment_timeout
- abort: two consecutive operational failures after 22_partial_identity; 43 cases remained
- provider attempts: 204
- latency p50/p95/max: 51,181 / 60,001 / 60,002 ms

The transport failure pattern was repeated server-side structured JSON/schema generation failure followed by bounded fallback attempts and rate-limit waiting. This is independent from the Mistral semantic overblocking result.

## Google replication

Model: gemini-3.5-flash-lite. This arm was non-gating.

It observed all 65 case records, with 63 successful provider cases and two assessment_timeout failures at 33_exact_binding_conflict and 44_prompt_injection_local_absence.

Semantic direction matched the Mistral overblocking signal:
- qualification risk misses: 0
- qualification spurious risk blocks: 25
- materialized exact: 37/65
- expected Relevant left Ambiguous: 7
- utility misses: 26
- wrong-target relevance retention: 0
- false relevance rejections: 0

## Final decision

All required top-level gates were false: operational completeness, correctness gate, utility, materialization, and qualification acceptance. v11 is therefore an immutable canonical FAIL.

No v11 rerun, rescore, or replacement tag is permitted. Independent holdout authoring remains prohibited.

The successor is v12:
- retain the two-key ownership boundary and fail-closed materialization v7;
- redefine qualification risk as a concrete local ambiguity signal rather than proof against hypothetical open-world risk;
- make Harness-owned canonical names/aliases authoritative local identity anchors;
- keep Present/Unresolved risk fail-closed once a concrete ambiguity cue exists;
- replace the single JsonObject fallback in this calibration runner with one strict raw-JSON Text fallback, preserving strict typed parsing and the two-call stage ceiling;
- use a fresh 73-case calibration before any holdout.
