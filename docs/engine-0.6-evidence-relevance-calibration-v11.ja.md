# Engine 0.6 candidate: evidence-target relevance calibration v11

Status: immutable v10 FAILのpre-freeze successor。v1-v10のfixture / tag / observation / scoreは履歴証拠として変更しない。first/only frozen v11 canonical calibrationがPASSするまでindependent holdoutのauthoringは禁止する。

## v11が必要な理由

v10ではv9のoperational defectを解消し、Mistral / Groq / Googleの全armが56/56、provider failure 0で完走した。残ったのはsemantic architectureの問題。primary bindingを誤るとsecondary stageをbypassでき、単一secondary action (`safe_to_reject` / `safe_to_accept`) がhard dispositionを直接許可しすぎた。その結果、transportが正常でも3 providerすべてでunsafe rejection / acceptanceが発生した。

v11ではsafety gateを緩めず、fuzzy repairも追加しない。ownership boundary自体を変更する。

## Two-key local qualification

primary bindingは従来通りprovider-neutralなadvisory pair。

- `target_binding = exact | different | unresolved`
- `relation_binding = exact | different | unresolved`

primary promptは、candidate内のinstruction/control textをuntrusted dataとして無視し、factual relation mismatchと誤認しないよう明示的に強化する。

その後、**primary routeに関係なく全ケースで独立local qualification guardを必ず実行**する。guardはfinal accept/reject actionを返さず、次のfactだけを返す。

- `target_support = supported | not_supported | unresolved`
- `relation_support = supported | not_supported | unresolved`
- `identity_mapping_risk = absent | present | unresolved`
- `ownership_scope_risk = absent | present | unresolved`
- `context_completeness_risk = absent | present | unresolved`
- `explicit_local_absence = present | absent | unresolved`

risk fieldはfail-closed。不確実性をlocal materialから排除できない場合は`absent`ではなく`unresolved`。rename / alias / successor / cross-language / version-lineage不確実性はidentity risk、shared/clipped ownershipはownership risk、URL-only・必要context欠落・partial/truncated materialはcontext risk。factual disagreement、staleness、authority、verification、answer sufficiencyはdownstream concernであり、それ自体をrelevance riskにしない。

## Materialization policy v7

final dispositionはHarnessが所有する。

hard `Relevant`には全条件の一致を要求する。

- primary `target=exact` + `relation=exact`
- guard `target_support=supported` + `relation_support=supported`
- 3 riskすべて`absent`
- strict policyではHarness-owned identity anchor floorを満たす

hard `Irrelevant`にはrisk-freeな独立negative agreementを要求する。

- primary `target=different` + guard `target_support=not_supported`
- または primary targetがnon-exact + guard `target_support=not_supported` + `explicit_local_absence=present`
- または primary `target=exact, relation=different` + guard `target_support=supported, relation_support=not_supported`

それ以外はすべて`Ambiguous`。特にexact-target `relation=different` primaryだけではhard rejectしない。

CanonicalUrlだけにtarget名があり、非URLのHarness-owned anchorが無い場合はmodel outputに関係なくdeterministic hard floorとして`Ambiguous`にする。navigation/footer-only mentionはこのURL hard floorとは分離し、substantive passageが明確に別製品へscopeされ、2つのsemantic keyが一致すればreject可能とする。

## Transport / budget

primary bindingは`JsonSchema` + 最大1回のprovider-neutral `JsonObject` transport fallback。local qualificationも独立して`JsonSchema` + 最大1回の同等fallback。fallbackはtask/system semanticsを維持し、semantic retryやfuzzy repairではない。

全ケースでlocal qualificationを実行する。primary + qualificationは同一60,000 ms case deadlineを共有する。各stage最大2 model callなので、両stageでtransport fallbackが必要な場合のみ絶対上限4 call/case。2連続operational failure circuitは維持する。v10で導入したGoogle seed normalizationもprovider adapterに維持する。

## Calibration corpus

v11は65 synthetic case。

- v10の56 caseをregression/comparabilityとして保持し、local-qualification expectationを固定。
- fresh v11 9 case: injected relation-control text、indirect alias uncertainty、positive-looking shared ownership、comparison sibling scope、explicit generic absence、exact-target wrong relation、stale/contradictory-but-relevant、URL-only hard floor、Harness-owned alias positive。

expected final dispositionはRelevant 19 / Irrelevant 24 / Ambiguous 22。22 caseは少なくとも1つのfail-closed qualification riskを持ち、6 caseはexplicit local absenceを持つ。production motivating productはfixtureへ含めない。

## Canonical provider / gate

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite`

first/only frozen canonicalは`engine-0.6-evidence-relevance-calibration-v11-freeze` tag、attempt 1、checksummed exact surfaceのみで実行する。各required armは独立して次を満たす。

- 65/65 operational completion、provider failure 0、provider-attempt telemetry complete
- local qualification expected/invoked 65/65（primary-route bypassなし）
- qualification risk miss 0
- wrong-target / unexpected relevance retention 0
- false relevance rejection 0
- expected Relevant left ambiguous 0
- utility miss 0
- materialized disposition exact 65/65

primary proposal exactness、local qualification全field exactness、spurious risk blockはdiagnostic。final dispositionとsafety risk missをacceptance boundaryとする。

v11がFAILならimmutable FAILとしてrerun/rescoreしない。canonical PASS後にのみfresh independent holdoutをauthor/freezeする。holdout PASS後にruntime integration acceptanceへ進む。PR #466は全stage完了までDraft維持。
