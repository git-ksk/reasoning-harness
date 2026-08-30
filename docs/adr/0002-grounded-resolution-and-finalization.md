# ADR-0002: Grounded resolution and finalization loop

- Status: Accepted
- Date: 2026-08-30

## Context

ADR-0001 established that the native harness runtime owns the execution protocol around a stochastic model. The implementation has since developed strong verification and diagnostic boundaries: harness-owned evidence, trusted verification receipts, typed contradiction/counterexample findings, evidence-aware causal diagnostics, assumption diagnostics, metamorphic robustness, and repeated-trial stability reporting.

Those capabilities make the harness effective at identifying unsupported, contradicted, or unresolved intermediate reasoning. They do not yet define the complete product loop from an unresolved finding to a grounded final answer.

If `accept | reject | unknown` is treated as the end of execution, the project risks becoming primarily an evaluator or post-hoc diagnostic tool. The intended product direction is broader: use verified intermediate state to control whether a stochastic model may continue toward a final answer, request additional evidence when support is insufficient, and ensure final rendering does not silently add unsupported claims.

## Decision

Reasoning Harness will evolve toward an **evidence-grounded reasoning runtime**.

The runtime owns the protocol that turns a task plus harness-owned evidence into a verified reasoning state and, when policy permits, a grounded final answer. A model remains a replaceable candidate generator and optional renderer. It never becomes the authority for evidence, verification, or final epistemic status.

Conceptually, the target loop is:

```text
task + harness-owned evidence
          |
          v
candidate generation
          |
          v
ground + verify + diagnose
          |
          +--> supported enough --------------------+
          |                                         |
          +--> unresolved / insufficient support    |
          |          |                              |
          |          v                              |
          |     resolution request                  |
          |          |                              |
          |     external evidence / verifier        |
          |          |                              |
          |          v                              |
          |     regenerate or revise                |
          |          |                              |
          |          +----> re-run harness ---------+
          |
          +--> refuted --> discard/revise --> re-run
                                                    |
                                                    v
                                               finalization
                                                    |
                                                    v
                                          grounded final answer
```

`unknown` remains a valid epistemic result. The runtime may stop with `unknown` or abstain when the resolution budget is exhausted, no trusted resolver is available, or policy forbids another attempt.

## Resolution boundary

A diagnostic finding is not itself permission to fetch arbitrary information or trust a new model assertion. The runtime should convert unresolved state into a typed resolution request that describes what is missing without pretending to know the answer.

A future provider-neutral resolution contract should be able to express requests such as:

- obtain trusted evidence for a proposition;
- obtain evidence for or against a causal relation;
- resolve a temporal, scope, or provenance mismatch;
- verify a proposition with an external deterministic oracle;
- revise or regenerate an inference after a hard refutation;
- request explicit human review when policy allows it.

The core runtime owns the request, budget, state transition, and re-verification semantics. Retrieval systems, web search, databases, MCP servers, compilers, test runners, human review systems, and domain-specific tools remain adapters outside the trusted core unless they return data through an explicitly trusted evidence/verifier boundary.

## Evidence acquisition is not authority

Returning data from a resolver does not automatically make that data trusted.

- Retrieval is an acquisition mechanism, not a verifier.
- A model-generated citation is not trusted evidence merely because it names a source.
- Candidate-authored provenance cannot elevate authority.
- A resolver must return evidence with provenance and authority metadata defined by the harness-owned input or verifier policy.
- Missing evidence remains `unknown`; the runtime must not fabricate completion to satisfy a resolution request.

This preserves the same authority boundary used by existing verification receipts and deterministic diagnostics.

## Repair and regeneration boundary

The runtime may ask a model to revise or regenerate a candidate after diagnostics, but the new candidate starts untrusted.

A repair loop may use prior findings as guidance, but it cannot:

- promote a soft finding into hard truth;
- preserve a previously rejected claim as verified merely because the model repeats it;
- allow the model to create trusted evidence or receipts;
- bypass validators, policy, or re-verification after revision.

Every repaired candidate crosses the same normalization, validation, verification, diagnostic, and policy boundaries as the original candidate.

## Finalization boundary

Final answer generation is a distinct phase from reasoning verification.

The target finalization contract is:

```text
verified ReasoningArtifact
          |
          v
answer renderer / optional model
          |
          v
claim coverage check
          |
          v
grounded final answer | unknown/abstain
```

A renderer may summarize, reorder, simplify, or adapt style, but it must not upgrade epistemic status or introduce unsupported factual claims.

The runtime should eventually verify that factual propositions in the rendered answer are covered by supported artifact propositions or are explicitly presented as assumptions/uncertainty according to policy. A renderer that introduces a new factual proposition must send that proposition back through the normal reasoning and verification loop rather than silently adding it to the final answer.

## Policy and termination

Resolution is bounded. The runtime owns explicit limits such as:

- maximum resolution attempts;
- model/token/time budgets;
- allowed resolver classes;
- required authority level;
- whether human review is allowed;
- whether unresolved claims force abstention or allow a qualified answer.

Budget exhaustion is not evidence. When the runtime cannot obtain sufficient support within policy, the correct outcome is `unknown`, a qualified partial answer, or abstention according to explicit policy.

## Research requirements

The resolution loop must be evaluated separately from raw diagnostic accuracy.

Important measurements include:

- answerable-case recovery: how often initially unresolved cases become supported after resolution;
- unsafe-final-answer rate: unsupported or contradicted factual claims that reach final output;
- evidence acquisition efficiency: additional calls/tokens/latency required per recovered case;
- resolution convergence: attempts required before supported, refuted, or exhausted;
- regression against direct generation and diagnose-only baselines;
- finalization coverage: proportion of final factual claims bound to supported artifact propositions.

Improving answerability is useful only if unsafe final answers do not increase.

## Consequences

Positive:

- the project remains more than a benchmark or post-hoc judge;
- existing diagnostics become actionable control signals inside a runtime loop;
- domain-specific retrieval can be integrated without moving domain logic into core;
- verified reasoning state becomes the source of truth for final answer construction;
- smaller/cheaper models can be evaluated on whether the surrounding protocol recovers grounded answers safely.

Costs:

- the runtime needs explicit resolution state, budgets, and termination semantics;
- finalization requires proposition coverage rather than prose-only rendering;
- live research becomes more expensive because recovery loops add model/tool calls;
- product claims must distinguish diagnose-only capability from implemented end-to-end grounded resolution.

## Non-goals

- making arbitrary open-world claims mathematically proven;
- embedding a general web crawler or RAG system into the core runtime;
- trusting an LLM judge, retriever, or renderer as a correctness authority;
- forcing every `unknown` case to resolve;
- hiding uncertainty to maximize answer rate;
- turning the harness into a generic agent framework.

## Relationship to ADR-0001

ADR-0001 remains authoritative for interface and packaging boundaries. This ADR clarifies what the native runtime is ultimately expected to own inside that boundary: not only diagnosis and `accept | reject | unknown`, but the bounded protocol for resolution, re-verification, and grounded finalization.
