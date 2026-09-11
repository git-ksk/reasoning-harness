# ネイティブインストーラーの契約

日本語 | [English](native-installers.md)

Reason CLI 0.5.0では、RustやCargoを入れていない一般ユーザー向けに1コマンドのネイティブインストーラーを追加します。これは製品配布の機能であり、Harness Engineのcorrectness semanticsは変更しません。

## 対応プラットフォーム

- Linux x86_64
- macOS arm64 / Apple Silicon
- macOS x86_64 / Intel
- Windows x86_64

未対応のOS / architectureでは、既存binaryを置き換える前に明示的に失敗します。

## リリース版での使い方

CLI / Engine分離後のrelease（`reason-vX.Y.Z`）では、native archiveと`SHA256SUMS`に加えて`install.sh` / `install.ps1`をRelease assetとして公開します。

Unix:

```bash
curl -fsSL https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.0/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.0/install.ps1 | iex
```

各tagに含まれるinstallerの既定versionは`reasoning-harness-cli` package versionと一致しなければなりません。release workflow自身がinstaller contract testを実行してからassetへ追加します。

scriptを取得した後でversionや配置先を明示することもできます。

```bash
./install.sh --version 0.5.0 --bin-dir "$HOME/.local/bin"
```

```powershell
./install.ps1 -Version 0.5.0 -BinDir "$env:LOCALAPPDATA\Programs\Reason\bin"
```

installerはshell profileやPATHを勝手に書き換えません。PATH追加が必要な場合は配置先を表示します。

## 完全性検証と置き換え

既存の`reason` executableを置き換える前に、installerは次を行います。

1. OS / architectureに対応する公開済みarchiveを選ぶ;
2. 同じGitHub Releaseからarchiveと`SHA256SUMS`を取得する;
3. archiveのSHA-256を照合する;
4. 期待する`reason` / `reason.exe`だけを展開対象として確認する;
5. binaryの`reason --version`が要求versionと一致することを確認する;
6. install先で一時stageしてから既存binaryを置き換える。

checksum不一致、archive形状不一致、version不一致、未対応platform、download failureでは、既存binaryを置き換えず停止します。

SHA-256検証は#371のintegrity baselineです。署名付きrelease provenance、利用可能なGitHub/Sigstore attestation、macOS / Windowsのplatform signing・notarizationは別P0の#382で扱います。一般向け0.5.0 release gateでは、その強いrelease identityまで通してからinstallerを最終的なtrusted distribution pathとして扱います。

## 過去リリースとの互換性

historical unified tag（`v0.1.0`〜`v0.4.2`）はimmutableのままです。release workflowから過去tagへinstaller fileを後付けしません。一方、既存native archiveと`SHA256SUMS`は公開済みなので、installer実装のcompatibility / smoke testでは`v0.4.2`を明示指定して実際にinstallできます。
