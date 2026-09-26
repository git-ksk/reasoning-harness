# Engine 0.6 evidence-target relevance calibration v11 — immutable result

Status: FAIL。v11の結果はimmutableとし、rerun / rescore / retagしない。

Freeze:
- candidate/tag commit: 6eebdd4c90cbb646cb67b06b0345a3c9a51d2208
- tag: engine-0.6-evidence-relevance-calibration-v11-freeze
- canonical GitHub Actions run: 36142920405
- run attempt: 1
- final-gate: failure

## Required Mistral arm

Model: ministral-8b-latest。

Operationalは65/65 successful provider cases、provider failure 0、qualification invocation 65/65、attempt telemetry complete。

Safety側は維持した。
- qualification risk miss: 0
- wrong-target relevance retention: 0
- false relevance rejection: 0

FAIL要因はutility/materialization。
- local qualification exact: 0/65
- qualification spurious risk block: 43
- materialized exact: 22/65
- expected Relevant left Ambiguous: 19
- utility miss: 43

root causeはmaterializerを緩和すべき問題ではなくguard contractの意味論矛盾。v11はsupplied local materialだけでriskを完全否定できなければunresolvedと要求したため、通常のopen-world uncertaintyまでlocal blockerへ変換した。さらにHarness-owned aliasはauthoritative local anchorであるべき一方、promptはalias equivalenceの記載・可能性そのものをidentity riskとして読める定義になっていた。

## Required Groq arm

Model: openai/gpt-oss-120b。

operational incompleteでnon-scorable。
- planned/completed: 65/22
- successful provider cases: 10
- failed provider cases: 12
- observed failure class: unsupported_capability 10、assessment_timeout 2
- 22_partial_identity後に2 consecutive operational failure circuitでabort、残り43
- provider attempts: 204
- latency p50/p95/max: 51,181 / 60,001 / 60,002 ms

失敗patternはserver-side structured JSON/schema generation failureの反復と、bounded fallback / rate-limit waitによるcase budget消費。Mistralのsemantic overblockingとは独立したtransport問題である。

## Google replication

Model: gemini-3.5-flash-lite。non-gating。

65 case recordを観測し、successful provider cases 63、assessment_timeout 2件。timeoutは33_exact_binding_conflictと44_prompt_injection_local_absence。

semantic方向はMistralのoverblocking signalと一致。
- qualification risk miss: 0
- qualification spurious risk block: 25
- materialized exact: 37/65
- expected Relevant left Ambiguous: 7
- utility miss: 26
- wrong-target relevance retention: 0
- false relevance rejection: 0

## Final decision

required top-level gateはoperational completeness / correctness / utility / materialization / qualification acceptanceすべてfalse。v11はimmutable canonical FAIL。

v11のrerun / rescore / replacement tagは禁止。independent holdout authoringも引き続き禁止する。

successorはv12。
- two-key ownership boundaryとfail-closed materialization v7は維持。
- qualification riskをhypothetical open-world risk否定証明ではなくconcrete local ambiguity signalへ再定義。
- Harness-owned canonical name / aliasをauthoritative local identity anchorとする。
- concrete ambiguity cueがある場合のPresent/Unresolvedは引き続きfail-closed。
- calibration runnerのsingle JsonObject fallbackをstrict raw-JSON Text fallbackへ変更し、typed parseとstage 2-call ceilingは維持。
- holdout前にfresh 73-case calibrationを実施する。
