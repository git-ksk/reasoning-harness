# Engine 0.6 evidence relevance holdout v11

Status: fresh independent acceptance corpus 準備済み / still unobserved。

Successor-v10 semantics は `engine-0.6-evidence-relevance-successor-v10-semantics-freeze` / `2d75c2d8f1710c0553c8b2993cad201566be7d5c` で固定済み。専用 runner は `engine-0.6-evidence-relevance-holdout-v11-runner-freeze` / `63e3e1bc5a319d2dccbbbaf2fcf643bc3a32dba5` で固定済み。

holdout-v11 の provider observation はまだ発生していない。

## Frozen profile

- binary: `reason-evidence-relevance-holdout-v11-study`
- configuration: `evidence-relevance-live-holdout-v11`
- suite: `evidence-relevance-holdout-v11`
- annotation protocol: `evidence-relevance-effective-qualification-v15-materialization-v28`
- fixed core: `evidence-relevance-fixed-core-v11`
- directory: `fixtures/evidence-relevance-holdout-v11`
- cases: 26
- issue: #468
- effective qualification: v15
- materialization: v28
- seed: 4681110

Historical V1-V10 binding は変更しない。

## Fresh corpus

corpus は runner-freeze tag の存在後にのみ author した。

Distribution:
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8

Authority mode:
- require_different: 9
- require_requested: 8
- forbid_different: 4
- preserve_risk: 4
- preserve_absence: 1

observed holdout-v1-v10 / successor development surface に対して case ID / canonical entity / task / exact signal / 8-token signal-window の再利用を禁止し、freshness test で強制する。

offline v15/v28 property test は conservative model output 下の negative-relation recovery、requested-relation recovery、instruction/control-schema の adversarial DifferentRelation 抑止、strict relation absence 保持、precommit label に対する exact materialization を確認する。

## Three-provider acceptance gate

one-shot canonical acceptance では3 providerを全て required に戻す:
- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`

各 required arm は26件全て完走し、以下を満たす必要がある:
- provider/protocol failure 0
- provider-arm latch なし
- Harness-owned identity/risk constraint を維持
- require_different -> different_relation
- require_requested -> requested_relation
- forbid_different -> different_relation ではない
- preserve_absence -> relation_absent
- preserve_risk -> frozen risk 維持
- final v28 materialization 26/26 exact
- wrong-target Relevant 0
- false relevance rejection 0
- relevant-left-Ambiguous 0
- utility miss 0

required arm の1件でも miss すれば immutable FAIL。rerun / rescore / relabel / tag move は行わない。

## Groq admission

canonical holdout-v10 run `37085574010` の実績から再計算:
- observed Groq tokens: 65,889
- Groq arm completed: `2026-10-03T03:31:56Z`
- prior configured start headroom: 55,000
- prior pacing: inter-case 300,000 ms + minimum request interval 10,000 ms
- v11 required start headroom: 55,000
- v11 self-budget: 70,000
- v11 pacing: inter-case 300,000 ms + minimum request interval 10,000 ms
- conservative not-before floor: `2026-10-03T09:17:01Z`

workflow は modeled floor 未達なら fail closed する。

## Freeze ordering

corpus/workflow/test surface は checksum、validate-only、freshness、fmt、Clippy `-D warnings`、v15/v28 replay/regression、workflow YAML parse、exact-head GitHub CI を全て通してから `engine-0.6-evidence-relevance-holdout-v11-freeze` annotated tag を push する。

tag push が first/only canonical provider observation を起動する。
