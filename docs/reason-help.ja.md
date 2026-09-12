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

## Plain / accessibility terminal presentation

`reason --plain`で、staticかつaccessibility-friendlyなhuman terminal outputを選べます。plain modeではprogress / decorative presentationを抑制しつつ、promptとCtrl+Cのsemanticsは維持します。`NO_COLOR`が存在する場合、`TERM=dumb`、またはhuman output streamがredirect / non-TTYの場合も同じplain policyを自動選択します。plain modeではANSI color、spinner animation、cursor-control依存を導入しません。

human textはterminal幅に合わせた危険なbyte truncationを行わず、Unicode内容を途中byteで切りません。JSON output contractはこのpresentation policyで変更しません。

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

low-level JSONを手書きせず、単一active local read-only MCP acquisition sourceを管理できます。

```text
reason mcp add inventory --program /path/to/mcp-server --arg=--stdio --tool lookup_item
reason mcp test inventory
reason mcp inspect inventory
reason mcp list
reason mcp remove inventory
```

`reason mcp test`はMCP negotiationと`tools/list` discoveryを行い、selected toolが`readOnlyHint=true`を宣言することを必須にします。**tool自体は実行しません。** 管理対象はuser-scoped non-secret configで、`inspect` / `list`はargument valueではなくcountだけを表示し、credential/tokenらしいsecret引数は`add`でrejectします。remote / Streamable HTTP OAuthは#386へ分離しています。

optional `reason-mcp` binaryは別surfaceで、Reasonのselected operationをexternal MCP clientへ公開するものです。MCP acquisition outputはauthorityではなくdataのままです。`docs/mcp-resolver.ja.md`と`docs/mcp-product-surface.ja.md`を参照してください。

## Update

`reason update --check`でavailability確認、`reason update`でprovenance検証済みupdate、`reason update --rollback VERSION`で明示rollbackを行います。full lifecycle contractは`docs/update-rollback-uninstall.ja.md`を参照してください。
