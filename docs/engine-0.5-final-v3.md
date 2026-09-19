# Harness Engine 0.5.0 final acceptance v3

Engine 0.5.0 is still pre-release. The package/core release coordinate remains Engine 0.4.2 until this line is closed and a versioned Engine 0.5.0 release is cut.

The earlier `engine-0.5.1-hardening-v1-freeze` run 35450688516 was named too early. It is retained as immutable pre-release evidence only; it is not an Engine 0.5.1 release or release candidate. Its findings belong to Engine 0.5.0 final hardening.

## Product candidate

- accepted pre-hardening control: `12292b92bcd7890b3a81fc53b2523d172c2bde3a`
- Engine 0.5.0 v3 product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- includes #446 deterministic explicit-fact session correction continuity
- includes #450 deterministic admitted exact-fact investigation materialization
- evaluator semantics from #445 separate finalization correctness from planner target-recall utility

Evaluation-only commits above the product candidate must leave `Cargo.toml`, `Cargo.lock`, and `crates/` byte-identical to that candidate.

## Why v3 exists

The first Engine 0.5.0 final matrix established a strong baseline, but a later fresh hardening observation exposed two avoidable stochastic dependencies:

- Qwen 3.8 27B could persist an explicit session fact but failed to restate it as a prior model claim before correction. #446 moved identity continuity into Harness control while preserving verification authority.
- After a validated read-only investigation action admitted exact structured evidence, Mistral 14B and Qwen still depended on model regeneration to restate the acquired proposition. #450 now adds a Harness-owned `Assumed` exact claim only when the selected capability/evidence mapping is mechanically unique; ordinary structured verification is still required for `Supported`/`Known`.

The prior hardening observation also confirmed #445's evaluator separation: GPT-OSS 120B correctly passed a grounded case with `target_recalled=false` because exact support and grounded exposure were present.

## Fresh v3 cases

- **Averiq**: answerable exact-key read-only investigation. Target: `engine050v3.averiq.route_endpoint = lane-947.sigma`.
- **Vardelis**: exact-key no-result investigation. It must fail closed without inventing `engine050v3.vardelis.registry_token = registry-731`.
- **Orivane**: session correction from `window-1409` to `window-1423` with persisted explicit-fact identity, typed invalidation, Harness-owned correction materialization, and zero external replay.

These identities and values were collision-checked against existing refs before surface creation.

## Required six-row matrix

All rows are required independently. No provider/model averaging, majority vote, or cross-model repair is allowed.

| target | provider/model | role |
| --- | --- | --- |
| `mistral-14b` | Mistral / `ministral-14b-latest` | affected required |
| `groq-qwen3.8-27b` | Groq / `qwen/qwen3.8-27b` | affected required |
| `mistral-8b` | Mistral / `ministral-8b-latest` | validated reference |
| `google-gemini-3.5-flash-lite` | Google / `gemini-3.5-flash-lite` | validated reference |
| `google-gemma-4-31b-it` | Google / `gemma-4-31b-it` | validated reference |
| `groq-gpt-oss-120b` | Groq / `openai/gpt-oss-120b` | validated reference |

Base seed: `823511` for every row. Provider pacing differences are operational only.

## Acceptance invariants

For the grounded Averiq case:

- at least one evidence item is admitted;
- exact target is `Supported`/`Known` in the final artifact;
- the final artifact contains a supported exact `harness_investigation_admitted_fact_*` claim, proving #450 actually executed;
- exact target is exposed grounded;
- unsupported exposed assertions = 0;
- planner `target_recalled` is recorded but is not a correctness conjunct.

For Vardelis:

- exact unsupported target is not fabricated;
- exact target is not exposed grounded;
- unsupported exposed assertions = 0.

For Orivane:

- start checkpoint contains exactly one persisted explicit user fact for the corrected key;
- typed correction and invalidation events exist;
- pending revalidation is false after correction;
- external calls replayed = 0;
- corrected target is exactly supported and grounded;
- final artifact contains the supported exact `harness_session_correction_target_*` claim;
- unsupported exposed assertions = 0.

Across each row: all 3 cases pass, correctness-boundary violations = 0, and session external-call replay = 0.

## Freeze and observation policy

Tag `engine-0.5-final-v3-freeze` freezes the fresh cases, configs, evaluator/scoring semantics, six-row matrix, seeds, workflow, docs, and checksum before credentials are exposed.

- workflow reruns are forbidden;
- first live launch per target is canonical;
- any workflow failure requires a new successor identity;
- historical v2 and the misnamed hardening-v1 evidence stay immutable;
- merge/version release happens only after raw reports and the aggregate matrix are preserved and reviewed.
