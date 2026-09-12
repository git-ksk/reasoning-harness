# Reason CLI help / examples / completions

日常利用の入口は自然言語taskです。

```text
reason "原因を説明し、verified factとunresolved pointを分けて"
```

human TTYでbare `reason`を実行するとinteractive REPLを開始します。最新managed sessionは`reason --continue`、選択またはID指定resumeは`reason --resume`です。

## Discoverability

`reason --help`はcommon pathを前面に出し、深いsurfaceへの導線を示します。command固有contractは`reason help <command>`、copy-paste例は`reason examples [topic]`で確認できます。

代表例:

```text
reason setup
reason --file notes.txt "このevidenceで何がsupportできる？"
reason session list
reason --format json "このclaimを確認して"
reason update --check
```

fileは別途admission / verificationされない限りuntrusted contextです。JSON / pipe modeは引き続きnon-interactiveかつdecoration-freeです。

structuredな`run`、`semantic-check`、`verify`、`schema`はadvanced product / automation用として残します。research/evaluationの`eval`、`eval-resolution`、`eval-judges`も削除せず、top-level helpでは意図的にde-emphasizeします。

`reason doctor`はPhase 2対象外なのでhelpでは案内しません。

## Shell completions

completion scriptはstdoutへ生成するだけで、shell設定を変更しません。

```text
reason completions bash
reason completions zsh
reason completions fish
reason completions powershell
```

導入は各shell標準のcompletion手順に従い、生成結果をredirect/sourceしてください。

## Auth / setup

初回provider / credential / model readinessは`reason setup`を使います。詳細surfaceは`reason auth --help`、`reason models`、`reason model --help`、`reason config --help`で確認できます。credentialはnative OS credential storeに保持し、config surfaceはnon-secretです。

## MCP

MCP product integrationは`reason mcp` subcommandではなく、別のoptional binaryです。

```text
reason-mcp --reason-command /path/to/reason
```

`reason-mcp`はselected operationをnative Reason runtimeへdelegateし、native product contractを維持します。MCP resultが自動的にtrusted authorityへ昇格することはありません。protocol boundaryは`docs/mcp-product-surface.ja.md`を参照してください。

## Update

`reason update --check`でavailability確認、`reason update`でprovenance検証済みupdate、`reason update --rollback VERSION`で明示rollbackを行います。full lifecycle contractは`docs/update-rollback-uninstall.ja.md`を参照してください。
