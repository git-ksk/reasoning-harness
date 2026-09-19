# ADR-0004: Harness-owned investigation action materialization

Status: candidate実装はevaluation対象としてaccept。最終adoptionにはIssue #283のfresh holdoutが必要。

## Context

bounded investigation runtimeでは、evidence admission、verification、finalization、budget、answer safetyはすでにHarness-ownedである。一方Issue #283以前のcommon fallback planner contractでは、modelがexact `(target_id, capability_id)` pairまで出力していた。

Issue #282のfrozen baselineでは、correctness boundaryは両routine providerで維持されたが、Googleの1 trialでuseful evidence acquisition後㑫inadmissibleなtarget/capability pairingが提案された。通常のHarness validatorがrejectしたためauthority leakageは無かったが、utilityはstochasticなexecutable-ID selectionに依存していた。

runtimeはmechanical materializationに必要な情報をすでに持つ。

- canonicalなadmitted investigation target ID;
- targetのexact `expected_fact_key`;
- configured read-only capabilityと明示`supported_fact_keys`;
- configured時の明示pselection_priority`;
- attempted target/capability pair;
- terminal/action budget;
- #233 globally unique selector;
- #249 exact-target post-`no_result` continuation;
- #261 precedence selector。

したがって目的は「planning全体をdeterministicにする」ことではない。modelは**どのunresolved targetを続けるか**を選んでよいが、Harnessがmechanically導出できるexact capability IDまでmodelに所有させない。

## Decision

新しい内部model-facing contractを追加する。

- intent contract: `reason-investigation-intent-v1`;
- materialization policy: `target-intent-materialization-v1`。

既存contractは維持する。

- runtime: `bounded-investigation-v1`;
- plan: `reason-investigation-plan-v1`;
- legacy executable action: `reason-investigation-action-v1`。

intent vocabularyは次だけに限定する。

```text
continue(target_id)
stop
```

`capability_id`、tool argument、evidence、authority、fact value、receipt、verdictは含めない。

1つのexact targetに対し、Harnessが`acquire(target_id, capability_id)`をmaterializeできるのは次をすべて満たす場合だけ。

1. stateがnon-terminalでaction budgetが残る;
2. exact target IDが存在;
3. target㑫non-empty exact `expected_fact_key`がある;
4. capabilityがuntried・明示read-only・そのexact keyを明示列挙;
5. candidate capabilityが1つだけ、または全candidateに明示priorityがありhighest priorityが一意。

priority欠落、tie、keyless target、wildcard-only compatibility、eligible capabilityなし、write-capable tool、terminal stateではmaterializeしない。

このruleはtarget identity間を選ばない。same-key sibling targetが複数materializableでも別identityのままintent schemaへ出し、modelがexact target IDを1つ選べる。Harnessはmerge/canonicalize/同一視しない。

## State machine

```text
unresolved investigation
        |
        v
#249 exact-target no_result continuation?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
begin bounded round
        |
        v
#233 globally unique exact pair?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
#261 globally unique precedence choice?
        | yes
        +----------------------> existing exact action -> validate
        |
        no
        v
materializable exact target IDを計算
        |
        +-- none ----------------------> legacy action-v1 model selector
        |
        v
intent-v1 model selector: continue(target_id) | stop
        |
        v
Harnessがtarget-intent-materialization-v1でexact capabilityをmaterialize
        |
        +-- refusal -------------------> typed refusal telemetry
        |                                same-round legacy action-v1 fallback
        |
        v
existing action-v1 proposal
        |
        v
existing validate_action
        |
        v
read-only acquisition
        |
        v
ordinary admission -> qualification -> verification -> finalization -> answer safety
```

same-round fallbackは意図的である。provider/schema fallbackがinvalid/stale intentを返してもauthorityは与えないが、既存action-v1 pathを試す前にround budgetだけを繰り返し消費させない。

## Authority boundary

```text
MODEL
  plan proposal -----------+
  target intent -----------|---- untrusted planning only
                           v
                    HARNESS CONTROL
              exact target identity lookup
              exact-key compatibility
              read-only filter
              attempted-pair filter
              unique priority rule
                           |
                           v
                 executable action proposal
                           |
                    existing validation
                           |
                           v
                 ACQUISITION ADAPTER
                           |
                    untrusted output
                           |
                           v
          admission / qualification / verification
                           |
                           v
                 finalization / answer safety
```

model intentもHarness materializationもevidence、truth、verification、answer authorityを生成しない。materializationが所有するのはread-only acquisition actionの選択だけ。

## Compatibility

additive changeとする。

- `reason-investigation-action-v1`は変更せずfallbackとして維持。
- `planner_calls`は従来どおりlegacy action-v1 model-selector callのみ。
- `planner_intent_calls`をintent-v1 call用に追加。
- `harness_intent_materializations`を新Harness policyによるexact action数として追加。
- typed intent refusal count/recordはdiagnostic-only additive telemetry。
- `intent_contract` / `materialization_policy`はdeserialization default付きadditive identity。
- natural JSONにはadditiveな`intent_generations` provider observationを出せる。
- investigation configにrequired fieldは追加しない。
- static resolver、MCP、admission、verification、finalization、answer-safety contractは変更しない。

materialization条件を満たせないcheckout/stateは既存bounded action-v1 selectorへfallbackする。release-level rollbackもpre-#283 Engine coordinateへ戻せばよく、stored config migrationは不要。

## Session / idempotency

materializationはaction validationと同じ`attempted_pairs`を参照し、`validate_action`をbypassしない。session persistenceは完了したresolution attemptを保存し、executable callbackを保存しない。resume/inspect/fork replayは記録済みstateを再構成するだけでexternal acquisitionを再実行せず、#283 regressionで`external_calls_replayed == 0`を明示確認する。

## Evaluation rule

Issue #282はconsumed pre-change diagnostic evidenceでありadoption holdoutには再利用しない。

final adoptionには別freezeのfresh successor identityが必要。

- exact pre-change controlとcandidateを比較;
- exercised path / utility / hard correctness / operational completenessを分離;
- intent/materialization pathが実際にexerciseされたことを確認;
- exact denominatorを出し、failed/incomplete observationも保存;
- correctness-boundary regression 0を要求;
- repeated model agreementをtruth扱いしない。

fresh holdoutでutility/stability benefitが確認できなければ、deterministic testがgreenでもcandidateを自動adoptしない。
