# Harness Engine 0.5.0 final cross-model acceptance v2 — 結果

canonical run `35435026552` は freeze tag `engine-0.5-final-v2-freeze`、commit `9f816536bc948fd278fd3d2bcbb837e9649b5edb` からSUCCESSで完了した。accepted Engine 0.5.0 product candidateは `7a91d272af1bab0a97bf80ed7bba027ff253d50a` のままで、evaluation-only commitはcandidateに対して `Cargo.toml` / `Cargo.lock` / `crates/` を変更していない。

## 判定

**最終validated-row gate: PASS。** predeclareした `validated_required` 4 rowはすべて独立にPASSした。aggregate matrixの `required_failures` は空で、provider/model平均や多数決は使っていない。

| Provider / model | Role | Finalization | Harness materialization | Correctness | Composite |
| --- | --- | --- | --- | ---: | --- |
| Mistral / `ministral-8b-latest` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | 0 | **PASS** |
| Google / `gemini-3.5-flash-lite` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/3 | 0 | **PASS** |
| Google / `gemma-4-31b-it` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 5/5 | 0 | **PASS** |
| Groq / `openai/gpt-oss-120b` | validated required | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/4 | 0 | **PASS** |
| Mistral / `ministral-14b-latest` | observed characterization | FAIL 2/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | 0 | **FAIL** |
| Groq / `openai/gpt-oss-20b` | observed characterization | PASS 3/3; replay 0 | PASS; action path yes; legacy planner 0; intent/materialize 4/4 | 0 | **PASS** |
| Groq / `qwen/qwen3.8-27b` | observed characterization | report未完了 | PASS; action path yes; legacy planner 0; intent/materialize 6/6 | materialization完了範囲で0 | **INCOMPLETE** |
| NVIDIA / `nvidia/nemotron-3.5-lightning-30b-a3b` | limited negative control | report未完了 | operationally incomplete; protocol + timeout; observed model identityなし | materialization集計で0 | **INCOMPLETE** |

このclassificationはmodelごとに独立している。observed / limited rowはrelease voteではなく、既存model-catalog labelの定義も変更しない。catalogの`validated`定義は引き続きcanonical v0.4.2 provider identityに紐づく。

## 残った観測

`ministral-14b-latest` はfresh `zqelora` caseでgroundedかつartifact-supportedな回答を出し、unsupported exposureも0だったが、そのcaseで必要なtarget-recall telemetryをevaluatorが観測できなかった。このためfinalizationは2/3でrowは`FAIL`。materialization componentはcorrectness violation 0でPASSしている。

`qwen/qwen3.8-27b` は#283 materialization componentをPASSし、same-key sibling分離、legacy executable-action planner call 0、correctness violation 0を維持した。一方finalizationはfresh session-correction caseのsession startでunique grounded prior targetを確立できず中断したため、semantic correctness failureではなく`INCOMPLETE`とする。

NVIDIA negative controlはcurrent semantic roleとの非互換を維持した。finalizationはstructured-output fallback後もinvalid candidate JSONで`protocol` failure、materializationは`protocol` 1 + `timeout` 1でoperationally incomplete、observed model identityも0だった。これはcatalogの`limited / known_incompatible`と整合し、release-blockingではない。

## Frozen provenance

- v1 pre-live freeze: `engine-0.5-final-v1-freeze` / `7ce08128754c7d0f9f8ac783625518bf8601d7b5`
- v1 run `35434927007`: Rust 1.88 runnerに`rustfmt`がなくcredential前に停止。live provider observation = 0
- v2 freeze: `engine-0.5-final-v2-freeze` / `9f816536bc948fd278fd3d2bcbb837e9649b5edb`
- v2 freeze tag object: `566ac48081bb0999bd4241ce7a10af20f2999aef`
- canonical v2 run: `35435026552` — SUCCESS
- preflight job: `105876114468` — SUCCESS
- final-gate job: `105879807991` — SUCCESS

rowごとのjob ID、artifact ID、artifact ZIP SHA-256、raw component report、aggregate matrix、incomplete rowのfailure stderrは `docs/observations/engine-0.5-final-v2/` に保存している。

## Claim boundary

これはEngine 0.5.0 semantic delta（#248 finalization bridge、#283 Harness-owned action materialization）に対するfresh one-canonical-observation compatibility / regression gateである。#282 repeated-trial reliabilityは専用のfreeze済みevidenceをsource of truthとして維持する。SLA、population-level reliability、model ranking、universal compatibilityを主張するものではない。

このcoordinateのpackageはまだHarness Engine `0.4.2`をreportする。この結果はEngine 0.5.0 semantic acceptance milestoneのcloseを支持するが、installed binaryがEngine `0.5.0`をreportすると主張するには別途version/package releaseが必要である。
