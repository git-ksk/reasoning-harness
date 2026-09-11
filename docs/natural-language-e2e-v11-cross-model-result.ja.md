# Natural-language E2E v11 — cross-model replication 結果

Issue #256 は、freeze 済み evaluator / scoring contract / corpus / seed / released product source を変更せず、released v0.4.1 natural-language E2E v11 surface を Google / Groq 対象へ cross-model replication した。

## Frozen coordinate

- replication Actions run: `34135141249`
- replication freeze tag: `natural-language-e2e-v11-cross-model-v1-freeze`
- replication freeze commit: `6a6db2fd436816d113303e52e0a28f7e0c6baf97`
- reference v11 tag / commit: `natural-language-e2e-v11-freeze` / `a758af17a998493c1005702365b100e05b05f95d`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- seed / max tokens: `57000` / `1024`
- fixed cases: `13`
- canonical Mistral reference: Actions `34129798774`, `ministral-8b-latest`

replication freeze と historical v11 reference は immutable のまま維持する。failed / incomplete target を同じ identity で rerun、repair、rescore、tuning しない。

## 結果概要

| target | completed | hard correctness | operational completeness | target recall | tool selection | trigger | #249 conformance | useful follow-up | grounded target coverage |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` reference | 13/13 | PASS | PASS | 0.60 | 0.80 | 1/3 | 1/1 | 1/1 | 0.00 |
| Google `gemma-4-31b-it` | 12/13 | PASS | FAIL | 0.80 | 0.90 | 3/3 | 3/3 | 3/3 | 0.20 |
| Google `gemini-3.5-flash-lite` | 13/13 | PASS | PASS | 1.00 | 0.60 | 0/3 | inconclusive | n/a | 0.20 |
| Groq `openai/gpt-oss-120b` | 0/13 | semantic violation 観測なし | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |
| Groq `qwen/qwen3.8-27b` | 0/13 | semantic violation 観測なし | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |
| Groq `openai/gpt-oss-20b` | 0/13 | semantic violation 観測なし | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |

semantic / correctness evidence と operational completeness は明示的に分離する。Gemma row は semantic evidence として有効だが operationally incomplete。Groq row は generic CLI から provider generation に入っておらず、model quality evidence ではない。

## Google 観測

### Gemma 4 31B

Gemma は predeclared no-result predecessor trigger 3件すべてを露出した。各 case は configured cache を先に選択し、typed `no_result` 観測後に既存 #249 exact-target continuation で対応 registry を選択した。conditional mechanism conformance は `3/3`、3件すべてで useful downstream evidence を得た。

一方で operational completeness は fail。`12/13` case complete、operational failure 4件、process-level operational failure 1件。generation failure class は `provider_unavailable` と `protocol`、process failure class は `provider_unavailable`。session persistence と external side-effect replay 0 は維持したが、該当 session path が完走しなかったため aggregate session-fork validity は false。この失敗は operational evidence として保持し、semantic failure へ読み替えない。

### Gemini 3.5 Flash-Lite

Gemini は `13/13` case を完走し、hard correctness / measurement / operations / report gate をすべて pass。target recall は `1.00` だったが tool-selection success は `0.60`。

predeclared follow-up 3件すべてで exact target を recall したにもかかわらず action 0、planner call 4回、`round_budget` stop となった。trigger reachability は `0/3`。したがって #249 mechanism denominator は 0 で、conformance は mechanism failure ではなく `inconclusive`。

target recall 自体が成功していても pre-trigger planner / action-selection gap が発生し、model 依存性が大きいことを直接示す evidence になった。

## Groq 観測

Groq 3 target はすべて provider generation 前に generic `reason` CLI が `--provider groq` を拒否し `0/13` となった。

`invalid value 'groq' for '--provider <PROVIDER>'; possible values: mistral, google, nvidia`

既存 `GroqAdapter` や dedicated external-information evaluation path の失敗ではない。観測された defect は generic natural-language CLI / provider wiring 欠落であり、v0.4.2 の #262 で扱う。Groq quota / rate-limit / model semantics にはこの frozen replication では到達していない。

## Cross-model 解釈

released v0.4.1 の #249 trigger が実際に露出した全 case では exact-target typed-`no_result` continuation は conformant だった。Mistral `1/1` + Gemma `3/3` = 観測 `4/4` conformant。completed semantic row では correctness-boundary violation、unsupported exposed assertion、unsupported structured claim、identity-unsafe admission、MCP authority self-promotion、session external side-effect replay はいずれも 0。

したがって product utility gap は #249 mechanism の malfunction evidence ではない。主要な pre-trigger residual は planner / action selection で、Mistral は 1/3、Gemini は 0/3、Gemma は 3/3 trigger exposure と大きな model 差が出た。downstream grounding residual も残り、useful follow-up evidence が grounded final answer へ安定してつながっていない。この finalization / grounding boundary は #248 / Harness Engine 0.5.0 に残し、v0.4.2 へ移動しない。

## Closeout decision

Issue #256 は immutable cross-model replication evidence として完了。これは同じ surface を tuning するための結果ではなく、v0.4.2 が上回るべき v0.4.1 baseline である。

次 patch の product target は次の2点だけとする。

1. #261: stochastic planner stall の前に、authority / identity boundary を変えない narrow Harness-owned deterministic read-only acquisition precedence を追加する。
2. #262: 既存 Groq adapter を generic natural-language `reason` provider path に露出する。

fresh successor measurement は #263 が所有する。新しい freeze 済み measurement でこの baseline に対する strict utility improvement と correctness / authority / identity / admission / session / final-answer safety no-regression を確認できない限り v0.4.2 は release しない。
