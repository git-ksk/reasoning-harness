# Exposed-text safety

Issue #210 closes the gap between the structured factual claims verified by Reasoning Harness and the text actually exposed to a user by the natural-language product path.

## Current policy

Machine identity: `harness-canonical-exposed-text-v1`.

`FinalAnswerCandidate` still contains both `text` and `factual_claims` for renderer compatibility, but they no longer have equal authority:

1. model-rendered `text` is advisory only;
2. `factual_claims` are checked against the final verified artifact under the existing grounded/uncertain rules;
3. if finalization may expose an answer, the exposed text is constructed by Harness code from the accepted factual claims and their modes;
4. target-local recovery text remains Harness-constructed from typed verified state;
5. renderer text is never copied into a `GroundedAnswer` or `QualifiedPartialAnswer`.

This makes an omitted declaration, contradictory renderer sentence, extra renderer-only factual assertion, or renderer-only certainty strengthening unable to change the guaranteed exposed answer surface. It does not make the model renderer authoritative and does not weaken evidence admission, qualification, verification, answer-safety, global verdict, or target-local recovery semantics.

## Product telemetry and wire contract

Natural-language JSON output uses `reason-natural-output-v3` and includes:

```json
"exposed_text": {
  "policy_id": "harness-canonical-exposed-text-v1",
  "renderer_text_exposed": false
}
```

`finalization.factual_claim_coverage` remains the structured-claim coverage metric. The `exposed_text` observation separately records how the user-visible text is produced, so structured coverage is no longer implicitly treated as a measurement of arbitrary renderer prose.

## Compatibility and migration

`reason-natural-output-v2` allowed verified structured claims and renderer `text` to diverge while still exposing the renderer text. Consumers that depended on natural renderer prose must not assume the same presentation under v3. They should treat `finalization.text` as the supported exposed answer and the structured artifact/claims as the inspectable correctness surface.

The exact wording of canonical text is presentation and may evolve under a future explicitly versioned policy; the authority rule must remain fail-closed.

## Reproducibility and rollback note

The pre-#210 behavior is reproducible at main commit `3a601c8` with `reason-natural-output-v2`. That baseline is retained only for historical reproduction. There is intentionally no runtime flag that re-enables model renderer text as grounded exposed output, because doing so would restore the P0 correctness gap.

If a future product change needs richer prose, it must receive a new exposed-text policy identity and must mechanically bind every exposed factual assertion to Harness-owned verified state before adoption.

Historical Stage-C, RSD2, and `product-external-info-v1/v2/v3/v4` observations are not rewritten. Their `unsupported grounded claims` metric remains a structured-claim metric unless an observation explicitly measured exposed-text safety.
