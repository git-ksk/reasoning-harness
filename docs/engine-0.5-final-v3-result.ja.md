# Harness Engine 0.5.0 final-v3 cross-model result

**状態:** まだversion release前のHarness Engine 0.5.0 product candidateに対する、ACCEPTED semantic / release-gate evidence。

canonical coordinate:

- freeze tag: `engine-0.5-final-v3-freeze`
- freeze commit: `063833f38c38225109586b3db92348563b3822f8`
- product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- control commit: `12292b92bcd7890b3a81fc53b2523d172c2bde3a`
- GitHub Actions run: `35457038163`
- aggregate matrix: `docs/observations/engine-0.5-final-v3/engine-0.5-final-v3-matrix.json`
- provenance: `docs/observations/engine-0.5-final-v3/provenance.json`

canonical workflowはpreflight / final gateを含めSUCCESS。required 6 model rowすべてが独立にPASSし、cross-model平均や多数決は使用していない。

## Result

| Target | Model | Role | Cases | Correctness violations | Session replay | Result |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `mistral-14b` | `ministral-14b-latest` | affected required | 3/3 | 0 | 0 | PASS |
| `groq-qwen3.8-27b` | `qwen/qwen3.8-27b` | affected required | 3/3 | 0 | 0 | PASS |
| `mistral-8b` | `ministral-8b-latest` | validated reference | 3/3 | 0 | 0 | PASS |
| `google-gemini-3.5-flash-lite` | `gemini-3.5-flash-lite` | validated reference | 3/3 | 0 | 0 | PASS |
| `google-gemma-4-31b-it` | `gemma-4-31b-it` | validated reference | 3/3 | 0 | 0 | PASS |
| `groq-gpt-oss-120b` | `openai/gpt-oss-120b` | validated reference | 3/3 | 0 | 0 | PASS |

aggregateは **6/6 row accepted、18/18 case PASS、correctness-boundary violation 0、session external-call replay 0**。

## Final-hardeningで確認した改善

### #445 — finalization correctnessとplanner utilityの分離

`target_recalled`はplanner/path telemetryとして保持し、exact supportとgrounded exposureがfinalization correctness contractを満たしている場合の必須conjunctから外した。

canonical v3では、Mistral 14BとGPT-OSS 120BがAveriqで`target_recalled=false`でもPASSした。exact artifact support、grounded exposure、unsupported exposure 0、Harness-owned investigation materializationはすべて成立している。

### #446 — explicit-fact session continuityの決定論化

persist済みexplicit user factをmodelがprior claimとして再生成しなくてもcorrectionへ進める。Qwen 3.8 27Bでは`start_prior_grounded=false`、`start_prior_explicit_fact_persisted=true`の状態から、`harness_materialized_correction_target=true`、exact support、grounded exposure、replay 0で完了した。

Orivaneでは6 rowすべてがsupported exact `harness_session_correction_target_*` pathをexerciseした。

### #450 — admitted exact-fact investigation materializationの決定論化

validated read-only investigation actionが機械的に一意なexact structured factをadmitした後、Harnessが`Assumed` exact claimを追加し、`Supported`への昇格は既存structured verificationだけに任せる。

Averiqでは6 rowすべてがsupported exact `harness_investigation_admitted_fact_*` pathをexerciseした。以前失敗していたMistral 14B / Qwen 3.8 27Bもfresh answerable investigationをPASS。

Vardelisのfail-closed性も全rowで維持された。admitted resultなし、supported targetなし、grounded target exposureなし、unsupported exposed assertion 0。

## 過去の命名訂正

先に実行した `engine-0.5.1-hardening-v1-freeze` run `35450688516` は、Engine 0.5.0がまだversion releaseされていないことを確認する前に命名したもの。immutableなpre-release evaluation provenanceとして残すが、Engine 0.5.1 release / release candidateではない。そのfindingは今回のEngine 0.5.0 final-hardening successorへ取り込んだ。

## Release boundary

このacceptanceでEngine 0.5.0のsemantic/correctness gateはcloseできる。ただし、このresultのproduct candidate時点では`reasoning-harness-core`のpackaged Engine coordinateはまだ0.4.2。

残るのはmechanical release closeoutのみ。accepted product/evaluation stackをmergeし、split-version policyに従ってEngine coordinateを0.5.0へbumpし、status/docsを更新、deterministic release regressionを通してimmutableなversioned Engine 0.5.0 tag/releaseを作成する。
