param(
    [switch]$Install
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$packageDir = Join-Path $root 'packaging\package'
$assetsDir = Join-Path $packageDir 'Assets'
$assetSource = Join-Path $root 'packaging\Assets'
$exe = Join-Path $root 'target\x86_64-pc-windows-gnu\release\droplet.exe'
$pfx = Join-Path $root '.toolchain\msix\Droplet-Dev.pfx'
$passwordFile = Join-Path $root '.toolchain\msix\pfx-password.txt'
$outDir = Join-Path $root 'apps\Droplet'
$msix = Join-Path $outDir 'Droplet.msix'

if (-not (Test-Path -LiteralPath $exe)) { throw 'Build the release executable first.' }
if (-not (Test-Path -LiteralPath $pfx)) { throw 'Missing the local development signing certificate.' }
if (-not (Test-Path -LiteralPath $passwordFile)) { throw 'Missing the local signing password file.' }

$makeappx = Get-ChildItem -LiteralPath (Join-Path $root '.toolchain\msix') -Recurse -Filter 'makeappx.exe' |
    Where-Object { $_.FullName -match '\\x64\\' } | Select-Object -First 1
$signtool = Get-ChildItem -LiteralPath (Join-Path $root '.toolchain\msix') -Recurse -Filter 'signtool.exe' |
    Where-Object { $_.FullName -match '\\x64\\' } | Select-Object -First 1
if (-not $makeappx -or -not $signtool) { throw 'The Windows SDK BuildTools are not available under .toolchain\msix.' }

New-Item -ItemType Directory -Force -Path $assetsDir | Out-Null
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Copy-Item -LiteralPath (Join-Path $root 'packaging\AppxManifest.xml') -Destination (Join-Path $packageDir 'AppxManifest.xml') -Force
Copy-Item -Path (Join-Path $assetSource '*.png') -Destination $assetsDir -Force
Copy-Item -LiteralPath $exe -Destination (Join-Path $packageDir 'Droplet.exe') -Force
Copy-Item -LiteralPath $exe -Destination (Join-Path $outDir 'Droplet.exe') -Force
if (Test-Path -LiteralPath $msix) { Remove-Item -LiteralPath $msix -Force }

& $makeappx.FullName pack /d $packageDir /p $msix /o
if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE." }

$password = (Get-Content -LiteralPath $passwordFile -Raw).Trim()
& $signtool.FullName sign /fd SHA256 /f $pfx /p $password /tr http://timestamp.digicert.com /td SHA256 $msix
if ($LASTEXITCODE -ne 0) {
    & $signtool.FullName sign /fd SHA256 /f $pfx /p $password $msix
    if ($LASTEXITCODE -ne 0) { throw "Signtool failed with exit code $LASTEXITCODE." }
}

$hash = (Get-FileHash -LiteralPath $msix -Algorithm SHA256).Hash.ToLowerInvariant()
$size = (Get-Item -LiteralPath $msix).Length
Write-Output "MSIX $size bytes SHA256 $hash"

if ($Install) {
    Add-AppxPackage -Path $msix -ForceApplicationShutdown
    Write-Output 'Installed Droplet MSIX.'
}
