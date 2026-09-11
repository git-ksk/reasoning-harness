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

## 保存・削除

backendにはprovider/default-account単位のsave / replace / deleteを実装済みです。ユーザー向けの`reason auth login/list/status/logout`はIssue #362で追加し、このbackendをそのまま使います。別のsecret storeは作りません。

credential replacementはOS store上の1つの論理updateとして扱い、deleteは選択provider accountだけに限定します。storage namingは将来のwork / personal named accountを追加してもraw secret bytesをconfig file経由でmigrationしなくてよい形です。

## trust boundary

provider credentialはoperational secretでしかありません。Keychain / Credential Manager / Secret Serviceからkeyを読めても、evidence、authority、verification receipt、trusted claimにはなりません。Harness Engine 0.4.2のcorrectness semanticsは変更しません。
