# Natural-language E2E v10 — v0.4.1 successor measurement

Issue #252 は `natural-language-e2e-v10` を、exact shipped v0.4.1 を測る fresh successor として定義します。v9 は `b42b287b57b3e7e6f19f69464639c5c77a1fe707` / `natural-language-e2e-v9-freeze` / Actions `34079947614` / artifact `10003417402` / digest `sha256:261dd6de3c05053bca3967a29b941cecd6fe4430ecba2dde01c282f5810b72a7` の immutable historical evidence のまま保持します。

v10 では v9 を再実行・再採点・tuning・修復しません。v9 と v10 の比較は structural / descriptive な後継観測であり、1回のsuccessor measurementを因果的な改善量として扱いません。

## Frozen product coordinate

- tag: `v0.4.1`
- commit: `29a9e4be6273dbffeda324e15517dc64930ad315`
- natural output: `reason-natural-output-v4`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `53000`
- max tokens: `1024`
- inter-case pacing: `1500ms`

measurement-only fileはrelease coordinateから追加できますが、`Cargo.toml`、`Cargo.lock`、`crates/` はreleased product coordinateとの差分 `0` を必須にします。

## 測定したいこと

v9 の Umber case は cache が typed `no_result`、registry は引き続き relevant、planner calls `4`、registry invocation `0`、stop `round_budget`、correctness violation `0`、operational failure `0` でした。これは correctness defect ではなく utility / avoidable abstention の residual として扱いました。

v0.4.1 Issue #249 は typed `no_result` 後の狭い Harness-owned continuation を追加しました。同一exact targetだけ、かつそのtargetの `expected_fact_key` をsupportする未試行のexplicit read-only capabilityがちょうど1件のときだけです。target merge、authority promotion、fact inference、finalization recoveryは行いません。

v10 は、このshipped mechanismがfresh surface上で実際にexerciseされ、通常のvalidation / acquisition / admission / verification / finalization / answer-safetyを壊していないかを測ります。

## #249 dedicated lane

fresh Cobalt lane は意図的な interventional mechanism-exercise fixture です。natural-language taskはv9と同じ構造を保ち、cacheを先に、`no_result` 後にregistryへ進むことを明示します。これは#249 continuation自体をmodelの自然な探索能力から切り分けて測るためです。このlaneを「modelが自然にこの順序を発見する」ことの証拠として解釈してはいけません。

このlaneの成功は次を同時に要求します。

- cacheが最初のcapabilityで typed `no_result` になること;
- registryだけが後続実行され、通常経路で `applied_evidence` または `verification_progress` になること;
- `harness_no_result_followup_selections == 1`;
- `planner_calls == 1`。これは`no_result`後に追加のstochastic action-planner callがないことを示す補助的なmechanism invariantであり、correctness/utility scoreではない;
- duplicate action executionがないこと;
- correctness-boundary violationとoperational failureが0であること。

telemetryだけでは成功判定しません。

## Freshness / contamination discipline

v10 は fresh investigation 8件 + session 3件です。pre-observation testで、観測済みpredecessorに対する case ID、exact task text、target key、fresh marker、configured source ref、provider base seed の再利用を機械的に拒否します。frozen historical refに対するfresh marker再利用も確認します。過去identifierを除外するためにassertionを弱めません。

v10 MCP non-promotion laneはdigest固定のofficial GitHub MCP imageを使い、`refs/tags/v0.4.1` の `CHANGELOG.md` を読む一方、case/source/target identityはfresh v10 identityにします。

## Deterministic pre-observation contracts

live credential投入前に、no-model/no-network fixture / admission preflight と relevant v0.4.1 deterministic contractsを通します。#249についてはpositive continuationに加え、same-key siblingのexact-target isolation、remaining explicit capabilityがexactly oneである条件、non-`no_result` outcome、keyless/wildcard capability、terminal budgetをnegative boundaryとして固定確認します。

workflowはv1-v9 frozen surface untouchedと、測定対象production source unchangedも確認します。

## Scoring separation

`hard_correctness_gate_passed` は correctness-boundary violation `0` を要求します。

`measurement_validity_passed` は別gateとして、11/11 operational completion、operational failure `0`、freshness/scope/authority/identity rejection `4/4`、MCP live `1/1`、#249 dedicated lane `1/1`（Harness telemetry / planner-call / no-duplicate contract込み）、session persistence/fork valid・external replay `0`、token-usage case coverage `>= 0.70` を要求します。

utility metricは観測値として分離し、観測後にgateへ合わせてtuningしません。

## Freeze / canonical-attempt discipline

corpus、checksum、evaluator、tests、scoring identity、provider/model、seed、budget、MCP coordinate、docs、workflowをlive observation前にfreezeします。live workflowは `natural-language-e2e-v10-freeze` を直接checkoutし、`HEAD` がfreeze-tag commitと一致することを必須にします。

checkout / identity / deterministic validation / build / pinned-MCP acceptance / credential presenceなど、live-case launch前の失敗はmeasurement surfaceへ入っていないためretry可能です。frozen evaluatorがconfigured providerで最初のlive caseをlaunchした時点から、そのrunをcanonicalとします。その後operational failureになっても結果として保持し、attempt marker、GitHub run identity、logs、artifactを保存します。この境界後は同じv10 identityのrerun / rescore / fixture・evaluator修復 / threshold tuningを行わず、semantic変更はsuccessor identityでのみ行います。

## Observation status

freeze準備時点では v10 live observation は未実行です。結果はpre-observation freeze完了後まで意図的に記載しません。
