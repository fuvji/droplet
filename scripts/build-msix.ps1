param([switch]$SkipSign)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$msixTools = Join-Path $root '.toolchain\msix'
$makeappx = Get-ChildItem -Path $msixTools -Recurse -Filter 'makeappx.exe' |
    Where-Object { $_.FullName -match '\\x64\\' } |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName
$signtool = Get-ChildItem -Path $msixTools -Recurse -Filter 'signtool.exe' |
    Where-Object { $_.FullName -match '\\x64\\' } |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $makeappx) { throw 'MakeAppx.exe was not found in .toolchain\msix.' }

$stage = Join-Path $root 'packaging\package'
$apps = Join-Path $root 'apps\droplet'
$assets = Join-Path $root 'packaging\Assets'
$manifest = Join-Path $root 'packaging\AppxManifest.xml'
$msix = Join-Path $apps 'droplet.msix'

$exe = @(
    (Join-Path $root 'target\x86_64-pc-windows-gnu\release\droplet.exe'),
    (Join-Path $root 'target\release\droplet.exe'),
    (Join-Path $root 'Droplet.exe')
) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
if (-not $exe) { throw 'Droplet.exe is missing. Build the release binary first.' }
if (-not (Test-Path -LiteralPath $assets)) { throw 'packaging\Assets is missing.' }
New-Item -ItemType Directory -Force -Path $stage, $apps | Out-Null
Remove-Item -LiteralPath $stage -Recurse -Force
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item -LiteralPath $manifest -Destination (Join-Path $stage 'AppxManifest.xml')
Copy-Item -LiteralPath $exe -Destination (Join-Path $stage 'droplet.exe')
Copy-Item -LiteralPath $exe -Destination (Join-Path $apps 'droplet.exe') -Force
Copy-Item -LiteralPath $assets -Destination $stage -Recurse

& $makeappx pack /d $stage /p $msix /o
if ($LASTEXITCODE -ne 0) { throw "MakeAppx failed with exit code $LASTEXITCODE." }

if ($SkipSign) {
    Write-Output "Unsigned package: $msix"
    exit 0
}

if (-not $signtool) { throw 'SignTool.exe was not found in .toolchain\msix.' }
$pfx = Join-Path $msixTools 'Droplet-Dev.pfx'
$cer = Join-Path $apps 'droplet-dev.cer'
$passwordFile = Join-Path $msixTools 'pfx-password.txt'
if (-not (Test-Path -LiteralPath $cer)) {
    $legacyCer = Join-Path $root 'dist\droplet-dev.cer'
    if (Test-Path -LiteralPath $legacyCer) {
        Copy-Item -LiteralPath $legacyCer -Destination $cer -Force
    }
}
if (-not (Test-Path -LiteralPath $pfx)) {
    $cert = New-SelfSignedCertificate `
        -Type CodeSigningCert `
        -Subject 'CN=Droplet Macro Suite' `
        -CertStoreLocation 'Cert:\CurrentUser\My' `
        -KeyExportPolicy Exportable `
        -KeyUsage DigitalSignature `
        -FriendlyName 'Droplet Macro Suite development signing' `
        -NotAfter (Get-Date).AddYears(3)
    $password = [Guid]::NewGuid().ToString('N')
    $secure = ConvertTo-SecureString -String $password -AsPlainText -Force
    Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $secure | Out-Null
    Export-Certificate -Cert $cert -FilePath $cer | Out-Null
    Set-Content -LiteralPath $passwordFile -Value $password -NoNewline
    Import-Certificate -FilePath $cer -CertStoreLocation 'Cert:\CurrentUser\TrustedPeople' | Out-Null
}
if (-not (Test-Path -LiteralPath $cer)) {
    $cert = Get-ChildItem 'Cert:\CurrentUser\My' |
        Where-Object { $_.Subject -eq 'CN=Droplet Macro Suite' } |
        Sort-Object NotAfter -Descending |
        Select-Object -First 1
    if (-not $cert) { throw 'The Droplet signing certificate is missing. Delete the PFX and rerun.' }
    Export-Certificate -Cert $cert -FilePath $cer | Out-Null
}
if (-not (Test-Path -LiteralPath $passwordFile)) { throw 'The local PFX password file is missing.' }
$pfxPassword = Get-Content -LiteralPath $passwordFile -Raw
& $signtool sign /fd SHA256 /f $pfx /p $pfxPassword $msix
if ($LASTEXITCODE -ne 0) { throw "SignTool failed with exit code $LASTEXITCODE." }
& $signtool verify /pa /q $msix
if ($LASTEXITCODE -ne 0) {
    Write-Warning 'The package is signed. Windows needs the same certificate in LocalMachine\TrustedPeople before App Installer will accept it. Run Install-Droplet.cmd and approve UAC; do not add the leaf certificate to the Root store.'
} else {
    Write-Output 'Package signature verified by the local trust provider.'
}
Write-Output "Portable app: $(Join-Path $apps 'droplet.exe')"
Write-Output "Signed package: $msix"
Write-Output "Development certificate: $cer"
Write-Output 'Install with: Install-Droplet.cmd (UAC prompt required once for certificate trust)'
