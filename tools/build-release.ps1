#Requires -Version 5.1
<#
.SYNOPSIS
    Build the release exe with stable path remapping and a normalized PE timestamp.

.DESCRIPTION
    A GNU PE build embeds its build paths in linker metadata. Without remapping,
    two clean builds from different checkout/target directories can differ even
    though the source is identical. This wrapper remaps the workspace and target
    directory to stable prefixes, builds the GNU release target, then zeroes the
    PE timestamp. The checked-in artifact is produced by this wrapper, and the
    release workflow runs the same command from the tagged checkout.
#>
param(
    [string]$TargetDir = "target"
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$target = [System.IO.Path]::GetFullPath((Join-Path $root $TargetDir))
New-Item -ItemType Directory -Force -Path $target | Out-Null

$oldTarget = $env:CARGO_TARGET_DIR
$oldFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    $env:CARGO_TARGET_DIR = $target
    $env:CARGO_ENCODED_RUSTFLAGS = @(
        "-Clink-self-contained=yes",
        "-Clinker-flavor=ld.lld",
        "-Clink-arg=--icf=all",
        "--remap-path-prefix=$root=/droplet",
        "--remap-path-prefix=$target=/target"
    ) -join [char]31

    Push-Location $root
    try {
        & cargo build --release --locked --target x86_64-pc-windows-gnu
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    } finally {
        Pop-Location
    }

    $artifact = Join-Path $target "x86_64-pc-windows-gnu\release\droplet.exe"
    & (Join-Path $PSScriptRoot "normalize-pe.ps1") -Path $artifact | Out-Host
    $hash = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Output "artifact: $artifact"
    Write-Output "sha256:   $hash"
} finally {
    $env:CARGO_TARGET_DIR = $oldTarget
    $env:CARGO_ENCODED_RUSTFLAGS = $oldFlags
}
