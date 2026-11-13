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
    public static extern IntPtr SendMessage(IntPtr hwnd, uint msg, IntPtr wParam, ref RECT lParam);

    [DllImport("user32.dll")]
    public static extern bool PostMessage(IntPtr hwnd, uint msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
}
"@

function New-RoundedPath([int]$Width, [int]$Height, [int]$Radius) {
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = [Math]::Max(1, $Radius * 2)
    $path.AddArc(0, 0, $d, $d, 180, 90)
    $path.AddArc($Width - $d, 0, $d, $d, 270, 90)
    $path.AddArc($Width - $d, $Height - $d, $d, $d, 0, 90)
    $path.AddArc(0, $Height - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    return $path
}

function Save-Panel([IntPtr]$Hwnd, [string]$Name, [int]$Radius) {
    $rect = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($Hwnd, [ref]$rect)) {
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
        # CopyFromScreen returns the rectangular composited surface. Preserve
        # the actual silhouette with a transparent rounded mask for the site.
        $masked = New-Object System.Drawing.Bitmap($width, $height, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $maskGraphics = [System.Drawing.Graphics]::FromImage($masked)
        $maskPath = New-RoundedPath $width $height $Radius
        try {
            $maskGraphics.Clear([System.Drawing.Color]::Transparent)
            $maskGraphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
            $maskGraphics.SetClip($maskPath)
            $maskGraphics.DrawImage($bitmap, 0, 0, $width, $height)
            $target = Join-Path $out $Name
            $masked.Save($target, [System.Drawing.Imaging.ImageFormat]::Png)
            $image = [System.Drawing.Image]::FromFile($target)
            try {
                if ($image.Width -ne 624 -or $image.Height -ne 408) {
                    throw "Saved capture is $($image.Width)x$($image.Height), expected 624x408."
                }
            } finally {
                $image.Dispose()
            }
        } finally {
            $maskPath.Dispose()
            $maskGraphics.Dispose()
            $masked.Dispose()
        }
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

# The PowerShell host may be DPI-unaware. Make it per-monitor v2 before it
# reads any window rect or screen pixels; otherwise Windows virtualizes a
# 120-DPI panel back to 624x408 and clips the capture.
[DropletTabs]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null

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

    # Force the panel to 100% for the canonical 624x408 site assets. This
    # exercises the same WM_DPICHANGED path used by a real monitor move.
    $current = New-Object DropletTabs+RECT
    if (-not [DropletTabs]::GetWindowRect($hwnd, [ref]$current)) {
        throw 'GetWindowRect failed before the DPI reset.'
    }
    $suggested = New-Object DropletTabs+RECT
    $suggested.Left = $current.Left
    $suggested.Top = $current.Top
    $suggested.Right = $current.Left + 624
    $suggested.Bottom = $current.Top + 408
    $dpiWord = 96 -bor (96 -shl 16)
    [DropletTabs]::SendMessage($hwnd, 0x02E0, [IntPtr]::new($dpiWord), [ref]$suggested) | Out-Null
    Start-Sleep -Milliseconds 600

    # Rail items start at y=52 with a 34px height and 36px pitch at 96 DPI.
    # These are physical client coordinates, matching App::on_pointer.
    $tabs = @(
        @{ Name = 'droplet-core.png';    Y = 69;  X = 50 },
        @{ Name = 'droplet-crystal.png'; Y = 105; X = 50 },
        @{ Name = 'droplet-assist.png';  Y = 141; X = 50 },
        @{ Name = 'droplet-system.png';  Y = 177; X = 50 }
    )

    foreach ($tab in $tabs) {
        Click-Tab $hwnd ([int]$tab.X) ([int]$tab.Y)
        # Must match ui::CORNER_RADIUS so the site mask and the window agree.
        Save-Panel $hwnd $tab.Name 8
        Write-Output "Captured $($tab.Name)"
    }
} finally {
    if (-not $process.HasExited) {
        Stop-Process -Id $process.Id
    }
}
