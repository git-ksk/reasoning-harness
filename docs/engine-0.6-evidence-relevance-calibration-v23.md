# Engine 0.6 evidence relevance calibration v23

Status: pre-freeze implementation candidate. No v23 freeze tag or live canonical observation exists.

## Goal

Promote Google to a required provider without weakening the authority-qualified gate. v23 keeps the frozen 48-case semantic surface from v22 and repairs one Harness-owned effective-qualification composition defect exposed by the immutable v22 Google observation.

## Frozen provider contract

All three providers are required:

- Mistral: `ministral-8b-latest`
- Groq: `openai/gpt-oss-120b`
- Google: `gemini-3.5-flash-lite`

Every required arm must complete all 48 cases and pass correctness, utility, materialization, and authority-qualified effective-qualification gates.

## Semantic delta: effective qualification v3

v22 case `76_v13_sibling_different_relation_no_cue` showed:

- expected proposal: `different / different`
- Google observed proposal: `different / exact`
- Google raw qualification: `distinct_target / different_relation / none`
- v2 effective qualification: `distinct_target / requested_relation / none`

The raw verifier already identified both axes correctly. v2 nevertheless preferred the proposal relation. v3 starts from v2 and changes the relation only when all of the following are true:

1. deterministic scope risk is `none`;
2. effective identity is `distinct_target`;
3. proposal says target `different` but relation `exact`;
4. raw qualification independently says `distinct_target / different_relation / none`;
5. the requested relation is not locally present.

Then the effective relation remains `different_relation`.

This is a generic corroboration rule. It does not key on provider, fixture ID, entity name, or exact wording. Materialization v16 is unchanged and the zero-risk authority gate remains strict.

## Immutable v22 replay proof

The sanitized successful observations from canonical v22 run `36338187291` are replayed under v3 before any freeze. All three providers must reach:

- 48/48 authority-qualified effective qualification;
- 48/48 materialized disposition;
- zero regression on the frozen 48-case surface.

The implementation candidate currently satisfies this proof.

## Groq admission plan

v22 observed 120,426 Groq tokens and completed at `2026-09-27T23:04:31Z`. Using the previously observed 200,000 TPD continuous-refill model and the prior `199,298 used` anchor at `2026-09-27T05:02:01Z`, modeled post-v22 headroom is about 30.6K tokens, assuming no material untracked organization usage.

v23 uses:

- modeled start headroom: 105K;
- earliest modeled floor: `2026-09-28T08:00:02Z` (`2026-09-28 17:00:02 JST`);
- Groq inter-case delay: 210 seconds;
- provider request spacing: 10 seconds;
- deliberate pacing across 48 cases: 10,350 seconds (2h52m30s);
- modeled refill during that pacing: about 23.96K tokens;
- modeled supply: about 128.96K tokens;
- observed-token hard cap: 128K;
- pre-case reserve: 4K;
- Groq job timeout: 240 minutes.

The 128K cap is above v22's observed 120,426-token full-arm demand while remaining below modeled supply. Missing usage telemetry under the guard remains fail-closed. Any known or suspected material organization-level Groq usage after the anchor invalidates this model and requires delaying or re-anchoring before freeze.

## Freeze rule

Before the first/only v23 canonical tag:

1. exact candidate must be clean and fully green;
2. frozen-surface checksum must pass;
3. v22 replay proof must pass for all three providers;
4. fresh Groq readiness must pass on the exact candidate, while remaining explicitly non-proof of TPD headroom;
5. the modeled Groq admission floor must have elapsed;
6. create the v23 freeze tag once and never rerun the canonical.

If any required provider is operationally incomplete or fails a required semantic gate, v23 is immutable FAIL and a new successor is required.
