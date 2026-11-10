param(
    [string]$OutDir = "site/public/assets"
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\x86_64-pc-windows-gnu\release\hydrogen.exe'
if (-not (Test-Path -LiteralPath $exe)) {
    throw 'Build the release executable before capturing screenshots.'
}

$out = Join-Path $root $OutDir
New-Item -ItemType Directory -Force -Path $out | Out-Null

Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;

public static class HydrogenTabs {
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
    $rect = New-Object HydrogenTabs+RECT
    if (-not [HydrogenTabs]::GetWindowRect($Hwnd, [ref]$rect)) {
        throw 'GetWindowRect failed.'
    }
    $width = $rect.Right - $rect.Left
    $height = $rect.Bottom - $rect.Top
    if ($width -le 0 -or $height -le 0) {
        throw 'The Hydrogen panel has an empty window rectangle.'
    }

    $bitmap = New-Object System.Drawing.Bitmap($width, $height)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    try {
        $hdc = $graphics.GetHdc()
        try {
            [HydrogenTabs]::PrintWindow($Hwnd, $hdc, 2) | Out-Null
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
    [HydrogenTabs]::PostMessage($Hwnd, 0x0201, [IntPtr]::new(1), $lParam) | Out-Null
    Start-Sleep -Milliseconds 100
    [HydrogenTabs]::PostMessage($Hwnd, 0x0202, [IntPtr]::Zero, $lParam) | Out-Null
    Start-Sleep -Milliseconds 350
}

$process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
try {
    Start-Sleep -Seconds 2
    $hwnd = [HydrogenTabs]::FindWindow('HydrogenOverlay', 'Hydrogen')
    if ($hwnd -eq [IntPtr]::Zero) {
        $hwnd = [HydrogenTabs]::FindWindow($null, 'Hydrogen')
    }
    if ($hwnd -eq [IntPtr]::Zero) {
        throw 'The Hydrogen panel window was not created.'
    }

    [HydrogenTabs]::ShowWindow($hwnd, 5) | Out-Null
    Start-Sleep -Milliseconds 700

    $dpi = [HydrogenTabs]::GetDpiForWindow($hwnd)
    if ($dpi -le 0) { $dpi = 96 }
    $scale = $dpi / 96.0

    # Rail items start at y=56 with a 40px pitch at 96 DPI. The click points
    # are client coordinates, matching App::on_pointer's coordinate space.
    $tabs = @(
        @{ Name = 'hydrogen-core.png';    Y = 75;  X = 56 },
        @{ Name = 'hydrogen-crystal.png'; Y = 115; X = 56 },
        @{ Name = 'hydrogen-assist.png';  Y = 155; X = 56 },
        @{ Name = 'hydrogen-system.png';  Y = 195; X = 56 }
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
