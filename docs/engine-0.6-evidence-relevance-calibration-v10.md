# Engine 0.6 candidate: evidence-target relevance calibration v10

Status: pre-freeze successor to immutable v9 FAIL. Historical v1-v9 fixtures, observations, tags, and scores remain immutable. Independent holdout authoring is prohibited until the first/only frozen v10 canonical calibration passes.

## Design correction

v9 established the desired safety boundary but mixed that boundary with a brittle explanation taxonomy and raw Text transport. v10 asks only the action question the Harness needs at a candidate-local boundary:

- negative safety decision: `safe_to_reject | abstain`;
- positive safety decision: `safe_to_accept | abstain`.

`safe_to_reject` does **not** assert that two product names are globally distinct. It means only that the supplied local candidate can be rejected for the exact Harness target/relation without discarding plausible target support. A passage clearly scoped to another product, navigation/comparison-only target mentions, or a generic passage with no target-local support may therefore be safely rejected when the supplied material itself does not raise identity equivalence. Explicit rename/alias/successor/cross-language/version-lineage uncertainty, mixed/shared ownership, and partial/truncated identity must abstain.

`safe_to_accept` means the supplied local document unit clearly binds the requested relation to the exact Harness target. Identity may be established in a source title, heading, or structured metadata while the relation appears in an adjacent excerpt. URL-only identity, unresolved shared-table ownership, and uncertain identity mapping must abstain. Factual contradiction, staleness, authority, verification, and answer sufficiency remain downstream concerns and are not relevance failures by themselves.

The Harness continues to own target identity, aliases, provenance, policy, and final disposition. Candidate content is untrusted data and cannot self-declare relevance or authority.

## Materialization policy v6

The primary target/relation binding remains advisory:

- `target=different|unresolved` -> invoke negative safety decision; `safe_to_reject` -> `irrelevant`, otherwise `ambiguous`;
- `target=exact, relation=different` -> `irrelevant` without a secondary call;
- `target=exact, relation=unresolved` -> `ambiguous`;
- strict identity mode still blocks positive promotion when no Harness-owned anchor exists;
- eligible `target=exact, relation=exact` -> invoke positive safety decision; `safe_to_accept` -> `relevant`, otherwise `ambiguous`.

This deliberately retains an independent positive boundary after the v8 exact/exact mixed-ownership failure. v10 fixes the semantics and transport before considering any future reduction in secondary calls.

## Structured transport and bounds

Primary binding remains maximum two model calls (`JsonSchema`, then the existing provider-neutral JSON-object fallback only when required).

Each secondary safety stage also uses a maximum of two model calls:

1. typed `JsonSchema` request;
2. at most one provider-neutral `JsonObject` transport fallback if schema capability is unsupported or the primary structured response is malformed.

The fallback preserves the original task, system policy, seed, token budget, and semantics. It is not a semantic retry, re-prompt, fuzzy repair, substring extraction, or third chance. A malformed fallback fails typed `protocol` closed.

Safety-decision output is a single-field JSON object and is capped by the existing 192-token calibration policy budget. Primary and secondary stages still share the 60,000 ms case deadline. The run circuit still opens after two consecutive operational failures.

Google seed handling is also corrected independently: the common Harness `u64` seed is normalized by the Google adapter into the provider-supported non-negative signed-32-bit domain before serialization. The abstract seed contract remains provider-neutral.

## Calibration corpus

v10 contains 56 synthetic cases:

- 47 v9 cases retained for regression/comparability with action-safety expectations;
- 9 fresh independently authored v10 cases: distributed title-to-body scope, stale-but-target-local evidence, sibling local scope, comparison/context-only target mention, generic no-target support, unresolved alias mapping, unresolved shared-row ownership, prompt injection inside otherwise clear positive evidence, and an exact/exact shared-row case that must positive-abstain.

Expected dispositions: 16 relevant, 21 irrelevant, 19 ambiguous. Negative safety expectations: 18 `safe_to_reject`, 17 `abstain`. Positive safety expectations: 16 `safe_to_accept`.

Production motivating product content is excluded.

## Provider roles and canonical gate

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite` with pacing and attempt telemetry.

The first/only frozen canonical run must use tag `engine-0.6-evidence-relevance-calibration-v10-freeze`, run attempt 1, and the exact checksummed surface. Each required arm must independently complete 56/56 with zero provider failures and satisfy:

- wrong-target relevance retention: 0;
- false relevance rejection: 0;
- expected Relevant left ambiguous: 0;
- utility misses: 0;
- materialized disposition exact: 56/56;
- unsafe negative rejection: 0;
- unsafe positive acceptance: 0.

Primary proposal accuracy and safety-decision exact accuracy remain diagnostic. The final Harness-owned disposition and unsafe action counts are the acceptance boundary.

A v10 FAIL becomes immutable and is not rerun/rescored. Only a canonical PASS permits authoring a fresh independent holdout; runtime acceptance follows only after that holdout passes. PR #466 remains Draft through all three stages.
