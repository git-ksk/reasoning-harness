# Engine 0.6 evidence relevance holdout successor v5

Status: immutable holdout v4 FAIL後のactive design / implementation candidate。

## Versioning

- effective qualification: v7
- materialization: v20
- historical v1-v4 qualification/materialization semanticsは変更しない
- immutable v4 observationはregression replay専用。historical artifactは書き換えない

## Successor rule

1. navigation-only target + stable repeated sibling owner:
   - strict Harness identity policy
   - deterministic scope risk none
   - target canonical name/aliasはnavigation/footerにだけ存在し、identity-capable local signalやcanonical URLには存在しない
   - sibling-looking subjectがheading/title系とbody/fact系の別signalで反復される
   - targetとidentity-family tokenを共有しつつ、non-genericな識別tokenを持つ
   - requested relationがlocalに存在する
   - v7は distinct_target / requested_relation を確定可能
   - advisory stageがexactへ誤投票してもv20はIrrelevantへmaterialize可能

2. explicit local separation + stable repeated sibling owner:
   - deterministic scope risk none
   - local textが反復siblingをHarness targetとは別service/productと明示する、または同等の明示的non-mappingを示す
   - requested relationがその反復siblingにlocalに存在する
   - v7はexact-vs-distinct advisory disagreementを越えてdistinct-target ownershipを確定可能
   - single unscoped different-nameだけでは不十分

3. deterministic target/relation absence:
   - deterministic scope risk none
   - bounded local unitがexact-target不在またはtarget-specific requested-relation不在を明示
   - v7は target_absent / relation_absent
   - 両advisory stageがoperationalに存在し、local qualifier側もtargetまたはrelation absenceを独立に報告した場合のみv20はIrrelevantへmaterialize可能
   - exact/exact positive conflictでqualifier-side absenceが無い場合はdeterministic Irrelevantへ落とさない
   - これはcandidate-local rejectionでありglobal product absenceの主張ではない

4. context-gap relation floor:
   - clipping / truncation / omitted local contextはtyped context_gapを維持
   - advisory relation bindingがunresolvedなら、missing contextを relation_absent にせずeffective relationをunresolvedへ戻す
   - terminal dispositionはAmbiguousを維持

5. 既存positive / ambiguity floor:
   - successor v4のrepeated authorized positive identityは維持
   - single-signal near siblingはAmbiguous
   - URL-only unnamed ownershipはAmbiguous
   - model outputはauthorityを作らずdeterministic scope riskを迂回できない

model stageは追加しない。

## Provider development protocol

Groq TPDをiteration bottleneckにしないため、successor developmentとfinal cross-provider validationを明確に分離する。

Development loop:
- required live provider: Mistral ministral-8b-latest + Google gemini-3.5-flash-lite
- Groqはcandidate-shaping live iterationから除外
- immutable Groq observationはoffline replay/regressionにのみ利用可
- development fixture/runはholdout evidenceではなく、#462 acceptanceを満たさない
- live developmentで繰り返し観測したfixtureは将来holdoutには使わない

Candidate freeze:
- generic deterministic controls PASS
- immutable v23 / holdout v1-v4 replayをsuccessor semanticsでPASS
- Mistral + Google development corpusでwrong-target Relevant 0かつrequired materialization/authority gate exact
- workspace tests / Clippy / rustfmt / workflow YAML / production special-case scan PASS
- 次のindependent holdout authoringより先にsuccessor semanticsをfreeze

Final validation:
- semantics freeze後にのみfresh independent holdout v5をauthor
- required providerはMistral + Google + Groqへ戻す
- Groqはiterative tuningではなく独立cross-provider validationとしてこの段階で使う
- canonical attemptは1回のみ。FAILはimmutable
- canonical直前のGroq admission/readiness確認は可。ただしholdout observationを先に消費しない

## Current offline regression evidence

- generic successor-v5 controls: 8/8 PASS
- immutable v23 replay: 48/48 x 3
- immutable holdout v1 replay: 26/26 x 3
- immutable holdout v2 replay: 26/26 x 3
- immutable holdout v3a replay: 26/26 x 3
- immutable holdout v4 captured observations: v7/v20で26/26 x 3
- historical v1-v4 FAIL observation自体は書き換えず保持
- successor replay wrong-target Relevant: 0
