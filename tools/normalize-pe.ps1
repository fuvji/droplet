#Requires -Version 5.1
<#
.SYNOPSIS
    Zero the PE header timestamp so repeated Windows GNU builds are byte-identical.

.DESCRIPTION
    The GNU linker used by this project writes the current time into the PE
    COFF header. Everything else in two clean release builds is identical. This
    helper is deliberately narrow: it validates the PE signature, zeroes only
    the four timestamp bytes, and rewrites the same file. It does not alter
    code, resources, signatures, or the PE checksum.

    Run the same helper in CI and locally before hashing or publishing an
    artifact. A reproducible build is a provenance gate, not a claim that
    Windows or the game has zero latency.
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$Path
)

$ErrorActionPreference = "Stop"
$resolved = (Resolve-Path -LiteralPath $Path).Path
$bytes = [System.IO.File]::ReadAllBytes($resolved)
if ($bytes.Length -lt 0x100) {
    throw "Not a PE file: $resolved"
}

$peOffset = [BitConverter]::ToInt32($bytes, 0x3C)
if ($peOffset -lt 0 -or $peOffset + 0x18 -gt $bytes.Length) {
    throw "Invalid PE offset: $peOffset"
}
if ($bytes[$peOffset] -ne 0x50 -or
    $bytes[$peOffset + 1] -ne 0x45 -or
    $bytes[$peOffset + 2] -ne 0 -or
    $bytes[$peOffset + 3] -ne 0) {
    throw "Missing PE signature: $resolved"
}

$timestamp = [BitConverter]::ToUInt32($bytes, $peOffset + 8)
if ($timestamp -ne 0) {
    for ($i = 0; $i -lt 4; $i++) {
        $bytes[$peOffset + 8 + $i] = 0
    }
    [System.IO.File]::WriteAllBytes($resolved, $bytes)
}

Write-Output ("normalized PE timestamp: {0}" -f $resolved)
