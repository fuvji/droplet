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

public static class DropletTabs {
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

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern uint GetDpiForWindow(IntPtr hwnd);
}
"@

function Save-Panel([IntPtr]$Hwnd, [string]$Name) {
    $rect = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($Hwnd, [ref]$rect)) {
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
            [DropletTabs]::PrintWindow($Hwnd, $hdc, 2) | Out-Null
        } finally {
            $graphics.ReleaseHdc($hdc)
        }
        $bitmap.Save((Join-Path $out $Name), [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}

function Click-Tab([IntPtr]$Hwnd, [int]$X, [int]$Y) {
    $lParam = [IntPtr]::new([int](($Y -shl 16) -bor ($X -band 0xFFFF)))
    [DropletTabs]::PostMessage($Hwnd, 0x0201, [IntPtr]::new(1), $lParam) | Out-Null
    Start-Sleep -Milliseconds 100
    [DropletTabs]::PostMessage($Hwnd, 0x0202, [IntPtr]::Zero, $lParam) | Out-Null
    Start-Sleep -Milliseconds 350
}

$process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
try {
    Start-Sleep -Seconds 2
    $hwnd = [DropletTabs]::FindWindow('DropletOverlay', 'Droplet')
    if ($hwnd -eq [IntPtr]::Zero) {
        $hwnd = [DropletTabs]::FindWindow($null, 'Droplet')
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw 'The Droplet panel window was not created.'
    }

    [DropletTabs]::ShowWindow($hwnd, 5) | Out-Null
    Start-Sleep -Milliseconds 700

    $dpi = [DropletTabs]::GetDpiForWindow($hwnd)
    if ($dpi -le 0) { $dpi = 96 }
    $scale = $dpi / 96.0

    # Rail items start at y=50 with a 34px height and 36px pitch at 96 DPI.
    # The click points are client coordinates, matching App::on_pointer's
    # coordinate space.
    $tabs = @(
        @{ Name = 'droplet-core.png';    Y = 67;  X = 50 },
        @{ Name = 'droplet-crystal.png'; Y = 103; X = 50 },
        @{ Name = 'droplet-assist.png';  Y = 139; X = 50 },
        @{ Name = 'droplet-system.png';  Y = 175; X = 50 }
    )

    foreach ($tab in $tabs) {
        Click-Tab $hwnd ([int]($tab.X * $scale)) ([int]($tab.Y * $scale))
        Save-Panel $hwnd $tab.Name
        Write-Output "Captured $($tab.Name)"
    }
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id
    }
}
