# Harness Engine 0.5.1 cross-model hardening acceptance v1

Issue #445 / #446 は、close済みHarness Engine 0.5.0 semantic baselineをretroactiveに書き換えないfollow-upである。freeze済みEngine 0.5.0 resultはimmutableのまま保持する。

2つのgapだけをauthority / correctnessを弱めずに検証する。

1. Finalization evaluator semantics (#445): planner target_recalledはpath / utility telemetryとして残す。一方、grounded finalizationはadmitted evidence、exact supported artifact state、exact grounded exposed output、unsupported-exposure safetyで判定する。
2. Session explicit-fact continuity (#446): persisted explicit_user_factが同じkeyで一意なら、訂正対象propositionのidentityをHarnessが決定論的に保持できる。修正後propositionはAssumed candidate claimとしてのみmaterializeし、通常のstructured verificationを通った場合だけSupported / groundedへ昇格できる。

## Product coordinate

- Engine 0.5.0 accepted control: 12292b92bcd7890b3a81fc53b2523d172c2bde3a
- Engine 0.5.1 hardening product candidate: 19ef4ac58cd1a3157313ace2e3f45ee7e36f29e5
- product change: PR #447 / Issue #446
- evaluator semantics prototype: PR #448 / Issue #445
- このevaluation surfaceではrelease/package versionを変更しない

product candidateより上のevaluation-only commitは Cargo.toml / Cargo.lock / crates/ をcandidateとbyte-identicalに保つ。

## Fresh cases

surface作成前に既存refとの衝突がないことを確認したfresh identityを3件使う。

- Velquor: answerable exact-key investigation。engine051.velquor.route_endpoint = lane-913.theta
- Tarvess: exact-key no-result。fail closed必須
- Oryndel: explicit fact window-1303からsessionを開始し、window-1319へcorrect。typed invalidation、external replay 0、exact verified grounding、unsupported exposure 0を要求

Oryndelのstart preconditionは一意なpersisted explicit-user-fact identityであり、model-generated prior claimではない。start_prior_groundedはmodel behavior telemetryとして別途残す。

## Required model matrix

6 rowすべてを独立requiredとする。provider/model平均、多数決、cross-model repairは使わない。

| target | provider/model | role |
| --- | --- | --- |
| mistral-14b | Mistral / ministral-14b-latest | affected required |
| groq-qwen3.8-27b | Groq / qwen/qwen3.8-27b | affected required |
| mistral-8b | Mistral / ministral-8b-latest | validated reference |
| google-gemini-3.5-flash-lite | Google / gemini-3.5-flash-lite | validated reference |
| google-gemma-4-31b-it | Google / gemma-4-31b-it | validated reference |
| groq-gpt-oss-120b | Groq / openai/gpt-oss-120b | validated reference |

全rowのbase seedは812411。providerごとのpacing差は運用上の差だけである。

## Acceptance invariants

各rowが3 caseすべてを完走し独立にPASSすること。

Grounded investigationはadmitted evidence、exact artifact support、exact grounded exposure、unsupported exposed assertion = 0を要求する。planner target_recalledは記録するがfinalization correctness conjunctにはしない。

Fail-closed investigationはexact supportやgrounded exposureを捏造せず、unsupported exposed assertion = 0を維持する。

Session correctionはstart checkpointにcorrected keyのexplicit_user_fact valueがちょうど1つpersistされていること、typed correction / invalidation event、correction後pending revalidation=false、external replay=0、corrected targetのexact support / grounding、unsupported exposure=0を要求する。modelがstart valueをgroundしたかは別telemetryとして残す。 final artifactにはharness_session_correction_target_ identityのexact supported Harness-owned correction claimも必須とし、deterministic pathが実際にexerciseされたことを証明する。

row全体でcorrectness-boundary violationとsession external-call replayはいずれも0必須。

## Canonical observation policy

tag engine-0.5.1-hardening-v1-freeze でcase、config、evaluator/scoring semantics、6-row model matrix、seed、workflow、checksumをcredential公開前にfreezeする。

- 各targetのfirst live launchがcanonical;
- workflow rerunは禁止;
- pre-live infrastructure failureを含むworkflow failureはすべて新しいsuccessor identityを要求;
- historical Engine 0.5.0 evidenceは変更・再classificationしない;
- merge / compatibility claimより前にraw report、stderr/stdout、attempt marker、job/artifact coordinate、aggregate matrixを保存する。

これは限定されたhardening deltaのacceptanceであり、SLA、model ranking、population-level reliability estimateではない。
