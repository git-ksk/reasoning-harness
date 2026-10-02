# Engine 0.6 evidence-target relevance calibration v16 — successor design

Status: implemented pre-freeze candidate `566a4b5a39ad937d9f43d006550ad7a84436a0f8`。v16 live observationは0。Independent holdout authoringは禁止継続。

## Fixed surface

- fixed core: evidence-relevance-fixed-core-v1
- case: 48件固定
- dispositionはv14前に選定したsemantic labelを維持
- case ID / synthetic entity / fixture exact phrase branch禁止
- v15はimmutableでありv16 semanticsによるrescoreは禁止

## Why v16 exists

v15はMistralが48/48 operational完走してもFAILした。verifierはblockerを過剰生成し、negative confirmationを不足させ、さらに1件のspurious positive confirmationとmaterialization v10のunresolved-primary rescueが組み合わさってwrong-target Relevantを生成した。v16では個別case tuningではなくcontract shapeを変更する。

## Primary proposal v5

Contract: `reason-evidence-relevance-binding-proposal-v5`。

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

Contract: `reason-evidence-local-qualification-v6`。Annotation protocol: `evidence-relevance-scope-verifier-v16`。

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

Policy: `target-evidence-relevance-binding-materialization-v11`。

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
5. explicit local absenceは、primaryの両axisがexact supportを主張せず、verifierがidentity_scope=target_absent + relation_scope=relation_absentを独立に返す場合もIrrelevantへmaterialize可能。これはnegative fail-closed evidenceでありpositive rescue authorityではない。
6. relation-negative Irrelevantはprimary exact target + relation_binding=different + verifier identity_scope=exact_target + verifier relation_scope in {different_relation, relation_absent} + scope riskなし。
7. その他のdisagreement / unresolved combinationはAmbiguous。

positive terminal dispositionはprimary/verifier完全agreementを必須にする。negative terminal dispositionはmatching negative scope evidenceまたはexplicit local absenceを要求し、unresolved primaryはpositive authorityを生成しない。model disagreementはutilityを落としてもauthorityを生成しない。

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

## Deterministic v16 annotation surface

fixed 48 caseのdispositionは14 Relevant / 18 Irrelevant / 16 Ambiguousを維持。expected verifier coverageを事前固定:
- identity_scope: exact_target 19 / distinct_target 10 / target_absent 5 / unresolved 14
- relation_scope: requested_relation 37 / different_relation 4 / relation_absent 5 / unresolved 2
- scope_risk: none 32 / identity_mapping 5 / ownership_scope 1 / context_gap 5 / multiple 5

v15からcase追加・relabelなし。

## Pre-live validation

freeze tag前のimplemented candidate validationはgreen:
- v16 fixed-core routing: 9/9 PASS
- evidence-relevance CLI runner: 22/22 PASS
- full workspace test: PASS、provider library最終行 153 passed / 0 failed / 1 ignored
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo fmt --all -- --check`: PASS
- `git diff --check`: PASS
- workflow YAML parse: PASS
- validate-only: 48 planned / 0 observed / `operational_abort=null` / `provider_arm_latch=null` / `validate_only_non_scorable`
- deterministic expected v16 annotationはexpected dispositionを変更せず48/48 exact materialization
- structural testでunresolved primaryからRelevantへ到達不能、non-none scope riskは必ずAmbiguousを証明

first/only frozen v16 canonicalもone-shot immutable。PASSまでindependent holdoutをauthorしない。
