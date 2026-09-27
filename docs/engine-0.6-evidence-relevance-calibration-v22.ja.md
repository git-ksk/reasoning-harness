# Engine 0.6 evidence-target relevance calibration v22 — operational TPD successor

Status: pre-freeze implementation candidate。v22はv21 semanticsをそのまま維持し、canonical admissionだけを変更する。fresh v22 suite/configuration、operational-equivalence test、tag-triggered workflow、TPD recovery-floor guardまで実装済み。v22 freeze tag / live observationはまだ存在しない。

v22はimmutable v21 run `36295631123` のsuccessorであり、v21 rerunでも結果の再解釈でもない。

## Objective

v21で実測したoperational failureを、semantic label / prompt / provider role / materializationをquota結果へ合わせずに除去する。

v21ではtiny Groq readiness 3発がcanonical直前にPASSしたにもかかわらず、organizationのTokens Per Day (TPD) headroomがfull required armには不足していた。したがってv22ではtiny readinessをtransport / credential / TPM / RPD evidenceに限定し、canonical tag消費前にconservativeなTPD recovery floorを追加する。

## Frozen semantic inheritance

v21から変更しないもの:
- fixed core `evidence-relevance-fixed-core-v1`、48件固定;
- 全case ID / expected proposal / expected local qualification / expected disposition;
- primary binding proposal v5;
- raw verifier v8;
- effective qualification v2 (`reason-evidence-relevance-effective-qualification-v2`);
- materialization v16 (`target-evidence-relevance-binding-materialization-v16`);
- authority-qualified gate semantics;
- deterministic local-risk classifier;
- target-negative terminal ruleとstrict Harness-owned identity floor;
- retry / timeout / provider latch / telemetry / public-safe diagnostic;
- required provider Mistral + Groq;
- Google full non-gating replication;
- first/only immutable canonical policy;
- canonical PASS前のindependent holdout禁止。

Google v21 case 76はnon-gating relation-axis diagnosticのまま維持する。target-negative terminalで最終Irrelevantは正しかったため、v22ではこの観測を理由にsemantic authority ruleを変更しない。

## Measured TPD behavior

v21 Groq quota responseで以下を保存できた。
- TPD limit: 200,000 tokens;
- used: 199,298;
- requested: 1,295;
- rejected request前のheadroom: 702;
- provider retry interval: 4m16.176s（256.176s）;
- TPM remaining: 8,000/8,000;
- RPD remaining: 974/1,000。

rejected requestの不足量は593 tokens。`593 / (200000 / 86400) = 256.176s`でprovider retry intervalと完全一致する。v22ではadmission planning上、このTPDを24時間連続補充token bucketとして扱う。

historical full-arm demandはtiny readinessより大幅に大きい。
- v20: 39 successful casesで97,259 Groq tokens;
- v21: 13 successful casesで31,648 Groq tokens;
- 現runner shapeの48件straight-line estimateは約116.9K–119.7K tokens。

v22 admission planningではconservativeに150K-token Groq headroomを目標とする。これはoperational planning boundでありsemantic thresholdではない。

## Canonical self-budget guard

recovery-floor admissionだけではprovider出力tokenの揺れを吸収できないため、v22はrequired Groq armに実行中のconservation guardも追加する。
- observed provider-token budget上限: 140,000 tokens;
- 次case開始前に必要なreserve: 4,000 tokens;
- v20/v21で観測したGroq 1 completed caseのtotal usage最大: 2,792 tokens;
- full armのhistorical projectionは約117K-120K。

各Groq case開始前にrunnerがそれまでのprovider-reported token usageを合算し、`consumed + 4,000 > 140,000`なら次のexternal requestを送らずprovider armをlatchして残りcaseを抑止する。guard有効中に過去model callのtoken usage telemetryが欠落していれば、0消費と仮定せずfail-closedでlatchする。

140K boundは150K admission targetより低く、fully recovered 200K bucketより十分小さい。organizationの同時利用が無ければ、想定外に消費が増えても約60Kを残す。guard発火時のcanonicalはimmutable FAILのままで、quota conservationのためにacceptanceを緩めない。

## TPD recovery floor

v21 Groq jobはbucket near-exhaustion観測直後の`2026-09-27T05:02:01Z`に完了した。この保守的anchorから24時間cooldownした時刻は:

- UTC: `2026-09-28T05:02:01Z`
- JST: `2026-09-28 14:02:01 +09:00`

