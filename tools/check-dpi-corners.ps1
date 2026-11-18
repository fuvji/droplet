#Requires -Version 5.1
<#
.SYNOPSIS
    Check the live 100%, 125%, and 150% layered-alpha panel captures.

.DESCRIPTION
    The layered mask is mirrored into one corner box. This check reads the
    actual PNGs, not the drawing code, and fails when an outer corner is opaque,
    when its alpha mirrors disagree, or when the straight edge vanished. The
    images are produced by tools/capture-tabs.ps1 from the running exe.
#>
param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [string]$Directory = "site/public/assets/dpi"
)

$ErrorActionPreference = "Stop"
$rootPath = (Resolve-Path -LiteralPath $Root).Path
$dirPath = Join-Path $rootPath $Directory
$captures = @(
    @{ Name = "droplet-core-100.png"; Width = 624; Height = 408; Radius = 18 },
    @{ Name = "droplet-core-125.png"; Width = 780; Height = 510; Radius = 23 },
    @{ Name = "droplet-core-150.png"; Width = 936; Height = 612; Radius = 27 }
)
$failures = [System.Collections.Generic.List[string]]::new()
$passes = 0

function Pass([string]$Message) {
    $script:passes++
    Write-Host "PASS  $Message" -ForegroundColor Green
}

function Fail([string]$Message) {
    $script:failures.Add($Message)
    Write-Host "FAIL  $Message" -ForegroundColor Red
}

Add-Type -AssemblyName System.Drawing
foreach ($capture in $captures) {
    $path = Join-Path $dirPath $capture.Name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        Fail "missing DPI capture: $Directory/$($capture.Name)"
        continue
    }

    $bitmap = [System.Drawing.Bitmap]::FromFile($path)
    try {
        if ($bitmap.Width -ne $capture.Width -or $bitmap.Height -ne $capture.Height) {
            Fail "$($capture.Name) is $($bitmap.Width)x$($bitmap.Height), expected $($capture.Width)x$($capture.Height)"
            continue
        }
        $w = $bitmap.Width
        $h = $bitmap.Height
        $r = [int]$capture.Radius

        $corners = @(
            @{ X = 0; Y = 0; Label = "top-left" },
            @{ X = $w - 1; Y = 0; Label = "top-right" },
            @{ X = 0; Y = $h - 1; Label = "bottom-left" },
            @{ X = $w - 1; Y = $h - 1; Label = "bottom-right" }
        )
        $transparent = $true
        foreach ($corner in $corners) {
            if ($bitmap.GetPixel($corner.X, $corner.Y).A -ne 0) {
                Fail "$($capture.Name) $($corner.Label) alpha is $($bitmap.GetPixel($corner.X, $corner.Y).A), expected 0"
                $transparent = $false
            }
        }
        if ($transparent) { Pass "$($capture.Name): four corners transparent" }

        $symmetric = $true
        for ($y = 0; $y -lt $r; $y++) {
            for ($x = 0; $x -lt $r; $x++) {
                $a = $bitmap.GetPixel($x, $y).A
                $mirrorX = $bitmap.GetPixel($w - 1 - $x, $y).A
                $mirrorY = $bitmap.GetPixel($x, $h - 1 - $y).A
                $mirrorXY = $bitmap.GetPixel($w - 1 - $x, $h - 1 - $y).A
                if ($a -ne $mirrorX -or $a -ne $mirrorY -or $a -ne $mirrorXY) {
                    Fail "$($capture.Name) asymmetrical alpha at ($x,$y): $a/$mirrorX/$mirrorY/$mirrorXY"
                    $symmetric = $false
                    break
                }
            }
            if (-not $symmetric) { break }
        }
        if ($symmetric) { Pass "$($capture.Name): corner alpha mirrors on both axes" }

        if ($bitmap.GetPixel($r, 0).A -eq 0) {
            Fail "$($capture.Name) straight top edge starts transparent at x=$r"
        } else {
            Pass "$($capture.Name): straight edge remains opaque"
        }
    } finally {
        $bitmap.Dispose()
    }
}

Write-Host ""
if ($failures.Count -gt 0) {
    Write-Host "DPI CORNER CHECK FAILED: $($failures.Count) issue(s), $passes passed." -ForegroundColor Red
    foreach ($failure in $failures) { Write-Host "  - $failure" -ForegroundColor Red }
    exit 1
}
Write-Host "DPI CORNER CHECK PASSED: $passes checks." -ForegroundColor Green
exit 0
