# Engine 0.6 evidence relevance independent holdout v12 result

Status: immutable canonical PASS.

Freeze tag: engine-0.6-evidence-relevance-holdout-v12-freeze
Freeze commit: 78c6894e871d9aa4dd79aef0a30c95647c075c4d
Canonical run: 37218652869, attempt 1 only
Cases: 26 (Relevant 8 / Irrelevant 9 / Ambiguous 9)
Required providers: Mistral + Google + Groq
Effective qualification: v17
Materialization: v30
Final gate: PASS
Holdout acceptance evidence: true

This result is immutable. Do not rerun, rescore, relabel, or move/recreate the freeze tag.

## Provider results

Mistral / ministral-8b-latest:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 22/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 44,843
- PASS

Google / gemini-3.5-flash-lite:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 25/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 46,022
- PASS

Groq / openai/gpt-oss-120b:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 21/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 63,336
- PASS

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- qualification gate: PASS
- materialization gate: PASS
- utility gate: PASS
- overall acceptance: PASS

The prior v11 Groq terminal failure class is covered by fresh generic target-owned/no-classifiable-relation controls in v12. No terminal miss remains. The fresh ownership-gap and relation-only controls also pass the frozen v17/v30 contract without identity/risk authority failure.

Issue #468 remains open and PR #469 remains Draft pending normal project integration/review policy; this document records the research acceptance result only.
