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

# Capture from a fresh, isolated profile. Using the caller's real config makes
# the published images depend on whatever bindings and toggles happen to be on
# this machine (and can preserve a legacy duplicate binding). The seeded
# profile makes every capture deterministic and demonstrates Anchor ownership
# of XButton1; normalize_bindings clears the old Breach owner on load.
$captureRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("droplet-capture-" + [Guid]::NewGuid().ToString("N"))
$captureData = Join-Path $captureRoot "Droplet"
New-Item -ItemType Directory -Force -Path $captureData | Out-Null
Set-Content -LiteralPath (Join-Path $captureData "config.ini") -Encoding ASCII -Value @"
[T]
v0=1
vP=1
v6Bn=XButton1
"@
$savedEnv = @{
    LOCALAPPDATA = $env:LOCALAPPDATA
    TEMP = $env:TEMP
    TMP = $env:TMP
}

Add-Type -AssemblyName System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @"
using System;
using System.Drawing;
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
    public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);

    [DllImport("user32.dll")]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateRectRgn(int left, int top, int right, int bottom);

    [DllImport("user32.dll")]
    public static extern int GetWindowRgn(IntPtr hwnd, IntPtr region);

    [DllImport("gdi32.dll")]
    public static extern bool PtInRegion(IntPtr region, int x, int y);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr obj);

    // Apply the live window's own region, not a hardcoded radius. This makes
    // the published PNG prove the real silhouette and cannot hide a square
    // frame with a decorative mask.
    public static void ApplyWindowRegionAlpha(Bitmap bitmap, IntPtr hwnd) {
        IntPtr region = CreateRectRgn(0, 0, 0, 0);
        try {
            if (GetWindowRgn(hwnd, region) == 0) {
                return;
            }
            for (int y = 0; y < bitmap.Height; y++) {
                for (int x = 0; x < bitmap.Width; x++) {
                    if (!PtInRegion(region, x, y)) {
                        bitmap.SetPixel(x, y, Color.Transparent);
                    }
                }
            }
        } finally {
            DeleteObject(region);
        }
    }
}
"@

function Save-Panel([IntPtr]$Hwnd, [string]$Name) {
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
        $hdc = $graphics.GetHdc()
        try {
            # PrintWindow captures the window rather than the desktop behind its
            # acrylic. CopyFromScreen would include unrelated windows that show
            # through the translucent panel.
            if (-not [DropletTabs]::PrintWindow($Hwnd, $hdc, 2)) {
                throw 'PrintWindow failed.'
            }
        } finally {
            $graphics.ReleaseHdc($hdc)
        }
        # Use the live region. The old hardcoded GraphicsPath could make a
        # square DWM frame look rounded in the screenshot even when the app
        # itself did not clip that frame.
        [DropletTabs]::ApplyWindowRegionAlpha($bitmap, $Hwnd)
        $target = Join-Path $out $Name
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

$process = $null
try {
    $env:LOCALAPPDATA = $captureRoot
    $env:TEMP = $captureRoot
    $env:TMP = $captureRoot
    $process = Start-Process -FilePath $exe -WorkingDirectory $root -PassThru
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
        # The alpha mask comes from GetWindowRgn, so the PNG follows the
        # actual window silhouette at the current DPI.
        Save-Panel $hwnd $tab.Name
        Write-Output "Captured $($tab.Name)"
    }
} finally {
    if ($null -ne $process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -PassThru | Wait-Process -Timeout 5 -ErrorAction SilentlyContinue
    }
    foreach ($name in $savedEnv.Keys) {
        if ($null -eq $savedEnv[$name]) {
            Remove-Item "Env:$name" -ErrorAction SilentlyContinue
        } else {
            Set-Item "Env:$name" $savedEnv[$name]
        }
    }
    Remove-Item -LiteralPath $captureRoot -Recurse -Force -ErrorAction SilentlyContinue
}
