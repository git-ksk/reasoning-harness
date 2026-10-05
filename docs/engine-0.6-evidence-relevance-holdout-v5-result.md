# Engine 0.6 evidence relevance independent holdout v5 result

Status: immutable canonical FAIL.

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v5-freeze`
- Freeze commit: `270c1907103c8ef85fa72875b47ff883feec9a86`
- Canonical run: `36694957246`, attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Google + Groq
- Effective qualification: v7
- Materialization: v20
- Operational: all three arms 26/26, provider failures 0
- Correctness hard gate: PASS on all providers; wrong-target Relevant = 0
- Effective authority qualification gate: PASS on all providers, 26/26
- Final gate: FAIL

This result is immutable. Do not rerun, rescore, relabel, move/recreate the freeze tag, delete/recreate the canonical identity, or reinterpret this holdout as PASS.

## Provider results

Mistral / `ministral-8b-latest`:
- operational: 26/26
- provider failures: 0
- effective authority qualification exact: 26/26
- materialized exact: 26/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 0
- tokens: 47,290
- latency p50/p95/max: 1,395 / 2,993 / 4,906 ms

Google / `gemini-3.5-flash-lite`:
- operational: 26/26
- provider failures: 0
- effective authority qualification exact: 26/26
- materialized exact: 26/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 0
- tokens: 48,843
- latency p50/p95/max: 6,730 / 7,941 / 14,583 ms

Groq / `openai/gpt-oss-120b`:
- operational: 26/26
- provider failures: 0
- effective authority qualification exact: 26/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 66,423
- latency p50/p95/max: 10,724 / 12,021 / 12,373 ms
- terminal miss:
  - `v5h15_negative_prompt_injection_absence`: expected Irrelevant -> Ambiguous

For the terminal miss, the frozen expected advisory proposal was `different / unresolved`, but Groq returned `exact / exact`. The raw local qualifier also returned `exact_target / requested_relation / none`. Harness-owned v7 effective qualification nevertheless deterministically recovered the expected `target_absent / relation_absent / none`, so the authority gate stayed exact. v20 then remained fail-closed because its explicit-local-absence rejection path still requires qualifier-side absence; with both advisory stages positive, it fell through to Ambiguous.

The candidate text contained an explicit local absence statement plus untrusted prompt-injection text. The failure does not show that the prompt injection was followed as policy. It shows that advisory positive agreement can still prevent terminal negative utility after deterministic Harness authority has already established local absence.

## Root cause

The remaining gap is materialization authority composition, not target qualification or provider operation.

1. Deterministic Harness-owned local absence can already recover `target_absent / relation_absent / none` even when both advisory stages return positive.
2. v20 still requires the raw local qualifier to independently expose target/relation absence before materializing Irrelevant.
3. That makes advisory output a de-facto veto over a stronger deterministic negative authority result.
4. Removing that veto must be narrow:
   - broad/generic catalog or landing-page wording is not sufficient negative authority;
   - visible truncation/context gaps, identity mapping uncertainty, ownership ambiguity, or other deterministic scope risk cannot create negative certainty;
   - contradictory positive factual evidence in the same bounded local unit blocks forced Irrelevant;
   - prompt-injection/control text remains untrusted data;
   - model output never creates negative authority;
   - no provider, fixture ID, synthetic entity, or exact holdout wording may enter production logic.

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- effective authority qualification: PASS
- utility: FAIL
- materialization: FAIL

Issue #462 remains open. Historical v5 artifacts and labels remain immutable. The canonical v5 observations may be copied into successor-v6 regression replay without changing the historical result.

## Successor direction

The design audit concludes that v7 effective qualification should remain unchanged because it passed the canonical authority gate 26/26 on all three providers and already produced the intended terminal-miss authority state. Successor v6 should therefore version materialization only as v21 unless implementation evidence reveals a separate qualification defect.

The successor-v6 design is recorded in `docs/engine-0.6-evidence-relevance-holdout-successor-v6.md`.
