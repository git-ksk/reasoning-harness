# Product external-information v4 cross-model replication

Issue #208 replicates the frozen `product-external-info-v4` matched-context four-arm measurement across additional model families. This is post-observation replication only: the v4 corpus, target propositions, scoring, evaluator semantics, MCP boundary, admission policy, and finalization logic remain byte-for-byte frozen from semantic head `e324ccbff6e818d205a734f06ccc8cac4b587588`.

## Models

- Mistral `mistral-small-latest`
- Google-hosted `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`

The canonical Ministral 8B result from run `34000216929` remains the original v4 observation and is used only as the reference row.

## Fixed conditions

- corpus: `product-external-info-v4`
- evaluator: `reason-product-external-info-v4`
- comparison: `matched-target-context-four-arm-v4`
- seed: `28000`
- max tokens: `1024`
- 21 cases: 18 semantic + 3 typed operational
- within each model run, arm 3 and arm 4 share one decoded acquisition snapshot per case
- raw and Harness arms receive the same task, exact target hypothesis, evidence requirement, and authority policy

The live external acquisition is repeated per model run because v4 has no post-freeze snapshot-injection interface. Cross-model conclusions therefore compare the same frozen endpoints and semantic contract, while exact external response bytes are guaranteed matched only within each arm-3/arm-4 pair, not across provider runs. A model-free acquisition preflight must pass immediately before the matrix begins.

## Reporting

For each model, report arm 3 (`raw_model_with_external`) versus arm 4 (`harness_with_mcp_external`): expected-grounded target coverage, expected-unknown preservation, false target abstention, unsupported grounded claims, missed target insufficiency, Harness unsafe-admission/authority-promotion counters, typed rejection telemetry, model token usage, model latency, accounted end-to-end latency, and operational failures.

Provider/protocol failures remain separate from semantic scores. No result may be repaired by changing v4 after observation.
