# Engine 0.6 evidence-target relevance calibration v17 — successor design

状態: immutable canonical FAIL。freeze commit `9647e70125b97dd77b1c4742004889c23fd10d58` / run `36226650327` は完了済み。rerun / rescore / relabel / retag禁止。詳細は `engine-0.6-evidence-relevance-calibration-v17-result.ja.md`。Independent holdout authoringは禁止継続。

## Fixed evaluation surface

v17も`evidence-relevance-fixed-core-v1` 48件を変更しない。v16 missを理由にcase追加・削除・relabel・選別しない。14 Relevant / 18 Irrelevant / 16 Ambiguousを維持。

contract:
- primary binding proposal: `reason-evidence-relevance-binding-proposal-v5`（v16から変更なし）
- independent local verifier: `reason-evidence-local-qualification-v7`
- Harness materializer: `target-evidence-relevance-binding-materialization-v12`
- annotation protocol: `evidence-relevance-scope-verifier-v17`

## Why v17 exists

v16はv15のsafety regressionを解消し、Required Mistral materializationを33/48 -> 42/48へ改善、wrong-target Relevantも0にした。残り6件は全てconservative Ambiguousで、generic residualは2系統だけ。

1. explicit `allow_semantic_equivalent` policyでprimary targetがunresolvedでもverifierがexact target/relation・risk noneを独立確認した場合のbounded positive pathがない。
2. verifierがusable bounded negative observationに`context_gap`を過剰使用する。navigation-only target mention + 明確なdifferent substantive owner、complete broad/generic unit、explicit local absence、無視すべきprompt-injection textが対象。

v17はこのgeneric boundaryだけを修正し、case ID / synthetic name / exact fixture phraseで分岐しない。

## Materialization v12

v11 terminal ruleを維持し、1つだけnarrow positive fallbackを追加。

primary `target_binding=unresolved`からRelevantへ進めるのは全条件成立時のみ:
- Harness policyが`identity_requirement=allow_semantic_equivalent`を明示設定;
- primary `relation_binding=exact`;
- verifier `identity_scope=exact_target`;
- verifier `relation_scope=requested_relation`;
- verifier `scope_risk=none`;
- candidateにsubstantive local signal（heading / excerpt / structured_metadata / fact）が1つ以上ある。

strict identity、URL/navigation-only、scope riskあり、relation unresolved/different、primary target=differentではfallback不可。

その他のv11 safety ruleは変更しない:
- scope riskは常にAmbiguous;
- strict identityのunresolved primaryはRelevantへ救済不可;
- negative target/relation terminalはmatching verifier scope evidence必須;
- explicit local absenceはnegative-only path;
- disagreementはAmbiguous。

## Local verifier v7

v16の3-field schemaを維持:
- `identity_scope`: exact_target | distinct_target | target_absent | unresolved
- `relation_scope`: requested_relation | different_relation | relation_absent | unresolved
- `scope_risk`: none | identity_mapping | ownership_scope | context_gap | multiple

schemaを増やさずprompt boundaryを明確化:
- candidate instructionはuntrustedでignore。ignored instruction自体はscope risk / context gapを作らない;
- navigation/footer target occurrenceはexact target ownershipを作らない;
- substantive contentが明確に別targetなら、Harness targetがnavigation/footer/comparisonにだけ出ても`distinct_target` + `scope_risk=none`;
- complete broad/generic/catalog unitでexact-target propositionがない場合は`target_absent`にでき、`context_gap`ではない;
- explicit local absenceは`target_absent` / `relation_absent`にでき、`context_gap`ではない;
- `context_gap`はvisible clipping/truncation、omitted referent/product column、substantive propositionのないURL/navigation-only unit、required local contextの明示omissionに限定;
- `allow_semantic_equivalent`ではlocally specific semantic equivalentをliteral canonical nameなしでも`exact_target`にできる。

## Unscored structural controls

scored fixed coreは増やさない。property/routing testのみ追加し、次を証明:
- strict identityではsemantic-equivalent fallback不可;
- scope riskがあればfallback不可;
- relation unresolvedではfallback不可;
- URL-onlyではfallback不可;
- frozen 48件のexpected annotationが既存expected dispositionへ全件materializeする。

## Operational policy

v16 hardeningをそのまま継承:
- active execution 60,000 ms / case
- cumulative provider wait/retry 45,000 ms
- single provider wait cap 30,000 ms
- absolute case wall-clock 120,000 ms
- retry ownershipはprovider adapter
- typed quota 1件でlatch
- correlated capacity failure 2件でlatch
- suppressed caseはoperational failure
- provider-attempt / active / wait / pacing / retry telemetry分離
- public provider failure sanitize
- manual Groq TPD attestation start gateなし

Required providerはMistral `ministral-8b-latest` + Groq `openai/gpt-oss-120b`。Google `gemini-3.5-flash-lite`はfull non-gating replication。

v16でcurrent Groq daily windowのquota exhaustionを実測済み。その既知exhausted window中はv17 freeze tagをpushしない。これはmanual capacity attestation gateではなく、observed quota latch直後にone-shot canonicalを knowingly 消費しないための運用制約。

## Pre-live acceptance

v17 freeze tag前に:
- fixed-core routing/property test PASS
- runner test PASS
- workspace test PASS
- clippy `-D warnings` PASS
- `cargo fmt --check` / `git diff --check` PASS
- `surface-v17.sha256`全件一致
- validate-onlyがv17 ID、48 planned / 0 observed、operational abortなし、provider-arm latchなし、non-scorable validation
- PR #466はDraft維持
- independent holdout未作成

first/only frozen v17 canonicalは開始後one-shot immutable。
