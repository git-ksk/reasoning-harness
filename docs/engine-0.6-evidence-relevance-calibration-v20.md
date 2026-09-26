# Engine 0.6 evidence-target relevance calibration v20 — design-only successor

Status: design only. No v20 runtime implementation, workflow, freeze tag, or live observation exists.

v20 follows immutable v19 run `36247789205`. It is not a replacement observation and must not reinterpret v19.

## Design objective

Preserve the v19 authority split that reached 48/48 on required Mistral, while addressing one predeclared relation-negative boundary exposed by Google replication. Keep quota handling unchanged rather than weakening acceptance because Groq exhausted daily quota.

## Frozen inheritance

Keep unchanged:
- fixed core `evidence-relevance-fixed-core-v1`, exactly 48 cases;
- all labels and expected dispositions;
- primary proposal v5;
- typed deterministic local-risk classifier;
- raw verifier v8 telemetry as diagnostic-only input;
- Harness-owned effective qualification as runtime/gating authority;
- strict positive identity floors;
- v19 target-negative terminal rule;
- provider retries, time budgets, attempt telemetry, quota/capacity latch behavior, and sanitization;
- required providers Mistral + Groq;
- full Google non-gating replication;
- one-shot canonical immutability and holdout prohibition before PASS.

## Proposed v20 semantic delta

Adopt only after generic pre-freeze proof.

When all are true:
- deterministic local risk is `none`;
- effective `scope_risk=none`;
- effective `identity_scope=exact_target`;
- effective `relation_scope=different_relation`;
- primary `target_binding=exact`;
- primary `relation_binding != exact`;
- no raw model safety risk is non-none;

then materialize Irrelevant rather than Ambiguous.

Fail closed when the primary relation is exact, identity is uncertain, any context/ownership/mapping risk exists, or a safety signal conflicts.

Why this candidate is allowed:
- it was explicitly identified before v19 live observation;
- Google v19 case 13 produced exactly this disagreement;
- the frozen core contains three `exact_target + different_relation + no risk` expectations, all Irrelevant and none Ambiguous;
- it adds no positive-admission path.

Implementation must receive a new materialization policy ID; v14 remains historical.

## Context-gap relation normalization: deferred

Google v19 showed three effective relation-scope mismatches under non-none `context_gap`. All three still materialized Ambiguous correctly. Do not fix these by weakening the qualification gate or matching fixture-specific phrases.

Before any normalization is considered, a generic deterministic rule must reproduce all frozen context-gap expectations, including both `requested_relation` and `unresolved`, without inferring through clipped text or omitted ownership. If that proof fails, v20 leaves this as non-gating diagnostic disagreement.

## Operational successor policy

Groq v19 failed on typed daily quota, not semantics. Therefore v20:
- keeps Groq required;
- keeps daily-quota fail-fast and immediate provider-arm latch;
- adds no retry storm or semantic retry;
- does not use a manual quota probe as acceptance evidence;
- does not rerun v19;
- may run a fresh v20 canonical only after implementation and every pre-freeze check is green in a later quota window.

If Groq hits quota again, that v20 canonical is also immutable FAIL. Any provider-role change requires separate independent evidence and cannot be made merely to pass #462.

## Required pre-freeze proof

Before any v20 freeze tag:
- no case growth or relabeling;
- generic property tests for the relation-negative rule, including conflicting-primary and safety-risk counterexamples;
- frozen v19 Mistral replay remains 48/48 effective qualification and 48/48 materialization;
- frozen v19 Groq successful observations remain 10/10 exact;
- frozen v19 Google successful observations improve from 45/46 to 46/46 materialization with zero unsafe Relevant, false rejection, or Relevant -> Ambiguous regression;
- the three Google context-gap relation mismatches are either resolved generically without regression or explicitly retained as non-gating diagnostic disagreement;
- v18/v19 regressions remain green;
- full workspace tests, Clippy `-D warnings`, fmt, surface checksum, and validate-only are green;
- public artifacts contain no private-project identifiers, secrets, or local paths.

No v20 live run is permitted during this design-only phase.

## Holdout boundary

Independent holdout authoring remains prohibited until a first/only frozen v20 canonical PASS. v19 artifacts are successor-design evidence only; they are not rescored and are not a holdout.
