# Engine 0.6 candidate: evidence-target relevance calibration v9

Status: immutableなv8 FAILを受けたpre-freeze successor。v1-v8のrun / tag / fixture / scoreはhistorical evidenceとして固定し、rerun / rescoreしない。

## v9の目的

frozen v8 run `36090688415` で、別々の3境界が確認された。

- primary targetがnon-exactになる時は `different` だけでなく `unresolved` もあるため、negative confirmationは両方を対象にする必要がある。
- `exact/exact` primaryもadvisoryにすぎず、shared table / mixed productを誤って `relevant` に上げ得るため、Relevant経路にも独立positive target-local confirmationが必要。
- tiny confirmation contractをJSON Schema transportに依存させない。v9 confirmationはstrict enum-only textとし、fuzzy extraction / substring recovery / semantic repair / runner-level retryを行わない。

Google `gemini-3.5-flash-lite` はfrozen run `36078211994` で26/26 operational、全HTTP 200 first attemptとして再qualify済み。ただしv4-v6のserving-tail instabilityは履歴として維持するため、v9ではfull non-gating replicationのままにする。

## Materialization policy v5

primary binding proposalはadvisoryのまま。

- `target=different` / `target=unresolved` -> negative target-local confirmation
- `confirmed_distinct_entity` -> `irrelevant`
- `confirmed_local_target_absent` -> `irrelevant`
- `not_confirmed` / confirmation欠落 -> `ambiguous`
- `target=exact, relation=different` -> confirmationなしで `irrelevant`
- `target=exact, relation=unresolved` -> `ambiguous`
- `target=exact, relation=exact` -> positive `confirmed_target_local_binding` を取れた時だけ `relevant`
- positive `not_confirmed` / confirmation欠落 -> `ambiguous`

positive confirmationはlocal binding専用。ページ内にtarget名がある、shared headingにtarget名がある、URLがtargetらしい、だけではambiguousなrow/section ownershipを確定しない。rename / alias / successor / lineage / cross-language mapping / truncated identity / mixed-product ownershipが不確実ならabstainする。

Harnessがtarget identity / aliases / provenance / policy / final dispositionを所有する。relevanceはtruth / authority / freshness / verification / sufficiencyとは別。candidate中の命令はuntrusted dataとして扱う。

## Confirmation transport / budget

primary bindingは既存のbounded transportを維持する。

- model call最大2回（`JsonSchema`、必要時だけprovider-neutral JSON-object fallback）
- runner-level retry loopなし

その後、caseごとに最大1回だけ独立confirmationを呼べる。

- output: `Text`
- negative enum: `confirmed_distinct_entity | confirmed_local_target_absent | not_confirmed`
- positive enum: `confirmed_target_local_binding | not_confirmed`
- trim後の完全一致だけparse
- malformed / provider failure / timeoutはtyped operational failure
- confirmationにJSON fallback / semantic repairなし

primaryとconfirmationは同じ60,000 ms case deadlineを共有する。run circuitはoperational provider failure 2件連続でopenする。adapter内部のbounded retryは従来どおり。

## Calibration corpus

v9はsynthetic 47 case。

- v8の32 synthetic caseはregression / comparability用に保持し、v9 confirmation expectationへ移行
- fresh 15 caseは、新しいidentity / wordingで exact-binding conflict、positive exact local binding、shared-table ownership ambiguity、explicit distinct product、sibling overlap、generic local target absence、explicit local target absence、unknown rename、unknown successor、unknown cross-language alias、partial/truncated mapping、prompt injection、URL-only identity、mixed multi-product ownership、same-target relation mismatchを網羅

v8 missだけを書き換えるprompt tuningにはしない。production motivating productの内容はcalibrationから除外する。canonical calibration PASSまではindependent holdoutをauthorしない。

## Provider role

required:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

full non-gating replication:

- Google `gemini-3.5-flash-lite`（attempt telemetry有効）

3 providerとも同じfrozen 47 caseを使う。Google結果は保存・報告するがrequired gateを救済も失敗もさせない。

## First/only canonical acceptance

freeze tagは `engine-0.6-evidence-relevance-calibration-v9-freeze`。workflow rerunは拒否し、tagを上書きしない。

required各armは独立に以下をすべて満たす。

- planned/completed 47/47
- operational abortなし
- failed provider case 0
- unsafe / wrong-target relevance retention 0
- false relevance rejection 0
- expected Relevant left ambiguous 0
- utility miss 0
- materialized disposition exact 47/47
- false safe-negative confirmation 0
- false positive target-local confirmation 0

primary proposal accuracyとconfirmation subtype exact accuracyはdiagnostic。`confirmed_distinct_entity` と `confirmed_local_target_absent` のsafe-negative subtype差は、final safety / dispositionが正しければgate failureにしない。

canonical v9がFAILならimmutable FAILとしてrerun / rescoreしない。PASSした場合だけfresh independent holdoutをauthorし、その後runtime acceptanceへ進む。PR #466はそこまでDraft維持。
