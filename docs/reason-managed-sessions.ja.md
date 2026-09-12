# Managed session durability

Reason CLI 0.5.0ではinteractive conversationの使い勝手をproduct-layer managed storeで扱い、各successful turnは従来どおりtyped `SessionFile` / `ReasoningThread` checkpointとして保持します。Harness Engine 0.4.2のauthority semanticsは変更しません。

## store / concurrency

managed sessionはReason user data/config root配下の`sessions/`に置きます。mutationとrecoveryではOS advisory lockを使います。loadしたsessionは、読み込んだexact bytesのSHA-256 digestもin-memoryだけで保持します。このdigestは永続化しません。save前にstore lockを取得し、現在のfile bytesとdigestを比較します。別processがsessionを更新・削除していた場合は、newer stateをsilent overwriteせず`session_conflict`で失敗します。

writeはnew temp file、`sync_all`、atomic replacementで行います。Unixはsame-filesystem `rename`、Windowsはreplace-existing / write-through指定の`MoveFileExW`を使います。対応platformではsession directoryもsyncします。crashで残った`.tmp-*`はstore lock取得中にだけ回収します。

## corruption / compatibility

`reason session list`はprovider / MCP / resolver / replayを一切呼ばず、compatible sessionとcorrupt / incompatible fileを分けて表示します。interactive pickerにはcompatible sessionだけを出します。

persistent contractは`reason-managed-session-v1`のまま維持します。concurrency metadataはin-memoryのみなので、#365で作ったformatはsupported earlier 0.5.x buildでも読めます。現在の0.5.x policyではdestructive migrationを行いません。将来older supported CLIで読めないformatが必要になった場合はnew contract/versionへ分離し、destructive migration前に明示的backup/exportを要求します。それ以外はupdate/rollback時にsilent rewriteせずrefuseします。

このdurability layerはhidden chain-of-thoughtやcredentialを保存しません。permission / retention / purge / ephemeral policyは#379で別途定義します。
