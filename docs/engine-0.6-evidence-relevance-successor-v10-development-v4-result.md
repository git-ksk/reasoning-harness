# Engine 0.6 evidence relevance successor-v10 development v4 result

Status: immutable development PASS.

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v4-freeze
- Freeze commit: d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0
- Run: 37111376390
- Candidate: effective qualification v15 / materialization v28
- Fresh surface: 24 cases
- Required development providers: Mistral + Google
- Groq: not observed; reserved for a later fresh independent holdout
- Result: PASS
- Holdout acceptance evidence: false

The annotated tag and attempt-1 observation are immutable. Do not rerun, rescore, relabel, or move the tag. This is development evidence only and is not holdout acceptance evidence.

## Preflight

The frozen surface passed checksum validation, validate-only, fmt, Clippy with `-D warnings`, core library tests, v4/v28 deterministic tests, historical replay, predecessor regression suites, and runner tests. Validate-only reported 24 planned / 0 completed and made no provider observation.

## Mistral

`ministral-8b-latest` completed 24/24 with runner rc 0.

- authority failures: 0
- identity/risk failures: 0
- materialization failures: 0
- proposal exact: 16/24
- raw local qualification exact: 14/24
- effective v15 local qualification exact: 24/24
- materialized v28 exact: 24/24
- total tokens: 41,237
- result: PASS

## Google

`gemini-3.5-flash-lite` completed 24/24 with runner rc 0.

- authority failures: 0
- identity/risk failures: 0
- materialization failures: 0
- proposal exact: 16/24
- raw local qualification exact: 17/24
- effective v15 local qualification exact: 23/24
- materialized v28 exact: 24/24
- total tokens: 42,426
- result: PASS

The one effective-qualification mismatch did not create an authority, identity/risk, or final-materialization failure; the precommitted final v28 contract remained exact 24/24.

## Adjudication

The development gate passed on both required providers. The result supports the bounded v15/v28 successor design:

- direct `defined as` Definition evidence can supply bounded other-relation authority under the exact-target/no-risk restrictions;
- instruction/control text cannot supply model-only negative authority;
- separate clean factual negative evidence survives an unrelated instruction segment;
- Harness-owned requested-relation authority survives instruction/control text;
- v28 composes the effective v15 state without reintroducing stale model proposal authority.

Historical replay remains bounded to the precommitted adjudicated corrections. Frozen v11/v14/v23/v27 and all prior immutable runs remain unchanged.

## Next step

Do not use this 24-case development corpus as acceptance evidence. Freeze v15/v28 successor semantics separately, then author a fresh independent holdout with no reuse of observed development surfaces. Groq may re-enter only on that fresh independent holdout after the successor semantics freeze.
