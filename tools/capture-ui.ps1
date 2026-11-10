param(
    [string]$OutDir = "site/public/assets"
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\x86_64-pc-windows-gnu\release\droplet.exe'
if (-not (Test-Path -LiteralPath $exe)) {
    throw 'Build the release executable before capturing screenshots.'
}

$out = Join-Path $root $OutDir
New-Item -ItemType Directory -Force -Path $out | Out-Null

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;

public static class DropletCapture {
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int Left, Top, Right, Bottom; }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string windowName);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);

    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hwnd, int command);

    [DllImport("user32.dll")]
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
}
"@

$process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
try {
    Start-Sleep -Seconds 2
    $hwnd = [DropletCapture]::FindWindow('DropletOverlay', 'Droplet')
    if ($hwnd -eq [IntPtr]::Zero) {
        $hwnd = [DropletCapture]::FindWindow($null, 'Droplet')
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw 'The Droplet panel window was not created.'
    }

    [DropletCapture]::ShowWindow($hwnd, 5) | Out-Null
    Start-Sleep -Milliseconds 700

    $rect = New-Object DropletCapture+RECT
    if (-not [DropletCapture]::GetWindowRect($hwnd, [ref]$rect)) {
        throw 'GetWindowRect failed.'
    }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) {
        throw 'The Droplet panel has an empty window rectangle.'
    }

    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $hdc = $graphics.GetHdc()
        try {
            [DropletCapture]::PrintWindow($hwnd, $hdc, 2) | Out-Null
        } finally {
            $graphics.ReleaseHdc($hdc)
        }
        $bitmap.Save((Join-Path $out 'droplet-core.png'), [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }

    Write-Output "Captured $width x $height panel screenshot in $out"
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id
    }
}
