# Resumable natural-language sessions

Issue #213 connects the existing typed `ReasoningThread` control plane to an explicit product session surface. It does **not** turn conversation history into evidence authority and does not introduce a second chat/runtime implementation.

Machine identity: `reason-session-v1`.

Continuation policy: `session-replay-only-acquisition-v1`.

## Commands

Start a session with the same natural-language options accepted by the normal product path:

```bash
reason session start \
  --store .reason/session.json \
  --provider mistral \
  --model ministral-8b-latest \
  "Determine the deployment region"
```

The first turn is executed by the native natural-language runtime. The resulting candidate, typed resolution attempts, accepted artifact, and safe checkpoint are recorded in `ReasoningThread`; the thread is then interrupted and the session file is atomically persisted.

Inspect or resume without re-running provider/tool acquisition:

```bash
reason session inspect --store .reason/session.json --format json
reason session resume  --store .reason/session.json --format json
```

Add later material:

```bash
reason session add --store .reason/session.json \
  --file incident-note.txt \
  --fact service.owner=platform \
  --hypothesis service.region=eu-west-1
```

Correct a prior explicit premise:

```bash
reason session correct --store .reason/session.json \
  --premise service.region=eu-west-1
```

Fork a safe checkpoint without mutating the source history:

```bash
reason session fork --store .reason/session.json \
  --checkpoint session-checkpoint-1 \
  --out .reason/alternative.json \
  --new-id alternative-lineage
```

Explicitly finalize a historical thread:

```bash
reason session close --store .reason/session.json
```

A finalized thread is immutable. Continue from an earlier safe checkpoint with `session fork`.

## What is persisted

The session file stores only explicit typed product/control state:

- `reason-session-v1` contract identity;
- provider/model/max-token and answer-safety identities used to start the session;
- start-time resolver/admission/trusted-verifier surface identities and config-source names;
- the serializable `ReasoningThread` event/checkpoint state;
- per-turn safe checkpoint IDs and Harness-finalized exposed answer results.

When an external acquisition actually ran, its already-recorded `ResolutionAttempt` carries adapter/admission config identities and cost/call telemetry. Replay reads those records; it does not invoke the adapter again.

Hidden chain-of-thought, private model scratch state, and raw renderer prose outside the existing Harness finalization contract are not session state.

## Add/correct trust semantics

`session add` and `session correct` are state changes, not evidence shortcuts.

- `--file` is persisted and reintroduced as `untrusted_context`; its prose cannot self-promote to a fact.
- `--fact KEY=VALUE` is explicit user structured evidence and still passes the ordinary Harness verification path.
- `--hypothesis KEY=VALUE` is a Harness-owned target, not proof of its value.
- a correction records typed `input_changed` and `input_state_invalidated` events before a replacement candidate/artifact can be accepted;
- stale finalization is cleared/suppressed while revalidation is pending.

The continuation turn reuses the persisted provider/model/safety identity and the native candidate -> grounding -> qualification -> verification -> finalization path.

## Replay and external side effects

`session inspect`, checkpoint replay, `session resume`, and `session fork` reconstruct stored state only. They do not re-run recorded external resolvers, MCP tools, trusted-command verifiers, or investigation acquisition.

For `session add` / `session correct`, v1 deliberately uses `session-replay-only-acquisition-v1`: it does **not** implicitly replay the start turn's resolver/MCP/investigation configuration. The new turn may call the persisted model for candidate generation/rendering, but acquisition must come from explicitly added material. This prevents "resume" from silently causing old external side effects again.

## Failure behavior

Before a continuation model call, input changes and their typed invalidation are atomically persisted. If the provider then fails, the previous final answer is not exposed again as current: session output reports `pending_revalidation: true` and omits stale `finalization`.

The prior safe checkpoint remains in history and can be forked for recovery. Invalid contract/runtime/safety identity fails closed as `session_incompatible` rather than silently reinterpreting the persisted state.

## Core vs product persistence

Core still owns no filesystem/database/cloud backend. `ReasoningThreadStore` remains abstract. `reason session` is the first thin product adapter: an explicit local JSON file with atomic replacement semantics. A future database/cloud store can implement the same control-plane invariants without changing evidence authority.
