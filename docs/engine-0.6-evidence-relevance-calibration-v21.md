# Engine 0.6 evidence-target relevance calibration v21 — design-only successor

Status: pre-freeze implementation candidate. Effective qualification v2, materialization v16, authority-qualified gating, frozen-core replay tests, and the v21 one-shot workflow are implemented. No v21 freeze tag or live observation exists.

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
- raw verifier exists with `identity_scope=exact_target`, `scope_risk=none`, and `relation_scope=different_relation`;
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

## Blocking-risk qualification gate v2

v20 also exposed a gate/authority mismatch: when deterministic effective `scope_risk` is non-none, materialization is already forced to Ambiguous, yet identity/relation axis mismatches still fail the canonical effective-qualification gate. That makes non-authoritative diagnostic fields gate required providers.

v21 versions the gate rather than hiding the telemetry:
- effective risk derivation remains mandatory for every case;
- risk misses and spurious risks remain zero-tolerance;
- when expected/effective risk is `none`, identity and relation must still match exactly;
- when a blocking risk is correctly non-none, identity/relation mismatches remain emitted as diagnostic metrics but are masked from the authority qualification gate because they cannot influence the terminal disposition.

Add separate authority-qualified metrics; do not overwrite the existing all-axis effective qualification telemetry. This is a general dominance rule for all blocking-risk categories, not a case-25 exception.

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
- context-gap relation mismatches stay fail-closed and may remain diagnostic; the new authority-qualified gate must still be 48/48;
- v18/v19/v20 regression suites, full package tests, all-target Clippy `-D warnings`, fmt, validate-only, and frozen surface checksum are green;
- no case-specific IDs, synthetic product names, or exact fixture phrases appear in production rules.

## Pre-freeze implementation evidence

The current candidate satisfies the deterministic/pre-freeze design proof:
- v21 keeps the fixed 48 cases and all frozen expected labels unchanged;
- effective qualification contract `reason-evidence-relevance-effective-qualification-v2` and materialization policy `target-evidence-relevance-binding-materialization-v16` are versioned successors;
- immutable v20 replay is sanitized and covers Mistral 48 successful observations, Groq 39 successful observations, and Google 48 successful observations;
- Mistral replay remains 48/48 authority-qualified and 48/48 materialized exact;
- Groq replay improves materialization from 38/39 under v15 to 39/39 under v16, reducing wrong-target Relevant from 1 to 0; `13_same_service_different_feature` is the only intended v15 -> v16 terminal disposition change;
- Google replay remains 48/48 materialized exact; its blocking-risk relation-axis disagreements remain visible in all-axis telemetry but are correctly non-authoritative under the separate authority-qualified gate;
- v18 regression: 17/17 PASS; v19 regression: 12/12 PASS; v20 regression: 16/16 PASS; v21 suite: 20/20 PASS; runner focused tests: 28/28 PASS;
- full `reasoning-harness-core`, `reasoning-harness-providers`, and `reasoning-harness-cli` package suites: PASS; providers recorded 153 passed / 1 ignored; CLI main suite recorded 205 passed / 3 ignored and all integration blocks passed;
- all-target Clippy with `-D warnings` for core/providers/cli: PASS;
- `cargo fmt --all -- --check`: PASS;
- v21 validate-only: 48 planned / 0 observed, `validate_only_non_scorable`;
- v21 canonical seed is versioned to `4626210`; provider roles, operational budgets, quota latch behavior, and one-shot immutability are unchanged.
- quota/rate-limit failures now persist a separate `provider_diagnostic` excerpt in the observation/checkpoint/result JSON. It preserves provider wording plus numeric limit/usage/retry/header evidence while forcibly redacting URLs, email addresses, organization/project/account/user/request identifiers, secret-like prefixes, and long opaque tokens; the compact `failure` classification remains unchanged.

The v21 candidate is not yet frozen. Standard PR CI and exact surface checksum must be green before any future tag is created.

## Operational boundary

v20 daily quota exhaustion does not authorize a rerun. v21 may receive one fresh canonical only after its new semantics are frozen and a later quota window is available. If Groq again reaches quota, that first/only v21 canonical is immutable FAIL.

Independent holdout authoring remains prohibited until canonical PASS.