この時刻より前にv22 freeze tagを作成してはいけない。

24時間floorは150K headroom推定回復時間（near-full観測から約17.9時間）より意図的に厳しい。tiny readinessでTPD headroomを直接測れたように扱わないため。

このfloorは下限であり保証ではない。v21観測後にGroq organizationでmaterialな追加利用があれば同じorganization-level allowanceを消費するため、bucket recovery assumptionは無効になる。追加利用が既知または疑わしい場合はgateを緩めずcooldownを延長する。

## Readiness semantics

freeze直前にexisting synthetic Groq readiness workflowをexact v22 candidate branchへ実行する。

PASSが意味するのは以下のみ:
- credential有効;
- model endpointがrequest受理;
- tiny requestに対するcurrent quota blockなし;
- bounded probe上でTPM/RPD headerが正常。

full-run TPD headroomの証明にはならず、そのように引用してはいけない。

quota / rate-limit failure時は新しいpublic-safe `provider_diagnostic` evidenceを保存する。raw unredacted provider bodyをpublic Actions logやcommitted artifactへ出さない。

## Fresh observation identity

v22はfresh versioned observation surfaceとする。
- suite: `evidence-relevance-calibration-v22`;
- configuration: `evidence-relevance-live-calibration-v22`;
- semanticsは不変なのでannotation protocolは`evidence-relevance-effective-qualification-v21`を維持;
- materialization v16維持;
- stochastic factorを増やさずoperational admission差分を分離するためcalibration seedもv21と同じ値を意図的に維持する。

新suite metadataはfrozen case/label変更を許可しない。

## Required pre-freeze proof

v22 tag前に必須:
- v22 case arrayがv21とsemantic-identical（48/48、relabel/growth/reorderなし）;
- v18/v19/v20/v21 semantic regression green;
- v22 operational-equivalence testで48 expected caseすべてv16 materialization一致;
- core/providers/CLI full package test green;
- all-target Clippy `-D warnings` / fmt / validate-only / exact surface checksum green;
- standard PR CI green;
- public-safety scan green;
- current timeが`2026-09-28T05:02:01Z`以降;
- exact candidate branchでfresh synthetic Groq readiness PASS;
- recovery anchor後にmaterialなGroq organization usageがある場合はcooldown延長済み。

## Pre-freeze implementation evidence

現在candidateはdeterministic / operational pre-freeze proofでgreen。
- v22 `cases`はfrozen v21とbyte-semantic JSON-equivalentで48/48、growth / relabel / reorder 0;
- annotation protocolは`evidence-relevance-effective-qualification-v21`、materializationはv16を維持し、v22でcore semantic production ruleは変更していない;
- v22 operational-equivalence suite: 3/3 PASS;
- regression: v18 17/17、v19 12/12、v20 16/16、v21 20/20 PASS;
- calibration runner focused: 30/30 PASS;
- validate-only: configuration `evidence-relevance-live-calibration-v22`、48 planned / 0 observed、`validate_only_non_scorable`;
- core full main suite 246/246 PASS + integration block全PASS;
- providers 153 passed / 1 ignored / 0 failed;
- CLI main suite 205 passed / 3 ignored / 0 failed + integration block全PASS;
- core/providers/CLI all-target Clippy `-D warnings`: PASS;
- `cargo fmt --all -- --check` / `git diff --check`: PASS;
- v22 frozen-surface checksumは明示40 filesを対象に再検証green;
- live workflowはcheckout/provider処理より前に`2026-09-28T05:02:01Z` recovery-floorをfail-closedで検証する;
- required Groq実行には140K observed-token cap + 4K pre-case reserveを追加し、usage telemetry欠損もfail-closedで扱う。

exact pushed candidate上のstandard PR CIとfresh pre-freeze Groq readinessはまだ必須。recovery-floor時刻前なのでv22 freeze tag作成は禁止継続。

## Canonical policy

v22 live workflowはfirst/only tag-triggeredを維持。workflow rerunを拒否し、TPD recovery floorより早く起動された場合はfail closedする。

required Groqが再度TPD/RPD/TPM quotaに達した場合、v22もimmutable FAIL。同じtagをrerunしない。保存した`provider_diagnostic`はsuccessor evidenceにのみ使う。

v22がrequired Mistral + Groq gateをPASSした場合もGoogleはこのcalibrationではreplication-onlyを維持する。その後にのみindependent holdoutを再開できる。
