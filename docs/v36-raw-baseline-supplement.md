# v36 supplemental raw-model baseline

[日本語](v36-raw-baseline-supplement.ja.md) | English

This is a **post-release supplemental comparison**, not a modification or rescore of the frozen v0.4.2 v36 release acceptance.

The purpose is to answer a simpler product question that the original v36 release gate did not measure directly:

> On the same 13 frozen v36 tasks, what changes when a model receives the raw task/context without Harness-owned admission, verification, and final-answer authority?

## Frozen source

The case surface is derived mechanically from:

- tag: `natural-language-e2e-v36-freeze`
- freeze commit: `57bea659d472a103cc48d86ddee7dfe4a41de790`
- v0.4.2 candidate: `9497b563ad914fada13d33e0c1a7fee549a1f1de`
- seed: `738214`
- 13 cases: 8 expected-grounded, 5 expected-unknown

The original v36 acceptance remains immutable. Its canonical Harness artifacts are referenced by Actions run ID and SHA-256 in `evaluation/v36-raw-baseline/harness-reference-v1.json`.

## What the raw arm sees

The raw arm receives:

1. the exact frozen user task;
2. a deterministic snapshot of the configured raw acquisition outputs or session-state transitions;
3. no expected target value except where that value naturally occurs inside the raw observation itself.

For investigation cases, the snapshot includes the configured resolver observations (including stale/scope/authority/source metadata and typed `no_result` observations). For the MCP case it includes the exact pinned `Cargo.toml` content as raw tool context. For session cases it includes the same explicit user-supplied state changes.

This intentionally removes tool-selection difficulty from the raw arm. The comparison therefore focuses on **final-answer utility and safety**, not planner quality.

## Metrics

| Metric | Meaning |
| --- | --- |
| v36 target-contract coverage | Of the 8 cases labeled `grounded` by the frozen v36 evaluator, how many outputs matched the evaluator-owned target value exactly? Higher is better on this synthetic surface. This is not a general open-world answer-accuracy metric. |
| Target-contract miss / false target abstention | Frozen `grounded` cases that did not reach the evaluator-owned target value. Lower is better for v36 utility. |
| Expected-unknown preservation | Of the 5 cases where the supplied raw observation is stale, wrong-scope, insufficient-authority, wrong-source, or otherwise non-authoritative, how many remained unknown? Higher is better. |
| Missed target insufficiency | Expected-unknown cases where the arm still gave a definite answer. Lower is better. |
| Wrong confident answer | Any definite raw answer that does not equal the expected target value. Lower is better. |

The original v36 planner metrics (`target recall`, `tool selection`, `trigger exposure`, `avoidable stall`) remain part of the frozen v36 release evidence and are **not** redefined by this supplement.

## Canonical Harness reference

Across the four preserved v36 candidate artifacts (Mistral, Groq, Gemini 3.5 Flash-Lite, Gemma 4 31B), the final-answer reference is identical on this 13-case interpretation:

- expected-grounded target coverage: `1/8 = 0.125`
- false target abstention: `7`
- expected-unknown preservation: `5/5 = 1.0`
- missed target insufficiency: `0`
- unsupported exposed assertions: `0`
- unsupported structured claims: `0`

That is deliberately conservative. The supplemental raw measurement is intended to show whether removing the Harness increases v36 target-contract coverage, unsafe certainty, or both. Because v36 contains synthetic planner/follow-up identities and values, this coverage must not be generalized into overall question-answer accuracy.

## Interpretation rule

This comparison must not be described as a new v36 release result. It is a later, separately identified product observation over the same frozen task surface.

A useful raw result is not automatically a safer result, and a safer Harness result is not automatically a more useful result. Report the utility and safety metrics side by side.
