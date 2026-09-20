# Harness Engine 0.5.0 final-v3 cross-model result

**Status:** ACCEPTED semantic/release-gate evidence for the still-unreleased Harness Engine 0.5.0 product candidate.

Canonical coordinates:

- freeze tag: `engine-0.5-final-v3-freeze`
- freeze commit: `063833f38c38225109586b3db92348563b3822f8`
- product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- control commit: `12292b92bcd7890b3a81fc53b2523d172c2bde3a`
- GitHub Actions run: `35457038163`
- aggregate matrix: `docs/observations/engine-0.5-final-v3/engine-0.5-final-v3-matrix.json`
- provenance: `docs/observations/engine-0.5-final-v3/provenance.json`

The canonical workflow, including preflight and final gate, completed successfully. All six required model rows passed independently; no cross-model averaging or majority vote was used.

## Result

| Target | Model | Role | Cases | Correctness violations | Session replay | Result |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `mistral-14b` | `ministral-14b-latest` | affected required | 3/3 | 0 | 0 | PASS |
| `groq-qwen3.8-27b` | `qwen/qwen3.8-27b` | affected required | 3/3 | 0 | 0 | PASS |
| `mistral-8b` | `ministral-8b-latest` | validated reference | 3/3 | 0 | 0 | PASS |
| `google-gemini-3.5-flash-lite` | `gemini-3.5-flash-lite` | validated reference | 3/3 | 0 | 0 | PASS |
| `google-gemma-4-31b-it` | `gemma-4-31b-it` | validated reference | 3/3 | 0 | 0 | PASS |
| `groq-gpt-oss-120b` | `openai/gpt-oss-120b` | validated reference | 3/3 | 0 | 0 | PASS |

Aggregate: **6/6 rows accepted, 18/18 cases passed, 0 correctness-boundary violations, 0 session external-call replay.**

## Final-hardening findings validated

### #445 — finalization correctness vs planner utility

`target_recalled` remains planner/path telemetry and is no longer incorrectly required when exact support and grounded exposure already satisfy the finalization correctness contract.

This distinction was exercised in the canonical v3 run: Mistral 14B and GPT-OSS 120B both passed the answerable Averiq case with `target_recalled=false`, while exact artifact support, grounded exposure, zero unsupported exposure, and the Harness-owned investigation materialization requirement all held.

### #446 — deterministic explicit-fact session continuity

The session correction no longer depends on the model restating a persisted explicit user fact before correction. Qwen 3.8 27B is the strongest observed case: `start_prior_grounded=false`, while `start_prior_explicit_fact_persisted=true`; the correction still completed with `harness_materialized_correction_target=true`, exact support, grounded exposure, and replay 0.

All six rows exercised the supported exact `harness_session_correction_target_*` path for Orivane.

### #450 — deterministic admitted exact-fact investigation materialization

After a validated read-only investigation action admits a mechanically unique exact structured fact, the Harness now adds an `Assumed` Harness-owned exact claim and relies on the existing structured verification pipeline for promotion.

All six rows exercised the supported exact `harness_investigation_admitted_fact_*` path for Averiq. The previously affected Mistral 14B and Qwen 3.8 27B rows both passed the fresh answerable investigation.

The fail-closed Vardelis case remained safe on all rows: no admitted result, no supported target, no grounded target exposure, and zero unsupported exposed assertions.

## Historical naming correction

The earlier `engine-0.5.1-hardening-v1-freeze` run `35450688516` was named before confirming that Harness Engine 0.5.0 had not yet been version-released. It is retained as immutable pre-release evaluation provenance only. It is **not** an Engine 0.5.1 release or release candidate, and its findings were incorporated into this Engine 0.5.0 final-hardening successor.

## Release boundary

This acceptance closes the semantic/correctness gate for Engine 0.5.0, but it does not by itself change the packaged engine coordinate. At the commit represented by this result, `reasoning-harness-core` still reports Engine 0.4.2.

The remaining closeout is mechanical release work: merge the accepted product/evaluation stack, bump the Engine coordinate to 0.5.0 under the split-version policy, update product/status documentation, run deterministic release regression, and create the immutable versioned Engine 0.5.0 tag/release.
