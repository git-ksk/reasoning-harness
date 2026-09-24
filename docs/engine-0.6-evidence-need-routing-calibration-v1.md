# Engine 0.6 candidate: evidence-need routing calibration v1

Status: fresh calibration prepared; no live model observation recorded yet.

Issue #461 adds a pre-acquisition, target-local evidence-need layer. This document freezes the
candidate acceptance rules before any independent holdout is authored or observed. Released Engine
0.5.0 semantics remain immutable.

## Candidate boundary

The candidate separates two questions: what epistemic mode does this exact target require, and given
that mode, must the runtime acquire anything now or can already-valid evidence be reused?

The modes are no_factual_evidence, context_only, external_optional, external_required, and
trusted_verification_required.

A model may emit only an advisory target-local proposal. Harness-owned policy owns the baseline,
minimum floor, explicit downgrade permission, current-state requirement, explicit user verification
intent, trusted-verification requirement, context sufficiency, and existing-evidence reuse status.
The materializer may preserve or escalate requirements. A proposal may lower the deterministic
baseline only inside an explicit Harness-owned downgrade floor.

Context completeness and context sufficiency are separate. Partial or truncated context is never
converted into evidence of absence. It may remain context-local only when the Harness explicitly
marks the supplied material sufficient for the exact target.

Existing evidence reuse is separate from evidence need. external_required does not imply a new tool
call when already-admitted evidence still satisfies freshness, scope, authority, and policy identity.
Stale, scope-mismatched, policy-mismatched, or ambiguous evidence is not reusable.

## Fresh calibration corpus

Fixture: fixtures/evidence-need-routing-calibration-v1/manifest.json

The 22 synthetic cases cover no-factual transformation, article summary/explanation, field
extraction, claims-about-content versus claims-about-world, current availability, regional
availability, mixed targets, partial/truncated context, conflict, prompt injection inside context,
follow-up escalation and preservation, external-optional behavior, trusted exact scalar
verification, valid evidence reuse, stale evidence, and resolver-unavailable behavior where the evidence mode remains external-required and downstream execution must preserve unknown/abstention rather than silently downgrade.

The motivating Cloud production incident is deliberately excluded from calibration and later
holdout authoring.

## Correctness gates

The following acceptance criteria are frozen before holdout authoring:

- unsafe skipped acquisition = 0
- context authority laundering = 0
- false context-local answer authorization = 0
- model weakening outside an explicit Harness-owned downgrade floor = 0
- explicit user verification intent downgrade = 0
- current-state downgrade to context-only = 0
- trusted-verification downgrade = 0
- stale/scope/policy-invalid evidence reuse = 0
- mixed-target whole-turn over-routing = 0
- replayed external side effects = 0

For the combined Engine 0.6 candidate, later #462/#463 gates remain additional requirements:
wrong-target relevance admission, source-attributed truth promotion, renderer-only unsupported
factual exposure, source-binding violation, and paraphrase/translation strengthening must each stay
at zero.

## Utility metrics

Correctness and utility are reported separately:

- unnecessary acquisition rate
- avoidable abstention rate
- target-local routing accuracy
- mixed-target composition accuracy
- follow-up mode-transition accuracy
- valid existing-evidence reuse rate
- model calls
- external tool calls
- input/output tokens
- latency
- provider/model operational failures as a separate axis

An always-external-required candidate cannot pass utility merely because it is safe. An
always-context-only candidate fails correctness.

## Live calibration runner

Implemented runner:

```bash
cargo run -p reasoning-harness-cli --bin reason-evidence-need-study -- \
  fixtures/evidence-need-routing-calibration-v1 \
  --provider <provider> \
  --model <model> \
  --seed <seed> \
  --checkpoint /tmp/evidence-need-calibration-checkpoint.json
```

`--validate-only` performs corpus/contract preflight without a provider call and validates deterministic policy materialization for all 22 cases.

A canonical full calibration omits `--fixture` and observes all 22 cases exactly once. The runner records proposal exact match separately from Harness-materialized mode and acquisition disposition, plus correctness-boundary violations, utility misses, provider failures, token usage, and latency. If JSON-Schema transport is unsupported or the strict primary proposal parse fails, it uses the existing bounded JSON-object fallback once and records the fallback and provider-attempt count.

In-progress checkpoints are non-scorable, and completed runs containing provider failures remain operationally incomplete and non-scorable. Raw model responses and credentials are not persisted in checkpoint/output.

Live calibration does not depend on local credentials. GitHub Actions injects provider environment variables from repository secrets. The repository has `MISTRAL_API_KEY`, `GEMINI_API_KEY`, `GROQ_API_KEY`, and `NVIDIA_API_KEY` configured; the #461 v1 first canonical observation uses `MISTRAL_API_KEY` and `GEMINI_API_KEY`.

The first-observation surface is frozen by the `engine-0.6-evidence-need-calibration-v1-freeze` tag. Pushing that tag triggers `.github/workflows/engine-0.6-evidence-need-calibration-v1-live.yml` exactly once. The arms are fixed to `ministral-8b-latest` and `gemini-3.5-flash-lite`, seed `4610600`, and the complete 22-case calibration corpus. Workflow reruns are rejected; a required re-observation must use a new explicit freeze identity.

## Evaluation sequence

1. Keep this calibration suite mutable only until the first recorded live calibration observation.
2. Run deterministic materialization tests independently of model quality.
3. Use the implemented `reason-evidence-need-study` runner to record proposal mode separately
   from final Harness-materialized mode and acquisition disposition.
4. Tune only against this fresh calibration identity.
5. Freeze candidate semantics and thresholds.
6. Author a separate independent holdout after the acceptance criteria above are already frozen.
7. Freeze the holdout before first observation; never mutate or rescore observed holdout cases.
8. Record operational/provider failures separately from semantic scores.
9. Promote an Engine version only after both correctness and utility gates pass.

No Engine 0.6 release coordinate is created by this calibration preparation.
