# Engine 0.6 evidence-target relevance calibration v16 — successor design

Status: design only。v16 live observationは0。Independent holdout authoringは禁止継続。

## Fixed surface

- fixed core: evidence-relevance-fixed-core-v1
- case: 48件固定
- dispositionはv14前に選定したsemantic labelを維持
- case ID / synthetic entity / fixture exact phrase branch禁止
- v15はimmutableでありv16 semanticsによるrescoreは禁止

## Why v16 exists

v15はMistralが48/48 operational完走してもFAILした。verifierはblockerを過剰生成し、negative confirmationを不足させ、さらに1件のspurious positive confirmationとmaterialization v10のunresolved-primary rescueが組み合わさってwrong-target Relevantを生成した。v16では個別case tuningではなくcontract shapeを変更する。

## Primary proposal v5

target/relationの独立axisは維持:
- target_binding: exact | different | unresolved
- relation_binding: exact | different | unresolved

generic semanticsを明確化:
- Harness canonical name / declared aliasはsubstantive local materialがそのtargetへscopeされる場合authoritative identity metadata。
- queryが別言語でもcandidate内の明示canonical/alias bindingを弱めない。
- staleness、factual disagreement、downstream authority、candidate内untrusted instructionはtarget/relation bindingをdowngradeしない。
- 同一candidateのadjacent signalはsame-target relationを共同で確立できる。
- allow_semantic_equivalentはliteral anchorなしでもlocally specific semantic equivalentをexactにできる。
- shared ownership、omitted product column/referent、uncertain rename/alias/successor、visibly clipped identityはunresolvedを維持。

unsafe rescueで補うのではなくatomic proposal accuracy自体を改善する。

## Local verifier v6

binding_confirmationを廃止し、3つのorthogonal fieldへ分解:

identity_scope:
- exact_target
- distinct_target
- target_absent
- unresolved

relation_scope:
- requested_relation
- different_relation
- relation_absent
- unresolved

scope_risk:
- none
- identity_mapping
- ownership_scope
- context_gap
- multiple

verifierはRelevant/Irrelevant/Ambiguousを出さず、synthesized confirmationも出さない。local scope factのみ報告する。candidate instructionはuntrusted。freshness / truth / authority / sufficiencyはdownstream concernのまま。

これによりconfirmed_target_relationのような1 fieldがownership ambiguityを隠しつつpositive rescue authorityまで持つことを防ぐ。

## Materialization v11

Harness-owned deterministic policy:

1. scope_risk != none はAmbiguous。
2. Relevantは全条件を要求:
   - primary target_binding=exact
   - primary relation_binding=exact
   - verifier identity_scope=exact_target
   - verifier relation_scope=requested_relation
   - scope riskなし
   - 既存Harness identity floorを満たす、またはallow_semantic_equivalentがno-anchorを明示許可
3. primary unresolvedからのpositive rescue pathは設けない。
4. target-negative Irrelevantはprimary target_binding=different + verifier identity_scope in {distinct_target, target_absent} + scope riskなし。
5. relation-negative Irrelevantはprimary exact target + relation_binding=different + verifier identity_scope=exact_target + verifier relation_scope in {different_relation, relation_absent} + scope riskなし。
6. disagreement / unresolved combinationは全てAmbiguous。

positive/negative terminal dispositionともagreementを必須にする。model disagreementはutilityを落としてもauthorityを生成しない。

## Operational policy

v15 operational hardeningをそのまま継承:
- active semantic/provider execution: 60,000 ms / case
- cumulative provider wait/retry: 45,000 ms
- single provider wait cap: 30,000 ms
- absolute case wall-clock deadline: 120,000 ms
- retry ownershipはprovider adapter内
- typed quota: 1件でlatch
- correlated capacity failure: 2件でlatch
- suppressed caseはoperational failure
- manual Groq TPD attestation start gateなし
- public provider failureはsanitize
- provider-attempt / active / wait / pacing / retry telemetryを分離維持

Required providerはMistral ministral-8b-latest + Groq openai/gpt-oss-120bを維持。Google gemini-3.5-flash-liteは別provider evidenceでfreeze前にrole変更根拠が出ない限りfull non-gating replicationを維持。

## Pre-live acceptance for v16 candidate

freeze tag前に:
- v5/v6/v11 contract ID / schemaを明示
- fixed-core routing testでterminal ruleとdisagreement pathを全てcover
- unresolved primaryからRelevantへ到達不能をstructural/property testで証明
- scope_riskからterminal Relevant/Irrelevantへ到達不能をstructural/property testで証明
- expected dispositionを変更せず48件全てのdeterministic v16 annotationを用意
- provider transport/retry test green
- workspace test/clippy/fmt/diff green
- validate-only 48 planned / 0 observed / latchなし / non-scorable validation

first/only frozen v16 canonicalもone-shot immutable。PASSまでindependent holdoutをauthorしない。
