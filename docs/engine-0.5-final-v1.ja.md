# Harness Engine 0.5.0 final cross-model acceptance v1

Issue #443 は Harness Engine 0.5.0 を締める最終evidence laneである。product実装は #248 / #282 / #283 までで完了済みで、このsurfaceではproduct codeを変更しない。live provider callより前にfreshなcompatibility matrixをfreezeし、各modelを独立に記録する。

## Product coordinate

- Engine 0.5.0 product candidate: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- immutable Engine 0.4.2 baseline: `d8940b4a98f11ec3e0968444fadc8bc90eae01ff`
- #283前control: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- evaluation時点のReason CLI: 0.5.2
- 別途versioned Engine 0.5.0 releaseでpackage coordinateを変更するまでは、installed/reported Harness Engineは0.4.2のまま

evaluation-only commitは `Cargo.toml` / `Cargo.lock` / `crates/` をproduct candidateとbyte-identicalに保つ。

## 最終delta acceptance

これは新しい一般model benchmarkではなく、immutableなv0.4.2 release gateを置き換えるものでもない。Engine 0.5.0で変わった意味境界を確認する。

1. **Finalization bridge** — #248由来のfresh 3ケース: exact grounded investigation、fail-closed no-result、session correction後のexact re-grounding + external replay 0。
2. **Harness-owned action materialization** — #283由来のfresh same-key-sibling 2ケース: grounded / stale。distinct sibling identity、relevant tool selection、correctness violation 0、`reason-investigation-intent-v1`、`target-intent-materialization-v1`、Harness materialization、legacy executable-action planner call 0を要求する。

repeated-trial reliabilityは再推定せず、freeze済み#282 `planner-reliability-v1`をsource of truthとする。今回のmatrixは各model 1 canonical observationのcompatibility / regression確認である。

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

validated 4 rowはそれぞれ独立にPASSが必要。model/provider平均、多数決、cross-model repairは使わない。observed / limited rowはevidenceであってrelease voteではない。

## Freeze / canonical policy

fresh identity `zqelora` / `zqmorin` / `zqaveth` / `zqneris` / `zqsolven` はsurface作成前に既存branch/tag historyとの衝突なしを確認済み。finalization base seedは `771101`、materializationは `771211`。

tag `engine-0.5-final-v1-freeze` でfixture、evaluator、role、seed、model matrix、workflowをcredential/live execution前にfreezeする。各targetのfirst live launchがcanonical。semantic failureは都合のよいrerun/rescoreをせず保存する。pre-live infrastructure retryは、そのtargetがlive-boundary markerを一度も書いていない場合だけ許可する。

各targetはcomponent reportとcomposite classificationを出す。

- `PASS`: 両componentがfreeze済みcontractを満たす。
- `FAIL`: reportは存在するが少なくとも一方が失敗。
- `INCOMPLETE`: canonical attempt後にcomponent reportが欠ける/parse不能。

`validated_required` rowは`PASS`だけがgateを満たす。README claimはraw reportとaggregate matrixを保存した後にだけ追加する。このstudyからSLA、population reliability、安定したlatency ranking、universal model compatibilityは主張しない。
