# Durable reasoning threads and deterministic replay

Issue #28 implements the durable control-plane contract from ADR-0003 without turning Reasoning Harness into a conversation store or generic agent session manager.

## Boundary

`ReasoningThread` persists only explicit typed runtime state:

- task identity and task text;
- untrusted `ReasoningCandidate` snapshots with stable candidate IDs/replacement lineage;
- accepted `ReasoningArtifact` state and verdict;
- `ReasoningPolicy` versions and deterministic #27 invalidation transitions;
- soft-judge observations as non-authoritative observations;
- already-executed #22 `ResolutionAttempt` records;
- checkpoints, interrupt/resume/fork control events;
- finalization results.

Hidden chain-of-thought is neither required nor represented by the contract.

## Append-oriented events

Events have monotonic sequence numbers, stable event IDs, and optional causation IDs. The current typed families are:

- `task_received`;
- `candidate_recorded`;
- `artifact_accepted`;
- `soft_finding_recorded`;
- `resolution_attempt_recorded`;
- `policy_changed`;
- `state_invalidated`;
- `checkpoint_created`;
- `interrupted`;
- `resumed`;
- `forked_from`;
- `answer_finalized`.

A policy change and its invalidation are deliberately separate events. Between them, replay reports `needs_reevaluation` and refuses checkpoint/finalization. The invalidation event must exactly match a deterministic re-run of #27 `apply_reasoning_policy`; an event log cannot manufacture a different authoritative artifact.

## Checkpoints

A `ReasoningCheckpoint` contains:

- stable checkpoint and thread IDs;
- thread schema version;
- event sequence;
- active policy version, when present;
- reconstructable harness-owned snapshot.

A checkpoint can be created only at an active accepted-state boundary. The stored snapshot must exactly equal deterministic replay at its `checkpoint_created` event. Mutated/stale checkpoints fail replay.

The snapshot preserves prior resolution-attempt records and soft observations. This preserves control/accounting context without re-executing any adapter.

## Interrupt and resume

Interrupt requires the latest safe checkpoint. Once interrupted, the thread is frozen: only `resume` is accepted. The interrupted snapshot is never treated as newly verified or finalized, and finalization text is cleared.

Resume restores the exact checkpoint snapshot and returns the thread to `active`. External resolver/tool side effects are not replayed. `resolution_attempt_recorded` is historical typed data only.

## Fork

Fork is non-destructive. It creates a new thread ID with:

- the same root lineage ID;
- parent thread ID;
- source checkpoint ID;
- a copied accepted checkpoint snapshot.

The source thread/history remains unchanged. Stable candidate identity is seeded from the checkpoint so a repaired/replacement candidate in the fork can reference the candidate it replaces.

A finalized source is immutable, but callers may fork from an earlier safe checkpoint to continue a new lineage.

## Policy interaction

When a thread already has an active `ReasoningPolicy`, a newly recorded accepted artifact must already be admissible under that policy. Replay re-applies #27 deterministically and rejects an event that attempts to bypass policy constraints.

For policy transitions, replay recomputes the full #27 `ReasoningPolicyTransition` from the prior accepted artifact and previous policy. The recorded transition must match exactly before its artifact/verdict become current state.

## Persistence backend

Core intentionally provides only the serializable contract plus the abstract `ReasoningThreadStore` load/save boundary. It includes no filesystem, database, cloud service, or retention policy.

Large-payload deduplication/content addressing may be implemented by a future backend or adapter without changing the authority semantics of replay.

## Replay safety invariants

1. event sequence and event IDs are validated deterministically;
2. checkpoint schema/thread/policy identity is validated;
3. interrupted threads cannot advance except by resume;
4. a pending policy change cannot advance except through matching deterministic invalidation;
5. finalized threads are immutable; continuing work requires fork;
6. resolver attempt records never call a resolver during replay;
7. soft findings remain observations and cannot become verification authority;
8. policy transitions are recomputed rather than trusted from serialized data;
9. hidden model reasoning is not part of persistent state.

The regression suite is credential-free and exercises checkpoint/resume equivalence, fork lineage, policy/invalidation replay, tamper rejection, interrupted/finalized gates, resolver-side-effect non-replay, and the absence of hidden-chain-of-thought fields.
