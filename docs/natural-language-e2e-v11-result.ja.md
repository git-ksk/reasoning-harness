# Natural-language E2E v11 — v0.4.1 Mistral canonical結果

Issue #254 は、immutable な v10 が released v0.4.1 / Issue #249 の typed `no_result` continuation の前提triggerへ到達しなかったため、fresh full-product successor として `natural-language-e2e-v11` をfreezeした。v11では trigger reachability、conditional mechanism conformance、downstream utilityを別々の観測として扱う。

## Canonical座標

- Actions run: `34129798774`, attempt 1
- freeze tag: `natural-language-e2e-v11-freeze`
- freeze commit: `a758af17a998493c1005702365b100e05b05f95d`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `57000`
- artifact: `10021729999`
- artifact digest: `sha256:5b84e62006e73d4bb9ee93a4793bad9b65d67f12b7a404ec325c55d14d2d13e3`
- result JSON digest: `sha256:f108582cddffb0eafd3d7ff122c1f27b22d30225f226b6aba69de8acc6e28383`
- attempt-marker digest: `sha256:f7b018d5621df82dd5a0e4d71fa25bbb19b24940957884b926db1897df747c34`

保存済みattempt markerは `unique-arclume-endpoint` で `live_case_launch_boundary_entered=true`、GitHub run `34129798774`、attempt `1` を記録している。したがってこのrunがcanonical v11 observationであり、in-placeのrerun / rescore / repair / tuningは禁止する。GitHub Actions runには実行logが保存され、uploaded artifactにはfreeze/preflight evidence、canonical result、stdout copy、attempt markerが保存されている。

## Aggregate結果

- completed cases: `13/13`
- hard correctness gate: **PASS**
- measurement observability / validity: **PASS**
- operational completeness: **PASS**
- report gate: **PASS**
- correctness-boundary violations: `0`
- exposed-text contract violations: `0`
- unsupported structured claims: `0`
- unsupported exposed assertions: `0`
- operational failures: `0`
- process operational failures: `0`
- typed operational action failures: `0`
- generation failures: `0`
- target recall: `0.60`
- tool-selection success: `0.80`
- investigation caseのgrounded-target coverage: `0.00` (`0/10`)
- false abstentions: `6`
- required admission rejections observed: `4/4`
- admission behavior coverage: `1.00`
- MCP path exposure: `1/1`
- session persistence valid: `true`
- session fork state valid: `true`
- session external calls replayed: `0`

frozen v11 semanticsではrun全体がscorableである。planner/grounding utilityの弱さはmeasurement dataとして残るが、correctness、operations、path observabilityを無効化しない。

## Trigger reachability / planner utility

predeclareしたfresh no-result follow-up 3件では、**trigger exposureは1/3**（`0.3333`）だった。

### `adaptive-emberquill-owner`

- target recalled: `true`
- planner calls: `1`
- first relevant capability: `emberquill-owner-cache`
- first relevant typed status: `no_result`
- trigger exposed: `true`
- selected sequence: `emberquill-owner-cache -> emberquill-owner-registry`
- actions executed: `2`
- stop reason: `resolved`

### `adaptive-fluxmere-owner`

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- trigger exposed: `false`
- trigger miss reason: `no_relevant_action`
- stop reason: `round_budget`

### `adaptive-glyntide-owner`

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- trigger exposed: `false`
- trigger miss reason: `no_relevant_action`
- stop reason: `round_budget`

後者2件はplanner selection utilityの観測であり、Issue #249 mechanism failureとして数えない。

## Conditional Issue #249 mechanism conformance

mechanism denominatorはtrigger-exposed caseのみなので **1**。

`adaptive-emberquill-owner` では:

- cacheがtyped `no_result`を返した;
- 直後のrelevant capabilityはsingle configured registryだった;
- `harness_no_result_followup_selections == 1`;
- relevant action sequenceにduplicate/reorderはなかった;
- mechanism classificationは `conformant`。

aggregate conditional mechanism resultは **1/1 conformant**、classificationは `all_exposed_conformant`。

これは、前提triggerへ自然言語経路で到達した1件において、released v0.4.1が#249 continuationを正しく実行した直接観測である。ただしdenominatorは1なので、このfrozen workload/model sliceについての記述的結果であり、広いmodel-level effect sizeを主張するものではない。

## Downstream utility

trigger-exposedしたEmberquillのregistry actionはadmitted evidenceを1件生成し、`verification_progress`へ進み、`downstream_followup_useful=true`だった。したがってdownstream utilityは **trigger-exposed opportunityでは1/1**、predeclareしたfollow-up 3件全体では **1/3**。

一方、Emberquill targetはgroundedせず、3件のfollow-up targetはいずれもgrounded finalizationに到達していない（`0/3`）。Emberquill finalizationも `unresolved` のため、mechanism conformanceをgrounded-answer successと同一視してはならない。

## Full E2E観測

### Correctness / admission

13件すべて完了し、correctness-boundary violationは0。必須4 rejection contract（stale evidence、scope expansion、authority-claim mismatch、untrusted source）はすべてexerciseされた。MCP laneではさらに `missing_observation_time` によりevidenceがrejectされた。reject/opaque evidenceがauthorityへself-promotionした例はない。

### MCP

`mcp-v11-v041-cargo-nonpromotion` はpinned GitHub MCP laneを1回invokeした。`mcp_path_exposed=true`、target recallはfalse、`mcp_output_authority_self_promotion=0`。v11/#247 semanticsではpath exposureをtarget utilityから独立して報告するため、target recall/grounding成功を主張せずにlaneの観測可能性を成立させている。

### Session

3件すべてstate semanticsを維持し、external call replayは0。`session-add-jadewisp` と `session-resume-fork-lumera` はgrounded answerへ到達した。`session-correct-krysal` はcorrection/persistence invariantを維持したがunresolvedで終了し、false abstentionを1件構成する。aggregateのsession persistence/fork-state checkはいずれもpass。

### Utility residual

残る主要課題はnatural-language utilityである。

- target recall: `0.60`;
- tool-selection success: `0.80`;
- investigation grounded-target successes: `0/10`;
- false abstentions: `6`;
- fresh no-result follow-up 3件のうち2件はtargetをrecallしたにもかかわらず、`round_budget`までactionを1回も選択しなかった。

これらは観測済み#249 conditional mechanism conformanceを否定しない。次の改善面はpost-`no_result` continuationではなく、planner/action selectionとdownstream groundingである。

## 解釈

canonical v11から主張できるのは以下。

1. このrunではreleased v0.4.1がfrozen v11 correctness boundaryとoperational completenessを維持した;
2. v11はfresh follow-up `1/3` で#249 predecessor triggerを実際に露出した;
3. そのtrigger exposureに条件づけると、released v0.4.1は観測1件中1件でexact-target Harness continuationを正しく実行した;
4. selected registryはadmitted evidenceとverification progressを作ったが、grounded targetには到達しなかった;
5. 残り2件のtrigger missはplanner/action-selection utility dataであり、#249 mechanism failureではない;
6. fresh full-product successor上でもMCP non-promotionとsession persistence/fork境界は維持された。

したがってIssue #254はtrigger-conditioned successor measurementとして完了できる。frozen v9/v10/v11はimmutable observationとして保持し、planner selectionまたはgrounded-answer utilityを改善する場合はv11をrerun/tuningせず、新しいissue/successorで扱う。
