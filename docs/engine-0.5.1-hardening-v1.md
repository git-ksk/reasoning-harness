# Harness Engine 0.5.1 cross-model hardening acceptance v1

Issues #445 and #446 are a non-retroactive follow-up to the closed Harness Engine 0.5.0 semantic baseline. The frozen Engine 0.5.0 result remains immutable.

This successor tests two gaps without weakening authority or correctness:

1. Finalization evaluator semantics (#445): planner target_recalled remains path/utility telemetry, but grounded finalization is judged by admitted evidence, exact supported artifact state, exact grounded exposed output, and unsupported-exposure safety.
2. Session explicit-fact continuity (#446): a unique persisted explicit_user_fact may deterministically preserve the identity of the proposition being corrected. The corrected proposition is materialized only as an Assumed candidate claim and must pass ordinary structured verification before it can become supported or grounded.

## Product coordinate

- Engine 0.5.0 accepted control: 12292b92bcd7890b3a81fc53b2523d172c2bde3a
- Engine 0.5.1 hardening product candidate: 19ef4ac58cd1a3157313ace2e3f45ee7e36f29e5
- product change: PR #447 / Issue #446
- evaluator semantics prototype: PR #448 / Issue #445
- release/package version is unchanged by this evaluation surface

Evaluation-only commits above the product candidate must leave Cargo.toml, Cargo.lock, and crates/ byte-identical to the product candidate.

## Fresh cases

Three fresh identities were collision-checked before surface creation:

- Velquor: answerable exact-key investigation, engine051.velquor.route_endpoint = lane-913.theta
- Tarvess: exact-key no-result investigation that must remain fail closed
- Oryndel: session starts from explicit fact window-1303, corrects to window-1319, and must preserve typed invalidation, zero external replay, exact verified grounding, and unsupported exposure = 0

The Oryndel start precondition is unique persisted explicit-user-fact identity, not a model-generated prior claim. start_prior_grounded remains telemetry so model behavior stays visible.

## Required model matrix

All six rows are required independently. There is no provider average, model average, majority vote, or cross-model repair.

| target | provider/model | role |
| --- | --- | --- |
| mistral-14b | Mistral / ministral-14b-latest | affected required |
| groq-qwen3.8-27b | Groq / qwen/qwen3.8-27b | affected required |
| mistral-8b | Mistral / ministral-8b-latest | validated reference |
| google-gemini-3.5-flash-lite | Google / gemini-3.5-flash-lite | validated reference |
| google-gemma-4-31b-it | Google / gemma-4-31b-it | validated reference |
| groq-gpt-oss-120b | Groq / openai/gpt-oss-120b | validated reference |

Base seed is 812411 for every row. Provider-specific pacing is operational only.

## Acceptance invariants

Every row must complete all 3 cases and pass independently.

Grounded investigation requires admitted evidence, exact artifact support, exact grounded exposure, and unsupported exposed assertions = 0. Planner target_recalled is recorded but is not a finalization correctness conjunct.

Fail-closed investigation must not fabricate exact support or grounded exposure, with unsupported exposed assertions = 0.

Session correction requires exactly one persisted explicit_user_fact value for the corrected key at start, typed correction and invalidation events, pending revalidation false after correction, external calls replayed = 0, corrected target exactly supported and grounded, and unsupported exposed assertions = 0. Whether the model grounded the start value remains separate telemetry. The final artifact must also contain the exact supported Harness-owned correction claim with the harness_session_correction_target_ identity, proving the deterministic path was exercised.

Across the full row, correctness-boundary violations and session external-call replay must both be 0.

## Canonical observation policy

Tag engine-0.5.1-hardening-v1-freeze freezes cases, configs, evaluator/scoring semantics, the six-row model matrix, seeds, workflow, and checksum before credentials are exposed.

- first live launch per target is canonical;
- workflow rerun is forbidden;
- any failed workflow, including pre-live infrastructure failure, requires a new successor identity;
- historical Engine 0.5.0 evidence is never modified or reclassified;
- raw reports, stderr/stdout, attempt markers, job/artifact coordinates, and aggregate matrix are preserved before merge or compatibility claim.

This acceptance tests a bounded hardening delta. It is not an SLA, model ranking, or population-level reliability estimate.
