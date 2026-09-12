# Reason 対話型ターミナルUX

Reason CLI 0.5.0開発ラインでは、Harness Engine 0.4.2のauthority / correctness semanticsを変えずに、日常利用向けterminal surfaceを追加します。

## dispatch boundary

- bare `reason`はstdinがTTYかつeffective outputがhuman-readableの場合だけinteractive REPLを起動します。
- `reason "TASK"`は従来どおりone-shot pathです。
- pipe/non-TTY stdinからREPLへ自動遷移しません。piped stdinは明示TASKに対するuntrusted contextのままです。
- config由来を含め`--format json`がeffectiveな場合はREPLへ自動遷移しません。automationが誤ってprompt待ちになることを防ぎます。
- structured subcommandは変更しません。

## command

- `/add <path>`: UTF-8 regular fileを現在のREPLの後続prompt向けuntrusted contextとして追加します。spaceを含むquoted pathはliteralに扱い、shell展開や評価は行いません。
- `/files`: 現在有効なcontext fileを表示します。
- `/clear`: context fileとin-memory conversation contextを消去します。
- `/help`: interactive commandを表示します。
- `/exit` / `/quit`: 終了します。
- 行末`\`でmultiline promptを継続します。

## authority / privacy

各promptは従来のHarness-owned natural execution pathを通ります。過去のuser/Reason exposed exchangeは後続promptへ`untrusted_context`としてだけ渡され、verified evidenceへ昇格しません。factを支えるには再検証が必要です。hidden chain-of-thoughtは保存も表示もしません。

#364ではshell-style prompt history fileもmanaged session fileも書きません。EOFはclean exitします。process-level Ctrl-Cでもpersistent interactive checkpoint自体が存在しないためhalf-written checkpointは残りません。managed continuation/checkpoint persistenceとcrash/concurrency durabilityは#365/#381、retention/purge policyは#379で扱います。
