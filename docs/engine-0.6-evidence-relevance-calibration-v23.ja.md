# Engine 0.6 evidence relevance calibration v23

Status: pre-freeze implementation candidate。v23 freeze tag / live canonical observationはまだ存在しない。

## Goal

Googleをrequired providerへ昇格しつつ、authority-qualified gateは緩めない。v22の48-case frozen semantic surfaceを維持し、immutable v22 Google observationで露出したHarness-owned effective-qualification compositionの欠陥1件だけを修正する。

## Frozen provider contract

3 provider全てrequired:

- Mistral: `ministral-8b-latest`
- Groq: `openai/gpt-oss-120b`
- Google: `gemini-3.5-flash-lite`

各required armは48/48完走し、correctness / utility / materialization / authority-qualified effective-qualification gateを全てPASSする必要がある。

## Semantic delta: effective qualification v3

v22 case `76_v13_sibling_different_relation_no_cue` では:

- expected proposal: `different / different`
- Google observed proposal: `different / exact`
- Google raw qualification: `distinct_target / different_relation / none`
- v2 effective qualification: `distinct_target / requested_relation / none`

raw verifierはidentity / relation両軸を既に正しく判定していたが、v2がproposal側relationを優先して上書きした。v3はv2を基礎とし、次を全て満たす場合だけrelationを補正する。

1. deterministic scope riskが`none`;
2. effective identityが`distinct_target`;
3. proposalがtarget `different` / relation `exact`;
4. raw qualificationが独立に `distinct_target / different_relation / none`;
5. requested relationがlocal evidenceに存在しない。

この場合effective relationを `different_relation` に保持する。

provider名、fixture ID、entity名、exact wordingには依存しないgeneric corroboration rule。materialization v16は変更せず、zero-risk authority gateもstrictなまま維持する。

## Immutable v22 replay proof

canonical v22 run `36338187291` のsuccessful observationをsanitized replayし、freeze前にv3で3 provider全てについて以下を要求する。

- authority-qualified effective qualification 48/48;
- materialized disposition 48/48;
- frozen 48-case surfaceへのregression 0。

現在のimplementation candidateはこのproofをPASSしている。

## Groq admission plan

v22 Groqは120,426 tokensを観測し、`2026-09-27T23:04:31Z` に完了した。以前観測したTPD 200,000のcontinuous-refill modelと、`2026-09-27T05:02:01Z` 時点 `199,298 used` のanchorを使うと、追加のmaterialなorganization usageが無い前提でv22終了時modeled headroomは約30.6K tokens。

v23では:

- modeled start headroom: 105K;
- earliest modeled floor: `2026-09-28T08:00:02Z`（JST `2026-09-28 17:00:02`）;
- Groq case間delay: 210秒;
- provider request spacing: 10秒;
- 48 casesのdeliberate pacing: 10,350秒（2時間52分30秒）;
- pacing中modeled refill: 約23.96K tokens;
- modeled supply: 約128.96K tokens;
- observed-token hard cap: 128K;
- pre-case reserve: 4K;
- Groq job timeout: 240分。

128K capはv22 full-arm実測120,426 tokensを上回りつつ、modeled supply未満に収める。guard有効時のusage telemetry欠損は引き続きfail-closed。anchor後のmaterialなorganization-level Groq追加利用が既知または疑わしい場合、このmodelは無効としてfreeze前にdelay / re-anchorする。

## Freeze rule

first/only v23 canonical tag前に:

1. exact candidateがcleanかつ全検証green;
2. frozen-surface checksum PASS;
3. 3 provider全てでv22 replay proof PASS;
4. exact candidate上でfresh Groq readiness PASS。ただしTPD headroom proofとは扱わない;
5. modeled Groq admission floor経過;
6. v23 freeze tagを1回だけ作成しcanonicalをrerunしない。

required providerのどれかがoperationally incompleteまたはrequired semantic gate FAILなら、v23はimmutable FAILとして新successorへ進む。
