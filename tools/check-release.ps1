#Requires -Version 5.1
<#
.SYNOPSIS
    Verify that the source tree, release artifact, docs, and static site agree.

.DESCRIPTION
    The checker uses Cargo.toml as the version source and the checked-in
    droplet.exe as the size/hash source. It also runs the release test suite by
    default, so a passing check covers the claims that are easiest to drift:
    version strings, binary metadata, test count, capture dimensions, and local
    links in the published site.

.PARAMETER Root
    Repository root. Defaults to the parent of this script.

.PARAMETER Version
    Optional expected version. When omitted, Cargo.toml supplies it.

.PARAMETER SkipTests
    Skip cargo test. Intended only for a quick documentation/link check; a
    release check should not use it.

.EXAMPLE
    powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-release.ps1
#>
param(
    [string]$Root = (Split-Path -Parent $PSScriptRoot),
    [string]$Version = "",
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path -LiteralPath $Root).Path
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

function FullPath([string]$Relative) {
    return [System.IO.Path]::GetFullPath((Join-Path $Root $Relative))
}

function Read-Text([string]$Relative) {
    $path = FullPath $Relative
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "missing file: $Relative"
    }
    return [System.IO.File]::ReadAllText($path)
}

function Check-Text([string]$Relative, [string]$Needle, [string]$Label) {
    try {
        $text = Read-Text $Relative
        if ($text.Contains($Needle)) {
            Pass $Label
        } else {
            Fail "$Label ($Relative does not contain '$Needle')"
        }
    } catch {
        Fail "$Label ($($_.Exception.Message))"
    }
}

function Check-Regex([string]$Relative, [string]$Pattern, [string]$Label) {
    try {
        $text = Read-Text $Relative
        if ([regex]::IsMatch($text, $Pattern)) {
            Pass $Label
        } else {
            Fail "$Label ($Relative does not match /$Pattern/)"
        }
    } catch {
        Fail "$Label ($($_.Exception.Message))"
    }
}

