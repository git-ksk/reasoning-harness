# Package managerからのインストール

日本語 | [English](package-managers.md)

Reason CLIは **0.5.2** からpackage-manager管理のinstallを正式に扱います。package-manager channelは`git-ksk/reasoning-harness`が公開するimmutableな`reason-v*` native archiveをそのまま利用し、Harness Engineを再buildしたりauthority / correctness semanticsを変更したりしません。

## Homebrew

upstream tapは`git-ksk/reason`です。

```bash
brew install git-ksk/tap/reason
```

upgrade / uninstallもHomebrew経由で行います。

```bash
brew upgrade git-ksk/tap/reason
brew uninstall git-ksk/tap/reason
```

formulaは公開済みmacOS arm64 / macOS x86_64 / Linux x86_64 archiveをplatformごとに選び、SHA-256を固定します。Cellar内のexecutableとsymlink lifecycleはHomebrewが所有します。

## WinGet

Windows Package Managerのidentifierは次です。

```text
git-ksk.Reason
```

default WinGet sourceへcommunity manifestが反映された後は次を使います。

```powershell
winget install --id git-ksk.Reason --exact
winget upgrade --id git-ksk.Reason --exact
winget uninstall --id git-ksk.Reason --exact
```

manifestは公開済み`reason-vX.Y.Z-windows-x86_64.zip`をportable packageとして使い、SHA-256を固定します。

## lifecycleの所有権

package managerがinstallしたexecutableは、そのpackage managerが唯一のmutation ownerです。Reason自身はpackage-manager-owned binaryを書き換えません。

- `reason update --check`は引き続き安全に利用でき、signed Reason releaseの確認だけ行えます。
- Homebrew / WinGet管理下で`reason update`や`reason update --rollback ...`を適用しようとすると、typed `external_package_manager` recoveryとして失敗します。
- `reason uninstall --dry-run`は引き続きnon-mutatingです。
- Homebrew / WinGet管理下で`reason uninstall`を適用しようとしても`external_package_manager`で失敗し、package manager側のuninstall commandを案内します。

これによりReasonのself-updaterがHomebrew CellarやWinGet portable-package stateをpackage managerの管理外で変更することを防ぎます。`install.sh` / `install.ps1`によるdirect installは、従来どおりReason自身のprovenance-verified self-update lifecycleを使います。

package-manager definitionはdistribution convenienceのみを追加します。package manager metadataがHarness Engine内部のevidence / correctness authorityになることはなく、current Engine 0.5.0でも同じです。
