# Engine 0.6 evidence-target relevance calibration v20 — frozen historical design

Status: historical frozen design. Canonical run `36283988719` is immutable FAIL; see [v20 immutable result](engine-0.6-evidence-relevance-calibration-v20-result.md). This document records the pre-freeze design and is not updated to make that result pass.

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

Implemented as materialization policy `target-evidence-relevance-binding-materialization-v15`, subject to the full pre-freeze proof below.

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

The implementation uses a new materialization policy ID (`target-evidence-relevance-binding-materialization-v15`); v14 remains historical and is replayed directly to prove the intended delta.

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

No v20 live run is permitted until the exact candidate commit is committed, pushed, standard PR CI is green, the frozen surface checksum revalidates, and an annotated v20 freeze tag is deliberately created.

## Pre-freeze implementation evidence

Current deterministic/pre-freeze evidence is green:
- v20 manifest contains the same 48 cases as v19 with no case growth or relabeling;
- materialization v15 adds only the exact-target / independently-different-relation terminal rejection described above;
- v20 generic/frozen-core/replay suite: 16/16 PASS;
- immutable v19 Mistral replay: 48/48 effective qualification and 48/48 materialization remain exact;
- immutable v19 Groq successful replay: 10/10 remains exact;
- immutable v19 Google successful replay: materialization improves from 45/46 under v14 to 46/46 under v15; the only v14 -> v15 disposition change is `13_same_service_different_feature`, Ambiguous -> Irrelevant;
- the three Google context-gap relation-scope disagreements remain diagnostic and still terminate Ambiguous; v20 does not normalize them;
- v18 regression: 17/17 PASS; v19 regression: 12/12 PASS; v20 suite: 16/16 PASS; runner focused tests: 23/23 PASS;
- full `reasoning-harness-core`, `reasoning-harness-providers`, and `reasoning-harness-cli` package test suites: PASS;
- `cargo fmt --all -- --check`: PASS;
- all-target Clippy with `-D warnings` for core/providers/cli: PASS;
- v20 validate-only: 48 planned / 0 observed, `validate_only_non_scorable`.

The v19 replay fixture is a sanitized compression of immutable run `36247789205`: successful case IDs are explicit, and only observed proposal/raw-qualification values that differed from the frozen expectation are stored as overrides. It contains no provider response bodies, credentials, or private-project data.

## Holdout boundary

Independent holdout authoring remains prohibited until a first/only frozen v20 canonical PASS. v19 artifacts are successor-design evidence only; they are not rescored and are not a holdout.
