# Update / rollback / uninstall

日本語 | [English](update-rollback-uninstall.md)

**Status:** Reason CLI 0.5.0開発ライン。split release（`reason-v*`）のみ対象です。

Reasonのlifecycle変更は、明示的・provenance検証済み・可逆であることを前提にします。historical unified releaseの`v0.4.2`以前はsplit-release attestation contractより前なので、self-update / rollback対象にはしません。

## 変更せず確認する

```bash
reason update --check
reason update --check --format json
```

`--check`は公開済み`reason-v*`を探索し、`release-manifest.json`だけを取得してGitHub OIDC/Sigstore attestationを検証します。Reason CLIとHarness Engineのversionを別々に表示し、native archiveのdownload・置換は行いません。

自動discoveryはSemVerのstable `reason-v*`だけを対象にします。`-beta` / `-rc`等のprereleaseは自動選択せず、試す場合は`--version`で明示します。

明示versionも確認できます。

```bash
reason update --check --version 0.5.1
```

## updateを適用する

```bash
reason update
```

interactive terminalではCLI / Engine identityを表示して確認を求めます。automationでは明示確認が必要です。

```bash
reason update --yes --format json
```

apply pathは次を順に検証します。

1. exact repository / signer workflow / `reason-vX.Y.Z` source refに対する`release-manifest.json` attestation;
2. manifest内のtarget CLI / Engine identity;
3. platform archiveのGitHub/Sigstore attestation;
4. archiveと`SHA256SUMS`のdigestをattested manifestと照合;
5. archive entryを`SHA256SUMS`とも照合;
6. Unixではarchive path safety、さらに展開後`reason --version` identity;
7. Unixでは同一directoryでatomic replacement、Windowsではprocess終了直後のstaged replacement。

Harness Engine SemVerが変わる場合は、次を明示しない限りapplyを拒否します。

```bash
reason update --allow-engine-change
```

これによりpresentation-onlyなCLI更新とreasoning/correctness engine変更をsilentに同一視しません。

## 明示rollback

rollbackは別operationで、現在より古いsplit releaseだけを受け付けます。

```bash
reason update --rollback 0.5.0
```

provenance、manifest、checksum、Engine change、confirmationはupdateと同じ契約です。`reason update`は古いversionを拒否してrollbackを案内し、rollbackは同一・新しいversionを拒否します。

## uninstall

まずpreviewできます。

```bash
reason uninstall --dry-run
```

実際のbinary削除:

```bash
reason uninstall
```

defaultは保守的です。

- native OS provider credentialは保持;
- `config.json` / `project-trust.json`は保持;
- explicit pathのsession fileは探索・削除しない;
- provider environment variableは変更しない。

明示purgeもできます。

```bash
reason uninstall --purge-data --purge-credentials
```

`--purge-data`はReason管理の`config.json`と`project-trust.json`だけに限定し、任意session fileを再帰探索しません。`--purge-credentials`はnative OS credential storeのsupported provider entryだけを削除します。non-interactive mutationでは`--yes`が必須です。

## Trust root

`reason-v*`のlifecycle mutationではSHA-256だけへfallbackしません。attestation検証にはGitHub CLI 2.93.0以上が必要です。詳細は[Release provenance](release-provenance.ja.md)を参照してください。
