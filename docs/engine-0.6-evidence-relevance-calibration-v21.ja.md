# Engine 0.6 evidence-target relevance calibration v21 — design-only successor

Status: design only。v21 runtime実装、workflow、freeze tag、live observationはまだ存在しない。

v21はimmutable v20 run `36283988719` のsuccessor。v20をrepair / rerun / reinterpretしない。

## Measured v20 gap

Required Groqはdaily quota前にsafety-criticalなrelation authority conflictを1件露出した。
- exact target identityはローカルにanchor済み;
- primary proposalは`relation=exact`;
- independent raw verifierは`relation=different_relation`かつ`scope_risk=none`;
- deterministic requested-relation cueは存在しない;
- effective qualification v1がprimary exact relationを`requested_relation`へ昇格;
- v15がexpected IrrelevantをRelevantとしてmaterializeした。

これはpromptだけのmissではなくHarness側のauthority ordering問題。deterministic local evidenceがrequested relationを確定していないのに、advisory primary outputが独立negative relation classificationを上書きできている。

## Frozen inheritance

変更しないもの:
- fixed core `evidence-relevance-fixed-core-v1` 48件;
- 全expected proposal / qualification / disposition label;
- primary proposal v5、raw verifier v8 prompt/contract;
- typed deterministic local-risk classifier;
- strict positive identity floorとv19/v20 target-negative safety behavior;
- context-gap / ownership / mappingのfail-closed behavior;
- provider retry、deadline、quota/capacity latch、telemetry、sanitization;
- required provider Mistral + Groq、Google full non-gating replication;
- one-shot canonical immutabilityとPASS前holdout禁止。

## Effective qualification v2

v1を黙って変更せず、`reason-evidence-relevance-effective-qualification-v2`を追加する。

変更するauthority boundaryは1つだけ。

以下を全て満たす場合:
- deterministic local risk = `none`;
- Harness identity evidenceからeffective identity = `exact_target`;
- deterministic requested-relation presence = false;
- raw verifierが存在し、`scope_risk=none`かつ`relation_scope=different_relation`;
- primary target = `exact`;

primary proposalが`relation=exact`でもeffective relationを`different_relation`とする。

primary exact relation単独では、このconflictを`requested_relation`へ昇格できない。

このauthorityはone-sided。raw verifierからpositive requested-relation authorityを新設せず、riskを越えて推測せず、context-gap normalizationも変更しない。

## Materialization v16

新しいmaterialization policy IDを追加し、v15はhistoricalのまま保持する。

上記exact-target relation-conflict shapeでは、primary relationがexactでもv16はIrrelevant terminalを許可できる。ただし以下が全て必要:
- deterministic risk = `none`;
- strict Harness target identityを満たす;
- effective qualification v2 = `exact_target + different_relation + none`;
- raw verifierも独立に`different_relation + none`;
- deterministic requested-relation cueがない。

context/ownership/mapping risk、strict identity anchor欠落、raw safety risk、deterministic requested-relation cueのいずれかがあればterminal rejectionしない。

## Why not simply trust the verifier

v21でraw verifier v8を一般的なauthorityにはしない。provider間のraw disagreementは依然多い。新しいauthorityは、target identityがHarness-owned exact、riskなし、requested relationのdeterministic cueなし、独立verifierが別relationを明示、という狭いnegative relation conflictだけに限定する。

これによりv19/v20のraw/effective telemetry分離を維持し、model-to-model disagreementを無制限なauthorityへ昇格させない。

## Context-gap normalization remains deferred

v20でも`context_gap`下のrelation-scope mismatchが残った（Groq case 25、Google case 25/26/80）が、最終dispositionはAmbiguousを維持した。v21でこれらをnormalizeしたりclipped/omitted contentを推測したりしない。

## Required pre-freeze proof

v21 freeze前に必須:
- fixed 48と全labelをbyte-semanticに維持;
- primary-exact/raw-different conflict、deterministic requested-relation counterexample、raw-risk counterexample、strict-identity counterexampleのgeneric property test;
- immutable v20 Mistral successful observationがeffective qualification 48/48、materialization 48/48維持;
- immutable v20 Groq successful observationがmaterialization 38/39 -> 39/39、wrong-target Relevant 1 -> 0;
- v20 Groq case 13だけが意図したterminal disposition change;
- v20 Googleがmaterialization 48/48、wrong-target Relevant / false rejection / Relevant -> Ambiguous regression 0を維持;
- context-gap relation mismatchはfail-closed維持、diagnosticとして残してよい;
- v18/v19/v20 regression、full package test、all-target Clippy `-D warnings`、fmt、validate-only、frozen surface checksumがgreen;
- production ruleにcase ID、synthetic product名、exact fixture phraseを入れない。

## Operational boundary

v20 daily quota exhaustionはrerun理由にならない。v21は新semanticをfreezeし、別quota windowが利用可能になってからfresh canonicalを1回だけ実行できる。Groqが再度quotaに達した場合、そのfirst/only v21 canonicalもimmutable FAIL。

canonical PASSまでindependent holdout authoringは禁止継続。
