# Engine 0.6 evidence-target relevance calibration v19 — pre-freeze candidate

Status: implementation candidate / 未freeze。v19 live calibration observationはまだ0件。v18 canonicalはimmutable FAILのままで、rescore / rerun / relabel / retagしない。

## Frozen inheritance

v19で変更しないもの:
- fixed core: `evidence-relevance-fixed-core-v1`、48件固定。
- primary proposal: `reason-evidence-relevance-binding-proposal-v5`。
- v18のscored case label / expected disposition。
- v18 operational deadline、adapter-owned retry、quota/capacity latch、provider-attempt telemetry、public-log sanitization。
- one-shot canonical policy。calibration PASSまでholdout authoring禁止。

## Materializerだけのsuccessorでは不足する理由

Required Mistral v18は48/48完走、materialized 47/48 exact、wrong-target Relevant retention 0まで改善した。一方raw local qualificationは26/48 exactで:
- scope-risk miss: 11
- spurious scope risk: 0
- identity-scope miss: 14
- relation-scope miss: 14

v18 final gateはこれら全fieldのmiss/spurious 0を要求していた。したがって `14_sibling_product_overlap` 向けのterminal ruleだけでdispositionを直してもqualification gateはredのまま。v19ではacceptanceを黙って緩めず、authority boundaryそのものを明示的にversion化する。

## v19 authority split

v19は2層を分離する。

1. **Raw model qualification telemetry**: verifier v8のraw出力はそのまま保持し、diagnosticとして継続scoreする。model missを上書き・隠蔽しない。
2. **Harness-owned effective qualification**: runtime materializationとv19 qualification gateが使用する新version contract。deterministic local factとbounded model evidenceをfail-closedに合成する。

最初のHarness-owned componentはtyped deterministic local-risk classifier:
- `none`
- `identity_mapping`
- `ownership_scope`
- `context_gap`
- `multiple`

v18のboolean local-risk floorを一般化する。deterministic riskがnon-noneならmodel出力より優先し、必ずAmbiguousを強制する。

effective identity/relation fieldもfail-closedを維持する。generic local structureとHarness-declared identity metadataからのみnormalize可能とし、clipped context、omitted ownership、uncertain alias/rename/successor mappingを推測で埋めない。

## Effective qualifierのpre-freeze acceptance

v19 freeze tag作成前に:
- fixed 48全件でeffective qualificationをderive/invokeする。
- effective identity-scope miss: 0。
- effective relation-scope miss: 0。
- effective scope-risk miss: 0。
- effective spurious scope risk: 0。
- raw verifier v8 metricsも別telemetryとして必ず出す。
- case ID、synthetic product名、exact fixture phraseをruleに使わない。
- generic negative / risk boundaryをproperty testで固定する。

genericにこの条件を満たせない場合はv19をfreezeしない。canonicalを消費するためにgateを緩和しない。

## Target-negative materialization

immutable v18 auditが支持する新しい一般rule:

以下をすべて満たす場合:
- deterministic local risk = none。
- effective `scope_risk=none`。
- effective `identity_scope=distinct_target`。
- primary `target_binding != exact`。

primary target axisがunresolvedでもIrrelevantへmaterialize可能とする。

primary target exact、effective/model safety risk non-none、deterministic local riskありのいずれかではこのruleを使わない。

fixed-core auditではこのshapeにexpected Ambiguousの衝突はなく、expected instanceはすべてIrrelevantだった。

provider観測で見えた次の2 shapeは、採用前に別property auditを行う:
- exact target + independently established different relation。
- riskなしのcomplete local relation absence。

個別caseを直せることだけを理由にv19へ事前採用しない。

## Pre-freeze implementation evidence

現在のv19 candidateはv18 gateを緩和せず、authority splitを実装している。

- Harness-owned effective qualification contract: `reason-evidence-relevance-effective-qualification-v1`
- materialization policy: `target-evidence-relevance-binding-materialization-v14`
- live-run configuration: `evidence-relevance-live-calibration-v19`
- calibration suite: `evidence-relevance-calibration-v19`
- annotation protocol: `evidence-relevance-effective-qualification-v19`
- raw verifier v8 outputは既存の `local_qualification_*` diagnostic metricsとしてそのまま保存・採点
- effective qualificationは別フィールド・別metricsで保存し、v19 qualification gateのauthorityとする

fixed 48に対するdeterministic pre-freeze evidenceはgreen。

- typed deterministic local-risk classification: expected 48/48
- effective identity/relation/risk qualification: expected 48/48
- expected materialization: 48/48
- immutable v18 Mistral mismatch replay: effective qualification 48/48 / materialization 48/48
- generic boundary/property tests: 12/12 PASS
- v18 regression suite: 17/17 PASS
- calibration runner focused tests: 23/23 PASS
- full package test suites (`core` / `providers` / `cli`): PASS
- all-target Clippy with `-D warnings` for all three workspace packages: PASS
- `cargo fmt --all -- --check`: PASS
- v19 validate-only: 48 planned / 0 observed / `validate_only_non_scorable`

replay evidenceはimmutable v18 observationを使うtest-only evidenceであり、v18のrescoreでも新規live observationでもない。

v19 freeze workflowは準備済みだが、exact annotated `engine-0.6-evidence-relevance-calibration-v19-freeze` tagが存在するまで起動しない。現時点でv19 freeze tagは存在しない。

## Operational policy

Groq daily quotaのmanual preflightは追加しない。v18同様、runtime typed quota failureをauthoritative signalとして即provider-arm latchする。retry storm、canonical rerun、replacement observationは禁止。

required providerはMistral + Groqを維持し、Googleは独立provider evidenceでfreeze前に方針変更しない限りfull non-gating replicationを維持する。

first/only frozen v19 canonicalがrequired operational / correctness / utility / materialization / effective-qualification gateをすべてPASSするまでindependent holdout authoringは禁止。
