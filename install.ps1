[CmdletBinding()]
param(
    [Parameter()]
    [string]$Version = "0.4.2",

    [Parameter()]
    [string]$BinDir = $(if ($env:LOCALAPPDATA) { Join-Path $env:LOCALAPPDATA "Programs\Reason\bin" } else { Join-Path $HOME ".local\bin" })
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$Repository = "git-ksk/reasoning-harness"
$SignerWorkflow = "git-ksk/reasoning-harness/.github/workflows/release-cli.yml"
$MinimumGhVersion = [Version]"2.93.0"

function Fail([string]$Message) {
    throw "reason installer: $Message"
}

function Verify-SplitReleaseProvenance([string]$ArchivePath) {
    $Gh = Get-Command gh -ErrorAction SilentlyContinue
    if (-not $Gh) {
        Fail "GitHub CLI >= $MinimumGhVersion is required to verify reason-v* release provenance"
    }
    $VersionLine = (& gh --version 2>$null | Select-Object -First 1)
    if ($VersionLine -notmatch '^gh version (\d+\.\d+\.\d+)') {
        Fail "could not determine GitHub CLI version; require gh >= $MinimumGhVersion"
    }
    $GhVersion = [Version]$Matches[1]
    if ($GhVersion -lt $MinimumGhVersion) {
        Fail "GitHub CLI $GhVersion is too old for trusted attestation verification; require >= $MinimumGhVersion"
    }
    $OldGhHost = $env:GH_HOST
    try {
        $env:GH_HOST = 'github.com'
        & gh attestation verify $ArchivePath `
            --repo $Repository `
            --signer-workflow $SignerWorkflow `
            --source-ref "refs/tags/$Tag" `
            --deny-self-hosted-runners *> $null
        if ($LASTEXITCODE -ne 0) {
            Fail "release provenance verification failed for $Archive; refusing to install"
        }
    } finally {
        $env:GH_HOST = $OldGhHost
    }
}

$Tag = $null
$ResolvedVersion = $Version
if ($Version -match '^reason-v(.+)$') {
    $Tag = $Version
    $ResolvedVersion = $Matches[1]
} elseif ($Version -match '^v(.+)$') {
    $Tag = $Version
    $ResolvedVersion = $Matches[1]
} elseif ($Version -match '^(\d+)\.(\d+)\.(\d+)([-+][0-9A-Za-z.-]+)?$') {
    $Major = [int]$Matches[1]
    $Minor = [int]$Matches[2]
    if ($Major -eq 0 -and $Minor -lt 5) {
        $Tag = "v$Version"
    } else {
        $Tag = "reason-v$Version"
    }
} else {
    Fail "invalid version/tag: $Version"
}

if ($ResolvedVersion -notmatch '^[0-9A-Za-z.+-]+$') {
    Fail "invalid version/tag: $Version"
}

if (-not [Environment]::Is64BitOperatingSystem) {
    Fail "unsupported Windows architecture; Reason CLI currently supports Windows x86_64 only"
}

$Architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
if ($Architecture -notin @('X64', 'x64')) {
    Fail "unsupported Windows architecture: $Architecture; Reason CLI currently supports Windows x86_64 only"
}

$Asset = "windows-x86_64"
$Archive = "reason-v$ResolvedVersion-$Asset.zip"
$BaseUrl = "https://github.com/$Repository/releases/download/$Tag"
$TempDir = Join-Path ([IO.Path]::GetTempPath()) ("reason-install-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    $ArchivePath = Join-Path $TempDir $Archive
    $ChecksumsPath = Join-Path $TempDir "SHA256SUMS"

    Write-Host "Installing Reason CLI $ResolvedVersion for $Asset..."
    try {
        Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/$Archive" -OutFile $ArchivePath
        Invoke-WebRequest -UseBasicParsing -Uri "$BaseUrl/SHA256SUMS" -OutFile $ChecksumsPath
    } catch {
        Fail "failed to download release assets from $Tag`: $($_.Exception.Message)"
    }

    if ($Tag.StartsWith('reason-v')) {
        Verify-SplitReleaseProvenance $ArchivePath
    }

    $ChecksumLines = Get-Content -LiteralPath $ChecksumsPath
    $MatchesForArchive = @($ChecksumLines | Where-Object { $_ -match ('^([0-9a-fA-F]{64})\s+\*?' + [Regex]::Escape($Archive) + '$') })
    if ($MatchesForArchive.Count -ne 1) {
        Fail "$Archive must appear exactly once in SHA256SUMS"
    }
    [void]($MatchesForArchive[0] -match '^([0-9a-fA-F]{64})')
    $Expected = $Matches[1].ToLowerInvariant()
    $Actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $ArchivePath).Hash.ToLowerInvariant()
    if ($Actual -ne $Expected) {
        Fail "checksum mismatch for $Archive; refusing to install"
    }

    $ExtractDir = Join-Path $TempDir "extract"
    Expand-Archive -LiteralPath $ArchivePath -DestinationPath $ExtractDir
    $Source = Join-Path $ExtractDir "reason-v$ResolvedVersion-$Asset\reason.exe"
    if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) {
        Fail "release archive does not contain the expected reason.exe"
    }

    $VersionOutput = (& $Source --version 2>$null | Out-String).Trim()
    if ($VersionOutput -ne "reason $ResolvedVersion") {
        Fail "downloaded binary reported '$VersionOutput', expected 'reason $ResolvedVersion'"
    }

    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    $Staged = Join-Path $BinDir (".reason.install." + [Guid]::NewGuid().ToString("N") + ".exe")
    Copy-Item -LiteralPath $Source -Destination $Staged
    $Destination = Join-Path $BinDir "reason.exe"
    Move-Item -LiteralPath $Staged -Destination $Destination -Force

    $InstalledVersion = (& $Destination --version 2>$null | Out-String).Trim()
    if ($InstalledVersion -ne "reason $ResolvedVersion") {
        Fail "installed binary failed its version smoke check"
    }

    Write-Host "Installed $Destination"
    Write-Host $InstalledVersion

    $PathEntries = @($env:PATH -split ';' | ForEach-Object { $_.TrimEnd('\\') })
    if ($PathEntries -notcontains $BinDir.TrimEnd('\\')) {
        Write-Host ""
        Write-Host "Add this directory to PATH to run reason from anywhere:"
        Write-Host "  $BinDir"
    }

    Write-Host ""
    if ($Tag.StartsWith('reason-v')) {
        Write-Host "Integrity: GitHub/Sigstore provenance and SHA-256 verified."
    } else {
        Write-Host "Integrity: SHA-256 verified for historical release (pre-attestation)."
    }
} finally {
    if (Test-Path -LiteralPath $TempDir) {
        Remove-Item -LiteralPath $TempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}
