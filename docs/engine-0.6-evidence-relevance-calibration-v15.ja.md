# Engine 0.6 evidence-target relevance calibration v15

状態: immutable v14 FAIL の pre-freeze successor。v15 live model observation はまだ0回。independent holdout authoringは禁止継続。

## 固定 evaluation surface

v15でも evidence-relevance-fixed-core-v1 48件を変更しない。v14のmodel missを理由にcase追加・削除・選別しない。dispositionは14 Relevant / 18 Irrelevant / 16 Ambiguousのまま。

semantic contract:
- primary binding proposal: reason-evidence-relevance-binding-proposal-v4
- independent verifier: reason-evidence-local-qualification-v5
- Harness materializer: target-evidence-relevance-binding-materialization-v10

v15 verifierの出力は2 fieldだけ:
- blocking_reason: none / identity_mapping / ownership_scope / context_gap / multiple
- binding_confirmation: none / confirmed_target_relation / confirmed_distinct_target / confirmed_different_relation / confirmed_local_absence

v14のmodel-authored explicit_local_absence を廃止する。primary negative target binding単独ではIrrelevantを確定できず、matching one-sided confirmationを要求する。primaryがpositive方向へunresolvedでも、independent verifierがexact target/relationをconfirmしHarness-owned identity floorを満たす場合だけRelevantへ救済できる。concrete blockerは常にAmbiguousへfail-closed。

Harness canonical name / alias matchはASCII aliasについてboundary-aware化する。anchorはidentity floor/checkであり、requested relationがtargetに属する独立証明にはしない。

## Operational budget v15

retry ownershipはprovider adapter内部に維持する。second retry loopをrunnerへ追加せず、既存bounded retryも削除しない。

1 caseのbudget:
- semantic/provider active execution: frozen policyの60,000 ms
- cumulative provider wait/retry: 45,000 ms
- single provider wait: 30,000 ms
- absolute case wall-clock: 120,000 ms

Mistral / Groq / Googleは以下をexecution telemetryとして分離:
- provider attempt started/completed
- active execution
- pacing wait
- retry wait

single/cumulative wait budgetを超えるprovider waitはsemantic deadlineまでsleepせずtyped rate_limitで終了する。daily quotaはtyped quotaのままshort-window transientとしてretryしない。

run-level:
- typed quota failure 1件でprovider armをlatchし、残りの確実なfailure callを抑止
- capacity failure (rate_limit, provider unavailable, transport/timeout, assessment/absolute timeout) 2件連続でprovider armをlatch
- suppressed caseはoperational failureのままでcanonical PASS不可

public artifactへ保存するrunner failureはorganization/project/account identifierやbilling URL等をsanitizeする。

## Groq full-run capacity gate

v14の48-case Mistral armは96 model callでtotal 67,610 tokensを使用した。v15 canonical前のGroq required daily headroomはconservativeに160,000 tokensとする。

tiny probe成功だけではheadroom証明にならない。canonical preflightはrepository variableを必須とする:

ENGINE_0_6_RELEVANCE_V15_GROQ_CAPACITY_ATTESTATION=engine-0.6-evidence-relevance-calibration-v15-freeze:160000

これはexact freeze tag向けoperator attestation。authoritative quota stateまたは他利用を隔離したfresh reset windowで160k以上を確保した場合のみ設定する。daily remaining capacityを確定できないならcanonicalを開始しない。

## Provider set / acceptance

provider roleはv14から変更しない:
- required: Mistral ministral-8b-latest
- required: Groq openai/gpt-oss-120b
- non-gating full replication: Google gemini-3.5-flash-lite

required armは48/48 provider success、provider-arm latchなし、operational abortなし、attempt telemetry completeが必須。

さらに各required armで:
- wrong-target Relevant 0
- false relevance rejection 0
- Relevant -> Ambiguous 0
- utility miss 0
- materialized exact 48/48
- blocker miss 0
- spurious blocker 0
- binding-confirmation miss 0
- spurious binding confirmation 0

first/only frozen canonicalはrerun / rescore / relabel / retagしない。PASSまでholdout authoringは禁止。
