# Harness Engine 0.6.0 release

Harness Engine 0.6.0は、split `engine-v*` namespaceで2つ目の独立Engine source releaseです。release closeoutではfreeze済みruntime behaviorを変更せず、integrated main上ですでにaccept済みのtarget-local evidence semanticsを明示的なEngine coordinateへpromotionします。

## Coordinate

- package owner: `reasoning-harness-core`
- Engine version: `0.6.0`
- release tag: `engine-v0.6.0`
- release namespace: `engine-v*`
- release closeout: #472 / milestone #8
- pre-release integrated candidate: `cfaa5592273716b0eb470f46744653db617e784a`
- このsource release時点のReason CLI package version: `0.5.3`
- provider implementation crate versionは内部実装coordinateのまま

公開済み`reason-v0.5.3` binaryはHarness Engine 0.5.0のままimmutableです。Engine 0.6.0を既存artifactへretrofitせず、将来の別Reason CLI releaseが新しいCLI coordinateで明示adoptできます。

## Accepted semantic delta

Engine 0.6.0は、既存hard verification authorityを維持しながら、context/external evidence周辺に3つのHarness-owned stageを追加します。

1. **Target-local evidence need（#461）。** acquisition前にtargetごとのtyped evidence requirement / acquisition dispositionを持つ。model proposalはadvisoryで、Harness-owned floorがexplicit external/current/trusted-verification requirementをcontext/localへ弱めることを禁止する。
2. **Evidence-target semantic relevance / relation qualification（#462/#468）。** retrieved materialをexact target identity、coarse relation、riskに対してHarness-owned compositionで評価する。relevance PASSはverification authorityを生成せず、ambiguous / wrong-target materialはfail closed。
3. **Source-attributed qualified prose（#463）。** relevant admitted proseは、bound sourceが何を述べているかというattributed statementとしてのみclaimをsupportできる。target/evidence/source/span identity、authority ceiling、conflict state、citation exposure、replay persistenceはHarness-ownedで、attributed prose単独では外部世界の`Known`/`Supported` truthを生成できない。

supporting PR #460はboundedなprovider-neutral JSON-Schema -> JSON-object transport fallbackを追加します。original task/system/budget/seed/reasoning preferenceを維持し、typed parse/validationを引き続き必須とするため、semantic repairやevidence/authority生成は行いません。

## Release evidence

新しいrelease-tuning surfaceを作らず、すでに観測済みのimmutable evidenceをrelease gateとして使用します。

### #461 evidence need

- calibration v3 freeze: `engine-0.6-evidence-need-calibration-v3-freeze`
- calibration run: `35957170730`
- independent holdout: `engine-0.6-evidence-need-holdout-v1-freeze`
- canonical holdout run: `35965160995`
- result: Mistral + Google 26/26 materialized mode / acquisition、correctness violation 0、utility miss 0、provider failure 0

### #462/#468 relevance / relation

- final accepted freeze: `engine-0.6-evidence-relevance-holdout-v12-freeze`
- freeze commit: `78c6894e871d9aa4dd79aef0a30c95647c075c4d`
- canonical run: `37218652869`, attempt 1
- required provider: Mistral、Google、Groq
- result: 各26/26、authority failure 0、identity/risk failure 0、materialization 26/26、wrong-target Relevant 0、false relevance rejection 0、Relevant-left-Ambiguous 0、utility miss 0

historical FAIL / operational relevance observationはimmutableのまま保持し、後からPASSへ読み替えません。

### #463 source attribution

- development v6 freeze: `engine-0.6-source-attribution-development-v6-freeze`
- development run: `37294100665` — PASS
- independent holdout: `engine-0.6-source-attribution-holdout-v2-freeze`
- freeze commit: `bb8d43616ee60370c099db6dda35f9a7f6d209f4`
- canonical run: `37326666360`, attempt 1 — PASS
- required provider: Mistral、Google、Groq
- result: 各18/18、useful attribution 6/6、citation coverage 100%、provider failure 0、truth-promotion / source-binding / renderer-only exposure / semantic-strengthening / wrong-target / missing-citation / replay-refetchの全hard gate 0

先行source-attribution holdout-v1 operational FAILはimmutableのまま、rerun / rescore / relabel / retagしていません。

## Release boundary

このreleaseで進めるのはEngine source coordinateだけです。新しいReason CLI binaryは公開せず、historical machine-contract identityや既存release / freeze artifactも変更しません。Reason CLIによるEngine 0.6.0 adoptionは別のproduct/lifecycle decisionとして扱います。