try {
    $cargoText = Read-Text "Cargo.toml"
    $cargoVersion = [regex]::Match($cargoText, '(?m)^version\s*=\s*"([^"]+)"')
    if (-not $cargoVersion.Success) {
        throw "Cargo.toml has no package version"
    }
    $cargoVersion = $cargoVersion.Groups[1].Value
    if ($Version -ne "" -and $Version -ne $cargoVersion) {
        Fail "requested version $Version does not match Cargo.toml $cargoVersion"
    }
    $ver = if ($Version -ne "") { $Version } else { $cargoVersion }
    Pass "version source: $ver"

    $lockText = Read-Text "Cargo.lock"
    if ([regex]::IsMatch($lockText, "(?m)^name = `"droplet`"\r?\nversion = `"$([regex]::Escape($ver))`"`r?$")) {
        Pass "Cargo.lock version"
    } else {
        Fail "Cargo.lock does not pin droplet $ver"
    }

    $appExe = FullPath "apps/droplet/droplet.exe"
    $siteExe = FullPath "site/public/downloads/droplet.exe"
    foreach ($artifact in @($appExe, $siteExe)) {
        if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) {
            Fail "missing artifact: $artifact"
        }
    }
    if (-not (Test-Path -LiteralPath $appExe -PathType Leaf) -or -not (Test-Path -LiteralPath $siteExe -PathType Leaf)) {
        throw "cannot continue without both release artifacts"
    }

    $appSize = (Get-Item -LiteralPath $appExe).Length
    $siteSize = (Get-Item -LiteralPath $siteExe).Length
    $appHash = (Get-FileHash -LiteralPath $appExe -Algorithm SHA256).Hash.ToLowerInvariant()
    $siteHash = (Get-FileHash -LiteralPath $siteExe -Algorithm SHA256).Hash.ToLowerInvariant()
    $sizeText = $appSize.ToString("N0", [System.Globalization.CultureInfo]::InvariantCulture)

    if ($appSize -eq $siteSize) { Pass "artifact sizes match" } else { Fail "artifact size mismatch: $appSize vs $siteSize" }
    if ($appHash -eq $siteHash) { Pass "artifact hashes match" } else { Fail "artifact hash mismatch: $appHash vs $siteHash" }
    Write-Host "      artifact: $sizeText bytes, SHA-256 $appHash"

    $sumsText = Read-Text "site/public/downloads/SHA256SUMS.txt"
    $sumMatch = [regex]::Match($sumsText, "(?im)^([0-9a-f]{64})\s+droplet\.exe\s*$")
    if ($sumMatch.Success -and $sumMatch.Groups[1].Value.ToLowerInvariant() -eq $appHash) {
        Pass "SHA256SUMS.txt hash"
    } else {
        Fail "SHA256SUMS.txt does not match the artifact hash"
    }
    Check-Text "site/public/downloads/SHA256SUMS.txt" "Droplet $ver" "SHA256SUMS.txt version"

    Check-Text ".github/workflows/release.yml" ('$expected = "' + $appHash + '"') "release workflow expected hash"
    Check-Text "site/public/downloads/README.txt" "Droplet $ver" "downloads README version"
    Check-Text "site/public/downloads/README.txt" "$sizeText bytes" "downloads README size"

    # Current-version anchors. Historical release notes intentionally keep
    # their original version numbers and sizes.
    Check-Text "README.md" "## What is in $ver" "README current feature heading"
    Check-Text "README.md" "**$sizeText bytes**" "README binary size"
    Check-Text "README.md" $appHash "README SHA-256"
    Check-Text "BENCHMARKS.md" "- Version: **$ver**" "BENCHMARKS version"
    Check-Text "BENCHMARKS.md" "**$sizeText bytes**" "BENCHMARKS binary size"
    Check-Text "BENCHMARKS.md" $appHash "BENCHMARKS SHA-256"
    Check-Text "CHANGELOG.md" "## $ver" "CHANGELOG version heading"
    Check-Text "CHANGELOG.md" "**$sizeText bytes**" "CHANGELOG binary size"
    Check-Text "CHANGELOG.md" $appHash "CHANGELOG SHA-256"
    Check-Text "docs/vault/droplet.md" "$ver portable exe" "vault product version"
    Check-Text "docs/vault/droplet.md" "$sizeText bytes" "vault product size"
    Check-Text "docs/vault/performance.md" "## $ver changes" "vault performance version"
    Check-Text "docs/vault/performance.md" "$sizeText B" "vault performance size"
    Check-Text "docs/vault/release.md" "## $ver" "vault release version"
    Check-Text "docs/vault/release.md" "$sizeText B" "vault release size"
    Check-Text "docs/vault/release.md" $appHash "vault release SHA-256"
    Check-Text "site/README.md" "$ver" "site README version"
    Check-Text "site/README.md" "$sizeText B" "site README binary size"
    Check-Text "releases/v$ver.md" "# Droplet $ver" "release note title"
    Check-Text "releases/v$ver.md" "**$sizeText bytes**" "release note binary size"
    Check-Text "releases/v$ver.md" $appHash "release note SHA-256"

    # Site metadata and visible claims.
    foreach ($page in @("site/public/index.html", "site/public/releases.html", "site/public/sponsors.html")) {
        Check-Text $page "brand-version`">$ver<" "$page brand version"
    }
    Check-Text "site/public/index.html" "<strong>$sizeText</strong>" "site binary size marker"
    Check-Text "site/public/index.html" "CORE / $ver" "site capture version"
    Check-Text "site/public/releases.html" "<em>$ver.</em>" "releases heading"
    Check-Text "site/public/releases.html" "Release / $ver" "releases card version"
    Check-Text "site/public/releases.html" $appHash "releases SHA-256"
    Check-Text "site/public/releases.html" "$sizeText<small>B</small>" "releases binary size"
    Check-Text "site/public/llms.txt" "Version: $ver" "llms version"
    Check-Text "site/public/llms.txt" "$sizeText bytes" "llms binary size"

    if (-not $SkipTests) {
        Push-Location $Root
        try {
            # cmd /c keeps Cargo's progress on stderr from becoming a
            # terminating NativeCommandError under $ErrorActionPreference=Stop.
            $testOutput = (& cmd /c "cargo test --release 2>&1") -join "`n"
            $testExit = $LASTEXITCODE
        } finally {
            Pop-Location
        }
        if ($testExit -ne 0) {
            Fail "cargo test --release failed"
            Write-Host $testOutput
        } else {
            $testMatches = [regex]::Matches($testOutput, "test result: ok\. (\d+) passed; (\d+) failed")
            if ($testMatches.Count -eq 0) {
                Fail "could not find a test result in cargo output"
            } else {
                $passed = 0
                $failed = 0
                foreach ($match in $testMatches) {
                    $passed += [int]$match.Groups[1].Value
                    $failed += [int]$match.Groups[2].Value
                }
                if ($failed -ne 0) {
                    Fail "release tests: $failed failed"
                } else {
                    Pass "release tests: $passed passed"
                }
                foreach ($doc in @(
                    "README.md", "BENCHMARKS.md", "CHANGELOG.md", "DEVELOPING.md",
                    "docs/vault/droplet.md", "docs/vault/performance.md",
                    "docs/vault/release.md", "site/public/index.html",
                    "site/public/releases.html", "site/public/llms.txt",
                    "releases/v$ver.md"
                )) {
                    Check-Text $doc "$passed / $passed" "test count in $doc"
                }
            }
        }
    } else {
        Write-Host "SKIP  cargo test (--SkipTests)" -ForegroundColor Yellow
    }

    # Captures are native-size product evidence. Checking dimensions catches a
    # downscaled or stretched replacement even when the filename is unchanged.
    Add-Type -AssemblyName System.Drawing
    $captures = @(
        "site/public/assets/droplet-core.png",
        "site/public/assets/droplet-crystal.png",
        "site/public/assets/droplet-assist.png",
        "site/public/assets/droplet-system.png"
    )
    foreach ($capture in $captures) {
        $path = FullPath $capture
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
            Fail "missing capture: $capture"
            continue
        }
        $image = [System.Drawing.Image]::FromFile($path)
        try {
            if ($image.Width -eq 624 -and $image.Height -eq 408) {
                Pass "capture 624x408: $capture"
            } else {
                Fail "capture dimensions $($image.Width)x$($image.Height), expected 624x408: $capture"
            }
        } finally {
            $image.Dispose()
        }
    }

    if (-not $SkipLinks) {
        $siteRoot = FullPath "site/public"
        $localLinks = 0
        $brokenLinks = 0
        $htmlFiles = Get-ChildItem -LiteralPath $siteRoot -Recurse -File -Filter "*.html"
        foreach ($file in $htmlFiles) {
            $base = $file.DirectoryName
            $html = [System.IO.File]::ReadAllText($file.FullName)
            foreach ($match in [regex]::Matches($html, '(?i)(?:href|src)\s*=\s*["'']([^"'']+)["'']')) {
                $link = $match.Groups[1].Value.Trim()
                if ($link -match '^(?:[a-z][a-z0-9+.-]*:|//|#)' -or $link -eq "") {
                    continue
                }
                $target = $link.Split('?')[0].Split('#')[0]
                if ($target -eq "") { continue }
                $localLinks++
                if ($target.StartsWith("/")) {
                    $resolved = Join-Path $siteRoot $target.TrimStart("/")
                } else {
                    $resolved = Join-Path $base $target
                }
                $resolved = [System.IO.Path]::GetFullPath($resolved)
                if (-not (Test-Path -LiteralPath $resolved)) {
                    Fail "broken local link in $($file.Name): $link"
                    $brokenLinks++
                }
            }
        }
        if ($brokenLinks -eq 0) {
            Pass "local HTML links resolve ($localLinks checked)"
        }

        # CSS url() references are easy to miss because they never appear in
        # the HTML link sweep.
        $cssLinks = 0
        $cssBroken = 0
        foreach ($css in Get-ChildItem -LiteralPath $siteRoot -Recurse -File -Filter "*.css") {
            $content = [System.IO.File]::ReadAllText($css.FullName)
            foreach ($match in [regex]::Matches($content, '(?i)url\(\s*["'']?([^"'')\s]+)')) {
                $target = $match.Groups[1].Value
                if ($target -match '^(?:data:|https?:|//|#|%23)' -or $target -eq "") { continue }
                $target = $target.Split('?')[0].Split('#')[0]
                if ($target -eq "") { continue }
                $cssLinks++
                $resolved = [System.IO.Path]::GetFullPath((Join-Path $css.DirectoryName $target))
                if (-not (Test-Path -LiteralPath $resolved)) {
                    Fail "broken CSS asset in $($css.Name): $target"
                    $cssBroken++
                }
            }
        }
        if ($cssBroken -eq 0) {
            Pass "local CSS assets resolve ($cssLinks checked)"
        }

        $manifestPath = Join-Path $siteRoot "site.webmanifest"
        if (Test-Path -LiteralPath $manifestPath) {
            try {
                $manifest = [System.IO.File]::ReadAllText($manifestPath) | ConvertFrom-Json
                $icons = @($manifest.icons)
                foreach ($icon in $icons) {
                    $src = [string]$icon.src
                    if ($src -eq "") { continue }
                    $resolved = [System.IO.Path]::GetFullPath((Join-Path $siteRoot $src))
                    if (-not (Test-Path -LiteralPath $resolved)) {
                        Fail "broken manifest icon: $src"
                    }
                }
                Pass "web manifest icons resolve"
            } catch {
                Fail "could not parse site.webmanifest: $($_.Exception.Message)"
            }
        }
    } else {
        Write-Host "SKIP  local site link scan (--SkipLinks)" -ForegroundColor Yellow
    }

    Write-Host ""
    if ($failures.Count -gt 0) {
        Write-Host "RELEASE CHECK FAILED: $($failures.Count) issue(s), $passes passed." -ForegroundColor Red
        foreach ($failure in $failures) {
            Write-Host "  - $failure" -ForegroundColor Red
        }
        exit 1
    }

    Write-Host "RELEASE CHECK PASSED: $passes checks for Droplet $ver." -ForegroundColor Green
    exit 0
} catch {
    Write-Host "RELEASE CHECK ERROR: $($_.Exception.Message)" -ForegroundColor Red
    exit 2
}
