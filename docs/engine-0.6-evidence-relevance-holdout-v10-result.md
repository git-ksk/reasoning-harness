# Engine 0.6 evidence relevance independent holdout v10 result

Status: immutable canonical FAIL.

Freeze tag: engine-0.6-evidence-relevance-holdout-v10-freeze
Freeze commit: 4aa3aac37ea1a4110e4bf69b498661bc3599a36a
Canonical run: 37085574010, attempt 1 only
Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
Required providers: Mistral + Google + Groq
Effective qualification: v11
Materialization: v23
Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, or reinterpret the canonical run as PASS.

## Provider results

Mistral / ministral-8b-latest:
- operational: 26/26, provider failures 0
- authority failures: 0
- identity/risk failures: 0
- materialized exact: 26/26
- wrong-target Relevant: 0
- utility misses: 0

Google / gemini-3.5-flash-lite:
- operational: 26/26, provider failures 0
- authority failures: 0
- identity/risk failures: 0
- materialized exact: 26/26
- wrong-target Relevant: 0
- utility misses: 0
- one diagnostic relation-label difference on v10h15_negative_availability_feature_support_non_frame: expected different_relation, effective unresolved
- this is a conservative forbid_requested variation and did not affect final materialization

Groq / openai/gpt-oss-120b:
- operational: 26/26, provider failures 0
- authority failures: 0
- identity/risk failures: 0
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility misses: 1
- relation-label differences: v10h15 and v10h18
- terminal miss: v10h18_negative_exact_target_numeric_non_frame, expected Irrelevant -> materialized Ambiguous

## v10h18 adjudication

The v10h18 expectation is not holdout drift.

Task: What hard limit applies to Ivory Stream?
Candidate evidence: Ivory Stream processed 93 jobs per project yesterday during routine telemetry.

The frozen pre-holdout successor-v9 development contract already established the same exact-target numeric-observation family as different_relation / Irrelevant in sv9d_19_exact_target_model_only_limit. The related sv9d_07_limit_numeric_observation control also expected different_relation.

Groq returned:
- proposal: exact target / unresolved relation
- local qualification: exact_target / relation_absent / none
- effective qualification: exact_target / relation_absent / none
- materialized disposition: Ambiguous

This is safe over-abstention, not an unsafe authority promotion. It is nevertheless a real utility/coverage miss under the frozen v11/v23 contract because the supplied telemetry observation is about another relation, not evidence of a hard limit.

## Final gate interpretation

Canonical result:
- operational completeness: PASS
- correctness hard gate: PASS
- qualification safety gate: PASS
- materialization gate: FAIL
- utility gate: FAIL
- overall: FAIL

No provider had an unsafe requested-relation promotion, wrong-target Relevant result, identity violation, or provider/protocol failure.

The remaining gap is cross-provider robustness on exact-target non-frame negative relation evidence. v11 safely refuses positive authority, but v23 can remain Ambiguous when a provider returns relation_absent rather than different_relation for an exact-target observation that is clearly about another relation.

## Next research direction

Do not tune v10, change its labels, or rerun it.

Any successor should use a fresh development surface independent of v10 and test a monotone, one-sided negative-relation mechanism for exact-target non-frame observations. It must:
- never create requested-relation authority;
- preserve Ambiguous for genuinely missing/truncated relation evidence;
- distinguish an affirmative observation about another relation from mere absence of requested-relation evidence;
- replay all frozen historical relation controls before any new holdout;
- use a fresh independent acceptance holdout only after successor semantics are frozen.

Issue #468 remains open.
