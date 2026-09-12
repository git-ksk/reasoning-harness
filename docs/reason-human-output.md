# Reason human answer presentation

Reason CLI 0.5.0 makes the Harness-owned result understandable without exposing hidden reasoning or turning model prose into authority. Harness Engine 0.4.2 correctness semantics are unchanged.

## Human sections

The normal human natural-language path renders:

1. **Answer** — only the already-finalized exposed answer for grounded/qualified results. Unresolved, requires-verification, and contradictory results use Harness-owned fixed wording instead of renderer prose.
2. **Verified facts** — canonical `key=value` propositions only, and only when the final Harness artifact marks the claim `known` or `supported`. Candidate/claim free-form statements are not reused as factual authority.
3. **Unresolved / qualified** — canonical propositions that remain inferred, assumed, unknown, contradicted, uncovered, or affected by typed evidence-qualification findings.
4. **Evidence / sources** — only evidence IDs referenced by verified claims, with source and provenance class. Raw evidence observations are not dumped into the presentation.
5. **Untrusted context** — file/stdin/conversation context is listed separately by source and explicitly marked as not supporting authority.
6. **Acquisition / verification notes** — typed admission and resolver failures are translated to concise operational language. Internal reasoning traces are not shown.

The compact footer retains finalization status, verified/unresolved counts, factual coverage, and the answer-safety configuration identity.

## Interactive inspection

After a completed interactive turn:

```text
/status
/evidence
```

`/status` reprints the verified facts, unresolved/qualified items, typed acquisition notes, and compact status. `/evidence` reprints supporting provenance, untrusted-context source labels, and acquisition notes. These commands do not invoke a model/resolver and do not change epistemic state.

After `reason -c` or `reason -r`, these views are reconstructed from the persisted typed `ReasoningThread` checkpoint and its finalization rather than from a stored presentation transcript.

## Machine output

`--format json` remains the machine-oriented contract and is unchanged by this presentation layer. The human view is derived from existing typed state; it is not a new authority-bearing wire format.
