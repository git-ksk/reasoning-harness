# Native installer contract

[日本語](native-installers.ja.md) | English

Reason CLI 0.5.0 introduces one-command native installers for users who do not have Rust or Cargo. The installers are a product-distribution surface; they do not change Harness Engine correctness semantics.

## Supported platforms

- Linux x86_64
- macOS arm64 / Apple Silicon
- macOS x86_64 / Intel
- Windows x86_64

Unsupported platform/architecture combinations fail before replacing any binary.

## Release usage

Starting with split CLI releases (`reason-vX.Y.Z`), the release workflow publishes `install.sh` and `install.ps1` alongside the native archives and `SHA256SUMS`.

Unix:

```bash
curl -fsSL https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.0/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.0/install.ps1 | iex
```

The version embedded in each tagged installer must match the `reasoning-harness-cli` package version. The release workflow runs the installer contract tests before attaching installer entrypoints.

For an explicit version or destination after downloading the script:

```bash
./install.sh --version 0.5.0 --bin-dir "$HOME/.local/bin"
```

```powershell
./install.ps1 -Version 0.5.0 -BinDir "$env:LOCALAPPDATA\Programs\Reason\bin"
```

The installer does not silently edit shell profiles or PATH. It prints the install directory when PATH guidance is required.

## Integrity and replacement behavior

Before replacing an existing `reason` executable, the installer:

1. selects the platform-specific published archive;
2. downloads that archive and `SHA256SUMS` from the same GitHub Release;
3. verifies the archive SHA-256;
4. extracts the expected `reason` / `reason.exe` path;
5. verifies the binary reports the requested CLI version;
6. stages the binary before replacing the destination.

A checksum mismatch, archive-shape mismatch, version mismatch, unsupported platform, or download failure stops the installation without replacing an existing binary.

SHA-256 verification is the #371 integrity baseline. Signed release provenance, GitHub/Sigstore attestations where practical, and platform signing/notarization are separate P0 distribution-security work in #382. The public 0.5.0 release gate requires that stronger release identity before the installer is treated as the final trusted distribution path.

## Historical releases

Historical unified tags (`v0.1.0` through `v0.4.2`) remain immutable. The release workflow does not retrofit installer files into those tags. The installer implementation can still target `v0.4.2` explicitly for compatibility/testing because the existing native archives and `SHA256SUMS` are already published.
