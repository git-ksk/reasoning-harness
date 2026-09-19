# Harness Engine 0.5.0 final cross-model acceptance v2 — result

Canonical run `35435026552` completed successfully from freeze tag `engine-0.5-final-v2-freeze` at commit `9f816536bc948fd278fd3d2bcbb837e9649b5edb`. The accepted Engine 0.5.0 product candidate remained `7a91d272af1bab0a97bf80ed7bba027ff253d50a`; evaluation-only commits did not change `Cargo.toml`, `Cargo.lock`, or `crates/` relative to that candidate.

## Disposition

**Final validated-row gate: PASS.** All four predeclared `validated_required` rows passed independently. `required_failures` is empty in the aggregate matrix; no provider/model averaging or majority vote was used.

| Provider / model | Role | Finalization | Harness materialization | Correctness | Composite |
| --- | --- | --- | --- | ---: | --- |
| Mistral / `ministral-8b-latest` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | 0 | **PASS** |
| Google / `gemini-3.5-flash-lite` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/3 | 0 | **PASS** |
| Google / `gemma-4-31b-it` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 5/5 | 0 | **PASS** |
| Groq / `openai/gpt-oss-120b` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/4 | 0 | **PASS** |
| Mistral / `ministral-14b-latest` | observed characterization | FAIL 2/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | 0 | **FAIL** |
| Groq / `openai/gpt-oss-20b` | observed characterization | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/4 | 0 | **PASS** |
| Groq / `qwen/qwen3.8-27b` | observed characterization | report not completed | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | 0 on completed materialization | **INCOMPLETE** |
| NVIDIA / `nvidia/nemotron-3.5-lightning-30b-a3b` | limited negative control | report not completed | operationally incomplete; protocol + timeout; no observed model identity | 0 on completed materialization accounting | **INCOMPLETE** |

These classifications are model-local. The observed/limited rows are not release votes and do not change the pre-existing model-catalog label definitions, which remain anchored to the canonical v0.4.2 provider identities.

## Residual observations

`ministral-14b-latest` produced a grounded and artifact-supported answer on the fresh `zqelora` case, with no unsupported exposure, but the evaluator did not observe the required target-recall telemetry for that case. That made the finalization component 2/3 and the row `FAIL`; the materialization component still passed with zero correctness violations.

`qwen/qwen3.8-27b` passed the #283 materialization component, including distinct same-key siblings, zero legacy executable-action planner calls, and zero correctness violations. Its finalization component aborted during the fresh session-correction case because session start did not establish the unique grounded prior target, so the row is `INCOMPLETE` rather than a semantic correctness failure.

The NVIDIA negative control remained incompatible with the current semantic roles. Finalization encountered invalid candidate JSON after the structured-output fallback (`protocol`), while the materialization run remained operationally incomplete with one `protocol` and one `timeout` failure and no observed model identity. This is consistent with the catalog's `limited / known_incompatible` status and is not release-blocking.

## Frozen provenance

- v1 pre-live freeze: `engine-0.5-final-v1-freeze` / `7ce08128754c7d0f9f8ac783625518bf8601d7b5`
- v1 run `35434927007`: stopped before credentials because Rust 1.88 on the runner lacked `rustfmt`; live provider observations = 0
- v2 freeze: `engine-0.5-final-v2-freeze` / `9f816536bc948fd278fd3d2bcbb837e9649b5edb`
- v2 freeze tag object: `566ac48081bb0999bd4241ce7a10af20f2999aef`
- canonical v2 run: `35435026552` — SUCCESS
- preflight job: `105876114468` — SUCCESS
- final-gate job: `105879807991` — SUCCESS

Per-row job IDs, artifact IDs, artifact ZIP SHA-256 values, raw component reports, the aggregate matrix, and failure stderr for incomplete rows are preserved under `docs/observations/engine-0.5-final-v2/`.

## Claim boundary

This is a fresh one-canonical-observation compatibility/regression gate over the Engine 0.5.0 semantic deltas (#248 finalization bridge and #283 Harness-owned action materialization), with #282 repeated-trial reliability retained from its own frozen evidence. It is not an SLA, a population-level reliability estimate, a model ranking, or proof of universal compatibility.

The package still reports Harness Engine `0.4.2` at this coordinate. This result supports closing the Engine 0.5.0 semantic acceptance milestone; a separate version/package release is required before claiming that installed binaries report Engine `0.5.0`.
