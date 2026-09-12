$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Installer = Join-Path $Root 'install.ps1'
$script:GhVersion = '2.93.0'
$script:AttestationSuccess = $true
$script:GhCalls = @()

function global:gh {
    if ($args.Count -gt 0 -and $args[0] -eq '--version') {
        Write-Output "gh version $script:GhVersion (mock)"
        $global:LASTEXITCODE = 0
        return
    }
    $script:GhCalls += [pscustomobject]@{ Host = $env:GH_HOST; Args = ($args -join ' ') }
    $global:LASTEXITCODE = if ($script:AttestationSuccess) { 0 } else { 1 }
}

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

$Temp = Join-Path ([IO.Path]::GetTempPath()) ('reason-installer-contract-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Temp | Out-Null
try {
    # Load verifier definitions without running the installer.
    $env:REASON_INSTALL_LIBRARY_ONLY = '1'
    . $Installer
    Remove-Item Env:REASON_INSTALL_LIBRARY_ONLY

    $Tag = 'reason-v0.5.0'
    $Archive = 'reason-v0.5.0-windows-x86_64.zip'
    $DummyArchive = Join-Path $Temp $Archive
    Set-Content -LiteralPath $DummyArchive -Value 'dummy' -NoNewline

    $script:GhCalls = @()
    $script:GhVersion = '2.93.0'
    $script:AttestationSuccess = $true
    Verify-SplitReleaseProvenance $DummyArchive
    Assert-True ($script:GhCalls.Count -eq 1) 'expected exactly one attestation verification call'
    $Call = $script:GhCalls[0]
    Assert-True ($Call.Host -eq 'github.com') 'GH_HOST was not pinned to github.com'
    Assert-True ($Call.Args.Contains('attestation verify')) 'attestation verify was not invoked'
    Assert-True ($Call.Args.Contains('--repo git-ksk/reasoning-harness')) 'repository identity was not pinned'
    Assert-True ($Call.Args.Contains('--signer-workflow git-ksk/reasoning-harness/.github/workflows/release-cli.yml')) 'signer workflow was not pinned'
    Assert-True ($Call.Args.Contains('--source-ref refs/tags/reason-v0.5.0')) 'source tag was not pinned'
    Assert-True ($Call.Args.Contains('--deny-self-hosted-runners')) 'self-hosted provenance was not denied'

    $script:GhVersion = '2.92.0'
    $Threw = $false
    try { Verify-SplitReleaseProvenance $DummyArchive } catch {
        $Threw = $true
        Assert-True ($_.Exception.Message.Contains('too old')) 'old gh was not rejected for version'
    }
    Assert-True $Threw 'old gh did not fail closed'

    # Full installer failure happens before checksum/extract/replacement, so an existing binary stays intact.
    $Release = Join-Path $Temp 'release'
    New-Item -ItemType Directory -Path $Release | Out-Null
    Copy-Item -LiteralPath $DummyArchive -Destination (Join-Path $Release $Archive)
    Set-Content -LiteralPath (Join-Path $Release 'SHA256SUMS') -Value ('0' * 64 + "  $Archive") -Encoding ascii
    function global:Invoke-WebRequest {
        param([switch]$UseBasicParsing, [string]$Uri, [string]$OutFile)
        $Name = [IO.Path]::GetFileName(([Uri]$Uri).AbsolutePath)
        Copy-Item -LiteralPath (Join-Path $script:MockReleaseDir $Name) -Destination $OutFile
    }
    $script:MockReleaseDir = $Release
    $script:GhVersion = '2.93.0'
    $script:AttestationSuccess = $false
    $FailBin = Join-Path $Temp 'bin-fail'
    New-Item -ItemType Directory -Path $FailBin | Out-Null
    $Existing = Join-Path $FailBin 'reason.exe'
    Set-Content -LiteralPath $Existing -Value 'old-binary' -NoNewline
    $Before = [IO.File]::ReadAllBytes($Existing)
    $Threw = $false
    try { . $Installer -Version '0.5.0' -BinDir $FailBin | Out-Null } catch {
        $Threw = $true
        Assert-True ($_.Exception.Message.Contains('provenance verification failed')) 'wrong attestation failure'
    }
    Assert-True $Threw 'attestation failure did not fail closed'
    $After = [IO.File]::ReadAllBytes($Existing)
    Assert-True ([Convert]::ToBase64String($Before) -eq [Convert]::ToBase64String($After)) 'existing binary changed after failed provenance'

    $global:LASTEXITCODE = 0
    Write-Output 'PowerShell installer provenance contract PASS'
} finally {
    Remove-Item Env:REASON_INSTALL_LIBRARY_ONLY -ErrorAction SilentlyContinue
    Remove-Item function:global:Invoke-WebRequest -ErrorAction SilentlyContinue
    Remove-Item function:global:gh -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $Temp -Recurse -Force -ErrorAction SilentlyContinue
}
