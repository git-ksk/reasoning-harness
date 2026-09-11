# プロジェクトtrust

日本語 | [English](project-trust.md)

**状態:** Reason CLI 0.5.0開発ラインで実装。tagged `v0.4.2` releaseにはまだ`reason trust` commandは含まれません。

Reasonはcurrent projectの`.reason/config.json`を読めます。このfileにはsubprocess、read-only MCP取得、investigation capabilityを設定できるため、**未trustのrepositoryへ移動しただけでそれらが自動有効化されてはいけません**。

## 何をtrustで保護する？

project configは影響範囲で分けます。

- provider、model、token上限、output formatなどの非secretな通常defaultだけを含むproject configはproject trustなしでも読める;
- `resolution.external_command`、`resolution.mcp_readonly`、`resolution.investigation`は、明示的なproject trustがない限りeffective configへmergeしない;
- `resolution.trusted_command`は、folderをtrustしていても**project configからは絶対に許可しない**。hard verifier authorityはuser configまたはcallerが明示した`--config PATH`からのみ設定する。

未trustまたはstaleなhigh-risk project configはfail-closedします。通常実行中にapproval promptを出さないため、non-interactive jobが入力待ちで止まることもありません。

## コマンド

```bash
reason trust status
reason trust add
reason trust list
reason trust revoke
reason trust revoke --all
```

`status` / `add` / `revoke`はdefaultでcurrent working directoryを対象にします。別folderなら`--project DIR`を指定できます。automationでは`--format json`を利用できます。

`reason trust add`が明示承認操作です。保存前にcanonical project path、現在のfingerprint、解決済みのexecutable/acquisition programを確認できます。

## trust recordは何に結び付く？

v1 trust recordは次へ結び付けます。

1. canonical project directory;
2. 正規化したhigh-risk `resolution` config;
3. 直接設定されたexecutable programのcanonical path;
4. その直接executable fileのSHA-256 identity。

high-risk configを変更した場合や、同じpathの直接executableを差し替えた場合はtrustが`stale`になります。再確認して`reason trust add`するまでproject acquisition configは有効化しません。repositoryを移動した場合も別project identityになります。canonical projectへのsymlink aliasは同一identityとして扱いますが、project root外へ逃げる`.reason/config.json` symlinkは拒否します。

project trustは**activation boundary**であり、sandboxでも、trusted executableが後から読む全fileを再帰的に署名する仕組みでもありません。acquisitionを有効化した後もHarnessのevidence / authority boundaryはそのまま適用されます。

## 保存先

trust recordは`reason-project-trust-v1`としてuser configと同じdirectoryの`project-trust.json`へ保存します。`REASON_HOME`利用時は`$REASON_HOME/project-trust.json`です。書き込みはtemporary fileからcommitし、Unixではuser-only permissionで作成します。

`--no-config`は引き続きhermeticなescape hatchです。user/project configを両方無視するため、project trustも参照しません。
