# Harness Engine 0.5.0 final acceptance v3

Engine 0.5.0 はまだpre-releaseである。package/coreのrelease coordinateは、このlineをcloseしてversioned Engine 0.5.0 releaseを切るまでEngine 0.4.2のまま維持する。

先に実行した `engine-0.5.1-hardening-v1-freeze` run 35450688516 は命名が早すぎた。これはimmutableなpre-release evidenceとしてのみ残し、Engine 0.5.1 release / release candidateとは扱わない。そこで得たfindingはEngine 0.5.0 final hardeningに属する。

## Product candidate

- pre-hardening control: `12292b92bcd7890b3a81fc53b2523d172c2bde3a`
- Engine 0.5.0 v3 product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- #446: explicit-fact session correction continuityをHarness側で決定論化
- #450: admitted exact-fact investigation materializationをHarness側で決定論化
- #445 evaluator semantics: finalization correctnessとplanner target-recall utilityを分離

product candidateより上のevaluation-only commitは `Cargo.toml` / `Cargo.lock` / `crates/` をcandidateとbyte-identicalに保つ。

## v3の目的

後続fresh hardening観測で、モデルへ再度言い直させていた2箇所が露出した。

- Qwen 3.8 27Bはexplicit session factをpersistできてもprior model claimへ再生成できずcorrection前に止まった。#446ではidentity continuityだけをHarness controlへ移し、verification authorityは従来のまま維持した。
- validated read-only investigation actionがexact structured evidenceをadmitした後も、Mistral 14B / Qwenは取得propositionをmodel regenerationで再度claim化する必要があった。#450ではselected capabilityとadmitted evidenceの対応が機械的に一意な場合だけHarness-owned `Assumed` claimを追加し、`Supported`/`Known`への昇格は既存structured verificationだけに任せる。

前回観測では#445の分離も確認できた。GPT-OSS 120Bは `target_recalled=false` でもexact supportとgrounded exposureが成立して正しくPASSした。

## Fresh v3 cases

- **Averiq**: answerable exact-key read-only investigation。targetは `engine050v3.averiq.route_endpoint = lane-947.sigma`。
- **Vardelis**: exact-key no-result。`engine050v3.vardelis.registry_token = registry-731` を捏造せずfail closedすること。
- **Orivane**: `window-1409`から`window-1423`へsession correction。persisted explicit-fact identity、typed invalidation、Harness-owned correction materialization、external replay 0を要求。

identity/valueはsurface作成前に既存refとの衝突がないことを確認済み。

## Required six-row matrix

6 rowすべてを独立requiredとする。provider/model平均、多数決、cross-model repairは禁止。

| target | provider/model | role |
| --- | --- | --- |
| `mistral-14b` | Mistral / `ministral-14b-latest` | affected required |
| `groq-qwen3.8-27b` | Groq / `qwen/qwen3.8-27b` | affected required |
| `mistral-8b` | Mistral / `ministral-8b-latest` | validated reference |
| `google-gemini-3.5-flash-lite` | Google / `gemini-3.5-flash-lite` | validated reference |
| `google-gemma-4-31b-it` | Google / `gemma-4-31b-it` | validated reference |
| `groq-gpt-oss-120b` | Groq / `openai/gpt-oss-120b` | validated reference |

全rowのbase seedは `823511`。provider pacing差は運用上の差だけである。

## Acceptance invariants

Averiq grounded case:

- admitted evidence >= 1;
- final artifactでexact targetが`Supported`/`Known`;
- supported exact `harness_investigation_admitted_fact_*` claimが存在し、#450 pathが実際にexerciseされたこと;
- exact targetをgrounded expose;
- unsupported exposed assertion = 0;
- planner `target_recalled`はtelemetryとして残すがcorrectness conjunctにはしない。

Vardelis:

- exact unsupported targetをfabricateしない;
- exact targetをgrounded exposeしない;
- unsupported exposed assertion = 0。

Orivane:

- start checkpointにcorrected keyのexplicit user factが一意にpersist;
- typed correction / invalidation eventが存在;
- correction後pending revalidation=false;
- external replay=0;
- corrected targetがexact supportedかつgrounded;
- supported exact `harness_session_correction_target_*` claimが存在;
- unsupported exposed assertion=0。

各row全体で3/3 PASS、correctness-boundary violation=0、session external-call replay=0を要求する。

## Freeze / observation policy

`engine-0.5-final-v3-freeze` でfresh case/config/evaluator/scoring/model matrix/seed/workflow/docs/checksumをcredential公開前にfreezeする。

- workflow rerunは禁止;
- 各targetのfirst live launchがcanonical;
- workflow failureは新successor identityを要求;
- historical v2および誤命名hardening-v1 evidenceはimmutable;
- raw reportとaggregate matrixを保存・reviewしてからmerge/version releaseへ進む。
