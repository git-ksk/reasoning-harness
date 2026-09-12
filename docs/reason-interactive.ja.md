# Reason 対話型ターミナルUX

Reason CLI 0.5.0開発ラインでは、Harness Engine 0.4.2のauthority / correctness semanticsを変えずに、日常利用向けterminal surfaceを追加します。

## dispatch boundary

- bare `reason`はstdinがTTYかつeffective outputがhuman-readableの場合だけmanaged interactive sessionを起動します。
- `reason "TASK"`は従来どおりone-shot pathです。
- `reason -c`はcanonical current project directoryで最後に更新されたcompatible managed sessionをcontinueします。
- `reason -r <id>`はstableなfull/short idでmanaged sessionをresumeします。TTYでbare `reason -r`ならnumeric pickerを開きます。
- `reason session list`はbacking file pathを露出せずmanaged sessionを一覧化し、`--format json`も利用できます。
- pipe/non-TTY stdinからREPLへ自動遷移せず、`-c/-r`もhuman TTY外ではprompt待ちせずfail closedします。
- config由来を含め`--format json`がeffectiveな場合はREPLへ自動遷移しません。
- structured subcommandと既存の低レベル`reason session ... --store` contractは互換維持します。

## plain / accessibility presentation

`reason --plain`はhuman TTY上のREPLを維持しつつ、decorative progressを抑制し、より単純なstatic human renderingを使います。`NO_COLOR`と`TERM=dumb`でも同じplain presentation policyを自動選択します。promptは通常のline input（`reason> ` / `... `）のままで、spinner / cursor-controlへ依存せず、Ctrl+Cも同じtyped cancellation semanticsを維持します。redirect / non-TTYとJSON invocationは引き続きnon-interactiveかつdecoration-freeです。

human outputをterminal widthに合わせてtruncateしないため、Unicode textをbyte途中で切らずそのまま出力します。

## command

- `/add <path>`: UTF-8 regular fileをpersisted untrusted context snapshotとして追加します。spaceを含むquoted pathはliteralに扱い、shell展開や評価は行いません。
- `/files`: 現在有効なpersisted untrusted context snapshotを表示します。
- `/status`: 直前のcompleted turnからverified fact、unresolved/qualified item、typed acquisition note、statusを表示します。
- `/evidence`: 直前のcompleted turnからsupporting evidence provenanceと、別表示されたuntrusted-context sourceを表示します。
- `/usage`: managed sessionの累積provider/resolver usageを表示します。設定済みbudgetはturn/resumeをまたいで適用します。
- `/clear`: 以後のpromptへ過去conversation/contextを持ち越さないようにします。既存のtyped turn history自体は削除しません。
- `/help`: interactive commandを表示します。
- `/exit` / `/quit`: 終了します。
- 行末`\`でmultiline promptを継続します。

## managed-session model

managed conversationはproduct layerの`reason-managed-session-v1` wrapperです。成功した各user promptは、それぞれ既存のtyped `SessionFile` / `ReasoningThread`とsafe checkpointとして保存します。Coreの「1 thread = immutable task」を崩してchat transcript化する実装は行いません。

managed checkpointが進むのはturnが成功した場合だけです。provider/model run失敗時は直前のpersist済みstateを維持します。resume時は保存済みprovider/model/max-token runtimeをpinし、互換しない明示overrideはsilent switchせずfailします。

過去のuser/Reason exposed exchangeは後続turnへ`untrusted_context`としてだけ渡され、過去回答に出たという理由でverified evidenceへ昇格しません。hidden chain-of-thoughtは保存も表示もしません。

## durability / privacy boundary

#365ではmanaged session選択、成功turn checkpoint、stable idまでを担当します。store locking、optimistic concurrency/generation、interrupted write recovery、corruption handling、update/rollback migrationは#381、private local permission、retention/purge、ephemeral/no-persist、outbound data disclosureは#379で扱います。
