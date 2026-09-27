# Engine 0.6 evidence-target relevance calibration v21 — design-only successor

Status: design only. No v21 runtime implementation, workflow, freeze tag, or live observation exists.

v21 follows immutable v20 run `36283988719`. It does not repair, rerun, or reinterpret v20.

## Measured v20 gap

Required Groq exposed one safety-critical relation-authority conflict before daily quota exhaustion:
- exact target identity was locally anchored;
- primary proposal said `relation=exact`;
- independent raw verifier said `relation=different_relation` with `scope_risk=none`;
- no deterministic requested-relation cue was present;
- effective qualification v1 promoted the primary exact relation to `requested_relation`;
- v15 then materialized Relevant for an expected Irrelevant case.

The defect is not a prompt-only miss. It is a Harness authority-ordering problem: advisory primary output can currently overwrite an independently negative relation classification when deterministic local evidence does not establish the requested relation.

## Frozen inheritance

Keep unchanged:
- fixed core `evidence-relevance-fixed-core-v1`, 48 cases;
- every expected proposal/qualification/disposition label;
- primary proposal v5 and raw verifier v8 prompts/contracts;
- typed deterministic local-risk classifier;
- strict positive identity floors and all v19/v20 target-negative safety behavior;
- context-gap / ownership / mapping fail-closed behavior;
- provider retries, deadlines, quota/capacity latches, telemetry, and sanitization;
- required providers Mistral + Groq and full non-gating Google replication;
- one-shot canonical immutability and holdout prohibition before PASS.

## Effective qualification v2

Introduce `reason-evidence-relevance-effective-qualification-v2` rather than mutating v1.

Only one authority boundary changes.

When all are true:
- deterministic local risk is `none`;
- effective identity is `exact_target` from Harness identity evidence;
- deterministic requested-relation presence is false;
- raw verifier exists with `scope_risk=none` and `relation_scope=different_relation`;
- primary target is `exact`;

then effective relation is `different_relation` even if the primary proposal says `relation=exact`.

The primary exact relation alone may no longer upgrade this conflict to `requested_relation`.

This is deliberately one-sided. It does not let raw verifier output create a positive requested-relation result, does not infer through risk, and does not change any context-gap normalization.

## Materialization v16

Introduce a new materialization policy ID. v15 remains historical.

For the exact-target relation-conflict shape above, v16 may terminally materialize Irrelevant even when the primary relation is exact, but only when:
- deterministic risk is `none`;
- strict Harness target identity is satisfied;
- effective qualification v2 is `exact_target + different_relation + none`;
- raw verifier independently reports `different_relation + none`;
- deterministic requested-relation presence is absent.

Any context/ownership/mapping risk, missing strict identity anchor, raw safety risk, or deterministic requested-relation cue blocks terminal rejection.

## Why not simply trust the verifier

v21 does not make raw verifier v8 generally authoritative. Raw verifier disagreements remain common across providers. The new authority applies only to a narrow negative relation conflict where:
- target identity is exact and Harness-owned;
- safety risk is absent;
- the requested relation has no deterministic local cue;
- the independent verifier explicitly classifies another relation.

This preserves the v19/v20 raw/effective telemetry split and prevents model-to-model disagreement from becoming unrestricted authority.

## Context-gap normalization remains deferred

v20 again showed relation-scope mismatches under `context_gap` (Groq case 25; Google cases 25/26/80). Their final dispositions remained Ambiguous. v21 must not use this successor to normalize those cases or infer through clipped/omitted content.

## Required pre-freeze proof

Before any v21 freeze:
- fixed 48 and all labels remain byte-semantically unchanged;
- generic property tests cover primary-exact/raw-different conflict, deterministic requested-relation counterexamples, raw-risk counterexamples, and strict-identity counterexamples;
- immutable v20 Mistral successful observations replay 48/48 effective qualification and 48/48 materialization;
- immutable v20 Groq successful observations replay from 38/39 to 39/39 materialization with wrong-target Relevant reduced 1 -> 0;
- v20 Groq case 13 is the only intended terminal disposition change;
- v20 Google remains 48/48 materialization with zero wrong-target Relevant, false rejection, or Relevant -> Ambiguous regression;
- context-gap relation mismatches stay fail-closed and may remain diagnostic;
- v18/v19/v20 regression suites, full package tests, all-target Clippy `-D warnings`, fmt, validate-only, and frozen surface checksum are green;
- no case-specific IDs, synthetic product names, or exact fixture phrases appear in production rules.

## Operational boundary

v20 daily quota exhaustion does not authorize a rerun. v21 may receive one fresh canonical only after its new semantics are frozen and a later quota window is available. If Groq again reaches quota, that first/only v21 canonical is immutable FAIL.

Independent holdout authoring remains prohibited until canonical PASS.
