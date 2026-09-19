# ADR-0004: Harness-owned investigation action materialization

Status: candidate implementation accepted for evaluation; final adoption requires the fresh Issue #283 holdout.

## Context

The bounded investigation runtime already keeps evidence admission, verification, finalization, budgets, and answer safety under Harness control. Before Issue #283, however, the common fallback planner contract still asked the model to emit an exact `(target_id, capability_id)` pair.

Frozen Issue #282 baseline evidence showed why that ownership split matters. Both routine providers remained correctness-safe, but one Google trial proposed an inadmissible target/capability pairing after useful evidence acquisition. The ordinary Harness validator rejected it, so no authority leaked, but utility still depended on stochastic executable-ID selection.

The runtime already knows the facts needed to materialize many executable actions mechanically:

- canonical admitted investigation target IDs;
- each target's exact `expected_fact_key`, when present;
- configured read-only capabilities and explicit `supported_fact_keys`;
- explicit `selection_priority`, when configured;
- attempted target/capability pairs;
- terminal/action budgets;
- the existing #233 globally unique selector;
- the existing #249 exact-target post-`no_result` continuation;
- the existing #261 precedence selector.

The design goal is therefore narrower than "make planning deterministic." The model may still decide **which unresolved target to continue**, but it should not own an exact capability ID when the Harness can derive that ID mechanically.

## Decision

Add a new internal model-facing contract:

- intent contract: `reason-investigation-intent-v1`;
- materialization policy: `target-intent-materialization-v1`.

Keep the existing public/internal contracts unchanged:

- runtime: `bounded-investigation-v1`;
- plan: `reason-investigation-plan-v1`;
- legacy executable action: `reason-investigation-action-v1`.

The intent vocabulary is deliberately small:

```text
continue(target_id)
stop
```

It contains no `capability_id`, tool argument, evidence field, authority field, fact value, receipt, or verdict.

For one exact target, the Harness may materialize an executable `acquire(target_id, capability_id)` only when all of the following hold:

1. the investigation state is non-terminal and action budget remains;
2. the exact target ID exists;
3. the target has a non-empty exact `expected_fact_key`;
4. candidate capabilities are untried, explicitly read-only, and explicitly list that exact key;
5. either exactly one candidate capability remains, or every candidate has explicit `selection_priority` and exactly one candidate has the highest priority.

Missing priority, a priority tie, keyless targets, wildcard-only compatibility, no eligible capability, write-capable tools, or terminal state are not materialized.

The rule never chooses among target identities. If multiple same-key sibling targets are materializable, they remain distinct entries in the intent schema and the model may choose one exact target ID. The Harness does not merge, canonicalize, or infer equivalence between them.

## State machine

```text
unresolved investigation
        |
        v
#249 exact-target no_result continuation available?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
begin bounded round
        |
        v
#233 globally unique exact pair?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
#261 globally unique precedence choice?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
compute materializable exact target IDs
        |
        +-- none ----------------------> legacy action-v1 model selector
        |
        v
intent-v1 model selector: continue(target_id) | stop
        |
        v
Harness materializes exact capability under target-intent-materialization-v1
        |
        +-- refusal -------------------> typed refusal telemetry
        |                                same-round legacy action-v1 fallback
        |
        v
existing action-v1 proposal
        |
        v
existing validate_action
        |
        v
read-only acquisition
        |
        v
ordinary admission -> qualification -> verification -> finalization -> answer safety
```

The same-round fallback is intentional. A provider/schema fallback that emits an invalid or stale target intent must not gain authority, but it also should not consume repeated bounded rounds before the already-existing action-v1 path gets a chance.

## Authority boundary

```text
MODEL
  plan proposal -----------+
  target intent -----------|---- untrusted planning only
                           v
                    HARNESS CONTROL
              exact target identity lookup
              exact-key compatibility
              read-only filter
              attempted-pair filter
              unique priority rule
                           |
                           v
                 executable action proposal
                           |
                    existing validation
                           |
                           v
                 ACQUISITION ADAPTER
                           |
                    untrusted output
                           |
                           v
          admission / qualification / verification
                           |
                           v
                 finalization / answer safety
```

Neither model intent nor Harness materialization creates evidence, truth, verification, or answer authority. Materialization only chooses an executable read-only acquisition action.

## Compatibility

This is additive.

- `reason-investigation-action-v1` remains unchanged and remains the fallback executable-action contract.
- `planner_calls` keeps its previous meaning: legacy action-v1 model-selector calls only.
- `planner_intent_calls` separately counts intent-v1 model calls.
- `harness_intent_materializations` separately counts exact executable actions produced by the new Harness policy.
- typed intent refusal counts/records are additive diagnostics.
- `intent_contract` and `materialization_policy` are additive telemetry identities with deserialization defaults.
- natural JSON may expose an additive `intent_generations` provider-observation array.
- investigation configuration does not gain a new required field.
- static resolver lanes, MCP contracts, evidence admission, verification, finalization, and answer-safety contracts are unchanged.

A checkout that cannot satisfy the materialization rule falls back to the existing bounded action-v1 selector. A release-level rollback can therefore return to the pre-#283 Engine coordinate without migrating stored configuration.

## Session and idempotency

Materialization reads the same `attempted_pairs` state used by action validation and never bypasses `validate_action`. Session persistence continues to store completed resolution attempts, not executable callbacks. Resume/inspect/fork replay reconstructs recorded state without replaying external acquisition; the #283 regression suite explicitly keeps `external_calls_replayed == 0`.

## Evaluation rule

Issue #282 is consumed pre-change diagnostic evidence and is not an adoption holdout.

Final adoption requires a separately frozen fresh successor identity that:

- compares exact pre-change control with the candidate;
- keeps exercised-path observability, utility, hard correctness, and operational completeness separate;
- demonstrates the intent/materialization path was actually exercised;
- reports exact denominators and preserves failed/incomplete observations;
- requires zero correctness-boundary regression;
- does not treat repeated-model agreement as truth.

If the fresh holdout does not demonstrate a utility/stability benefit, the candidate is not adopted merely because deterministic tests pass.
