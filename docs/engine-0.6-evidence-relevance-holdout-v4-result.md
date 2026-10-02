# Engine 0.6 evidence relevance independent holdout v4 result

Status: immutable canonical FAIL.

- Freeze tag: engine-0.6-evidence-relevance-holdout-v4-freeze
- Freeze commit: 4e60db418175be36f1e910f52be88c4cb3b34830
- Canonical run: 36650257492, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Groq + Google
- Effective qualification: v6
- Materialization: v19
- Operational: all three arms 26/26, provider failures 0
- Correctness hard gate: PASS on all providers; wrong-target Relevant = 0
- Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, or reinterpret this holdout identity as PASS.

## Provider results

Mistral ministral-8b-latest:
- effective authority qualification exact: 25/26
- materialized exact: 24/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 2
- tokens: 47,537
- latency p50/p95/max: 1,228 / 2,305 / 2,819 ms
- terminal misses:
  - v4h11_negative_navigation_target_other_subject: expected Irrelevant -> Ambiguous
  - v4h12_negative_generic_catalog_absence: expected Irrelevant -> Ambiguous

Groq openai/gpt-oss-120b:
- effective authority qualification exact: 26/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 67,212
- latency p50/p95/max: 10,636 / 10,999 / 11,099 ms
- terminal miss:
  - v4h15_negative_prompt_injection_absence: expected Irrelevant -> Ambiguous

Google gemini-3.5-flash-lite:
- effective authority qualification exact: 25/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 49,165
- latency p50/p95/max: 6,974 / 10,311 / 19,898 ms
- terminal miss:
  - v4h14_negative_explicit_separate_service: expected Irrelevant -> Ambiguous

v4h22_ambiguous_truncated_relation remained correctly materialized Ambiguous for all providers, but Groq and Google reported relation_absent under a deterministic context_gap; this is an authority-qualification mismatch even though the terminal disposition is safe.

## Root cause

The remaining gap is systematic negative-authority composition, not provider operation.

1. Navigation/footer-only target names are non-owning context. A stable repeated heading/body subject may establish a different local owner even if advisory model stages vote exact.
2. Deterministic local target/relation absence is usable negative evidence. An advisory exact/exact vote must not force Ambiguous when the bounded unit itself explicitly establishes local absence.
3. An explicit statement that a repeated sibling subject is a separate/distinct service or product from the Harness target is Harness-owned negative identity evidence.
4. A visible truncation/context gap cannot be converted into relation_absent; when relation binding is unresolved, the effective relation must remain unresolved.
5. These corrections must not weaken the wrong-target Relevant hard gate, infer global absence, or treat a single near-name signal as sufficient distinct-target authority.

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- utility: FAIL
- materialization: FAIL
- qualification: FAIL

Issue #462 remains open. Successor semantics may replay these immutable observations for regression only. Fresh independent holdout v5 authoring is prohibited until successor-v5 semantics are frozen.
