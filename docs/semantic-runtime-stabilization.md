# Semantic runtime stabilization

Issue #73 hardens the frozen `semantic-decidability-d3-v1` candidate for runtime use without changing
its semantic contract, calibration fixtures, holdout-v5 labels, thresholds, or provider-specific
behavior.

## Frozen runtime identities

The core owns the runtime identity rather than accepting it from model output or provider metadata.
The stabilization API freezes these identifiers:

| Coordinate | Identity |
| --- | --- |
| characterized rollback baseline | `soft-semantic-v3` |
| R2 model-facing materialization contract | `materialization-r2-v1` |
| deterministic decidability contract | `deterministic-explicit-typed-preconditions-v1` |
| D3 runtime candidate | `semantic-decidability-d3-v1` |
| identity schema | `semantic-runtime-identity-v1` |

`SemanticRuntimeProfile::SemanticDecidabilityD3V1` records the baseline, R2 contract, deterministic
gate, and rollback configuration together. `SemanticRuntimeProfile::SoftSemanticV3` remains the
compiled default during the stabilization PR. After that change passed CI, a separate reviewable
runtime-adoption PR switched the default to D3. The already characterized v3 profile remains an
explicit rollback selection.

`run_semantic_runtime` provides both profiles behind one provider-neutral API. The D3 branch executes
the unchanged R2 semantic materialization, evaluates the harness-owned typed decidability gate, and
can only preserve the base decision or force `abstain`. It cannot repair malformed provider output,
promote an abstention, create trusted evidence, or change verdict authority.

## R2/D3 capability preflight

`reason-semantic-preflight` performs a bounded series of protocol-only synthetic requests that is
independent of every calibration and holdout corpus. The default is three probes and every probe must
pass for an overall `compatible` result, so intermittent R2 materialization failures cannot be hidden
by one lucky response. A successful preflight means only that the provider/model emitted payloads
accepted by `materialization-r2-v1`; the observed semantic decisions are reported but never scored.

```text
cargo run -q --locked -p reasoning-harness-cli --bin reason-semantic-preflight -- \
  --provider mistral \
  --model ministral-8b-latest \
  --probes 3
```

The output separates:

- `compatible`: every requested R2 decision-only protocol probe parsed successfully;
- `incompatible`: a protocol/capability failure such as forbidden model-owned materialization fields;
- `operationally_incomplete`: credentials, quota, rate limit, timeout, transport, provider availability,
  truncation, or other operational failure prevented a capability conclusion.

The manual `semantic-d3-capability-preflight` workflow exposes the same probe. It is deliberately
separate from the historical frozen D2 and holdout-v5 workflows, so a compatibility check does not
rewrite an adoption study's provider-call plan.

## Typed operational telemetry

R2 materialization failures now use `MaterializationFailureClass` rather than CLI-local strings. The
serialized classes distinguish setup, credentials, transport, provider error, rate limit, quota,
provider unavailable, timeout, provider protocol, unsupported capability, materialization protocol,
truncation protocol, and provider generation error.

`reason-decidability-study` keeps the per-case typed failure and also emits `failure_counts`. Provider
or protocol failures remain operational evidence and never become `finding`, `no_finding`, or
`abstain` observations.

## Partial-result preservation

Long decidability studies can opt into atomic progress preservation with `--checkpoint <path>`. The
checkpoint is rewritten immediately before and after every provider call using a same-directory
temporary file and rename. It contains the immutable study/candidate identity, provider/model,
started/completed/successful/failed counts, the currently active fixture/trial/seed if a call is in
flight, typed case failures, usage/latency already observed, and all completed cases.

Checkpoint semantic status is explicit:

- `partial_do_not_score` while execution is still in progress;
- `operationally_incomplete_do_not_score` if execution terminates normally with failed provider calls;
- `full_study_complete` only when every expected provider call completed successfully.

This preserves evidence such as a timeout after a partially completed model run without allowing the
partial rows to masquerade as the frozen study's semantic denominator. Existing complete-trial metric
rules remain unchanged.

## Adoption and rollback

The stabilization PR deliberately left the default unchanged until its CI gate passed. Runtime
adoption then occurred as a separate reversible change: `DEFAULT_SEMANTIC_RUNTIME_PROFILE` now
selects `SemanticDecidabilityD3V1`, so `run_default_semantic_runtime` executes the R2 materialization
plus deterministic D3 gate. `SemanticRuntimeProfile::SoftSemanticV3` remains directly selectable for
rollback, and the low-level v3 semantic-judge API is not rewritten or removed. No provider-specific
semantic branch is permitted.

Rollback therefore does not require changing fixtures, prompts, thresholds, or recorded research:
a caller can select `SoftSemanticV3` explicitly, or the default constant can be reverted to that
profile in one isolated runtime change.

The observed holdout-v4/v5 corpora remain immutable research history. Neither stabilization nor
adoption may use their content for prompt tuning, relabelling, threshold selection, or calibration.

## Live operational smoke

The adopted runtime has a separate synthetic smoke surface that is deliberately outside the D2 and
holdout-v5 research corpora. `reason-semantic-runtime-smoke` executes two bounded operational cases
against a selected provider/model:

- a deterministic `permit` case with a clear counterexample, where D3 must preserve the R2 base
  decision; and
- a matched deterministic `force_abstain` case with the same model-visible semantic context but a
  missing harness-owned proposition binding, where D3 must return `abstain` regardless of the R2
  base decision.

The same cases also execute the explicit `soft-semantic-v3` rollback profile and require it to
remain operationally executable. Fixture dispositions are recomputed before provider initialization,
and live failures are reported with the existing typed operational classes. The smoke surface is not
semantic calibration, does not score correctness, and must not consume or modify frozen holdouts.
