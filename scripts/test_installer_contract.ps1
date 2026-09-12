$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$Installer = Join-Path $Root 'install.ps1'
$script:MockReleaseDir = $null
$script:GhVersion = '2.93.0'
$script:AttestationSuccess = $true
$script:GhCalls = @()

function global:Invoke-WebRequest {
    param(
        [switch]$UseBasicParsing,
        [string]$Uri,
        [string]$OutFile
    )
    $Name = [IO.Path]::GetFileName(([Uri]$Uri).AbsolutePath)
    Copy-Item -LiteralPath (Join-Path $script:MockReleaseDir $Name) -Destination $OutFile
}

function global:gh {
    if ($args.Count -gt 0 -and $args[0] -eq '--version') {
        Write-Output "gh version $script:GhVersion (mock)"
        $global:LASTEXITCODE = 0
        return
    }
    $script:GhCalls += [pscustomobject]@{
        Host = $env:GH_HOST
        Args = ($args -join ' ')
    }
    $global:LASTEXITCODE = if ($script:AttestationSuccess) { 0 } else { 1 }
}

function New-MockRelease([string]$Base, [string]$Version = '0.5.0') {
    $Release = Join-Path $Base 'release'
    $Package = Join-Path $Base "reason-v$Version-windows-x86_64"
    New-Item -ItemType Directory -Path $Release, $Package -Force | Out-Null
    $Exe = Join-Path $Package 'reason.exe'
    $Source = @"
using System;
public static class Program {
    public static void Main(string[] args) { Console.WriteLine("reason $Version"); }
}
"@
    Add-Type -TypeDefinition $Source -Language CSharp -OutputAssembly $Exe -OutputType ConsoleApplication
    Copy-Item (Join-Path $Root 'README.md') $Package
    Copy-Item (Join-Path $Root 'LICENSE') $Package
    Copy-Item (Join-Path $Root 'CHANGELOG.md') $Package
    $Archive = Join-Path $Release "reason-v$Version-windows-x86_64.zip"
    Compress-Archive -Path $Package -DestinationPath $Archive
    $Hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $Archive).Hash.ToLowerInvariant()
    Set-Content -LiteralPath (Join-Path $Release 'SHA256SUMS') -Value "$Hash  $([IO.Path]::GetFileName($Archive))" -Encoding ascii
    return $Release
}

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

$Temp = Join-Path ([IO.Path]::GetTempPath()) ('reason-installer-contract-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $Temp | Out-Null
try {
    $script:MockReleaseDir = New-MockRelease $Temp
    $BinDir = Join-Path $Temp 'bin-success'
    $script:GhCalls = @()
    $script:AttestationSuccess = $true
    . $Installer -Version '0.5.0' -BinDir $BinDir | Out-Null
    $Installed = Join-Path $BinDir 'reason.exe'
    Assert-True (Test-Path -LiteralPath $Installed) 'split release was not installed'
    Assert-True ((& $Installed --version | Out-String).Trim() -eq 'reason 0.5.0') 'installed version mismatch'
    Assert-True ($script:GhCalls.Count -eq 1) 'expected exactly one attestation verification call'
    $Call = $script:GhCalls[0]
    Assert-True ($Call.Host -eq 'github.com') 'GH_HOST was not pinned to github.com'
    Assert-True ($Call.Args.Contains('attestation verify')) 'attestation verify was not invoked'
    Assert-True ($Call.Args.Contains('--repo git-ksk/reasoning-harness')) 'repository identity was not pinned'
    Assert-True ($Call.Args.Contains('--signer-workflow git-ksk/reasoning-harness/.github/workflows/release-cli.yml')) 'signer workflow was not pinned'
    Assert-True ($Call.Args.Contains('--source-ref refs/tags/reason-v0.5.0')) 'source tag was not pinned'
    Assert-True ($Call.Args.Contains('--deny-self-hosted-runners')) 'self-hosted provenance was not denied'

    $FailBin = Join-Path $Temp 'bin-fail'
    New-Item -ItemType Directory -Path $FailBin | Out-Null
    $Existing = Join-Path $FailBin 'reason.exe'
    Set-Content -LiteralPath $Existing -Value 'old-binary' -NoNewline
    $Before = [IO.File]::ReadAllBytes($Existing)
    $script:AttestationSuccess = $false
    $Threw = $false
    try {
        . $Installer -Version '0.5.0' -BinDir $FailBin | Out-Null
    } catch {
        $Threw = $true
        Assert-True ($_.Exception.Message.Contains('provenance verification failed')) 'wrong attestation failure'
    }
    Assert-True $Threw 'attestation failure did not fail closed'
    $After = [IO.File]::ReadAllBytes($Existing)
    Assert-True ([Convert]::ToBase64String($Before) -eq [Convert]::ToBase64String($After)) 'existing binary changed after failed provenance'

    $script:AttestationSuccess = $true
    $script:GhVersion = '2.92.0'
    $OldBin = Join-Path $Temp 'bin-old-gh'
    New-Item -ItemType Directory -Path $OldBin | Out-Null
    $Threw = $false
    try {
        . $Installer -Version '0.5.0' -BinDir $OldBin | Out-Null
    } catch {
        $Threw = $true
        Assert-True ($_.Exception.Message.Contains('too old')) 'old gh was not rejected for version'
    }
    Assert-True $Threw 'old gh did not fail closed'

    Write-Output 'PowerShell installer provenance contract PASS'
} finally {
    Remove-Item function:global:Invoke-WebRequest -ErrorAction SilentlyContinue
    Remove-Item function:global:gh -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $Temp -Recurse -Force -ErrorAction SilentlyContinue
}
