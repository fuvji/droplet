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
    public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wParam, ref RECT lParam);

    [DllImport("user32.dll")]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
"@

# Read and capture in physical pixels; an unaware host would virtualize the
# window and produce a clipped 624x408 bitmap at 125% DPI.
[DropletCapture]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null

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

    # Canonical 100% panel size for the README and site.
    $current = New-Object DropletCapture+RECT
    if (-not [DropletCapture]::GetWindowRect($hwnd, [ref]$current)) {
        throw 'GetWindowRect failed before the DPI reset.'
    }
    $suggested = New-Object DropletCapture+RECT
    $suggested.Left = $current.Left
    $suggested.Top = $current.Top
    $suggested.Right = $current.Left + 624
    $suggested.Bottom = $current.Top + 408
    $dpiWord = 96 -bor (96 -shl 16)
    [DropletCapture]::SendMessage($hwnd, 0x02E0, [IntPtr]::new($dpiWord), [ref]$suggested) | Out-Null
    Start-Sleep -Milliseconds 600

    $rect = New-Object DropletCapture+RECT
    if (-not [DropletCapture]::GetWindowRect($hwnd, [ref]$rect)) {
        throw 'GetWindowRect failed.'
    }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -ne 624 -or $height -ne 408) {
        throw "Expected the native 624x408 capture, got ${width}x${height}."
    }

    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $graphics.CopyFromScreen(
            $rect.Left,
            $rect.Top,
            0,
            0,
            $bitmap.Size,
            [System.Drawing.CopyPixelOperation]::SourceCopy
        )
        $target = Join-Path $out 'droplet-core.png'
        $bitmap.Save($target, [System.Drawing.Imaging.ImageFormat]::Png)
        $image = [System.Drawing.Image]::FromFile($target)
        try {
            if ($image.Width -ne 624 -or $image.Height -ne 408) {
                throw "Saved capture is $($image.Width)x$($image.Height), expected 624x408."
            }
        } finally {
            $image.Dispose()
        }
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
