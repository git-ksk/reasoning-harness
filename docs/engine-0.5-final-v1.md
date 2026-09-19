# Harness Engine 0.5.0 final cross-model acceptance v1

Issue #443 is the final evidence lane for Harness Engine 0.5.0. Product implementation is complete through #248, #282, and #283; this surface changes no product code. It freezes one fresh compatibility matrix before any live provider call and reports each model independently.

## Product coordinate

- Engine 0.5.0 product candidate: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- immutable Engine 0.4.2 release baseline: `d8940b4a98f11ec3e0968444fadc8bc90eae01ff`
- pre-#283 control: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- Reason CLI at evaluation time: 0.5.2
- installed/reported Harness Engine remains 0.4.2 until a separate versioned Engine 0.5.0 release changes the package coordinate

Evaluation-only commits must leave `Cargo.toml`, `Cargo.lock`, and `crates/` byte-identical to the product candidate.

## Final delta acceptance

This is not a new general model benchmark and does not replace the immutable v0.4.2 release gate. It covers the Engine 0.5.0 semantic deltas:

1. **Finalization bridge** — three fresh #248-derived cases: exact grounded investigation, fail-closed no-result, and session correction with exact re-grounding and zero external replay.
2. **Harness-owned action materialization** — two fresh #283-derived same-key-sibling cases: grounded and stale. Candidate acceptance requires distinct sibling identity, relevant tool selection, zero correctness violations, `reason-investigation-intent-v1`, `target-intent-materialization-v1`, Harness materialization, and zero legacy executable-action planner calls.

Repeated-trial reliability is not re-estimated; frozen #282 `planner-reliability-v1` remains the source of truth for that question. This matrix is one canonical compatibility/regression observation per model.

## Model matrix

| target | provider/model | catalog role | Engine 0.5.0 gate |
| --- | --- | --- | --- |
| `mistral-8b` | Mistral / `ministral-8b-latest` | validated | required |
| `mistral-14b` | Mistral / `ministral-14b-latest` | observed | characterization |
| `google-gemini-3.5-flash-lite` | Google / `gemini-3.5-flash-lite` | validated | required |
| `google-gemma-4-31b-it` | Google / `gemma-4-31b-it` | validated | required |
| `groq-gpt-oss-120b` | Groq / `openai/gpt-oss-120b` | validated | required |
| `groq-qwen3.8-27b` | Groq / `qwen/qwen3.8-27b` | observed | characterization |
| `groq-gpt-oss-20b` | Groq / `openai/gpt-oss-20b` | observed | characterization |
| `nvidia-nemotron` | NVIDIA / `nvidia/nemotron-3.5-lightning-30b-a3b` | limited / known incompatible | bounded negative control |

All four validated rows must PASS independently. There is no model/provider averaging, majority vote, or cross-model repair. Observed and limited rows are evidence, not release votes.

## Freeze and canonical policy

Fresh identities `zqelora`, `zqmorin`, `zqaveth`, `zqneris`, and `zqsolven` were collision-checked against existing branch/tag history before surface creation. Finalization uses base seed `771101`; materialization uses `771211`.

Tag `engine-0.5-final-v1-freeze` freezes fixtures, evaluators, roles, seeds, model matrix, and workflow before credentials/live execution. The first live launch per target is canonical. Semantic failure is preserved without convenience rerun/rescore. Pre-live infrastructure retry is allowed only when that target never crossed its live-boundary marker.

Each target produces component reports plus one composite classification:

- `PASS`: both components satisfy their frozen contracts.
- `FAIL`: reports exist but at least one contract fails.
- `INCOMPLETE`: a component report is absent/unparseable after the canonical attempt.

Only `PASS` satisfies a `validated_required` row. README claims are added only after raw reports and the aggregate matrix are preserved. This study does not establish an SLA, population reliability, stable latency ranking, or universal model compatibility.
