# ADR-0003: Reasoning control plane, policy, and durable threads

- Status: Accepted
- Date: 2026-08-30

## Context

ADR-0002 established a bounded grounded-resolution loop. The next architectural question is how that loop should behave across longer-lived work: policy changes, interruption/resume, repair cycles, evidence invalidation, and client reconnection.

Mature agent harnesses provide useful control-plane patterns without implying that Reasoning Harness should become a generic agent framework. Current Codex architecture separates technical sandbox boundaries from approval policy, keeps thread lifecycle and persistence in the runtime, and exposes an event stream that clients can reconnect to. LangGraph similarly treats thread-scoped checkpoints as durable execution state and models resume/fork as checkpoint operations. Claude Code exposes resumable/forkable sessions, lifecycle hooks, permissions, and optional subagents. These are architectural influences only; none is a correctness authority or runtime dependency.

## Decision

Reasoning Harness will add a small reasoning-specific control plane around the existing artifact and resolution runtime. The core concepts are:

1. `ReasoningPolicy` — explicit permissions and promotion/escalation rules.
2. `ReasoningThread` — durable identity for a reasoning session across runs and repairs.
3. typed append-oriented events — provenance needed to reconstruct accepted runtime state.
4. explicit checkpoint/resume/fork semantics.
5. policy-change invalidation and deterministic re-evaluation of affected state.

The project will **not** add a second resolver/evidence-provider abstraction. Acquisition continues through the #22 `ResolutionResolver -> EvidenceAdmissionPolicy / TrustedResolutionVerifier` boundary.

## Mapping from mature harness concepts

| Mature harness concept | Reasoning Harness analogue | Decision |
|---|---|---|
| execution sandbox | evidence/inference promotion policy | adopt as `ReasoningPolicy` |
| approval policy | promotion/escalation policy | adopt inside `ReasoningPolicy` |
| thread/session | durable `ReasoningThread` | adopt |
| event stream | typed reasoning/provenance events | adopt |
| checkpoint | reconstructable verified runtime snapshot | adopt |
| resume/fork | continue or branch from a stable checkpoint | adopt |
| tool boundary | #22 resolver/admission/verifier boundary | reuse, do not duplicate |
| scoped instructions | global/domain/run policy layering | adopt as generic policy composition |
| post-action validation | proposition -> evidence -> edge -> artifact -> final-answer ladder | adopt |
| retry/repair | #22 repair + full re-verification | already adopted |
| skills/subagents | specialist semantic workers | defer pending benchmark evidence |

## Evidence and inference sandbox

A reasoning sandbox constrains what model-proposed state may affect grounded output. It is policy, not a separate execution environment.

Initial modes are conceptual presets over one policy object:

- `strict`: only directly verified propositions may be promoted.
- `bounded`: explicitly permitted deterministic inference classes may produce derived support.
- `exploratory`: unverified hypotheses may remain in working state but cannot enter grounded factual output.

The implementation should prefer explicit capabilities/fields over a mode enum once real requirements are known. A preset must never bypass existing verifier authority.

## Promotion and escalation policy

`ReasoningPolicy` owns permitted state transitions, not truth. Policy may choose among:

- retain verified state;
- retain qualified/uncertain state;
- request evidence;
- request deterministic verification;
- request repair/regeneration;
- request human review;
- reject;
- terminate unknown/abstain.

Soft semantic findings remain advisory even when policy uses them as a trigger for more work. They cannot directly create a verification receipt or hard finding.

## ReasoningThread and event model

A `ReasoningThread` is the durable container for one reasoning investigation. It must not require storing or exposing hidden chain-of-thought. Persisted state consists only of explicit typed runtime artifacts and control events.

Candidate event families include:

- task/question received;
- evidence acquired/admitted/qualified;
- candidate proposed/replaced;
- claim or edge verified/refuted;
- diagnostic finding raised;
- resolution/repair/human-review requested;
- policy changed;
- state invalidated;
- checkpoint created;
- answer finalized.

Events should carry stable entity IDs and causal references sufficient to reconstruct the accepted working/verified state. Large raw payloads may be content-addressed rather than duplicated into every event.

## Interrupt, resume, and fork

- **interrupt** records a safe checkpoint boundary without converting incomplete work into evidence.
- **resume** continues from a checkpoint under an explicitly known policy version.
- **fork** creates a new thread lineage from a prior checkpoint while preserving the original history.
- replay/reconstruction must be deterministic for harness-owned state under the same schema/policy versions.

Side effects in external resolvers remain adapter-owned and must be idempotent or externally deduplicated; replaying harness state must not imply replaying external side effects.

## Policy composition and invalidation

Policy is layered generically:

```text
global policy
  -> domain policy
  -> task/run policy
```

Core knows only generic fields such as authority thresholds, freshness/scope requirements, allowed inference capabilities, resolver classes, and escalation rules. Domain-specific source names or taxonomies stay outside core.

A policy change is append-only history, not an edit of past events. The runtime computes which accepted evidence, receipts, claims, edges, and finalizations are no longer admissible and emits explicit invalidation events. Downstream dependent state is re-evaluated before reasoning may resume.

A stricter policy must never silently preserve a conclusion whose support no longer qualifies.

## Validation ladder and invalidation propagation

Validation widens from the smallest affected unit:

1. proposition/schema validity;
2. evidence qualification and evidence-to-claim support;
3. inference/causal edge validity;
4. local dependency-chain consistency;
5. artifact-level consistency and decision;
6. final factual-claim coverage.

When upstream support changes, dependent verification/derived state is invalidated before wider checks run. This is dependency propagation, not model self-correction.

## Repair authority invariant

Repair remains the #22 runtime primitive. A replacement candidate:

- starts untrusted;
- may receive prior findings only as context;
- inherits no verification authority;
- re-enters normalization, validation, qualification, verification, diagnostics, policy, and finalization;
- invalidates downstream state when its upstream propositions differ.

## Deferred and rejected concepts

Deferred until measured benefit exists:

- skills;
- subagents/multi-agent orchestration;
- specialist evidence-seeker/critic agents as first-class core concepts;
- generic workflow graph DSL.

Rejected:

- persisting hidden chain-of-thought as a runtime requirement;
- a parallel evidence-provider interface competing with #22 resolvers;
- letting policy, confidence, retrieval, or semantic judges manufacture verification authority;
- domain-specific policy rules in generic core.

## Implementation sequencing

This ADR creates two focused future implementation tracks:

1. #27 — `ReasoningPolicy` composition, capability/promotion rules, and dependency invalidation. **Implemented.**
2. #28 — `ReasoningThread` events, checkpoint/resume/fork, and deterministic reconstruction. **Implemented.**

Both control-plane tracks are now implemented. #13 soft-judge calibration is also implemented; typed soft findings may be recorded as non-authoritative thread observations but never gain replay authority.

## Research references

- OpenAI, “Unlocking the Codex harness: how we built the App Server”: https://openai.com/index/unlocking-the-codex-harness/
- OpenAI, “Running Codex safely at OpenAI”: https://openai.com/index/running-codex-safely/
- LangGraph persistence/interrupt/time-travel documentation: https://docs.langchain.com/oss/python/langgraph/persistence and https://docs.langchain.com/oss/python/langgraph/use-time-travel
- Claude Code Agent SDK/session documentation: https://docs.anthropic.com/en/docs/claude-code/sdk

These references motivate control-plane separation only. Reasoning Harness keeps its own provider-neutral correctness and authority model.
