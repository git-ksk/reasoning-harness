# プロバイダー認証情報の安全な保存

日本語 | [English](secure-credentials.md)

**状態:** Reason CLI 0.5.0開発ラインで実装。tagged `v0.4.2` releaseはまだprovider environment variableのみを使います。

Reasonはprovider API keyを`reason-config-v1`、project file、session、evidence、authority stateへ保存しません。0.5.0のcredential backendはOS-nativeなsecure storeを使います。

- macOS: Keychain Services;
- Windows: Windows Credential Manager;
- Linux/*nix: 利用可能な場合はSecret Service。

OS storeを使えない場合に**平文fileへfallbackしません**。

## 実行時の優先順位

providerごとのcredential解決順は固定です。

1. provider environment variableが存在すればそれを使う;
2. 存在しなければReasonのOS-store entryを読む;
3. どちらにもなければtyped credential errorで失敗する。

CI、container、server、headless環境ではenvironment variableを引き続き正式にサポートします。

| Provider | Environment variable | OS-store account |
| --- | --- | --- |
| Mistral | `MISTRAL_API_KEY` | `provider:mistral:account:default` |
| Google / Gemma | `GEMINI_API_KEY` | `provider:google:account:default` |
| Groq | `GROQ_API_KEY` | `provider:groq:account:default` |
| NVIDIA Hosted NIM | `NVIDIA_API_KEY` | `provider:nvidia:account:default` |

service identityは`io.github.git-ksk.reason-cli.credentials.v1`としてversioningし、account identityは`provider:<provider>:account:<name>`形式にします。0.5.0は`default`から始めるため、将来`work` / `personal` accountを追加してもraw secret bytesのmigrationは不要です。Googleとcompatibility selectorの`Gemma`は意図的に同じGoogle credentialを共有します。

environment variableが存在するのに空または不正な場合は、OS storeへ黙ってfallbackせず失敗します。明示overrideの設定ミスで別credential sourceが勝手に選ばれるのを防ぎます。

## 失敗時の扱い

- credentialなし: `credentials`;
- OS credential serviceが利用不可 / locked / unsupported: `credential_store_unavailable`;
- OS store内のdataが不正など: `credential_store_error`。

error messageやJSON failureへcredential値を出しません。Secret Serviceがないheadless Linuxでは、provider environment variableを使うかplatform credential serviceを用意する案内を返し、平文credential fileは作りません。

## `reason auth`で管理する

Reason CLI 0.5.0開発ラインでは、次のuser-facing commandを使います。

```bash
# TTYでは入力を非表示。対話terminalならprovider省略でpickerも使える。
reason auth login mistral

# automation向け。secretを通常argvには載せない。
reason auth login mistral --from-env
printf '%s\n' "$MISTRAL_API_KEY" | reason auth login mistral --stdin

reason auth status mistral
reason auth list
reason auth logout mistral
```

既存OS-store credentialがある場合、`login`は`--replace`を明示しない限り上書きしません。`status` / `list`が返すのはsource/state（`environment`、`os_store`、`missing`、typedなinvalid/unavailable state）のみで、mask済み断片・prefix・suffixを含めcredential値は一切表示しません。`logout`は選択provider/default-accountのOS-store entryだけを削除し、environment variableは変更しません。environment variableが残っていればlogout後もruntimeではそれが優先されます。

interactive入力はhidden/no-echo TTYです。non-interactiveでは`--stdin`または`--from-env`を使い、`--api-key` / `--secret` / `--password`のようなsecret-valued argv flagは意図的に提供しません。credential replacementはOS store上の1つの論理updateとして扱い、deleteは選択provider accountだけに限定します。storage namingは将来のwork / personal named accountを追加してもraw secret bytesをconfig file経由でmigrationしなくてよい形です。

## trust boundary

provider credentialはoperational secretでしかありません。Keychain / Credential Manager / Secret Serviceからkeyを読めても、evidence、authority、verification receipt、trusted claimにはなりません。Harness Engine 0.4.2のcorrectness semanticsは変更しません。
