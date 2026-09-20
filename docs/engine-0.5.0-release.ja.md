# Harness Engine 0.5.0 release

Harness Engine 0.5.0は、最後のunified release `v0.4.2`以降で最初に独立versioningするEngine source releaseです。

## Coordinate

- package owner: `reasoning-harness-core`
- Engine version: `0.5.0`
- release tag: `engine-v0.5.0`
- release namespace: `engine-v*`
- このsource release時点のReason CLI package version: `0.5.2`
- provider implementation crate version: `0.4.2`

既に公開済みの`reason-v0.5.2` binaryはimmutableで、Harness Engine 0.4.2をreportし続けます。Engine 0.5.0を既存artifactへretrofitせず、将来のReason CLI releaseが明示的にadoptできます。

## Accepted semantic delta

Engine 0.5.0にはaccepted verified-investigation lineとfinal hardeningを含みます。

- finalization/grounding bridgeとauthority-bound exposed factual output;
- repeated-trial planner reliabilityとdeterministic executable-action ownership;
- planner target-recall telemetryとfinalization correctnessの分離（#445）;
- explicit-fact session correction continuityの決定論化（#446）;
- validated read-only acquisition後のadmitted exact-fact investigation materialization決定論化（#450）。

provider/model固有のcorrectness branchは追加していません。evidence admission、verification、answer-safety、fail-closed、external-call replay boundaryは引き続きHarness-ownedです。

## Release evidence

release gateはimmutableな`engine-0.5-final-v3-freeze` surfaceです。

- freeze commit: `063833f38c38225109586b3db92348563b3822f8`
- product candidate: `d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90`
- canonical GitHub Actions run: `35457038163`
- required rows: Mistral 14B、Qwen 3.8 27B、Mistral 8B、Gemini 3.5 Flash-Lite、Gemma 4 31B、GPT-OSS 120B
- result: 6/6 row、18/18 case、correctness-boundary violation 0、session external replay 0

詳細は[Engine 0.5.0 final-v3 result](engine-0.5-final-v3-result.ja.md)および`docs/observations/engine-0.5-final-v3/`配下のraw evidenceを参照してください。

## Historical note

先に作成した`engine-0.5.1-hardening-v1-freeze` identityは、Engine 0.5.0がまだversion releaseされていないことを確認する前に命名したものです。immutableなpre-release evaluation provenanceとしてのみ残し、Engine 0.5.1 release / release candidateとは扱いません。
