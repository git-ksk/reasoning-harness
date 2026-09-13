# Package-manager installation

[日本語](package-managers.ja.md) | English

Reason CLI supports package-manager-owned installations starting with **0.5.2**. The package-manager channels reuse the same immutable `reason-v*` native archives published by `git-ksk/reasoning-harness`; they do not rebuild Harness Engine or change its authority/correctness semantics.

## Homebrew

The upstream tap is `git-ksk/homebrew-tap` (`git-ksk/tap`):

```bash
brew install git-ksk/tap/reason
```

Upgrade and uninstall through Homebrew:

```bash
brew upgrade git-ksk/tap/reason
brew uninstall git-ksk/tap/reason
```

The formula selects the published macOS arm64, macOS x86_64, or Linux x86_64 archive and pins its SHA-256. Homebrew owns the Cellar executable and symlink lifecycle.

## WinGet

The Windows Package Manager identifier is:

```text
git-ksk.Reason
```

After the community manifest is available from the default WinGet source:

```powershell
winget install --id git-ksk.Reason --exact
winget upgrade --id git-ksk.Reason --exact
winget uninstall --id git-ksk.Reason --exact
```

The manifest uses the published `reason-vX.Y.Z-windows-x86_64.zip` archive as a portable package and pins its SHA-256.

## Lifecycle ownership

Package managers remain authoritative for the executable they install. Reason intentionally does not overwrite a package-manager-owned binary:

- `reason update --check` remains safe and may inspect a newer signed Reason release;
- applying `reason update` or `reason update --rollback ...` from a Homebrew/WinGet-owned executable fails with the typed `external_package_manager` recovery path;
- `reason uninstall --dry-run` remains non-mutating;
- applying `reason uninstall` from a Homebrew/WinGet-owned executable also fails with `external_package_manager` and points to the manager-native uninstall command.

This behavior prevents Reason's self-updater from mutating Homebrew Cellar state or WinGet portable-package state behind the package manager's back. Direct installs from `install.sh` / `install.ps1` continue to use Reason's provenance-verified self-update lifecycle.

The package-manager definitions add distribution convenience only. They do not grant package-manager metadata any evidence or correctness authority inside Harness Engine 0.4.2.
