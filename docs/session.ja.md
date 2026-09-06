# 再開可能な自然言語セッション

Issue #213 は、既存の typed `ReasoningThread` control plane を明示的な product session surface に接続する。conversation history を evidence authority にする機能ではなく、別系統の chat/runtime を新設するものでもない。

machine identity: `reason-session-v1`

continuation policy: `session-replay-only-acquisition-v1`

## コマンド

通常の自然言語 product path と同じ option で session を開始する。

```bash
reason session start \
  --store .reason/session.json \
  --provider mistral \
  --model ministral-8b-latest \
  "deployment regionを確認して"
```

最初のturnはnative natural-language runtimeで実行する。candidate、typed resolution attempt、accepted artifact、安全なcheckpointを`ReasoningThread`へ記録し、threadをinterruptしてsession fileをatomicに保存する。

provider/tool acquisitionを再実行せずにinspect/resumeできる。

```bash
reason session inspect --store .reason/session.json --format json
reason session resume  --store .reason/session.json --format json
```

後から資料・structured inputを追加する。

```bash
reason session add --store .reason/session.json \
  --file incident-note.txt \
  --fact service.owner=platform \
  --hypothesis service.region=eu-west-1
```

以前の明示premiseを訂正する。

```bash
reason session correct --store .reason/session.json \
  --premise service.region=eu-west-1
```

source historyを変更せずsafe checkpointからforkする。

```bash
reason session fork --store .reason/session.json \
  --checkpoint session-checkpoint-1 \
  --out .reason/alternative.json \
  --new-id alternative-lineage
```

historical threadを明示的にfinalizeする。

```bash
reason session close --store .reason/session.json
```

finalized threadはimmutableである。継続する場合は以前のsafe checkpointから`session fork`する。

## 永続化するもの

session fileが保持するのは明示的なtyped product/control stateだけである。

- `reason-session-v1` contract identity;
- session開始時のprovider/model/max-token/answer-safety identity;
- start時のresolver/admission/trusted-verifier surface identityとconfig source名;
- serializableな`ReasoningThread` event/checkpoint state;
- turnごとのsafe checkpoint IDとHarness-finalized exposed answer result。

external acquisitionを実行済みなら、記録済み`ResolutionAttempt`にadapter/admission config identity、cost/call telemetryが残る。replayはそのrecordを読むだけでadapterを再invokeしない。

hidden chain-of-thought、private model scratch state、既存Harness finalization contract外のraw renderer proseはsession stateに含めない。

## add/correctのtrust semantics

`session add`と`session correct`はstate changeであり、evidence shortcutではない。

- `--file`は保存後も`untrusted_context`として再投入し、proseからfactを自己昇格させない。
- `--fact KEY=VALUE`は明示user structured evidenceだが、通常のHarness verification pathを通る。
- `--hypothesis KEY=VALUE`はHarness-owned targetであって、そのvalueの証明ではない。
- correctionはreplacement candidate/artifactをacceptする前にtyped `input_changed` / `input_state_invalidated` eventを記録する。
- revalidation中はstale finalizationをclear/suppressする。

continuation turnは保存済みprovider/model/safety identityを使い、同じnative candidate -> grounding -> qualification -> verification -> finalization pathへ戻る。

## replayと外部side effect

`session inspect`、checkpoint replay、`session resume`、`session fork`は保存済みstateを再構成するだけで、記録済みexternal resolver、MCP tool、trusted-command verifier、investigation acquisitionを再実行しない。

`session add` / `session correct`のv1は意図的に`session-replay-only-acquisition-v1`を使う。start turnのresolver/MCP/investigation設定を暗黙replayしない。新turnでは保存済みmodelをcandidate generation/renderingに使えるが、acquisitionは明示追加されたmaterialだけから行う。resumeだけで過去のexternal side effectが再発することを防ぐためである。

## failure時

continuation model callの前にinput changeとtyped invalidationをatomic保存する。providerがその後失敗しても、以前のfinal answerをcurrentとして再表示しない。session outputは`pending_revalidation: true`を返し、stale `finalization`を省略する。

以前のsafe checkpointはhistoryに残るためfork recoveryできる。contract/runtime/safety identityが一致しなければ、persisted stateを勝手に再解釈せず`session_incompatible`でfail closedする。

## coreとproduct persistence

core自体は引き続きfilesystem/database/cloud backendを持たず、`ReasoningThreadStore`はabstractのままである。`reason session`は最初の薄いproduct adapterであり、明示local JSON fileをatomic replaceする。将来database/cloud storeへ差し替えてもevidence authority semanticsは変えない。
