param(
    [string]$Package,
    [switch]$NoPause
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
if (-not $Package) {
    $Package = Join-Path $root 'apps\Hydrogen\Hydrogen.msix'
}
$pkg = (Resolve-Path -LiteralPath $Package).Path

$signature = Get-AuthenticodeSignature -LiteralPath $pkg
if (-not $signature.SignerCertificate) {
    throw "The package has no signing certificate: $pkg"
}
$certificate = $signature.SignerCertificate

$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($identity)
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# App Installer checks the machine TrustedPeople store. If this exact leaf is
# already trusted, a normal user can update the package without another UAC
# prompt. Only the certificate-trust step needs elevation on a fresh machine.
$trusted = $false
$readStore = New-Object Security.Cryptography.X509Certificates.X509Store(
    'TrustedPeople',
    [Security.Cryptography.X509Certificates.StoreLocation]::LocalMachine
)
try {
    $readStore.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadOnly)
    $trusted = [bool]($readStore.Certificates |
        Where-Object { $_.Thumbprint -eq $certificate.Thumbprint } |
        Select-Object -First 1)
} catch {
    $trusted = $false
} finally {
    $readStore.Close()
}

if (-not $trusted) {
    if (-not $isAdmin) {
        $arguments = @(
            '-NoProfile'
            '-ExecutionPolicy', 'Bypass'
            '-File', ('"{0}"' -f $PSCommandPath)
            '-Package', ('"{0}"' -f $pkg)
        )
        if ($NoPause) {
            $arguments += '-NoPause'
        }
        $child = Start-Process -FilePath 'powershell.exe' -Verb RunAs -ArgumentList $arguments -PassThru -Wait
        if (-not $NoPause) {
            Read-Host 'Press Enter to close'
        }
        exit $child.ExitCode
    }

    # Trust only the signing leaf in TrustedPeople; never add a self-signed leaf
    # to Root. This is the supported sideload path, not a security bypass.
    $writeStore = New-Object Security.Cryptography.X509Certificates.X509Store(
        'TrustedPeople',
        [Security.Cryptography.X509Certificates.StoreLocation]::LocalMachine
    )
    $writeStore.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
    try {
        $present = $writeStore.Certificates |
            Where-Object { $_.Thumbprint -eq $certificate.Thumbprint } |
            Select-Object -First 1
        if (-not $present) {
            $writeStore.Add($certificate)
        }
    } finally {
        $writeStore.Close()
    }
}

$existing = Get-AppxPackage -Name 'HydrogenMacroSuite'
$manifestVersion = [version]([xml](Get-Content -LiteralPath (Join-Path $root 'packaging\AppxManifest.xml') -Raw)).Package.Identity.Version
$alreadyCurrent = $existing -and $existing.Version -eq $manifestVersion
if (-not $alreadyCurrent) {
    try {
        Add-AppxPackage -Path $pkg -ForceApplicationShutdown -ForceUpdateFromAnyVersion
    } catch {
        if (-not $existing) {
            throw
        }
        Remove-AppxPackage -Package $existing.PackageFullName
        Add-AppxPackage -Path $pkg -ForceApplicationShutdown
    }
}

$installed = Get-AppxPackage -Name 'HydrogenMacroSuite'
if (-not $installed) {
    throw 'Hydrogen did not register after Add-AppxPackage.'
}

Write-Output ('Installed Hydrogen ' + $installed.Version)
Write-Output ('Package: ' + $installed.PackageFullName)
Write-Output 'Launch Hydrogen from the Start menu. The panel starts in the tray.'
if (-not $NoPause) {
    Read-Host 'Press Enter to close'
}
